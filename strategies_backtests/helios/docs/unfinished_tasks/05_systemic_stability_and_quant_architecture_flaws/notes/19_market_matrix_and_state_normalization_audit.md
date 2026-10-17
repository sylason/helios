# KIỂM TOÁN KIẾN TRÚC MA TRẬN TRẠNG THÁI & CHUẨN HÓA DỮ LIỆU (MARKET MATRIX & STATE NORMALIZATION AUDIT)

> **Mã tài liệu:** `NOTE-19-MATRIX-NORMALIZATION-AUDIT`  
> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Phạm vi kiểm toán:** `matrix.rs`, `sensors.rs`, `normalization.rs`, `config.rs`, `ferro_trade/indicators/calculate/bulk.rs`  
> **Đối chiếu chéo:** `docs/audit_01/04_matrix/`, `05_matrix/`, `15_semantic_mismatch_audit/`, `audit_01/ROADMAP.md`, `notes/07`, `notes/08`  
> **Mục tiêu:** Kiểm toán toàn diện tính toàn vẹn toán học, hành vi vi cấu trúc, bẫy suy biến phân vị, dị tật forward-fill đa khung thời gian và dọn dẹp triệt để các tàn dư chỉ báo/gen ma.

---

## 🧭 1. BẢNG ĐỐI CHIẾU TRẠNG THÁI TỔNG THỂ (STATUS MAPPING)

| Mã Lỗi / Task | Vị Trí File & Dòng Code | Hiện Trạng Thực Tế Trong Codebase | Phân Loại & Mức Độ |
| :--- | :--- | :--- | :---: |
| **`TODO-MAT-01`**<br>(Squeeze Intensity) | `core_v3/matrix.rs`<br>`L55-63, L81` | Đã dùng công thức phân số bậc 2 $\frac{1}{1 + (BB/KC)^2}$, **nhưng vẫn bị đẩy qua `rolling_percentile`** $\to$ Vẫn sập bẫy tụt về 0.50 khi nén dài! | ⚠️ **FIX DỞ DANG (BẪY MỚI)**<br>*(Rất nghiêm trọng)* |
| **`TODO-MAT-04`**<br>(Chaos Index) | `core_v3/matrix.rs`<br>`L83-84, L98-104` | Đã đổi sang $c\_r \times (1.0 - \text{entropy})$, **nhưng cả hai vẫn chạy rolling percentile trên nến rải phẳng** $\to$ Nhiễu nền 0.25 khi flat. | ⚠️ **FIX MỘT NỬA**<br>*(Nhiễu nền flat market)* |
| **`TODO-MAT-02`**<br>(KAMA Slope ATR) | `core_v3/matrix.rs`<br>`L87-90` | Vẫn dùng `(s / scale_atr).clamp(-1.0, 1.0)`, **chưa chuyển sang Tanh scaling** $\tanh(s / \text{scale\_atr})$. Bão hòa sớm tại $\pm 1.00$. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Nghiêm trọng)* |
| **`TODO-SEN-11`**<br>(Float Epsilon) | `utils/normalization.rs`<br>`L77, L88, L98` | `dedup_by` dùng `(*a - *b).abs() < f64::EPSILON` lệch pha với `binary_search_by` dùng `total_cmp` $\to$ Nến hợp lệ bị trả về `NaN`! | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Chí tử)* |
| **`TODO-FT-02`**<br>(Z-Score Flat) | `ferro_trade/.../bulk.rs:128`<br>`utils/normalization.rs:236` | Khi $\sigma \le 10^{-9}$ và $|X - \mu| = 0$, code trả về `f64::NAN` thay vì $Z = 0.0$ $\to$ Nhiễm độc `NaN` toàn hệ thống khi thị trường đứng im. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Chí tử)* |
| **`TODO-MAT-03`**<br>(Ghost Indicators) | `core_v3/matrix.rs`<br>`L12-22` | Đã xóa 4 chỉ báo ma (`liquidity`, `price_z_score`, `chaos`, `trend_axis`) và 4 bản clone thừa (`vol_axis`, `trend_dir`, `flow_axis`, `volume_force`). | 🟢 **ĐÃ FIX TRONG CODE**<br>*(Đã tinh giản sạch)* |
| **`TODO-CFG-05`**<br>(Ghost Genes) | `core_v3/config.rs`<br>`L393-418` | Đã thu hồi 5 gen ma khỏi `MatrixConfig` (`volume_lookback`, `momentum_lookback`, `adx_trend_threshold`, `vol_z_sensitivity`, `z_score_sensitivity`). DNA còn 102 chiều. | 🟢 **ĐÃ FIX TRONG CODE**<br>*(Đạt chuẩn DNA)* |
| **`FLAW-MTF-FF`**<br>(Forward-Fill Trap) | `core_v3/matrix.rs`<br>`L81-85` | Chạy `rolling_percentile` trực tiếp trên chuỗi 1m rải phẳng (Step Function) $\to$ Méo mó phân phối xác suất CDF, bách phân vị nhảy giật cục 15m/4h. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Lỗi kiến trúc cốt lõi)* |
| **`ORPHAN-SENS`**<br>(Cảm biến mồ côi) | `core_v3/mod.rs`<br>`L114-120, L181-200` | `sensors.kama_slope` được tính nhưng 0 ai đọc; `bb_middle_slope_adj` tính 2 lần kẹp biên lặp lại; `kama_slope` bị cắm nhầm chu kỳ `ind.atr.period`. | 🟡 **LỖI MỚI PHÁT HIỆN**<br>*(Ô nhiễm tham số chéo)* |

