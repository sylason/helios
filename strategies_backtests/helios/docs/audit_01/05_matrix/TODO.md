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
| **2** | `TASK-MAT-02` | Tái cấu trúc công thức Squeeze Intensity thoát khỏi bẫy 0.50 (Xóa clone `vol_axis`) | **CRITICAL** | 🟢 Đã xử lý | `core_v3/matrix.rs:58–67`, `risk.rs:320` |
| **3** | `TASK-MAT-03` | Đồng bộ hóa dải giá trị `trend_strength` về $[0.0, 1.0]$ và khử số âm | **HIGH** | 🟢 Đã xử lý | `core_v3/matrix.rs:75–86` |
| **4** | `TASK-MAT-04` | Triệt tiêu Deep Clone & Xóa biến nhân bản `trend_dir`, chuẩn hóa `kama_slope_atr` | **HIGH** | 🟢 Đã xử lý | `core_v3/matrix.rs`, scanners, `mod.rs` |
| **5** | `TASK-MAT-05` | Khóa cứng đồng bộ Timeframe giữa Bollinger Bands và Keltner Channels | **HIGH** | 🟢 Đã xử lý | `core_v3/config.rs:114–124`, `core_v3/mod.rs:142–195` |
| **6** | `TASK-MAT-06` | Chuẩn hóa phòng thủ NaN an toàn tuyệt đối, triệt tiêu bẫy `unwrap_or` | **MEDIUM** | 🟢 Đã xử lý | `core_v3/matrix.rs:51–73, 108–124` |
| **7** | `TASK-MAT-07` | Giữ lại `entropy`, xóa bỏ chỉ báo ma (`liquidity`, `chaos`) và gen ma `volume_lookback` | **MEDIUM** | 🟡 Đang xử lý (Phần 1: Xong) | `core_v3/matrix.rs`, `core_v3/config.rs:266` |
| **8** | `TASK-MAT-08` | Bổ sung kẹp biên an toàn cho `bb_width_norm`, `bb_middle_slope_adj` | **LOW** | 🟢 Đã xử lý | `core_v3/matrix.rs:58, 139`, `core_v3/mod.rs:257` |
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

### 🟢 TASK-MAT-02: Tái cấu trúc công thức Squeeze Intensity thoát khỏi bẫy 0.50 (Inverse Quadratic Soft Squeeze & Xóa clone `vol_axis`) - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-02` (Critical).
- **Kết quả thực hiện:**
  - [x] Đã tái cấu trúc công thức đo Squeeze thành hàm phân số bậc 2 liên tục trơn tru:
    $$\text{sqz\_raw} = \frac{1.0}{1.0 + (BB_w / KC_w)^2}$$
    + Khi $BB_w \ll KC_w$ (nén cực đại): $\text{sqz\_raw} \to 1.0$.
    + Khi $BB_w = KC_w$ (ngưỡng Carter kinh điển): $\text{sqz\_raw} = 0.50$.
    + Khi $BB_w \gg KC_w$ (bung nén/bão): $\text{sqz\_raw} \to 0.0$ (không bị ép về $0.0$ phẳng lỳ, bảo toàn độ dốc biến thiên liên tục).
  - [x] Triệt tiêu vĩnh viễn bẫy kẹt $0.50$ của `rolling_percentile`: khi biến động mở rộng, `squeeze_intensity` rơi chuẩn xác về vùng đáy $[0.01 \dots 0.15]$.
  - [x] Tiện tay xóa sổ hoàn toàn biến clone dư thừa `vol_axis` khỏi struct `MarketMatrixV3`, loại bỏ 1 lệnh deep clone vector trên heap.
  - [x] Chuyển `risk.rs:320` sang đọc trực tiếp `matrix.squeeze_intensity`:
    `let sqz_multiplier = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0).max(0.0);`
    Giải phóng điểm nghẽn Break-Even: Ngưỡng kích hoạt BE không còn bị thổi phồng ảo thêm $50\%$ trong bão, giúp bot chốt giữ hòa vốn kịp thời khi có lợi nhuận.
  - [x] Bổ sung unit test `test_squeeze_continuous_no_half_stuck` kiểm chứng bung nén mạnh $BB_w = 3.1 \times KC_w$ thì `squeeze_intensity` rơi về $< 0.20$ (PASS).
  - [x] Cập nhật test `test_orthogonal_matrix_nan_resilience` đổi sang kiểm tra `squeeze_intensity` (PASS).
  - [x] Kiểm thử tự động: 100% tests `helios` (66/66) và `bot_engine` (39 passed, 1 ignored) vượt qua.

