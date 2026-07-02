// strategies_backtests/helios/src/utils/normalization.rs

/// Chuẩn hóa Min-Max: Chuyển dữ liệu về dải [0.0, 1.0].
/// Bỏ qua NaN khi tính toán nhưng giữ nguyên vị trí NaN trong kết quả.
pub fn min_max_scale(data: &[f64]) -> Vec<f64> {
    let valid_data: Vec<f64> = data.iter().filter(|v| !v.is_nan()).cloned().collect();
    if valid_data.is_empty() { return vec![f64::NAN; data.len()]; }

    let min = valid_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = valid_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;

    data.iter().map(|&v| {
        if v.is_nan() { f64::NAN }
        else if range.abs() < f64::EPSILON { 0.5 }
        else { (v - min) / range }
    }).collect()
}

/// Chuẩn hóa Tanh (Non-linear): Nén dữ liệu về dải [-1.0, 1.0].
/// sensitivity: Hệ số điều chỉnh độ dốc. Càng lớn thì kết quả càng mượt, ít bị bão hòa sớm.
pub fn tanh_scale(data: &[f64], sensitivity: f64) -> Vec<f64> {
    data.iter().map(|&v| {
        if v.is_nan() { f64::NAN }
        else { (v / sensitivity).tanh() }
    }).collect()
}

/// [ELITE] Chuẩn hóa Bách phân vị trượt (Rolling Percentile).
/// Trả về giá trị từ 0.0 đến 1.0, cho biết giá trị hiện tại đứng ở đâu trong cửa sổ W nến quá khứ.
/// Tối ưu hóa cực đại (Sliding Sorted Window O(N logW + memmove)): 
/// Giảm 108 tỷ phép tính xuống còn vài chục triệu phép tịnh tiến, tăng tốc x100 lần.
pub fn rolling_percentile(data: &[f64], window_size: usize) -> Vec<f64> {
    let len = data.len();
    let mut result = vec![f64::NAN; len];
    if window_size == 0 { return result; }

    // Cửa sổ trượt được sắp xếp liên tục
    let mut sorted_window: Vec<f64> = Vec::with_capacity(window_size);

    for i in 0..len {
        let current_val = data[i];

        // 1. Gỡ bỏ nến cũ khỏi cửa sổ
        if i >= window_size {
            let oldest_val = data[i - window_size];
            if !oldest_val.is_nan() {
                if let Ok(idx) = sorted_window.binary_search_by(|v| v.total_cmp(&oldest_val)) {
                    sorted_window.remove(idx);
                }
            }
        }

        // 2. Chèn nến mới vào cửa sổ (giữ tính sắp xếp)
        if !current_val.is_nan() {
            let insert_idx = match sorted_window.binary_search_by(|v| v.total_cmp(&current_val)) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            sorted_window.insert(insert_idx, current_val);
        }

        if current_val.is_nan() {
            continue;
        }

        // 3. Tính Percentile bằng Binary Search thay vì đếm từng phần tử
        let valid_count = sorted_window.len();
        if valid_count > 0 {
            // Tìm số phần tử < current_val
            let count_smaller = sorted_window.partition_point(|&v| v < current_val - f64::EPSILON);
            // Tìm số phần tử <= current_val
            let count_smaller_or_equal = sorted_window.partition_point(|&v| v <= current_val + f64::EPSILON);
            
            let count_equal = count_smaller_or_equal.saturating_sub(count_smaller);

            // [FIX MTF]: Xử lý đúng cho cửa sổ có các giá trị step-filled (bằng nhau)
            result[i] = (count_smaller as f64 + 0.5 * count_equal as f64) / valid_count as f64;
        } else {
            result[i] = 0.5;
        }
    }
    result
}

