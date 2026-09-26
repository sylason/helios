# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO): ABSORPTION SCANNER

> **Mô-đun**: `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`  
> **Dự án**: Helios Quantitative Trading Strategy V3.3  
> **Tài liệu tham chiếu**: [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_absorption/technical_audit.md)  
> **Mục tiêu**: Loại bỏ 100% bẫy bắt đỉnh/đáy ngược phân phối/tích lũy, triệt tiêu Hard Drawdown -26.6%, chuẩn hóa thứ nguyên DNA và cân bằng trọng tài Arbiter.

---

## 📋 BẢNG TỔNG HỢP NHIỆM VỤ

| Mã Nhiệm Vụ | Mức Độ Ưu Tiên | Mô Tả Tóm Tắt | Tệp Tin Ảnh Hưởng | Trạng Thái |
|---|---|---|---|---|
| **TASK-ABS-01** | 🔴 **CRITICAL** | Tái cấu trúc logic xác định hướng theo Wyckoff (%B + CLV Spring/Upthrust) | `absorption.rs` (L69–75) | ✅ **ĐÃ HOÀN THÀNH** |
| **TASK-ABS-02** | 🔴 **CRITICAL** | Đồng bộ hóa thứ nguyên & Mô hình Nỗ lực vs Kết quả (Effort vs Result) | `absorption.rs` (L54, 58), `config.rs` (L547) | ✅ **ĐÃ HOÀN THÀNH** |
| **TASK-ABS-03** | 🔴 **CRITICAL** | Vá lỗ hổng Arbiter: Cấm lệnh SL chặt cướp quyền khi Confidence thấp | `arbiter.rs` (L174–182) | ⏳ CHỜ XỬ LÝ (Module Arbiter) |
| **TASK-ABS-04** | 🟡 **HIGH** | Thắt chặt dải DNA `struct_sl_buffer_atr` [0.10, 2.50] | `config.rs` (L554–557) | ✅ **ĐÃ HOÀN THÀNH** |
| **TASK-ABS-05** | 🟡 **HIGH** | Thắt chặt ngưỡng dốc KAMA `max_adverse_slope` [0.02, 0.25] | `config.rs` (L559–564), `absorption.rs` (L81–94) | ✅ **ĐÃ HOÀN THÀNH** |
| **TASK-ABS-06** | 🟡 **HIGH** | Đồng bộ hóa chu kỳ VWMA & SMA nội tại (Internalized Ratio) | `sensors.rs`, `core_v3/mod.rs` (L215) | ⏳ CHỜ XỬ LÝ |
| **TASK-ABS-07** | 🟠 **MEDIUM** | Tách cơ chế Break-Even và Time-Decay Trailing riêng cho Absorption | `risk.rs` (L301–325, L423–447) | ⏳ CHỜ XỬ LÝ (Module Risk) |
| **TASK-ABS-08** | 🟠 **MEDIUM** | Dọn dẹp gen chết `max_initial_sl_pct` hoặc tích hợp nội tại | `config.rs` (L538–540), `absorption.rs` | ⏳ CHỜ XỬ LÝ |
| **TASK-ABS-09** | 🟢 **LOW** | Bổ sung Unit Tests kiểm thử phân định Tích lũy vs Phân phối & DNA Width = 97 | `core_v3/mod.rs` (tests) | ✅ **ĐÃ HOÀN THÀNH** |
| **TASK-ABS-10** | 🔴 **CRITICAL** | Triệt tiêu bẫy đệm kép SL (`ABS-SL-DOUBLE-PADDING`), đưa SL về neo trực tiếp `low - buffer` | `absorption.rs` (L122–133) | ⏳ CHUYỂN GIAO SANG PHÂN HỆ `06_risk` |

---

## 🛠️ CHI TIẾT CÁC NHIỆM VỤ CẦN THỰC HIỆN

---