---

### 🟢 TASK-MAT-03: Đồng bộ hóa dải giá trị `trend_strength` về $[0.0, 1.0]$ và khử số âm - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-03` (High).
- **Kết quả thực hiện:**
  - [x] Đã tái cấu trúc công thức tính `trend_strength` sang chuẩn hóa tuyến tính thuần khiết:
    $$\text{trend\_strength} = \left(\frac{\text{ADX}}{100.0}\right).\text{clamp}(0.0, 1.0)$$
    + Triệt tiêu hoàn toàn số âm $[-1.0, 0.0]$, đưa dải đo về đúng tính chất đại lượng vô hướng $[0.0, 1.0]$ ($\text{ADX} = 25.0 \implies 0.25$).
    + Xóa bỏ hoàn toàn vector cấp phát đệm thừa `adx_centered: Vec<f64>` và phép trừ `adx - 20.0` trong vòng lặp.
    + Chấm dứt ô nhiễm dữ liệu chéo: không còn mượn `z_score_sensitivity` để chia cho ADX.
  - [x] Thống nhất sau phiên `/grill-me`: Giữ nguyên 100% `config.rs` (không can thiệp vào dải gene của `RangeOscillator` vì scanner này đang ngủ đông).
  - [x] Bổ sung unit test `test_trend_strength_linear_bounds` kiểm chứng 100% các mốc ADX từ NaN, số âm, $15.0 \to 0.15$ đến $120.0 \to 1.0$ (kẹp trần) (PASS).
  - [x] Kiểm thử tự động: 100% tests `helios` (67/67) và `bot_engine` (39 passed, 1 ignored) vượt qua.

---

### 🟢 TASK-MAT-04: Triệt Tiêu Deep Clone & Xóa Sổ Biến Nhân Bản `trend_dir` (Clean Code Triệt Để) - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-04` (High).
- **Kết quả thực hiện:**
  - [x] Đã xóa bỏ hoàn toàn trường `pub trend_dir: Arc<Vec<f64>>` khỏi struct `MarketMatrixV3`, chấm dứt tình trạng 2 biến song sinh chứa cùng 1 nội dung.
  - [x] Đã loại bỏ hoàn toàn lệnh `let trend_dir = kama_slope_atr.clone();` và cấp phát `Arc::new(trend_dir)` trong `MarketMatrixV3::from_sensors`.
  - [x] Chuyển đổi 100% các Scanner sang đọc trực tiếp trường duy nhất chuẩn xác `matrix.kama_slope_atr`:
    + `absorption.rs`: Đổi sang `matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0)`.
    + `momentum_ignition.rs`: Đổi sang `matrix.kama_slope_atr.get(i).copied()`.
    + `range_osc.rs`: Đổi sang `matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0)` kèm phòng vệ bounds checking.
  - [x] Cập nhật toàn bộ 14 mock unit tests trong `core_v3/mod.rs` từ `matrix.trend_dir` sang `matrix.kama_slope_atr`.
  - [x] Kiểm thử tự động:
    + `helios`: 67/67 unit tests PASS ($100\%$).
    + `bot_engine`: 39 passed, 1 ignored ($100\%$).
    + `walk_forward_smart_seed`: Cargo check PASS (Exit code 0).
- **Hiện trạng ban đầu:**
  - Tại `core_v3/matrix.rs:102, 126, 128`, mã nguồn gọi `.clone()` trên các `Vec<f64>`:
    + `let vol_axis = squeeze_intensity.clone();` (Đã xóa ở TASK-MAT-02)
    + `let trend_dir = kama_slope_atr.clone();` (Đã xử lý dứt điểm ở task này)
    + `let volume_force = flow_axis.clone();` (Đã xóa ở TASK-MAT-01)
  - Thay vì giải pháp chắp vá Arc aliasing, việc xóa bỏ hoàn toàn `trend_dir` đã tuân thủ 100% các nguyên tắc Clean Code: Single Source of Truth (SSOT), Intention-Revealing Names và DRY.

---

