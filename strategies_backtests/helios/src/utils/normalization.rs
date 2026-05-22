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
pub fn tanh_scale(data: &[f64], sensitivity: f64) -> Vec<f64> {
    data.iter().map(|&v| {
        if v.is_nan() { f64::NAN }
        else { (v * sensitivity).tanh() }
    }).collect()
}

/// Tính toán cường độ Squeeze (Nén lò xo).
/// Intensity = 1.0 - (BB_Width / KC_Width)
/// Càng nén mạnh (BB nằm sâu trong KC) thì Intensity càng tiến về 1.0.
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

        let bb_w = bu - bl;
        let kc_w = ku - kl;
        
        let ratio = if kc_w > 1e-9 { bb_w / kc_w } else { 1.0 };
        let val = (1.0 - ratio).max(0.0).min(1.0);
        intensity.push(val);
    }

    intensity
}

/// Tính toán Z-Score cho một chuỗi dữ liệu (FIX #6).
/// Z = (Giá trị hiện tại - Trung bình quá khứ) / Độ lệch chuẩn quá khứ.
pub fn calculate_z_score_series(data: &[f64], lookback: usize) -> Vec<f64> {
    let len = data.len();
    let mut result = vec![f64::NAN; len];
    if len < lookback || lookback == 0 { return result; }

    for i in lookback..len {
        let window = &data[i-lookback..i]; 
        let valid_window: Vec<f64> = window.iter().filter(|v| !v.is_nan()).cloned().collect();
        if valid_window.is_empty() { continue; }

        let count = valid_window.len() as f64;
        let mean: f64 = valid_window.iter().sum::<f64>() / count;
        let variance: f64 = valid_window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let current = data[i];
        if !current.is_nan() && std_dev > 1e-9 {
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