### TASK-ABS-01: Tái Cấu Trúc Logic Xác Định Hướng Lệnh — Mô Hình Dynamic Quant (`%B` + `CLV` Spring/Upthrust)
- **Mức độ**: 🔴 **CRITICAL (Cốt Tử)**
- **Vấn đề**: Code tại dòng 69–75 chỉ dùng `vwma >= bb_middle && close >= bb_middle` để vào Long. Điều này dẫn tới việc mua ngay đỉnh phân phối của cá mập khi thị trường ở nửa trên dải nén và bán tháo ngay đáy tích lũy.
- **Tệp tin**: [`absorption.rs:L64-76`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L64-L76)
- **Giải pháp kỹ thuật (Phương án B — Dynamic Quant `%B` + `CLV`)**:
  1. **Tính toán 2 đại lượng phi thứ nguyên (Dimensionless Metrics)** kèm bảo vệ chia cho 0:
     ```rust
     let bb_width = (bb_upper - bb_lower).max(1e-5);
     let pct_b = (close - bb_lower) / bb_width;

     let candle_range = (high - low).max(1e-5);
     let clv = (2.0 * close - high - low) / candle_range;
     ```
  2. **Các ngưỡng Dynamic từ cấu hình DNA (`abs_cfg`)**:
     - `spring_tolerance`: Dung sai râu quét thanh khoản ngoài dải (ví dụ `0.10`).
     - `min_clv`: Ngưỡng lực nến rút chân/đè đầu tối thiểu (ví dụ `0.20`).
  3. **Điều kiện Long (Tích lũy Gom hàng — Wyckoff Spring)**:
     - Vị trí: `pct_b >= -spring_tolerance && pct_b <= 0.45` (Nửa dưới dải nén, cho phép nhúng râu quét thanh khoản).
     - Lực đỡ: `clv >= min_clv` (Nến rút chân chứng minh Smart Money mua hấp thụ).
     - Dòng tiền: `vwma < bb_middle` (VWMA nằm ở vùng giá thấp).
  4. **Điều kiện Short (Phân phối Xả hàng — Wyckoff Upthrust)**:
     - Vị trí: `pct_b >= 0.55 && pct_b <= (1.0 + spring_tolerance)` (Nửa trên dải nén, cho phép quét đỉnh).
     - Lực xả: `clv <= -min_clv` (Nến bị đè đầu chứng minh Smart Money xả hàng).
     - Dòng tiền: `vwma > bb_middle` (VWMA nằm ở vùng giá cao).
  5. **Hóa giải 4 Edge Cases chí mạng**:
     - *Edge Case 1 (Mẫu số = 0)*: Kẹp `.max(1e-5)` cho cả `candle_range` và `bb_width`.
     - *Edge Case 2 (Doji 2 đầu `CLV ≈ 0`)*: Tự động bỏ qua vì không vượt qua `min_clv`.
     - *Edge Case 3 (Siêu nén Super Squeeze)*: Điểm `f_sqz` và `f_vol` bảo vệ, không đủ điểm sẽ không kích hoạt.
     - *Edge Case 4 (Hấp thụ thất bại)*: Bảo vệ bởi Structural SL `low.min(bb_lower) - buffer`.
- **Definition of Done (DoD)**:
  - 100% không vào lệnh Long trên các cây nến cắm đầu hoặc khi giá ở nửa trên dải nén.
  - Phân biệt rõ ràng giữa Spring (quét đáy rút chân) và Upthrust (quét đỉnh đè đầu).
  - Khắc phục hoàn toàn lỗi chia cho 0 và NaN bypass.

---

### TASK-ABS-02: Đồng Bộ Hóa Thứ Nguyên & Mô Hình Nỗ Lực vs Kết Quả (Effort vs Result — Volume/Spread Ratio)
- **Mức độ**: 🔴 **CRITICAL (Cốt Tử)**
- **Vấn đề**: `vol_z_threshold` mang dải `[0.20, 5.00]` nhưng lại chia cho `matrix.volume_force` (vốn đã bị nén trong `[0.0, 1.0]`). Đồng thời `matrix.rs` dùng `z_val.abs()` biến volume cạn kiệt thành volume đột biến, và không phân biệt được nến xả lũ (Marubozu) với nến hấp thụ (Narrow Spread).
- **Tệp tin**:
  - [`absorption.rs:L42, L54, L58`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L42)
  - [`matrix.rs:L105-114`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L105-L114)