### 🟢 TASK-MAT-05: Khóa cứng đồng bộ Timeframe giữa Bollinger Bands và Keltner Channels (Serde-Safe) - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-05` (High).
- **Kết quả thực hiện:**
  - [x] Bổ sung hỗ trợ thuộc tính `#[dna(skip)]` vào proc macro `DnaMappableMacro` trong `ferro_trade_derive/src/lib.rs`.
  - [x] Gắn `#[dna(skip)]` và `#[serde(default = "default_timeframe")]` lên trường `timeframe` của `KeltnerParams` trong `core_v3/config.rs:114–124`:
    + Loại bỏ vĩnh viễn gen `indicators.keltner.timeframe` khỏi không gian tìm kiếm GA Optimizer.
    + Thu gọn số chiều DNA từ **103 về đúng 102 chiều**.
    + Đảm bảo tính tương thích ngược Serde JSON 100% (Serde-Safe), không làm crash IPC hay hỏng deserialization khi nạp JSON cũ.
  - [x] Đồng bộ hóa 100% cấu hình và runtime trong `core_v3/mod.rs`:
    + Tại đầu hàm `init()`: `self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;`
    + Tại `update_from_dna()`: `self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;`
    + Khởi tạo Keltner Channel ép buộc sử dụng `ind.bbands.timeframe`:
      ```rust
      let (kc_u, _kc_m, kc_l) = ctx.keltner(
          ind.bbands.timeframe, "KC", 
          ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone()
      );
      ```
  - [x] Cập nhật unit test `test_dna_width_and_parameter_defs_integrity` kiểm chứng `DNA_WIDTH == 102` và `parameter_defs` không còn chứa `keltner.timeframe` (PASS).
  - [x] Bổ sung unit test chuyên sâu `test_keltner_bbands_timeframe_synchronization` kiểm chứng đồng bộ sau khi nạp DNA và kiểm thử 2 kịch bản deserialize JSON cũ/mới (PASS).
  - [x] Kiểm thử toàn diện:
    + `helios`: 68/68 unit tests PASS ($100\%$).
    + `bot_engine`: 39 passed, 0 failed, 1 ignored ($100\%$).
    + `walk_forward_smart_seed`: Cargo check PASS (Exit code 0).
- **Ý nghĩa định lượng:**
  Triệt tiêu hoàn toàn $75\%$ tổ hợp lệch pha khung thời gian ngẫu nhiên (12/16 trường hợp) giữa BB và KC. Bảo toàn nguyên vẹn tính hình học của Carter Squeeze $\text{ratio} = BB_w / KC_w$, giải phóng Playbook `Absorption` và `RiskManager` khỏi các tín hiệu nén ảo.

---

### 🟢 TASK-MAT-06: Chuẩn hóa phòng thủ NaN an toàn tuyệt đối, triệt tiêu bẫy `unwrap_or` - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-06` (Medium).
- **Kết quả thực hiện:**
  - [x] Áp dụng triệt để **Phương Án C (In-situ Defensive Arithmetic & Semantic Neutral Fallback)** trực tiếp trong `MarketMatrixV3::from_sensors` mà không tạo helper rác `sanitize_f64`.
  - [x] Phòng thủ chia cho 0 và lọc NaN an toàn tuyệt đối:
    + `bb_width_norm`: Chỉ tính khi `bm > 1e-9 && bb_w.is_finite()`, ngược lại trả về $0.0$; kẹp trần an toàn `.clamp(0.0, 1.0)`.
    + `sqz_raw`: Kiểm tra mẫu số $KC_w > 10^{-9}$ và tử số $BB_w$ hữu hạn; rơi về phần tử trung tính vật lý $0.0$ khi thiếu biến động hoặc dữ liệu rỗng.
    + `trend_strength`: Kiểm tra `adx_v.is_finite() && adx_v > 0.0`, chuẩn hóa mượt về $[0.0, 1.0]$.
    + `kama_slope_atr`: Kiểm tra `s.is_finite()`, chuẩn hóa và kẹp biên an toàn $[-1.0, 1.0]$.
    + `trend_axis`: Kiểm tra `adx_v.is_finite() && adx_v > 0.0`, chuẩn hóa $[0.0, 1.0]$.
    + `chaos_index`: Kiểm tra cả `c_r_raw` và `ent_raw` hữu hạn, rơi về mức an toàn $0.0$.
  - [x] Unit test kiểm chứng: `test_matrix_safe_math_nan_resilience` và `test_zero_volatility_flat_bars` xác nhận không có bất kỳ giá trị NaN/Inf nào lọt qua ma trận.
  - [x] 100% test cases của `helios` (71/71) và `bot_engine` (39/39) PASS hoàn hảo.