---

## 🔬 2. MỔ XẺ CHI TIẾT CÁC LỖ HỔNG TOÁN HỌC & KIẾN TRÚC

### 2.1. Squeeze Index: Công Thức Mới Nhưng Vẫn Sập Bẫy Suy Biến 0.50
- **Hiện trạng công thức (`matrix.rs:55-63, 81`):**
  $$\text{sqz\_raw}[i] = \frac{1.0}{1.0 + \left(\frac{\max(BB_w, 0.0)}{KC_w}\right)^2}$$
  Sau đó:
  ```rust
  let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
  ```
- **Bản chất toán học của bẫy tụt về 0.50 (Prolonged Squeeze Degradation):**
  1. Bản thân hàm $\frac{1}{1 + (BB_w / KC_w)^2}$ đã là một đại lượng vô hướng chuẩn mực thuộc $[0.0, 1.0]$:
     - Khi nén cực đại ($BB_w \ll KC_w$): $\text{sqz\_raw} \to 1.0$.
     - Khi chạm biên Carter ($BB_w = KC_w$): $\text{sqz\_raw} = 0.50$.
     - Khi bung nén bão lớn ($BB_w \gg KC_w$): $\text{sqz\_raw} \to 0.0$.
  2. Khi thị trường nén chặt kéo dài hơn cửa sổ trượt $W = \text{sqz\_lb}$ (ví dụ 20 nến hoặc 100 nến, đặc biệt với nến MTF 15m/4h rải phẳng ra 1m):
     - Tất cả các phần tử trong cửa sổ trượt đều có giá trị bằng nhau xấp xỉ $0.90$.
     - Thuật toán `rolling_percentile` tính:
       $$\text{count\_smaller} = 0, \quad \text{count\_equal} = W \implies \text{percentile} = \frac{0 + 0.5 \times W}{W} = \mathbf{0.50}$$
  3. **Hệ quả chí tử:** Đúng vào khoảnh khắc thị trường nén chặt nhất để chuẩn bị kích nổ xu hướng thì `squeeze_intensity` lại **tự động sụt giảm từ 0.90 về 0.50**!
     - Bộ lọc Chop Zone trong `trend_pb.rs:64` bị lừa là thị trường đã thoát nén, vội vàng vào lệnh sớm khi giá vẫn đang tích lũy đi ngang.
     - `absorption.rs` bị mất điểm tựa cường độ nén, bỏ qua cơ hội gom hàng chân sóng.