- **Giải pháp kỹ thuật (Phương án 2 — Wyckoff Effort vs Result)**:
  1. **Đọc trực tiếp Raw Volume Z-Score** từ `sensors.vol_z_score[i]`.
  2. **Chuẩn hóa biên độ nến (Normalized Spread)** qua ATR kèm kẹp sàn/trần an toàn:
     ```rust
     let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
     let norm_spread = (candle_range / current_atr).clamp(0.20, 2.0);
     ```
  3. **Công thức tính điểm hấp thụ khối lượng ($f_{\text{vol}}$)**:
     ```rust
     let raw_vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
     let f_vol = if raw_vol_z > 0.0 {
         let effort_vs_result = raw_vol_z / norm_spread;
         (effort_vs_result / vol_thresh).tanh()
     } else {
         0.0 // Volume cạn kiệt (Z-score âm) -> Không có lực hấp thụ
     };
     ```
  4. **Hóa giải Edge Cases**:
     - *Edge Case 1 (Mẫu số `norm_spread` $\to 0$)*: Kẹp sàn `0.20` chống chia cho 0 hoặc khuếch đại vô hạn.
     - *Edge Case 2 (Nến xả lũ Marubozu `norm_spread > 2.0`)*: Mẫu số lớn tự động dập tắt điểm $f_{\text{vol}}$, loại bỏ hoàn toàn bẫy bắt dao rơi.
- **Definition of Done (DoD)**:
  - Khắc phục 100% mâu thuẫn thứ nguyên toán học giữa tử số và mẫu số.
  - Triệt tiêu hoàn toàn hiện tượng kích hoạt lệnh khi volume cạn kiệt hoặc khi nến xả lũ dài ngoằng.

---

### TASK-ABS-03: Vá Lỗ Hổng Arbiter Cướp Quyền Lệnh Khi Confidence Thấp
- **Mức độ**: 🔴 **CRITICAL (Cốt Tử)**
- **Vấn đề**: Trong `arbiter.rs`, nếu hai scanner cùng đề xuất một chiều (ví dụ Long), scanner nào có SL gần giá hiện tại hơn (`dist_new < dist_curr`) sẽ thắng 100%, bất chấp độ tự tin (Confidence) kém xa đối thủ.
- **Tệp tin**: [`arbiter.rs:L173-183`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L173-L183)
- **Giải pháp kỹ thuật**:
  - Bổ sung biên an toàn tự tin tối thiểu: Scanner có khoảng cách SL ngắn hơn chỉ được chiến thắng nếu điểm tự tin của nó không thua kém đối thủ quá $0.10$:
    ```rust
    if proposal.signal_type == current_prop.signal_type {
        let dist_new = proposal.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);
        let dist_curr = current_prop.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);

        // Chỉ ưu tiên SL gần hơn nếu độ tự tin không bị bỏ quá xa
        if dist_new < dist_curr && conf >= (*current_conf - 0.10) {
            best = Some((proposal, conf));
        } else if (dist_new - dist_curr).abs() < 1e-6 && conf > *current_conf {
            best = Some((proposal, conf));
        }
    }
    ```
- **Definition of Done (DoD)**:
  - `TrendPB` (confidence 0.90) không còn bị `Absorption` (confidence 0.22) cướp lệnh trong xu hướng mạnh.

---

### TASK-ABS-04: Thắt Chặt Dải DNA Của `struct_sl_buffer_atr`
- **Mức độ**: 🟡 **HIGH (Quan Trọng)**
- **Vấn đề**: Dải gen DNA của `struct_sl_buffer_atr` hiện tại từ `0.25` đến `10.00` ATR. Mức 10 ATR cộng thêm biên dưới Bollinger Bands tạo khoảng cách SL lên tới 12 ATR ($\approx 6-8\%$ giá), gây vỡ trận tài khoản khi dính Hard SL.
- **Tệp tin**: [`config.rs:L554-557`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L554-L557)
- **Giải pháp kỹ thuật**:
  - Giảm dải DNA xuống mức an toàn cho chiến thuật bắt breakout vùng nén:
    ```rust
    /// Đệm ATR dưới vùng nén hấp thụ thanh khoản (0.10 -> 2.50 ATR, Step 0.10 -> 25 nấc)
    #[dna(min = 0.10, max = 2.50, step = 0.10)]
    #[serde(default = "default_absorption_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,
    ```
  - Cập nhật hàm mặc định `default_absorption_sl_buffer_atr() -> f64 { 0.80 }` (thay vì 2.0).
- **Definition of Done (DoD)**:
  - Mức rủi ro tối đa cho mỗi lệnh Absorption không vượt quá 3.5 ATR kể từ giá vào lệnh.