- **Hiện trạng:**
  - Mẫu hình `.copied().unwrap_or(0.0)` không bắt được `f64::NAN` vì giá trị trả về là `Some(f64::NAN)`.
  - Kết quả phụ thuộc vào quirk ngầm của IEEE 754 trong `.max(0.0).min(1.0)`.
- **⚠️ CẢNH BÁO RỦI RO TIỀM TÀNG CỦA `sanitize_f64` TRONG TƯƠNG LAI:**
  1. *Bẫy che giấu bug gốc (Silent Defect Masking):* Nếu chỉ báo bị lỗi logic toán học (như chia cho 0, config period sai, warmup lỗi), `sanitize_f64` sẽ âm thầm nuốt lỗi và thay bằng `fallback`. Hệ thống có thể chạy ngầm trên $100\%$ dữ liệu giả mà không có bất kỳ cảnh báo nào.
  2. *Bẫy sai lệch ngữ cảnh (Contextual Fallback Mismatch):* Áp dụng cẩu thả `fallback = 0.0` cho mọi chỉ báo sẽ gây thảm họa:
     - MFI (dao động 0–100): fallback phải là mức trung tính `50.0` (thay vì `0.0` báo hiệu quá bán ảo).
     - Đường giá KAMA/SMA: fallback phải là `current_close` (thay vì `0.0` làm sập giá kích hoạt SL khẩn cấp).
     - Độ lệch chuẩn $\sigma$ / Độ rộng ATR: fallback phải là $\epsilon = 10^{-5}$ (thay vì `0.0` gây lỗi chia cho 0 ở phép tính sau).
  3. *Điểm mù của GA Optimizer:* Khiến thuật toán di truyền không đào thải được các bộ tham số dị tật sinh ra chuỗi NaN kéo dài.
  4. *Quy tắc phối hợp 2 tầng (Defense-in-Depth):*
     - **Tầng 1 (Entry Gate):** Hard Warmup Gate & Data Sufficiency Check (bắt buộc đủ $\ge 43,200$ nến cho các khung lớn như 4h/12h; từ chối chạy bot nếu thiếu dữ liệu).
     - **Tầng 2 (Runtime Seatbelt):** Safe Math Guards tại chỗ bảo vệ hệ thống khỏi đột tử do các dị tật toán học giữa chừng (chia cho 0 khi nến đơ, căn bậc hai số âm do floating-point, sàn lag).

---

### 🟢 TASK-MAT-07: Đại Phẫu Rác Thừa Matrix: Xóa Bỏ 4 Chỉ Báo Ma, 3 Bản Sao Clone và 3 Gen DNA Ma - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-07` (Medium).
- **Kết quả thực hiện Phần 1 (Cắt bỏ 3 Gen DNA Ma):**
  - [x] Gắn `#[dna(skip)]` và giữ `#[serde(default)]` cho 3 gen: `momentum_lookback`, `volume_lookback`, `adx_trend_threshold` trong `core_v3/config.rs` (Serde-Safe 100%).
  - [x] Thu gọn không gian tìm kiếm GA từ **102 về đúng 99 chiều** (`DNA_WIDTH = 99`).
  - [x] Khóa cứng kiểm tra độ dài `dna.len() != HeliosConfigV3::DNA_WIDTH` trong `core_v3/mod.rs:131`, đồng bộ với chốt chặn an toàn sẵn có của `AssetActor`.
  - [x] Cập nhật unit test `test_dna_width_and_parameter_defs_integrity` xác nhận 3 gen ma đã biến mất khỏi Search Space.
  - [x] 100% tests của `helios` (71/71) và `bot_engine` (39/39) PASS hoàn hảo; `walk_forward_smart_seed` biên dịch sạch.
