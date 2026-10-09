# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (ACTIONABLE TODO): CẤU HÌNH & KHÔNG GIAN DNA HELIOS V3.3
**Module:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`  
**Tài liệu kiểm toán gốc:** [technical_audit.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/09_core_v3_config/technical_audit.md)  
**Ngày lập kế hoạch:** 19/09/2026  
**Chuyên gia kiểm toán:** Quantitative & Systems Auditor  

---

## 🎯 BẢNG TỔNG QUAN ƯU TIÊN THỰC THI

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-CFG-01` | Loại bỏ Gen Ma `volume_lookback`, `z_score_sensitivity` và dọn sạch `MatrixConfig` (DNA về 102) | **CRITICAL** | 🟢 Đã xử lý | `core_v3/config.rs:274–287`, `core_v3/mod.rs:1325` |
| **2** | `TASK-CFG-02` | Sửa lỗi toán học dải gen `momentum.rsi_threshold` chống đảo lộn Short/Long | **CRITICAL** | 🔴 Chưa xử lý | `core_v3/config.rs:408–409`, `scanners/momentum_ignition.rs:55–60` |
| **3** | `TASK-CFG-03` | Rút ngắn dải gen `absorption.struct_sl_buffer_atr` từ 10.0 ATR về 2.50 ATR | **HIGH** | 🔴 Chưa xử lý | `core_v3/config.rs:554–555`, `core_v3/risk.rs:44–50` |
| **4** | `TASK-CFG-04` | Hiệu chỉnh cận dưới `momentum.vol_z_threshold` & `reversal.exhaustion_vol_drop` | **HIGH** | 🔴 Chưa xử lý | `core_v3/config.rs:412, 499`, `scanners/` |
| **5** | `TASK-CFG-05` | Cấu hình hóa các Hằng số ma thuật (Magic Numbers) trong Arbiter và Risk | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/config.rs:300–335`, `arbiter.rs:115, 128`, `risk.rs:327` |
| **6** | `TASK-CFG-06` | Bổ sung `#[serde(default)]` cấp độ struct chống panic khi deserialize JSON thiếu trường | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/config.rs:253, 361, 401, 485, 537, 635` |
| **7** | `TASK-CFG-07` | Dọn dẹp các trường rác tàn dư (Dead Fields) và xóa struct mồ côi `WarmupConfig` | **LOW** | 🔴 Chưa xử lý | `core_v3/config.rs:310–327, 588–600, 846–850` |
| **8** | `TASK-CFG-08` | Wire logic thực thi cho `mega_trend_max_cycles` trong `risk.rs` | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/config.rs:714`, `core_v3/risk.rs:257–260` |

---

## 📋 CHI TIẾT CÁC ĐẦU VIỆC CẦN THỰC HIỆN

### 🟢 TASK-CFG-01: Loại bỏ Gen Ma `volume_lookback`, `z_score_sensitivity` khỏi `MatrixConfig` và hạ DNA_WIDTH về 102 - [ĐÃ XỬ LÝ ✅]
- **Mã lỗi liên đới:** `VULN-CFG-01` (Critical).
- **Hiện trạng:**
  - Tại [`core_v3/config.rs:266–269`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L266-L269), trường `volume_lookback` có gắn attribute `#[dna(min = 100.0, max = 1000.0, step = 30.0)]`, chiếm 1 chiều gen DNA với 31 nấc tìm kiếm.
  - Tuy nhiên, trong [`core_v3/matrix.rs:76–88`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L76-L88), hàm `MarketMatrixV3::from_sensors` chỉ đọc `trend_lookback`, `squeeze_lookback`, `momentum_lookback`, và `z_score_sensitivity`. Trường `volume_lookback` **hoàn toàn không được đọc ở bất kỳ đâu** trong `core_v3`.
  - Hậu quả: Thuật toán di truyền (GA) phân bổ tài nguyên tìm kiếm vô ích vào một gen ma không có bất kỳ tác động nào đến fitness function.
