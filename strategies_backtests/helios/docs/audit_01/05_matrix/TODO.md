# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (ACTIONABLE TODO): MARKET MATRIX CORE V3
**Module phụ trách:** [`matrix.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs)  
**Tài liệu kiểm toán gốc:** [technical_audit.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_matrix/technical_audit.md)  
**Ngày lập kế hoạch:** 19/09/2026  
**Chuyên gia kiểm toán:** Quantitative & Systems Auditor  

---

## 🎯 BẢNG TỔNG QUAN ƯU TIÊN THỰC THI

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-MAT-01` | Sửa lỗi nghịch đảo độ nhạy `vol_z_sensitivity` trong `flow_axis` | **CRITICAL** | 🔴 Chưa xử lý | `core_v3/matrix.rs:105–114` |
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

### 🔴 TASK-MAT-01: Sửa lỗi nghịch đảo độ nhạy `vol_z_sensitivity` trong `flow_axis`
- **Mã lỗi liên đới:** `FLAW-MAT-01` (Critical).
- **Hiện trạng:**
  - Tại `core_v3/matrix.rs:106`, mã nguồn gọi:
    ```rust
    let vol_z_scale = normalization::tanh_scale(&sensors.vol_z_score, 1.0 / vol_z_sens);
    ```
  - Do hàm `tanh_scale` chia dữ liệu cho tham số độ nhạy (`v / sensitivity`), việc truyền `1.0 / vol_z_sens` khiến công thức trở thành $\tanh(v \times \text{vol\_z\_sens})$.
  - Hậu quả: Đảo ngược $180^\circ$ ý đồ quant. Khi tăng `vol_z_sensitivity` để giảm độ nhạy thì hệ thống lại biến thành siêu nhạy, bão hòa kịch trần $\approx 1.0$ ngay cả với biến động khối lượng vi mô.
  - Ngoài ra, mã nguồn cộng gộp cảm tính $50\%$ MFI vào `flow_axis`, vi phạm triệt để nguyên tắc trực giao độc lập của bản Đặc tả Kỹ thuật.
- **Hành động cần làm:**
  1. Đổi tham số truyền vào: `tanh_scale(&sensors.vol_z_score, vol_z_sens)`.
  2. Bỏ việc cộng gộp MFI vào `flow_axis`:
     $$\text{flow\_axis}[i] = \tanh\left(\frac{|\text{vol\_z\_score}[i]|}{\text{vol\_z\_sens}}\right)$$
  3. Để MFI độc lập trong `sensors.mfi` cho các Scanner cần dòng tiền tự truy vấn.
- **Tiêu chuẩn nghiệm thu (Acceptance Criteria):**
  - Viết unit test xác nhận khi `vol_z_score = 1.5` và `vol_z_sens = 1.5`, giá trị $\tanh(1.5 / 1.5) = \tanh(1.0) \approx 0.76159$.
  - Khi `vol_z_sens` tăng lên $3.0$, giá trị giảm xuống $\tanh(1.5 / 3.0) = \tanh(0.5) \approx 0.46211$ (đúng chiều co giãn).

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

### 🟡 TASK-MAT-07: Giữ lại `entropy` cho `chaos_index`, xóa bỏ chỉ báo ma (`liquidity`, `chaos`) và gen ma `volume_lookback`
- **Mã lỗi liên đới:** `FLAW-MAT-07` (Medium).
- **Hiện trạng:**
  - `matrix.liquidity`, `matrix.chaos` được tính toán bằng các thuật toán Fenwick Tree tốn kém nhưng không có module nào trong `core_v3` sử dụng.
  - `matrix.entropy` (Kaufman Efficiency Ratio) trước đây bị bỏ xó tại dòng 122, nhưng nay là **chỉ số sống còn để phân biệt sóng định hướng với bão Whipsaw** trong `TASK-MAT-10`.
  - `MatrixConfig.volume_lookback` có gen DNA chiếm 31 nấc tìm kiếm nhưng mã nguồn `matrix.rs` không hề đọc giá trị này.
- **Hành động cần làm:**
  1. Giữ lại phép tính `entropy` trong `from_sensors` để phục vụ `TASK-MAT-10`.
  2. Xóa bỏ hoặc vô hiệu hóa các dòng tính toán `liquidity`, `chaos` không sử dụng.
  3. Xóa macro `#[dna(...)]` trên trường `volume_lookback` trong `core_v3/config.rs`.
- **Tiêu chuẩn nghiệm thu:**
  - Tổng số gen DNA của hệ thống giảm thêm 1 gen ma.
  - Phép tính `entropy` được tái sử dụng thành công trong `chaos_index`.

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