---

### TASK-ABS-05: Thắt Chặt Ngưỡng Dốc KAMA `max_adverse_slope`
- **Mức độ**: 🟡 **HIGH (Quan Trọng)**
- **Vấn đề**: Ngưỡng trần `0.50` cho phép lệnh Long đi ngược dòng một con thác giảm mạnh có độ dốc KAMA $-0.30$.
- **Tệp tin**: [`config.rs:L559-564`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L559-L564)
- **Giải pháp kỹ thuật**:
  - Thu hẹp dải DNA từ `[0.05, 0.50]` về **`[0.02, 0.25]` (bước nhảy `0.01`)**.
  - Mặc định: `0.10`.
  - Nếu độ dốc KAMA chuẩn hóa lệch quá $0.10$ ATR, cấm tuyệt đối mở lệnh ngược hướng.
- **Definition of Done (DoD)**:
  - Triệt tiêu hoàn toàn lệnh SHORT tại đỉnh sóng tăng (như nến 35672 trong log `v3_9`).

---

### TASK-ABS-06: Đồng Bộ Hóa Chu Kỳ VWMA & SMA Nội Tại
- **Mức độ**: 🟡 **HIGH (Quan Trọng)**
- **Vấn đề**: `vwma` và `bb_middle` hiện chạy trên 2 timeframe và lookback độc lập do DNA tự do chọn, dẫn đến so sánh khập khiễng lệch pha.
- **Tệp tin**:
  - [`core_v3/mod.rs:L215`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L215)
  - [`absorption.rs:L44-45`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L44-L45)
- **Giải pháp kỹ thuật**:
  - Tính toán đường SMA cùng chu kỳ với VWMA ngay tại nến hoặc sử dụng trực tiếp tỷ số nội tại:
    $$\Delta_{\text{VWMA-SMA}} = \text{VWMA}(N) - \text{SMA}(N)$$ trên cùng 1 Timeframe thống nhất (ưu tiên timeframe của Bollinger Bands).
- **Definition of Done (DoD)**:
  - Phép so sánh dòng tiền loại bỏ 100% hiện tượng lệch pha MTF.

---

### TASK-ABS-07: Tách Cơ Chế Trailing Stop & Break-Even Riêng Cho Absorption
- **Mức độ**: 🟠 **MEDIUM (Trung Bình)**
- **Vấn đề**: `ABSORPTION` hiện đang dùng chung logic Break-Even với `TREND_PB` (yêu cầu lãi $1.0 - 4.5$ ATR nhân thêm $1.5$ lần hệ số nén), khiến lệnh hầu như không bao giờ được kéo về hòa vốn.
- **Tệp tin**: [`risk.rs:L301-325`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L301-L325)
- **Giải pháp kỹ thuật**:
  - Tách nhóm riêng cho `ABSORPTION`: Là chiến thuật bứt phá từ vùng nén, nếu giá chạy được `1.5 ATR`, lập tức dời SL về `Entry + Fee Buffer`.
  - Giảm thời gian chờ siết SL khi underwater (`time_countdown_bars`) dành riêng cho Absorption xuống tối đa 60 nến (thay vì 360 nến).
- **Definition of Done (DoD)**:
  - Lệnh Absorption sau cú bùng nổ thất bại sẽ được cắt lỗ hòa vốn hoặc cắt lỗ ngắn, không bị ngâm vốn 500 nến dẫn đến Hard SL.

---

### TASK-ABS-08: Dọn Dẹp Gen Chết `max_initial_sl_pct`
- **Mức độ**: 🟠 **MEDIUM (Trung Bình)**
- **Vấn đề**: Trường `max_initial_sl_pct` khai báo trong `TacticalAbsorptionParams` nhưng không được sử dụng trực tiếp bên trong `absorption.rs`.
- **Tệp tin**: [`config.rs:L538-540`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L538-L540)
- **Giải pháp kỹ thuật**:
  - Đưa `max_initial_sl_pct` vào trong `absorption.rs` để kẹp trần SL ngay tại khâu tạo `Proposal`, không để lọt SL rộng sang Arbiter và RiskManager.
- **Definition of Done (DoD)**:
  - 100% gen trong `TacticalAbsorptionParams` đều có tác dụng trực tiếp đến quyết định của scanner.