- **Hành động cần làm:**
  1. Xóa bỏ attribute `#[dna(...)]` trên trường `volume_lookback` trong `MatrixConfig` (hoặc loại bỏ hoàn toàn trường này khỏi struct nếu không cần tương thích backward).
  2. Cập nhật bài test `test_dna_width_and_parameter_defs_integrity` trong [`core_v3/mod.rs:887`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L887): Chuyển `assert_eq!(width, 95)` thành `assert_eq!(width, 94)`.
  3. Bổ sung assert xác minh `defs` không chứa `matrix.volume_lookback`.
- **Tiêu chuẩn nghiệm thu (Acceptance Criteria):**
  - `HeliosConfigV3::DNA_WIDTH` trả về đúng 94.
  - Test suite `cargo test --lib strategy::core_v3` vượt qua $100\%$.

---

### 🔴 TASK-CFG-02: Sửa lỗi toán học dải gen `momentum.rsi_threshold` chống đảo lộn Short/Long
- **Mã lỗi liên đới:** `VULN-CFG-02` (Critical).
- **Hiện trạng:**
  - Tại [`core_v3/config.rs:408–409`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L408-L409):
    ```rust
    #[dna(min = 1.0, max = 99.0, step = 3.0)]
    pub rsi_threshold: f64,
    ```
  - Trong [`momentum_ignition.rs:55, 60`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L55-L60):
    - Long: `f_rsi_long = 1.0 / (1.0 + exp(-0.15 * (rsi_mom - rsi_thresh)))`
    - Short: `f_rsi_short = 1.0 / (1.0 + exp(-0.15 * ((100.0 - rsi_thresh) - rsi_mom)))`
  - Khi GA sinh ra $\text{rsi\_threshold} \le 40.0$ (ví dụ 25.0):
    - Khi thị trường giảm mạnh ($\text{RSI} = 35.0$), hiệu số $35 - 25 = +10 \implies f_{\text{rsi\_long}} > 0.80$, kích hoạt **Long bùng nổ xung lực ngay trong xu hướng downtrend sập đáy**!
    - Ngược lại, ngưỡng Short thành $100 - 25 = 75.0$. Khi thị trường tăng cực mạnh ($\text{RSI} = 65.0$), hiệu số $75 - 65 = +10 \implies f_{\text{rsi\_short}} > 0.80$, kích hoạt **Short ngay đỉnh uptrend phi mã**!
- **Hành động cần làm:**
  1. Khóa cận dưới của `momentum.rsi_threshold` tối thiểu $\ge 55.0$ và cận trên $\le 85.0$:
     ```rust
     /// RSI Xung Lực Bùng Nổ (55.0 -> 85.0, Step 1.0 -> 31 nấc)
     #[dna(min = 55.0, max = 85.0, step = 1.0)]
     pub rsi_threshold: f64,
     ```
  2. Đảm bảo bước nhảy $1.0$ cho độ mịn cao hơn trong việc bám đỉnh xung lực.
- **Tiêu chuẩn nghiệm thu:**
  - GA không bao giờ sinh ra `rsi_threshold < 55.0`.
  - Viết unit test xác minh: Khi RSI < 50, điểm `score_long` của `MomentumIgnitionScanner` không bao giờ đủ điều kiện kích hoạt proposal.

---

### 🟠 TASK-CFG-03: Rút ngắn dải gen `absorption.struct_sl_buffer_atr` từ 10.0 ATR về 2.50 ATR
- **Mã lỗi liên đới:** `VULN-CFG-03` (High).
- **Hiện trạng:**
  - Tại [`core_v3/config.rs:554–555`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L554-L555):
    ```rust
    #[dna(min = 0.25, max = 10.00, step = 0.25)]
    pub struct_sl_buffer_atr: f64,
    ```
  - Dải gen lên tới $10.0 \times \text{ATR}$ (40 nấc). Với đòn bẩy $20\text{x}$, khoảng cách $10$ ATR thường vượt quá $-100\%$ ROE.
  - Tuy nhiên, `RiskManager::clamp_sl_price` luôn kẹp cứng trần SL ở mức $1.0\% \text{ giá} \approx 1.5 - 2.0 \text{ ATR}$.
  - Toàn bộ khoảng giá trị từ $2.5$ ATR đến $10.0$ ATR (chiếm $30/40$ nấc tìm kiếm) đều cho ra cùng một kết quả SL sau khi clamp, gây lãng phí nghiêm trọng tài nguyên tìm kiếm của GA.