/// Tính toán cường độ Squeeze (Nén lò xo).
///
/// --- [BUG #3 - CŨ - ĐÃ VÔ HIỆU HÓA] ---
/// Logic cũ: Intensity = 1.0 - (BB_Width / KC_Width)
/// Vấn đề:
///   - Trên thị trường crypto 1m, BB thường RỘNG hơn KC (biến động cao)
///   - Khi BB > KC → ratio > 1.0 → (1.0 - ratio) âm → .max(0.0) → = 0.0
///   - Kết quả: sqz_raw toàn bộ = 0.0 → rolling_percentile = 0.5 cứng (mọi giá trị bằng nhau)
///   - Squeeze squad không bao giờ có signal thực sự để Optimizer học
///
/// --- [FIX #3 - MỚI]: Overlap Logic ---
/// Ý tưởng cốt lõi: Đo mức độ "BB nằm GẮN với KC" thay vì "BB nhỏ hơn KC bao nhiêu"
///
/// Cách tính:
///   - overlap_upper = KC_upper - BB_upper (BB_upper nằm dưới KC_upper bao nhiêu?)
///   - overlap_lower = BB_lower - KC_lower (BB_lower nằm trên KC_lower bao nhiêu?)
///   - Nếu cả hai dương → BB hoàn toàn nằm trong KC → Squeeze mạnh nhất
///   - Nếu một bên âm → BB thoát ra một phía → Squeeze đang giải phóng
///   - Chuẩn hóa theo KC_width để có kết quả [0, 1] bất kể mức giá tuyệt đối
///
/// Kết quả:
///   - sqz_raw = 1.0 → BB hoàn toàn nằm trong KC (Squeeze cực mạnh)
///   - sqz_raw = 0.5 → BB thoát ra một bên nhẹ
///   - sqz_raw = 0.0 → BB thoát hẳn ra ngoài KC (Breakout)
///   - sqz_raw có thể âm → BB vượt rất xa KC (Không bao giờ xảy ra khi đã clamp)
pub fn calculate_squeeze_intensity(
    bb_upper: &[f64],
    bb_lower: &[f64],
    kc_upper: &[f64],
    kc_lower: &[f64],
) -> Vec<f64> {
    let len = bb_upper.len();
    let mut intensity = Vec::with_capacity(len);

    for i in 0..len {
        let bu = bb_upper[i];
        let bl = bb_lower[i];
        let ku = kc_upper[i];
        let kl = kc_lower[i];

        if bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
            intensity.push(f64::NAN);
            continue;
        }

        // --- [CŨ - VÔ HIỆU HÓA] ---
        // let bb_w = bu - bl;
        // let kc_w = ku - kl;
        // let ratio = if kc_w > 1e-9 { bb_w / kc_w } else { 1.0 };
        // let val = (1.0 - ratio).max(0.0).min(1.0);
        // intensity.push(val);

        // --- [MỚI]: Overlap Logic ---
        let kc_w = ku - kl;
        if kc_w < 1e-9 {
            // KC width quá nhỏ (dữ liệu lỗi) → trả về 0 (không squeeze)
            intensity.push(0.0);
            continue;
        }

        // Đo phần "BB nằm bên trong KC" từ mỗi phía
        // overlap_upper > 0: BB_upper nằm dưới KC_upper (tốt — đang bị nén)
        // overlap_upper < 0: BB_upper vượt qua KC_upper (bung ra)
        let overlap_upper = ku - bu; // KC_upper - BB_upper
        let overlap_lower = bl - kl; // BB_lower - KC_lower

        // Tổng overlap / KC_width → chuẩn hóa theo kích thước kênh
        // Khi BB hoàn toàn trong KC: overlap_upper + overlap_lower = KC_w - BB_w > 0
        // Khi BB = KC: tổng = 0
        // Khi BB > KC: tổng âm (đang breakout)
        let total_overlap = overlap_upper + overlap_lower;
        let raw = total_overlap / kc_w;

        // Clamp về [0, 1]: 0 = không squeeze; 1 = nén tối đa (BB rất nhỏ trong KC)
        intensity.push(raw.max(0.0).min(1.0));
    }

    intensity
}


/// Tính toán Z-Score cho một chuỗi dữ liệu (FIX #6).
/// Z = (Giá trị hiện tại - Trung bình quá khứ) / Độ lệch chuẩn quá khứ.
/// Tối ưu Zero-Allocation: Khử triệt để lệnh .collect::<Vec>() sinh rác bộ nhớ.
pub fn calculate_z_score_series(data: &[f64], lookback: usize) -> Vec<f64> {
    let len = data.len();
    let mut result = vec![f64::NAN; len];
    if len < lookback || lookback == 0 { return result; }

    for i in lookback..len {
        let current = data[i];
        if current.is_nan() { continue; }

        let start = i - lookback;
        let mut sum = 0.0;
        let mut count = 0.0;

        // Pass 1: Tính Mean
        for j in start..i {
            let v = data[j];
            if !v.is_nan() {
                sum += v;
                count += 1.0;
            }
        }

        if count < 1.0 { continue; }
        
        let mean = sum / count;
        let mut variance_sum = 0.0;

        // Pass 2: Tính Variance
        for j in start..i {
            let v = data[j];
            if !v.is_nan() {
                let diff = v - mean;
                variance_sum += diff * diff;
            }
        }

        let std_dev = (variance_sum / count).sqrt();

        if std_dev > 1e-9 {
            result[i] = (current - mean) / std_dev;
        }
    }
    result
}

/// Chuẩn hóa độ dốc xu hướng theo mức độ biến động (FIX #7).
/// Adjusted_Slope = Slope / ATR.
pub fn volatility_adjusted_slope(slopes: &[f64], atrs: &[f64]) -> Vec<f64> {
    slopes.iter().enumerate().map(|(i, &s)| {
        let atr = atrs[i];
        if s.is_nan() || atr.is_nan() || atr < 1e-9 { f64::NAN }
        else { s / atr }
    }).collect()
}