- **Kết quả thực hiện Phần 2 (Đại phẫu `MarketMatrixV3` & Cắt bỏ 4 Chỉ Báo Ma):**
  - [x] Cắt bỏ 4 chỉ báo ma khỏi struct `MarketMatrixV3`: `liquidity`, `price_z_score`, `chaos`, `trend_axis`.
  - [x] Cắt bỏ 1 bản sao clone thừa `kama_slope_adj` (hệ thống dùng `kama_slope_atr`).
  - [x] Đóng gói 2 biến nội bộ rò rỉ `chaos_ratio` và `entropy` thành biến cục bộ `let` bên trong `from_sensors` chỉ để tính `chaos_index`.
  - [x] Struct `MarketMatrixV3` co gọn từ **13 trường xuống đúng 6 trường vật lý cốt lõi**: `kama_slope_atr`, `chaos_index`, `trend_strength`, `bb_middle_slope_adj`, `squeeze_intensity`, `bb_width_norm`.
  - [x] Đồng bộ chốt chặn lưới biên nến `self.matrix.chaos_index.len() <= i` tại `core_v3/mod.rs:269`.
  - [x] Cập nhật test suite: `test_orthogonal_matrix_nan_resilience`, `test_matrix_independence_no_zero_sum`, `test_super_trend_low_chaos`, `test_matrix_safe_math_nan_resilience` và `test_volume_zscore_differentiation_from_price`.
  - [x] Kiểm thử toàn diện: 100% tests `helios` (71/71) và `bot_engine` (39 passed, 1 ignored) PASS hoàn hảo; `walk_forward_smart_seed` biên dịch exit code 0.
- **BẢNG CHỨNG THỰC TOÀN DIỆN 15 TRƯỜNG DỮ LIỆU TRONG `MarketMatrixV3`:**
  *(Quét trực tiếp 100% mã nguồn trên toàn bộ thư mục `strategies_backtests/helios/src/strategy/core_v3/`)*

  | STT | Trường trong `MarketMatrixV3` | File nào đọc? Dòng nào? | Mục đích nghiệp vụ thực tế | Đánh giá tình trạng |
  | :---: | :--- | :--- | :--- | :--- |
  | **1** | `kama_slope_atr` | `scanners/trend_pb.rs:47` | Đo độ dốc KAMA chuẩn hóa theo ATR để bắt nhịp hồi | 🟢 **ACTIVE CORE** (Đang dùng tích cực) |
  | **2** | `trend_axis` | **KHÔNG CÓ SCANNER NÀO ĐỌC!** *(Chỉ có test nội bộ `matrix.rs:182`)* | Map từ ADX nhưng bị bỏ hoang hoàn toàn | 🧟 **ZOMBIE 100% (Chỉ báo ma)** |
  | **3** | `vol_axis` | `risk.rs:320`: `let sqz_multiplier = matrix.vol_axis...` | Nới rộng ngưỡng kích hoạt hòa vốn (Break-Even) khi có nén | ⚠️ **CÓ DÙNG** (nhưng là clone 100% từ `squeeze_intensity`) |
  | **4** | `chaos_index` | `risk.rs:103`, `arbiter.rs` | Phạt điểm tín hiệu và thắt chặt SL khi gặp bão Whipsaw | 🟢 **ACTIVE CORE** (Đang dùng tích cực) |
  | **5** | `trend_dir` | `scanners/absorption.rs:107`, `momentum_ignition.rs:86`, `range_osc.rs:122` | Đo hướng xu hướng KAMA để Veto cản tàu / bắt dao rơi | 🟢 **ACTIVE CORE** (Clone từ `kama_slope_atr`) |
  | **6** | `trend_strength` | `scanners/range_osc.rs:69` | Chặn RangeOsc khi thị trường có trend mạnh | 🟡 **CÓ DÙNG** (cần sửa số âm ở TASK-MAT-03) |
  | **7** | `kama_slope_adj` | **KHÔNG CÓ SCANNER NÀO ĐỌC!** *(Scanner đọc từ `sensors`)* | Passthrough trung chuyển dữ liệu thừa thãi | 🧟 **PASSTHROUGH THỪA** |
  | **8** | `bb_middle_slope_adj` | `scanners/range_osc.rs:74` | Kiểm tra độ dốc trục giữa BB Middle (SMA 20) | 🟢 **ACTIVE CORE** (Đang dùng trong Range) |
  | **9** | `squeeze_intensity` | `scanners/absorption.rs:48` | Cường độ nén Squeeze cho Absorption | 🟢 **ACTIVE CORE** (Đang dùng tích cực) |
  | **10** | `chaos` | **HOÀN TOÀN KHÔNG CÓ BẤT KỲ FILE NÀO ĐỌC!** | Tính `natr / 100.0` rồi bỏ xó | 🧟 **ZOMBIE 100% (Chỉ báo ma)** |
  | **11** | `chaos_ratio` | `mod.rs:257`: `.len() <= i` | Chỉ dùng để kiểm tra độ dài buffer nến | 🧟 **BIẾN NỘI BỘ RÒ RỈ** (Dùng tính `chaos_index`) |
  | **12** | `entropy` | **KHÔNG CÓ SCANNER NÀO ĐỌC!** | Chỉ dùng nội bộ tính `chaos_index` | 🧟 **BIẾN NỘI BỘ RÒ RỈ** (Dùng tính `chaos_index`) |
  | **13** | `liquidity` | **HOÀN TOÀN KHÔNG CÓ BẤT KỲ FILE NÀO ĐỌC!** | Chạy cây Fenwick `rolling_percentile` trên MFI mỗi nến rồi vứt xó! | 🧟 **ZOMBIE SIÊU LÃNG PHÍ CPU 100%!** |
  | **14** | `bb_width_norm` | `scanners/range_osc.rs:53` | Kiểm tra bề rộng dải Bollinger tối thiểu | 🟢 **ACTIVE CORE** (Đang dùng trong Range) |
  | **15** | `price_z_score` | **KHÔNG CÓ SCANNER NÀO ĐỌC!** | Chạy `tanh_scale` nặng nề nhưng bỏ xó | 🧟 **ZOMBIE 100% (Chỉ báo ma)** |