- **Hành động cần làm:**
  1. Điều chỉnh dải gen của `TacticalAbsorptionParams.struct_sl_buffer_atr`:
     ```rust
     /// Đệm ATR dưới vùng nén hấp thụ thanh khoản (0.20 -> 2.50 ATR, Step 0.10 -> 24 nấc)
     #[dna(min = 0.20, max = 2.50, step = 0.10)]
     #[serde(default = "default_absorption_sl_buffer_atr")]
     pub struct_sl_buffer_atr: f64,
     ```
  2. Cập nhật `default_absorption_sl_buffer_atr` về giá trị hợp lý $0.80$ (thay vì $2.0$).
- **Tiêu chuẩn nghiệm thu:**
  - Số nấc tìm kiếm của gen này giảm từ 40 xuống 24 nấc, tập trung $100\%$ vào dải có độ nhạy thực tế.

---

### 🟠 TASK-CFG-04: Hiệu chỉnh cận dưới `momentum.vol_z_threshold` & `reversal.exhaustion_vol_drop`
- **Mã lỗi liên đới:** `VULN-CFG-04` (High).
- **Hiện trạng:**
  1. `momentum.vol_z_threshold` ([`config.rs:412`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L412)): Dải `[0.10, 1.00]`. Vì `flow_force` nền tảng thị trường đã là $\approx 0.35$, khi gen chạm $0.10$, hàm `tanh(flow_force / 0.10) \approx 0.998$, làm mất hoàn toàn khả năng phân biệt volume bùng nổ thật.
  2. `reversal.exhaustion_vol_drop` ([`config.rs:499`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L499)): Dải `[0.01, 0.99]`. Khi chạm $0.01$, $1.0 - \tanh(\text{vol} / 0.01) = 0.0$ vĩnh viễn, làm tê liệt điểm volume của Reversal Scanner.
  3. `absorption.squeeze_threshold` ([`config.rs:543`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L543)): Dải `[0.01, 0.99]`. Cận dưới $0.01$ là phi thực tế cho việc xác định Squeeze.
- **Hành động cần làm:**
  1. Sửa `vol_z_threshold` trong `TacticalMomentumParams`:
     ```rust
     #[dna(min = 0.35, max = 0.90, step = 0.05)] // 12 nấc
     pub vol_z_threshold: f64,
     ```
  2. Sửa `exhaustion_vol_drop` trong `TacticalReversalParams`:
     ```rust
     #[dna(min = 0.20, max = 0.80, step = 0.05)] // 13 nấc
     pub exhaustion_vol_drop: f64,
     ```
  3. Sửa `squeeze_threshold` trong `TacticalAbsorptionParams`:
     ```rust
     #[dna(min = 0.40, max = 0.95, step = 0.05)] // 12 nấc
     pub squeeze_threshold: f64,
     ```
- **Tiêu chuẩn nghiệm thu:**
  - Hàm `tanh()` trong các scanner không bị bão hòa sớm hoặc triệt tiêu về 0.

---

### 🟡 TASK-CFG-05: Cấu hình hóa các Hằng số ma thuật (Magic Numbers) trong Arbiter và Risk
- **Mã lỗi liên đới:** `VULN-CFG-05` (Medium).
- **Hiện trạng:**
  - `arbiter.rs:128`: `let macro_thresh = 0.10;` gán cứng.
  - `arbiter.rs:115–116`: `trend_axis >= 0.40` gán cứng.
  - `arbiter.rs:188`: `diff > 0.15` gán cứng.
  - `risk.rs:327`: `fee_buffer_pct = 0.0008` gán cứng.
  - `mod.rs:298`: Cooldown Max Cap `60` nến gán cứng.
- **Hành động cần làm:**
  1. Thêm các trường vào `ArbiterConfig`:
     - `pub macro_veto_slope: f64` (mặc định 0.10).
     - `pub macro_veto_trend_axis: f64` (mặc định 0.40).
     - `pub conflict_margin: f64` (mặc định 0.15).
  2. Thêm các trường vào `RiskBaseParams` hoặc `RiskStopParams`:
     - `pub fee_buffer_pct: f64` (mặc định 0.0008).
     - `pub max_cooldown_bars: usize` (mặc định 60).
  3. Wire các trường này vào logic thực thi trong `arbiter.rs`, `risk.rs`, và `mod.rs`.
- **Tiêu chuẩn nghiệm thu:**
  - Không còn hằng số ma thuật hardcoded trong luồng ra quyết định VETO và quản trị rủi ro.

---

### 🟡 TASK-CFG-06: Bổ sung `#[serde(default)]` cấp độ struct chống panic khi deserialize JSON thiếu trường
- **Mã lỗi liên đới:** `VULN-CFG-06` (Medium).
- **Hiện trạng:**
  - `MatrixConfig`, `TacticalPullbackParams`, `TacticalMomentumParams`, `TacticalReversalParams`, `TacticalAbsorptionParams`, `RiskStopParams` có rất nhiều field không có `#[serde(default = "...")]`.
  - Khi người dùng nạp một file cấu hình partial JSON (chỉ ghi đè một số tham số), Serde sẽ trả về lỗi `missing field ...`.
