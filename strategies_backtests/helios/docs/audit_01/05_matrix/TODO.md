# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (ACTIONABLE TODO): MARKET MATRIX CORE V3
**Module phụ trách:** [`matrix.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs)  
**Tài liệu kiểm toán gốc:** [technical_audit.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_matrix/technical_audit.md)  
**Ngày lập kế hoạch:** 19/09/2026  
**Chuyên gia kiểm toán:** Quantitative & Systems Auditor  

---

## 🎯 BẢNG TỔNG QUAN ƯU TIÊN THỰC THI

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-MAT-01` | Tái cấu trúc trục volume: Xóa `flow_axis` & gen `vol_z_sensitivity` (Occam) | **CRITICAL** | 🟢 Đã xử lý | `core_v3/matrix.rs`, `scanners/momentum_ignition.rs` |
| **2** | `TASK-MAT-02` | Tái cấu trúc công thức Squeeze Intensity thoát khỏi bẫy 0.50 | **CRITICAL** | 🔴 Chưa xử lý | `core_v3/matrix.rs:60–68, 83` |
| **3** | `TASK-MAT-03` | Đồng bộ hóa dải giá trị `trend_strength` về $[0.0, 1.0]$ và khử số âm | **HIGH** | 🔴 Chưa xử lý | `core_v3/matrix.rs:71–81` |
| **4** | `TASK-MAT-04` | Triệt tiêu Deep Clone `Vec<f64>` thừa thãi, chuyển sang Arc aliasing | **HIGH** | 🔴 Chưa xử lý | `core_v3/matrix.rs:102, 126, 128` |
| **5** | `TASK-MAT-05` | Khóa cứng đồng bộ Timeframe giữa Bollinger Bands và Keltner Channels | **HIGH** | 🔴 Chưa xử lý | `core_v3/config.rs:103–124` |
| **6** | `TASK-MAT-06` | Chuẩn hóa phòng thủ NaN an toàn tuyệt đối, triệt tiêu bẫy `unwrap_or` | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/matrix.rs:51–73, 108–124` |
| **7** | `TASK-MAT-07` | Giữ lại `entropy`, xóa bỏ chỉ báo ma (`liquidity`, `chaos`) và gen ma `volume_lookback` | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/matrix.rs`, `core_v3/config.rs:266` |
| **8** | `TASK-MAT-08` | Bổ sung kẹp biên an toàn cho `bb_width_norm`, `bb_middle_slope_adj` | **LOW** | 🔴 Chưa xử lý | `core_v3/matrix.rs:58, 139` |
| **9** | `TASK-MAT-09` | Xây dựng bộ Unit Tests chuyên sâu bao phủ các lỗi toán học & edge cases | **HIGH** | 🔴 Chưa xử lý | `core_v3/matrix.rs:152–300` |
| **10** | `TASK-MAT-10` | Tái cấu trúc `chaos_index` bằng Kaufman ER (`entropy`) trị dứt điểm bẫy Super Trend | **CRITICAL** | 🟢 Đã xử lý | `core_v3/matrix.rs:116–124` |

---

## 📋 CHI TIẾT CÁC ĐẦU VIỆC CẦN THỰC HIỆN

### 🟢 TASK-MAT-01: Tái Cấu Trúc Trục Khối Lượng: Xóa Bỏ Hộp Đen `flow_axis` / `volume_force` (Occam's Scalpel) - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-01` (Critical).
- **Kết quả thực hiện:**
  - [x] Đã xóa bỏ hoàn toàn trường `flow_axis` và clone `volume_force` khỏi `MarketMatrixV3`, loại bỏ 2 vector cấp phát rác trên heap.
  - [x] Đã bỏ macro `#[dna(...)]` trên `vol_z_sensitivity` trong `MatrixConfig`, đưa số chiều DNA về 103 chuẩn xác.
  - [x] Chuyển `MomentumIgnition` sang đọc trực tiếp `sensors.vol_z_score` qua hàm chuẩn hóa `sat_unbounded(raw_vol_z, vol_thresh)`.
  - [x] Chuyển `RangeOscillator` sang đọc trực tiếp `sensors.vol_z_score` quy đổi an toàn qua `sat_unbounded(raw_vol_z, 2.0)`.
  - [x] Kiểm thử tự động: 100% tests của `helios` (65/65 tests) và `bot_engine` (39/39 tests) đã PASS.
- **Hiện trạng:**
  - Tại `core_v3/matrix.rs:106`, mã nguồn gọi:
    ```rust
    let vol_z_scale = normalization::tanh_scale(&sensors.vol_z_score, 1.0 / vol_z_sens);
    ```
  - Do hàm `tanh_scale` chia dữ liệu cho tham số độ nhạy (`v / sensitivity`), việc truyền `1.0 / vol_z_sens` khiến công thức trở thành $\tanh(v \times \text{vol\_z\_sens})$.
  - Hậu quả: Đảo ngược $180^\circ$ ý đồ quant. Khi tăng `vol_z_sensitivity` để giảm độ nhạy thì hệ thống lại biến thành siêu nhạy, bão hòa kịch trần $\approx 1.0$ ngay cả với biến động khối lượng vi mô.
  - Ngoài ra, mã nguồn cộng gộp cảm tính $50\%$ MFI vào `flow_axis`, tạo ra sàn đáy ảo $0.25$ ngay cả khi khối lượng bằng 0, làm tê liệt `RangeOscillator`.
  - **Phát hiện kiểm toán chuyên sâu:** Trong toàn bộ hệ thống, chỉ có 2 scanner đọc `flow_axis` / `volume_force` (`MomentumIgnition` và `RangeOscillator`), trong khi `Absorption` (scanner cần volume nhất) lại đọc trực tiếp `sensors.vol_z_score`. Việc duy trì `MatrixConfig.vol_z_sensitivity` tạo ra bẫy trùng lặp siêu tham số (Collinearity) với `mom_cfg.vol_z_threshold`.
- **Phương án Giải quyết Tối ưu Nhất (Occam's Scalpel - Chém Tận Gốc):**
  1. **Xóa bỏ hoàn toàn** trường `flow_axis` và bản sao nhân bản `volume_force` khỏi struct `MarketMatrixV3`.
  2. **Xóa bỏ macro `#[dna(...)]`** trên trường `vol_z_sensitivity` trong `MatrixConfig` (cắt giảm 1 chiều DNA thừa thãi).
  3. **Tại `momentum_ignition.rs`:** Đọc trực tiếp `sensors.vol_z_score`:
     ```rust
     let raw_vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
     let f_vol = sat_unbounded(raw_vol_z, mom_cfg.vol_z_threshold);
     ```
     *(Dải DNA của `mom_cfg.vol_z_threshold` là `[1.0, 3.5]` vốn dĩ được thiết kế nguyên bản cho Z-Score!)*
  4. **Tại `range_osc.rs`:** Đọc trực tiếp `sensors.vol_z_score` kèm chuẩn hóa an toàn:
     ```rust
     let raw_vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
     let vol_norm = sat_unbounded(raw_vol_z, 2.0);
     if vol_norm > range_cfg.volume_max_threshold { return None; }
     ```
     *(Lưu ý: RangeOsc và Anomaly hiện tại đang tắt gen DNA trong `config.rs`, nhưng vẫn cần xử lý phòng thủ để không bị Unit Mismatch khi bật lại).*
- **Phân tích 4 Edge Cases Chí Tử:**
  - *Edge Case 1 (DNA Length Shift):* Độ dài DNA giảm 1 (từ 104 -> 103). Cần dọn sạch DB SQLite cũ để tránh lỗi Deserialize (đã có cơ chế Storage Cleanup tự động).
  - *Edge Case 2 (Z-Score âm trong Momentum):* Khi $Z \le 0.0$, `sat_unbounded(val, thresh)` trả về $0.0$ ngay lập tức vì `val <= 0.0` (khối lượng yếu thì điểm xung lực = 0, hoàn hảo).
  - *Edge Case 3 (Unit Mismatch trong RangeOsc):* `volume_max_threshold` có dải DNA $[0.01, 0.99]$. Không được so sánh trực tiếp với $Z$-Score thô mà phải qua `sat_unbounded(raw_vol_z, 2.0)` để bảo toàn dải phần trăm.
  - *Edge Case 4 (Gãy Unit Tests trong matrix.rs):* Cập nhật các bài test cũ đang assert `flow_axis`.
- **Tiêu chuẩn nghiệm thu (Acceptance Criteria):**
  - Struct `MarketMatrixV3` không còn chứa 2 vector cấp phát rác `flow_axis` và `volume_force`.
  - `MomentumIgnition` nhận điểm bùng nổ khối lượng chuẩn xác từ `sensors.vol_z_score`.
  - Tất cả các unit test và integration tests trong `helios` pass 100%.

---

### 🔴 TASK-MAT-02: Tái cấu trúc công thức Squeeze Intensity thoát khỏi bẫy 0.50
- **Mã lỗi liên đới:** `FLAW-MAT-02` (Critical).
- **Hiện trạng:**
  - Tại `core_v3/matrix.rs:60–68`, biểu thức `(ku - bu) + (bl - kl) / kc_w` hoàn toàn đồng nhất đại số với $1.0 - (BB_w / KC_w)$.
  - Khi thị trường biến động ($BB_w > KC_w$), giá trị bị kẹp về $0.0$.
  - Khi `sqz_raw` toàn bộ là $0.0$, thuật toán `rolling_percentile` trả về bách phân vị **cứng $0.50$** (vì $\text{count\_smaller} = 0$, $\text{count\_equal} = W \implies (0 + 0.5W)/W = 0.50$).
  - Squeeze Scanner ngộ nhận thị trường luôn nén trung bình ngay giữa tâm bão biến động mạnh.
- **Hành động cần làm:**
  1. Tái cấu trúc công thức đo Squeeze thành hàm liên tục:
     $$\text{sqz\_ratio} = \frac{KC_w}{BB_w}$$
     $$\text{sqz\_raw} = \tanh\left(\frac{KC_w}{BB_w \times \text{factor}}\right) \in [0.0, 1.0]$$
  2. Khi $BB_w \gg KC_w$ (thị trường bung rộng, biến động cực đại), $\text{sqz\_raw} \to 0.0$.
  3. Đảm bảo hàm `rolling_percentile` phân biệt rõ ràng giữa giai đoạn co thắt thật sự và giai đoạn dải mở rộng.
- **Tiêu chuẩn nghiệm thu:**
  - Unit test kiểm tra khi $BB_w = 2.0 \times KC_w$, kết quả `sqz_raw` phải tiến sát về $0.0$, và chuỗi `squeeze_intensity` không bị kẹt ở $0.50$.

---

### 🟠 TASK-MAT-03: Đồng bộ hóa dải giá trị `trend_strength` về $[0.0, 1.0]$ và khử số âm
- **Mã lỗi liên đới:** `FLAW-MAT-03` (High).
- **Hiện trạng:**
  - Tại `core_v3/matrix.rs:72, 81`, `trend_strength` được tính bằng `tanh((adx - 20.0) / z_sens)`. Khi ADX $< 20.0$ (thị trường sideway/không trend), giá trị trả về âm sâu xuống tới $-1.0$.
  - Trong khi đó, định nghĩa trường tại dòng 22 ghi rõ: `0.0 to 1.0 (ADX Tanh Scaled)`.
  - Scanner `scanners/range_osc.rs:62` kiểm tra `trend_strength > max_trend`, khi gặp số âm có thể bị sai lệch logic chặn.
  - Đồng thời, tồn tại song song cả `trend_axis` và `trend_strength` cùng đo ADX nhưng theo hai chuẩn khác nhau.
- **Hành động cần làm:**
  1. Chuẩn hóa lại công thức `trend_strength` về $[0.0, 1.0]$ không có số âm:
     $$\text{trend\_strength}[i] = \tanh\left(\frac{\text{ADX}[i]}{\text{adx\_trend\_threshold}}\right) \in [0.0, 1.0]$$
  2. Đồng nhất hóa `trend_axis` và `trend_strength` hoặc phân định rõ: `trend_dir` là hướng $[-1.0, 1.0]$, `trend_strength` là độ mạnh $[0.0, 1.0]$.
- **Tiêu chuẩn nghiệm thu:**
  - Unit test khẳng định với mọi giá trị ADX từ $0.0$ đến $100.0$, `trend_strength` luôn thuộc $[0.0, 1.0]$, không bao giờ $< 0.0$.

---

### 🟠 TASK-MAT-04: Triệt tiêu Deep Clone `Vec<f64>` thừa thãi, chuyển sang Arc aliasing
- **Mã lỗi liên đới:** `FLAW-MAT-04` (High).
- **Hiện trạng:**
  - Tại `core_v3/matrix.rs:102, 126, 128`, mã nguồn gọi `.clone()` trên các `Vec<f64>`:
    + `let vol_axis = squeeze_intensity.clone();`
    + `let trend_dir = kama_slope_atr.clone();`
    + `let volume_force = flow_axis.clone();`
  - Gây lãng phí $> 12 \text{ MB}$ bộ nhớ heap cho mỗi lần backtest, tích lũy thành $> 60 \text{ GB}$ rác trong quá trình chạy GA Optimizer.
- **Hành động cần làm:**
  1. Thay thế việc nhân bản mảng bằng việc chia sẻ con trỏ `Arc`:
     ```rust
     let slope_arc = Arc::new(kama_slope_atr);
     let flow_arc = Arc::new(flow_axis);
     let sqz_arc = Arc::new(squeeze_intensity);

     Self {
         kama_slope_atr: slope_arc.clone(),
         trend_dir: slope_arc,
         flow_axis: flow_arc.clone(),
         volume_force: flow_arc,
         vol_axis: sqz_arc.clone(),
         squeeze_intensity: sqz_arc,
         ...
     }
     ```
  2. Giảm chi phí nhân bản từ $O(N)$ phân bổ lại mảng về $O(1)$ tăng reference counter (8 bytes).
- **Tiêu chuẩn nghiệm thu:**
  - `Arc::ptr_eq(&matrix.kama_slope_atr, &matrix.trend_dir)` trả về `true`.
  - `Arc::ptr_eq(&matrix.vol_axis, &matrix.squeeze_intensity)` trả về `true`.
  - `Arc::ptr_eq(&matrix.flow_axis, &matrix.volume_force)` trả về `true`.

---

### 🟠 TASK-MAT-05: Khóa cứng đồng bộ Timeframe giữa Bollinger Bands và Keltner Channels
- **Mã lỗi liên đới:** `FLAW-MAT-05` (High).
- **Hiện trạng:**
  - Tại `core_v3/config.rs:109, 120`, cả `BbandsParams` và `KeltnerParams` đều có gen DNA riêng để chọn `timeframe`.
  - GA Optimizer có thể chọn `bbands.timeframe = 5m` và `keltner.timeframe = 4h`, làm phá hủy hoàn toàn mối tương quan hình học của mô hình Squeeze.
- **Hành động cần làm:**
  1. Trong `KeltnerParams`, loại bỏ macro `#[dna(...)]` khỏi trường `timeframe` hoặc loại bỏ hoàn toàn trường này.
  2. Tại `core_v3/mod.rs:174`, ép buộc Keltner Channel sử dụng chung `ind.bbands.timeframe` với Bollinger Bands:
     ```rust
     let (kc_u, _kc_m, kc_l) = ctx.keltner(
         ind.bbands.timeframe, // Đồng bộ 100% với Bollinger Bands
         "KC", ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone()
     );
     ```
- **Tiêu chuẩn nghiệm thu:**
  - GA Optimizer không còn khả năng cấu hình lệch pha timeframe giữa BB và KC.
  - Squeeze luôn được đo lường trên cùng một khung nến nhất quán.

---

### 🟡 TASK-MAT-06: Chuẩn hóa phòng thủ NaN an toàn tuyệt đối, triệt tiêu bẫy `unwrap_or`
- **Mã lỗi liên đới:** `FLAW-MAT-06` (Medium).
- **Hiện trạng:**
  - Mẫu hình `.copied().unwrap_or(0.0)` không bắt được `f64::NAN` vì giá trị trả về là `Some(f64::NAN)`.
  - Kết quả phụ thuộc vào quirk ngầm của IEEE 754 trong `.max(0.0).min(1.0)`.
- **Hành động cần làm:**
  1. Viết một helper inline an toàn:
     ```rust
     #[inline(always)]
     fn sanitize_f64(val: Option<f64>, fallback: f64) -> f64 {
         match val {
             Some(v) if v.is_finite() => v,
             _ => fallback,
         }
     }
     ```
  2. Áp dụng helper này cho toàn bộ quá trình đọc dữ liệu từ `sensors` (`natr`, `bb`, `kc`, `adx`, `mfi`, `z_score`).
- **Tiêu chuẩn nghiệm thu:**
  - Unit test với mảng đầu vào chứa `f64::NAN`, `f64::INFINITY`, `f64::NEG_INFINITY` xác nhận ma trận đầu ra không có bất kỳ giá trị phi thực nào lọt qua.

---

### 🟡 TASK-MAT-07: Đại Phẫu Rác Thừa Matrix: Xóa Bỏ 4 Chỉ Báo Ma, 3 Bản Sao Clone và 3 Gen DNA Ma
- **Mã lỗi liên đới:** `FLAW-MAT-07` (Medium).
- **Hiện trạng kiểm toán toàn diện (11/17 trường thừa thãi & 3 gen DNA ma):**
  - **4 Chỉ báo ma (Zombie Indicators - 0 ai đọc trong toàn bộ chiến lược):**
    1. `trend_axis`: Map từ ADX nhưng không có scanner nào đọc (chỉ xuất hiện trong test nội bộ).
    2. `liquidity`: **CỰC KỲ LÃNG PHÍ**: Chạy cây Fenwick `rolling_percentile` trên MFI mỗi nến nhưng vứt xó!
    3. `chaos`: Tính từ `natr / 100.0`, hoàn toàn vô dụng.
    4. `price_z_score`: Tính bằng hàm `tanh_scale` nặng nề nhưng không module nào sử dụng.
  - **3 Bản sao nhân bản (Duplicate Clones - Tốn RAM heap allocations vô ích):**
    1. `vol_axis`: Deep clone 100% từ `squeeze_intensity` (chỉ dùng duy nhất ở `risk.rs:320`).
    2. `trend_dir`: Deep clone 100% từ `kama_slope_atr`.
    3. `volume_force`: Deep clone 100% từ `flow_axis`.
    *(Kèm trường `kama_slope_adj` trên matrix: Passthrough từ sensors nhưng các scanner toàn đọc thẳng từ `sensors`, không ai đọc từ `matrix`).*
  - **3 Gen DNA Ma (Phantom Genes) trong `MatrixConfig` (`core_v3/config.rs`):**
    1. `volume_lookback`: Khai báo gen 31 nấc tìm kiếm, nhưng trong toàn bộ `matrix.rs` **không có chữ `volume_lookback` nào**!
    2. `momentum_lookback`: Dùng tính `liquidity`, mà `liquidity` là Zombie $\implies$ GA tối ưu vô nghĩa!
    3. `adx_trend_threshold`: Dùng tính `trend_axis`, mà `trend_axis` là Zombie $\implies$ GA tối ưu vô nghĩa!
- **Hành động cần làm:**
  1. Giữ lại phép tính `entropy` và `chaos_ratio` trong `from_sensors` để phục vụ `TASK-MAT-10` (nhưng cân nhắc giữ cục bộ thay vì public clone nếu có thể).
  2. Xóa bỏ hoặc vô hiệu hóa các dòng tính toán 4 chỉ báo ma `liquidity`, `chaos`, `price_z_score`, `trend_axis`.
  3. Xóa các bản clone thừa `vol_axis`, `trend_dir`, `volume_force`, thay bằng dùng trực tiếp `squeeze_intensity` và `kama_slope_atr`.
  4. Xóa macro `#[dna(...)]` trên 3 gen ma `volume_lookback`, `momentum_lookback`, `adx_trend_threshold` trong `core_v3/config.rs` (giúp GA giảm thêm 3 chiều tìm kiếm vô ích).
- **Tiêu chuẩn nghiệm thu:**
  - Tổng số gen DNA của hệ thống giảm thêm 3 gen ma vô tác dụng.
  - Triệt tiêu hoàn toàn các phép tính Cây Fenwick và Tanh chạy vô ích cho các trường rác.
  - Bộ nhớ RAM heap allocations của Matrix giảm tối đa.

---

### 🟢 TASK-MAT-08: Bổ sung kẹp biên an toàn cho `bb_width_norm`, `bb_middle_slope_adj`
- **Mã lỗi liên đới:** `FLAW-MAT-08` (Low).
- **Hiện trạng:**
  - `bb_width_norm` và `bb_middle_slope_adj` không có kẹp biên trên, có thể bùng nổ vô tận khi flash crash.
- **Hành động cần làm:**
  1. Kẹp biên `bb_width_norm`: `.clamp(0.0, 1.0)`.
  2. Kẹp biên `bb_middle_slope_adj`: `.clamp(-2.0, 2.0)` (độ dốc SMA 20 trên ATR).
- **Tiêu chuẩn nghiệm thu:**
  - Giá trị không bao giờ tràn biên ngoài dải quy định ngay cả khi mô phỏng dữ liệu giá nhảy vọt $50\%$.

---

### 🟠 TASK-MAT-09: Xây dựng bộ Unit Tests chuyên sâu bao phủ các lỗi toán học & edge cases
- **Mã lỗi liên đới:** QA Verification.
- **Hành động cần làm:**
  - Viết các test cases mới trong `matrix.rs:tests`:
    1. `test_vol_z_sensitivity_scaling_direction`: Kiểm chứng tăng sensitivity làm giảm giá trị tanh (đúng chiều).
    2. `test_squeeze_continuous_no_half_stuck`: Kiểm chứng khi $BB_w > KC_w$, Squeeze không bị kẹt ở $0.50$.
    3. `test_trend_strength_strictly_non_negative`: Kiểm chứng khi ADX $= 5.0$, `trend_strength >= 0.0`.
    4. `test_arc_aliasing_no_deep_clones`: Kiểm chứng địa chỉ bộ nhớ pointer equality của các trường trùng lặp.
    5. `test_infinite_and_nan_sanitization`: Kiểm chứng khả năng phòng thủ số vô cực và NaN.
- **Tiêu chuẩn nghiệm thu:**
  - `cargo test --package helios matrix` vượt qua 100% các bài test mới.

---

### 🟢 TASK-MAT-10: Tái Cấu Trúc `chaos_index` Bằng Kaufman ER (`entropy`) (Đã hoàn thành)
- **Mã lỗi liên đới:** `FLAW-MAT-10`, `VULN-ARB-CHAOS` (Critical).
- **Hiện trạng:**
  - Công thức cũ tại `core_v3/matrix.rs:122`: `0.60 * c_r + 0.40 * z_abs` đánh đồng độ lớn biến động và độ lệch chuẩn với bão Whipsaw.
  - Khi có Super Trend bùng nổ, `chaos_index = 1.0`, bóp chết điểm tự tin của các Scanner theo xu hướng.
- **Hành động đã thực hiện:**
  1. Tái cấu trúc công thức:
     $$\text{chaos\_index} = c\_r \times (1.0 - \text{entropy})$$
  2. Khử hoàn toàn hằng số ma thuật (như 0.5, 0.6, 0.4): Nếu trong giai đoạn Warmup (`is_nan()`), trả về $0.0$.
- **Kết quả nghiệm thu:**
  - `test_super_trend_low_chaos`: Đạt chuẩn (khi entropy $\ge 0.95$, chaos_index $\le 0.03$, không bị phạt oan).
  - `test_whipsaw_discrimination_chaos_index`: Đạt chuẩn (khi bão Whipsaw bùng nổ, chaos_index $> 0.50$ và duy trì $> 0.35$).
  - 65/65 tests trong `helios` passed. `bot_engine` check sạch sẽ.

---

### 🔮 TASK-MAT-11: Nghiên cứu Kiến trúc Mô hình C (Regime-Playbook Affinity Matrix) (Định hướng tương lai)
- **Bản chất kiến trúc:**
  - Qua phân tích phản biện First-Principles (ngày 23/09/2026), xác định ranh giới: **Arbiter là sàn đấu thầu phân xử khách quan (Auctioneer), KHÔNG được ôm đồm logic bối cảnh vĩ mô để trở thành hộp đen Meta-Controller.**
  - Việc đánh giá tương thích giữa Playbook và Chế độ thị trường thuộc về **Tầng Nhận Thức Thị Trường (Market Matrix Layer)** hoặc nội tại từng Scanner, không thuộc về Arbiter.
- **Đặc tả Mô hình C (Regime-Playbook Affinity Matrix):**
  - Ma trận $4 \times 4$ ánh xạ giữa Véc-tơ Trạng Thái Thị Trường 4 Chiều (4-State Simplex $\vec{R} = [w_{trend}, w_{range}, w_{compress}, w_{chaos}]$) và từng Playbook:
    $$\text{Confidence}_i = \text{raw\_score}_i \times \text{Affinity}(\text{Playbook}_i, \vec{R})$$
    + $\text{TrendPB, Momentum} \implies \text{Affinity cao với } w_{trend}$
    + $\text{Absorption} \implies \text{Affinity cao với } w_{compress}$
    + $\text{RangeOsc} \implies \text{Affinity cao với } w_{range}$
    + $\text{Reversal} \implies \text{Affinity cao với } w_{chaos} \land \text{kiệt sức}$
- **Trạng thái:** 💡 Đã ghi nhận kiến trúc vào tài liệu thiết kế. Tạm thời hoãn triển khai để giữ Arbiter tinh gọn, tập trung xử lý Điểm Yếu 3 & 4 của Arbiter theo chuẩn Single-Gate.