- **🚨 PHÁT HIỆN ĐẶC BIỆT VỀ `vol_axis` & `risk.rs:320`:**
  - Tại `risk.rs:320`: `let sqz_multiplier = matrix.vol_axis.get(i)...; let min_be_roe = base_be_roe * (1.0 + sqz_multiplier);`
  - Do `vol_axis` clone từ `squeeze_intensity` vốn bị kẹt cứng ở $0.50$ (lỗi `TASK-MAT-02`), ngưỡng kích hoạt Break-Even luôn bị **thổi phồng thêm 50%**, khiến các lệnh thắng không dời được SL về hòa vốn kịp thời và bị quay đầu cắn Hard SL!
- **Hành động cần làm:**
  1. Giữ lại phép tính `entropy` và `chaos_ratio` cục bộ trong `from_sensors` để phục vụ `TASK-MAT-10` (không cần lưu public Arc).
  2. Xóa bỏ hoặc vô hiệu hóa các dòng tính toán 4 chỉ báo ma `liquidity`, `chaos`, `price_z_score`, `trend_axis`.
  3. Đổi `risk.rs:320` sang đọc trực tiếp `matrix.squeeze_intensity`, rồi xóa bản clone thừa `vol_axis`.
  4. Đổi các scanner sang đọc trực tiếp `matrix.kama_slope_atr`, rồi xóa bản clone thừa `trend_dir`.
  5. Xóa macro `#[dna(...)]` trên 3 gen ma `volume_lookback`, `momentum_lookback`, `adx_trend_threshold` trong `core_v3/config.rs` (giúp GA giảm thêm 3 chiều tìm kiếm vô ích).
- **Tiêu chuẩn nghiệm thu:**
  - Tổng số gen DNA của hệ thống giảm thêm 3 gen ma vô tác dụng.
  - Triệt tiêu hoàn toàn các phép tính Cây Fenwick và Tanh chạy vô ích cho các trường rác.
  - Bộ nhớ RAM heap allocations của Matrix giảm tối đa.

---

### 🟢 TASK-MAT-08: Bổ sung kẹp biên an toàn cho `bb_width_norm`, `bb_middle_slope_adj` - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `FLAW-MAT-08` (Low).
- **Kết quả thực hiện:**
  - [x] Kẹp biên `bb_width_norm`: `.clamp(0.0, 1.0)` ngăn chặn bung band vô tận khi xảy ra Flash Crash.
  - [x] Kẹp biên `bb_middle_slope_adj`: `.clamp(-2.0, 2.0)` tại cả 2 tầng:
    + Tầng khởi tạo `core_v3/mod.rs:257`: tính độ dốc SMA 20 trên ATR.
    + Tầng ma trận `core_v3/matrix.rs:136`: chuẩn hóa an toàn kèm kiểm tra `is_finite()`, rơi về $0.0$ nếu thiếu dữ liệu.
  - [x] Unit test kiểm chứng: `test_bb_width_norm_clamped` PASS $100\%$ khi mô phỏng râu nến kéo giãn band 19.8x.
- **Hiện trạng:**
  - `bb_width_norm` và `bb_middle_slope_adj` không có kẹp biên trên, có thể bùng nổ vô tận khi flash crash.
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