- **Giải pháp dứt điểm (Occam's Razor):**
  - **Bỏ hoàn toàn bước bọc qua `rolling_percentile`!** Sử dụng trực tiếp giá trị thực phi thứ nguyên $\text{sqz\_raw} \in [0.0, 1.0]$.
  - Tiết kiệm $100\%$ chi phí duy trì cấu trúc dữ liệu cây Fenwick phức tạp.

---

### 2.2. Chaos Index: Nhiễu Nền Flat Market & Lệch Pha Khung Thời Gian
- **Công thức hiện tại (`matrix.rs:98-104`):**
  $$\text{chaos\_index}[i] = c\_r \times (1.0 - \text{entropy})$$
  với $c\_r = \text{rolling\_percentile}(\text{chaos\_ratio}, W)$ và $\text{entropy} = \text{rolling\_percentile}(\text{ER}, W)$.
- **Vấn đề 1: Nhiễu nền Flat Market (0.25 Floor Noise):**
  - Khi thị trường hoàn toàn êm đềm, không biến động:
    - Chuỗi $ER \approx 0.0$ phẳng lì $\implies \text{rolling\_percentile}(ER) = 0.50$ (do tất cả các phần tử bằng nhau).
    - Chuỗi $\text{chaos\_ratio}$ phẳng lì $\implies \text{rolling\_percentile}(\text{chaos\_ratio}) = 0.50$.
    - Kết quả: $\text{chaos\_index} = 0.50 \times (1.0 - 0.50) = \mathbf{0.25}$!
  - Khi ER dao động nhỏ giọt quanh 0: $\text{entropy} \to 0.0 \implies noise = 1.0 - 0.0 = 1.0 \implies \text{chaos\_index} = 0.50 \times 1.0 = \mathbf{0.50}$.
  - Dải giá trị thực tế bị co hẹp về $[0.25, 1.00]$, làm cho ngưỡng `chaos_veto_threshold` ($0.50 \sim 0.70$) của `RegimeGatekeeper` trở nên cực kỳ nhạy cảm và dễ Veto oan các phiên tích lũy lành mạnh.
- **Vấn đề 2: Lệch pha đa khung thời gian (Temporal Desynchronization):**
  - $\text{chaos\_ratio}$ được tính theo `ind.natr.timeframe` (có thể là 4h).
  - $\text{efficiency\_ratio}$ được tính theo `ind.kama.timeframe` (có thể là 15m).
  - Khi có 1 cây nến 4h biến động mạnh vì tin tức, $\text{chaos\_ratio}$ giữ mức cao suốt 240 nến 1m (4 giờ). Trong suốt 4 giờ đó, khung 15m đã hạ nhiệt và đi vào vùng tích lũy phẳng ($ER \to 0 \implies noise \to 1.0$). Nhân lại ra $\text{chaos\_index} > 0.70$, khiến Gatekeeper khóa chặt bot không cho giao dịch suốt 4 tiếng giữa vùng tích lũy hoàn hảo!

---

### 2.3. KAMA Slope Scaling: Bão Hòa Độ Dốc Clamp $\pm 1.0$ (`TODO-MAT-02`)
- **Đoạn mã hiện tại (`matrix.rs:87-90`):**
  ```rust
  let scale_atr = if config.matrix.kama_slope_scale_atr <= 0.0 { 0.20 } else { config.matrix.kama_slope_scale_atr };
  let kama_slope_atr: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
      if s.is_finite() { (s / scale_atr).clamp(-1.0, 1.0) } else { 0.0 }
  }).collect();
  ```
- **Hệ quả vi cấu trúc:**
  - `kama_slope_adj` trên khung 15m/1h khi có sóng lớn thường đạt $0.40 \dots 1.50$.
  - Chia cho `scale_atr = 0.20` sẽ ra $2.0 \dots 7.5$.
  - Lệnh `.clamp(-1.0, 1.0)` kẹp cứng toàn bộ các giá trị này tại **$+1.00$ hoặc $-1.00$** suốt hàng chục nến liên tiếp.
  - Ma trận mất hoàn toàn khả năng phân biệt giữa:
    - Một xu hướng tăng bền vững lành mạnh (độ dốc vừa phải, $s / \text{scale} \approx 1.5$).
    - Một cú tăng dựng đứng kiệt sức (Climax Parabolic Run, $s / \text{scale} \ge 5.0$).
- **Giải pháp chuyển đổi:** Chuyển sang hàm trơn hyperbol:
  $$\text{kama\_slope\_atr} = \tanh\left(\frac{s}{\text{scale\_atr}}\right)$$

---

### 2.4. Động Cơ Chuẩn Hóa `normalization.rs` & `ferro_trade`

#### 🔴 Lỗ hổng 1: Lệch pha Float Epsilon trong `rolling_percentile` (`TODO-SEN-11`)
- **Vị trí code:** `strategies_backtests/helios/src/utils/normalization.rs:77, 88, 98`.
- **Đoạn mã nguy hiểm:**
  ```rust
  unique_vals.sort_unstable_by(|a, b| a.total_cmp(b));
  unique_vals.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON); // <-- Dùng EPSILON
  ...
  if let Ok(rank) = unique_vals.binary_search_by(|v| v.total_cmp(&oldest_val)) // <-- Dùng total_cmp!
  ```
- **Kịch bản phát sinh lỗi:**
  - Nếu hai giá trị $v_1, v_2$ có $|v_1 - v_2| < \text{EPSILON}$, `dedup_by` loại bỏ $v_2$, chỉ giữ $v_1$.
  - Khi gặp nến mang giá trị $v_2$, `binary_search_by` dùng `total_cmp` so sánh bit chính xác. Vì $v_1 \neq v_2$ về mặt bit representation, hàm trả về `Err(insert_idx)`.
  - Nhánh `if let Ok(rank)` bị bypass hoàn toàn:
    1. Nến mới không được chèn vào cây Fenwick $\implies$ `result[i]` giữ nguyên `f64::NAN`!
    2. Khi nến trượt ra khỏi window, `binary_search_by` cũng trả về `Err` $\implies$ nến cũ không được gỡ khỏi cây Fenwick, làm sai lệch toàn bộ biến đếm `valid_window_count` và các giá trị phân vị về sau!
- **Giải pháp:** Sử dụng hàm so sánh nhất quán giữa khâu dedup và tìm kiếm nhị phân (Custom Float Comparator với dung sai $\epsilon = 10^{-7}$).

#### 🔴 Lỗ hổng 2: Phép chia 0/0 sinh `NaN` khi Flat Market trong Z-Score (`TODO-FT-02`)
- **Vị trí code:**
  1. `ferro_trade/src/indicators/calculate/bulk.rs:128`:
     ```rust
     if s > 0.0 { (val - m) / s } else { f64::NAN } // <-- LỖI: Trả về NaN!
     ```
  2. `strategies_backtests/helios/src/utils/normalization.rs:236-238`:
     ```rust
     if std_dev > 1e-9 { result[i] = (current - mean) / std_dev; } // <-- LỖI: std_dev <= 1e-9 thì giữ nguyên NaN!
     ```
- **Kịch bản lỗi:**
  - Khi thị trường phẳng lì (giá đứng im, nến Doji liên tiếp), độ lệch chuẩn $\sigma = 0.0$ và sai phân $|X - \mu| = 0.0$.
  - Theo toán học thống kê: Khi giá nằm ngay tại đường trung bình và không có độ phân tán, $Z\text{-Score} = \mathbf{0.0}$.
  - Việc trả về `f64::NAN` làm lây nhiễm NaN vào `sensors.z_score` và `sensors.vol_z_score`, khiến các điều kiện so sánh trong `trend_pb.rs`, `reversal.rs`, `anomaly.rs` bị đánh sập toàn bộ.
- **Giải pháp:** Khi $s \le 10^{-9}$, nếu $|val - m| < 10^{-9}$ thì trả về $0.0$, chỉ trả về `NaN` nếu dữ liệu gốc là `NaN`.

---

### 2.5. Tàn Dư Mồ Côi & Lỗi Mới Phát Hiện Trong `core_v3/mod.rs`
1. **`sensors.kama_slope` mồ côi 100% (`mod.rs:114-120`):**
   - Hàm `ctx.kama_slope` trả về cặp `(k_slope, k_slope_adj)`.
   - `s.kama_slope = k_slope` được lưu vào sensors nhưng **trong toàn bộ hệ thống không có bất kỳ dòng code nào đọc trường này** (các module chỉ đọc `kama_slope_adj` hoặc `kama_slope_atr`).
2. **Ngộ độc chéo chu kỳ ATR vào độ dốc KAMA (`mod.rs:117`):**
   ```rust
   let (k_slope, k_slope_adj) = ctx.kama_slope(
       ind.kama.timeframe, "KAMA_Slope", 
       ind.kama.short_period, 2, ind.kama.long_period, 
       ind.kama.short_period, ind.atr.period // <-- BẪY CROSS-POLLUTION!
   );
   ```
   - Tham số cuối cùng là chu kỳ ATR để chuẩn hóa độ dốc KAMA vĩ mô (15m/1h/4h), nhưng lại bị cắm biến `ind.atr.period` (vốn là chu kỳ của `atr_sl` vi mô 1m/3m/5m dùng cho Stop Loss).
   - Khi GA tối ưu chu kỳ SL vi mô, nó vô tình làm méo mó luôn độ dốc KAMA vĩ mô!
3. **Cấp phát tính toán 2 lần kẹp biên `bb_middle_slope_adj`:**
   - Tại `mod.rs:181-200`: Tính vòng lặp qua `k_bars = 5` và kẹp `.clamp(-2.0, 2.0)`.
   - Tại `matrix.rs:110-112`: Lại chạy thêm một vòng lặp `.map(|&s| s.clamp(-2.0, 2.0)).collect()` lặp lại y hệt!
   - Chỉ có duy nhất `range_osc.rs:74` đọc trường này, trong khi RangeOsc đang bị tắt trong cấu hình GA.

---

### 2.6. Dị Tật Forward-Fill & Lệch Pha MTF (`notes/08`)
- **Cơ chế:** Tầng `ferro_trade` tính các chỉ báo MTF (15m, 1h, 4h) rồi dùng `map_indicator_to_base_tf` rải phẳng thành mảng 1m dưới dạng **Hàm Bậc Thang (Step Function)**.
- **Hệ quả khi chạy `rolling_percentile` trên mảng rải phẳng:**
  1. **Ảo ảnh mẫu quan sát (Sample Inflation):** Trong một cửa sổ 300 nến 1m của khung 15m, thực chất chỉ có $300 / 15 = 20$ quan sát độc lập. Mỗi quan sát bị nhân bản 15 lần.
  2. **Bóp méo hàm phân phối tích lũy CDF:** Cây Fenwick lưu trữ các giá trị trùng lặp cơ học, làm cho các bước nhảy xác suất bị phân mảnh và giật cục.
  3. **Tê liệt nội nến:** Suốt 14 phút giữa 2 lần đóng nến 15m, các chỉ số bách phân vị này hoàn toàn đóng băng, không phản ánh bất kỳ động lực học vi mô nào của dòng tiền.

---

## ⚖️ 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành Phần / Đề Xuất | Ưu Điểm | Khuyết Điểm | Edge Cases Nguy Hiểm & Giải Pháp |
| :--- | :--- | :--- | :--- |
| **Bỏ `rolling_percentile` ở Squeeze** | • Khử $100\%$ bẫy tụt về 0.50 khi nén lâu.<br>• Giữ nguyên độ nén thực $0.90$ khi thị trường tích lũy kéo dài.<br>• Tối ưu CPU, bỏ cây Fenwick. | • Dải giá trị không phân bố đều kiểu Percentile, phụ thuộc vào tỷ lệ $BB/KC$. | **Edge Case:** $KC_w \to 0$ khi thị trường đóng băng hoàn toàn.<br>$\implies$ Đã có sẵn guard `kc_w.max(1e-6)` trong code. |
| **Tanh Scaling cho KAMA Slope** | • Giữ được gradient trơn tru khi trend bùng nổ mạnh.<br>• Không bị bão hòa cứng tại $\pm 1.00$. | • Cần chọn hệ số chia `scale_atr` hợp lý ($0.20 \sim 0.50$). | **Edge Case:** Độ dốc cực đại $\to \infty$.<br>$\implies \tanh(\infty) = 1.0$, hoàn toàn an toàn và tiệm cận mượt mà. |
| **Custom Float Comparator trong Fenwick** | • Khử $100\%$ lỗi `NaN` do lệch pha giữa `f64::EPSILON` và `total_cmp`. | • Tăng thêm một vài phép so sánh float trong binary search. | **Edge Case:** Giá trị float quá gần nhau $< 10^{-7}$.<br>$\implies$ Gộp chung vào cùng một bucket phân vị là hành vi chuẩn xác. |
| **Fix Z-Score Flat Market Fallback 0.0** | • Khử triệt để hiện tượng lây nhiễm `NaN` toàn hệ thống khi flat market. | • Không có. | **Edge Case:** Mảng đầu vào toàn bộ là `NaN`.<br>$\implies$ Kiểm tra `val.is_nan()` trước khi trả về $0.0$. |
| **Đồng bộ Timeframe cho Chaos Index** | • Khử hiện tượng bão ảo do nến 4h giật 1 cây nhưng nến 15m đi ngang. | • Giảm 1 bậc tự do độc lập của GA. | **Edge Case:** Người dùng cấu hình tay 2 timeframe lệch nhau.<br>$\implies$ Tự động cảnh báo hoặc fallback về timeframe nhỏ hơn. |

---

## 📋 4. DANH MỤC CÔNG VIỆC CẦN TRIỂN KHAI (TODO ACTION ITEMS)

- [ ] **Task MAT-01:** Bỏ `rolling_percentile` khỏi `squeeze_intensity` trong `matrix.rs:81`, gán trực tiếp `Arc::new(sqz_raw)`.
- [ ] **Task MAT-02:** Triển khai Tanh scaling cho `kama_slope_atr`: `(s / scale_atr).tanh()` tại `matrix.rs:89`.
- [ ] **Task MAT-03:** Khắc phục Float Epsilon mismatch trong `normalization.rs:77, 88, 98`.
- [ ] **Task MAT-04:** Sửa fallback Z-Score khi flat market ($\sigma \le 1e-9$) trả về $0.0$ thay vì `NaN` trong `bulk.rs:128` và `normalization.rs:236`.
- [ ] **Task MAT-05:** Dọn sạch cảm biến mồ côi `sensors.kama_slope` và gỡ ô nhiễm tham số `ind.atr.period` khỏi `ctx.kama_slope` tại `core_v3/mod.rs:117`.
- [ ] **Task MAT-06:** Ràng buộc đồng bộ Timeframe giữa `NATR` và `KAMA` để triệt tiêu lệch pha thời gian trong `chaos_index`.