---

### TASK-ABS-09: Xây Dựng Bộ Kiểm Thử Tự Động (Unit Tests) Cho Absorption
- **Mức độ**: 🟢 **LOW (Bảo Trì)**
- **Vấn đề**: Thiếu các unit test giả lập kịch bản phân phối đỉnh và tích lũy đáy để kiểm chứng thuật toán.
- **Tệp tin**: [`absorption.rs:tests`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs)
- **Nội dung kiểm thử cần viết**:
  1. `test_accumulation_bottom_spring_triggers_long`: Giả lập nến rút chân pinbar tại đáy Bollinger Bands với volume lớn $\implies$ Phải ra tín hiệu LONG.
  2. `test_distribution_top_upthrust_triggers_short`: Giả lập nến quét đỉnh đè đầu tại đỉnh Bollinger Bands với volume lớn $\implies$ Phải ra tín hiệu SHORT.
  3. `test_runaway_dump_veto`: Giả lập KAMA slope âm sâu $\implies$ VETO tuyệt đối không được Long.
  4. `test_flat_market_zero_volume_no_signal`: Giả lập Volume Z-score âm $\implies$ Điểm score phải bằng 0.
- **Definition of Done (DoD)**:
  - `cargo test --package helios --lib strategy::core_v3::scanners::absorption` chạy vượt qua 100% (Đã pass 4/4 tests).

---

## 🔮 CÁC ĐIỂM MÙ ĐỊNH LƯỢNG MỚI PHÁT HIỆN TỪ FIRST-PRINCIPLES
 
- [ ] **TASK-ABS-10: Đo đạc thời gian nén biến động (Squeeze Duration / Age Filter) [TẠM HOÃN / POSTPONED]**
  - **Trạng thái**: ⏸️ **TẠM HOÃN XEM XÉT SAU** (Tránh Over-engineering; bản thân Bollinger Bands đã tích hợp độ trễ 20 nến).
  - **Vấn đề**: Scanner chỉ nhìn điểm nén tại nến hiện tại `squeeze_intensity[i]`.
  - **Kế hoạch**: Sẽ quay lại đánh giá sau khi đã hoàn thành các mô-đun và sửa hết các lỗi cốt lõi khác.

- [ ] **TASK-ABS-11: Kiểm tra mẫu hình cạn kiệt khối lượng trước đó (Prior Volume Dry-Up) [TẠM HOÃN / POSTPONED]**
  - **Trạng thái**: ⏸️ **TẠM HOÃN XEM XÉT SAU** (Tránh Over-engineering; Z-score volume đã tự động phản ánh độ biến thiên so với nền trước đó).
  - **Vấn đề**: Chỉ nhìn volume nến hiện tại.
  - **Kế hoạch**: Sẽ quay lại đánh giá sau khi hoàn thành các mô-đun khác.

- [x] **TASK-ABS-12: Khử bỏ toàn bộ các hằng số hardcoded trong `absorption.rs` [HOÀN THÀNH 100%]**
  - **Mức độ**: 🟡 **HIGH (Quan Trọng)**
  - **Kết quả giải quyết**:
    1. Bổ sung 2 tham số DNA độc lập: `max_boundary_pct_b` $[0.15, 0.48]$ và `squeeze_weight` $[0.20, 0.80]$.
    2. Nâng `DNA_WIDTH` từ 97 lên **99 chiều**.
    3. Thay thế hardcode `%B <= 0.45` và `%B >= 0.55` bằng `max_boundary_pct_b` độc lập với `spring_tolerance`.
    4. Thay thế hardcode tỷ trọng 50/50 bằng tổ hợp lồi: `w_sqz * f_sqz + (1.0 - w_sqz) * f_vol`.
    5. Khử bỏ trần spread 2.0, chỉ giữ Epsilon sàn toán học 0.10 chống chia cho 0.
    6. Khử bỏ hardcode sàn SL 0.002, bàn giao toàn quyền kẹp sàn/trần cho `RiskManager::clamp_sl_price`.
    7. 49/49 unit tests passed 100%.