- **Hành động cần làm:**
  1. Thêm attribute `#[serde(default)]` lên đầu mỗi struct:
     ```rust
     #[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
     #[serde(default)]
     pub struct MatrixConfig { ... }

     #[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
     #[serde(default)]
     pub struct RiskStopParams { ... }
     ```
  2. Viết unit test: Deserialize chuỗi JSON rỗng `"{}"` sang `HeliosConfigV3` và xác nhận kết quả khớp hoàn toàn với `HeliosConfigV3::default()`.
- **Tiêu chuẩn nghiệm thu:**
  - Không bao giờ bị panic hoặc lỗi deserialize khi nạp partial config từ JSON/TOML.

---

### 🟢 TASK-CFG-07: Dọn dẹp các trường rác tàn dư (Dead Fields) và xóa struct mồ côi `WarmupConfig`
- **Mã lỗi liên đới:** `VULN-CFG-07` (Low).
- **Hiện trạng:**
  - `ArbiterConfig` còn chứa 6 trường cũ từ thời Arbiter phi tuyến (`min_multiplier_veto`, `scanner_weight`, v.v.).
  - `RiskBaseParams` còn chứa `sl_min_dist_pct`, `base_lot_size`, `nuke_leverage`.
  - `WarmupConfig` đứng cô lập cuối file không thuộc struct nào.
- **Hành động cần làm:**
  1. Xóa struct mồ côi `WarmupConfig`.
  2. Đánh dấu `#[deprecated]` hoặc loại bỏ các field thừa không còn sử dụng trong `ArbiterConfig` và `RiskBaseParams`.
  3. Đảm bảo không làm gãy các file test hoặc example cũ.
- **Tiêu chuẩn nghiệm thu:**
  - Mã nguồn trong sạch, không có cảnh báo dead code hoặc unused field.

---

### 🟡 TASK-CFG-08: Wire logic thực thi cho `mega_trend_max_cycles` trong `risk.rs`
- **Mã lỗi liên đới:** `VULN-CFG-08` (Medium).
- **Hiện trạng:**
  - Trường `mega_trend_max_cycles: usize` (dòng 714) được định nghĩa với `default = 10` nhưng trong `risk.rs:257–260` hoàn toàn không được kiểm tra.
  - Lệnh siêu sóng không có cơ chế giới hạn chu kỳ, dẫn đến nguy cơ ngâm lệnh quá lâu khi sóng đã chuyển sang giai đoạn phân phối.
- **Hành động cần làm:**
  1. Cập nhật điều kiện kích hoạt Mega Trend trong `risk.rs`:
     ```rust
     let max_cycles = if risk_cfg.mega_trend_max_cycles == 0 { 10 } else { risk_cfg.mega_trend_max_cycles };
     let is_mega_trend = current_pnl > 0.0 
         && (current_pnl >= mega_trend_trigger_pnl || current_roe >= 0.50)
         && decay_cycles <= max_cycles;
     ```
  2. Bổ sung unit test kiểm chứng: Khi `decay_cycles > mega_trend_max_cycles`, chế độ Mega Trend tự động hạ cấp về Trailing thông thường.
- **Tiêu chuẩn nghiệm thu:**
  - `mega_trend_max_cycles` được wire hoàn chỉnh vào logic và có test case kiểm thử.