- [ ] **TASK-ABS-13: Thiết lập mục tiêu chốt lời tự nhiên (Natural Target TP) [CHUYỂN SANG RISK MANAGEMENT]**
  - **Trạng thái**: 🔄 **CHUYỂN SANG `risk.rs`** (Theo đúng ranh giới phân tách trách nhiệm - Separation of Concerns: Scanner chỉ phát hiện cơ hội & SL cấu trúc, Risk Manager độc quyền quản lý Take Profit và Trailing Stop).
  - **Kế hoạch**: Sẽ xử lý trong mô-đun kiểm toán Risk Management (`06_risk`).

- [ ] **TASK-ABS-14: Sửa SL Double Padding gây R:R Âm (CRITICAL — Phát hiện từ WFSS Log 2026-09-22)**
  - **Mức độ**: 🔴 **CRITICAL**
  - **Bằng chứng:** Log `WFSS_arbiter_weakpoint2.log` — ROE thua trung bình **-26.1%** vs ROE thắng **+14.6%** → R:R ≈ 1:0.43.
  - **Vấn đề (`absorption.rs:122-133`):**
    ```rust
    let base_support = low.min(bb_lower); // Tầng 1: neo tận biên dưới BB
    let sl = base_support - buffer;        // Tầng 2: lại trừ thêm buffer ATR (tới 2.5 ATR)!
    ```
    Kết quả: SL = `min(low, bb_lower) - 2.5 ATR` → cách entry **4–5 ATR**. Kết hợp `max_initial_sl_pct` trần 40% ROE trên 20x đòn bẩy → lỗ tối đa 1 lệnh lên đến 40% vốn ký quỹ.
  - **Giải pháp (chọn 1 trong 2):**
    - **Phương án A (đơn giản):** Bỏ `bb_lower`, chỉ neo theo `low - buffer` (như `TrendPB`). SL = `low - (struct_sl_buffer_atr * ATR)`.
    - **Phương án B (giữ nguyên neo BB nhưng không cộng đệm):** `sl = bb_lower - (0.1 * ATR)` (buffer tối thiểu chống quét sát biên).
  - **Sửa thêm:** Giảm trần DNA `max_initial_sl_pct` từ `[0.04, 0.40]` xuống `[0.04, 0.20]` (tương đương 20% ROE tối đa trên 20x = 1% giá).
  - **Definition of Done:**
    - ROE thua trung bình của lệnh Absorption < 20% trên backtest ETH 30 ngày.
    - Profit Factor của Absorption > 1.0.

- [ ] **TASK-ABS-15: Siết Chặt Cận Dưới DNA Absorption Chống Playbook Bias (HIGH — Phát hiện từ WFSS Log 2026-09-22)**
  - **Mức độ**: 🟡 **HIGH**
  - **Bằng chứng:** Log `WFSS_arbiter_weakpoint2.log` — ABSORPTION chiếm **90% lệnh** (9/10), 5 playbook còn lại 0 lệnh.
  - **Nguyên nhân gốc rễ (xác nhận bởi đọc `config.rs`):**
    - `min_clv: 0.00 → 0.60` (DNA min = 0.00 → không cần CLV nến hấp thụ)
    - `vol_z_threshold: 0.20 → 5.00` (DNA min = 0.20 → gần như volume bình thường cũng đủ)
    - `min_activation_score: 0.01 → 0.99` (DNA min = 0.01 → GA tối ưu về cận dưới → spam tín hiệu)
    - `max_boundary_pct_b: 0.15 → 0.48` (DNA max = 0.48 → cho phép nến ở nửa trên dải BB vẫn kích hoạt)
    GA kẹt cực trị cục bộ do Smart Seeding → tiến hóa tăng cường ABSORPTION thêm thay vì đa dạng.
  - **Giải pháp — Siết cận dưới:**
    ```toml
    min_activation_score: [0.20 → 0.90]  # Thay vì [0.01 → 0.99]
    min_clv:              [0.15 → 0.60]  # Thay vì [0.00 → 0.60]
    vol_z_threshold:      [1.00 → 5.00]  # Thay vì [0.20 → 5.00]
    ```
  - **Definition of Done:**
    - Trên backtest ETH 30 ngày, không có playbook nào chiếm > 60% lệnh.
    - Ít nhất 3 playbook có ≥ 1 lệnh trong paper trading 7 ngày.

---
*Kế hoạch hành động này là tài liệu chuẩn hướng dẫn lập trình viên sửa đổi mã nguồn Helios V3.*
