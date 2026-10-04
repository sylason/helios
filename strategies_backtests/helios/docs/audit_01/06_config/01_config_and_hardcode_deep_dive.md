# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU: CẤU HÌNH DNA, MAGIC NUMBERS & ĐỘ BAO QUÁT TÀI SẢN (CONFIG & HARDCODE AUDIT)

> **Files kiểm toán:**  
> - [`config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)  
> - Các file Scanners ([`trend_pb.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs), [`momentum_ignition.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs), [`reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs), [`absorption.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs))  
> - [`gatekeeper.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs), [`arbiter.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs), [`matrix.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs), [`risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)  
> **Mục tiêu:** Rà soát triệt để các hằng số bị gán cứng (Magic Numbers), kiểm tra tính tương thích với mọi asset (từ Forex thanh khoản sâu đến Micro-cap Altcoin), và đánh giá độ bao quát [Min, Max] cùng độ mịn (Step) của không gian DNA Optimizer.

---

## 🧭 1. ĐỐI CHIẾU KIẾN TRÚC `config.rs` (CẤU TRÚC GENE DNA)

Toàn bộ cấu hình `HeliosConfigV3` bao gồm 5 nhánh lớn:
1. **`IndicatorsConfig`**: Cấu hình các cảm biến MTF (KAMA, ADX, RSI trend/rev/mom, BBands, Keltner, Stoch, ATR, NATR, MFI, VWMA).
2. **`MatrixConfig`**: Cấu hình lookback và scale các trục chuẩn hóa.
3. **`ArbiterConfig`**: Cấu hình phê duyệt tự tin và Cầu Dao Gatekeeper.
4. **`TacticalSniperConfig`**: Cấu hình 6 chiến đội Scanners (Trend, Momentum, Range, Reversal, Anomaly, Absorption).
5. **`RiskManagementConfig`**: Gồm `RiskBaseParams` (vốn, đòn bẩy) và `RiskStopParams` (SL, TSL, BE, TTP, Mega Trend).

### ⚠️ PHÁT HIỆN 1: NHÓM GEN MA & CẤU HÌNH BỊ CẮT NGỌN (ZOMBIE / SKIPPED GENES)
- **Cấu hình Matrix:**
  - `momentum_lookback` và `volume_lookback`: Đang bị `#[dna(skip)]` và gán cứng `default = 20`.
  - `adx_trend_threshold`: Bị `#[dna(skip)]` và gán cứng `25.0`.
  - `vol_z_sensitivity`: Không có `#[dna]`, gán cứng `1.50`.
- **Cấu hình Arbiter:**
  - 6 trường trọng số và ngưỡng regime cũ (`scanner_weight`, `min_confidence_trending/ranging/compression/chaos`, `min_multiplier_veto`) bị comment out hoàn toàn khỏi DNA.
  - `chaos_hysteresis_delta` ($0.10$), `conflict_confidence_margin` ($0.15$), `max_chaos_penalty` ($0.50$): Hoàn toàn **không có `#[dna]`**, là các hằng số tĩnh.
- **Cấu hình Risk Base:**
  - **100% tham số vốn và đòn bẩy (`RiskBaseParams`) KHÔNG CÓ DNA:** `percent_per_trade: 0.20`, `leverage: 20.0`, `min_order_usd: 21.0`, `max_cap_fraction: 0.25`, `taker_fee_rate: 0.0004`, `slippage_allowance_pct: 0.0002`.
- **Cấu hình Cooldown Sentinel:**
  - `re_entry_cooldown_bars` ($15$), `max_cooldown_cap_bars` ($45$), `be_tolerance_cooldown_bars` ($3$), `true_be_roundtrip_fee_pct` ($0.13\%$), `hard_loss_roe_threshold` ($-5\%$): **Không có DNA**, cố định 100%.

---

## 🔍 2. BẢNG TỔNG KẾT MAGIC NUMBERS (CÁC GIÁ TRỊ HARDCODE TRONG TOÀN BỘ CODEBASE)

Khi chiến lược được kỳ vọng thích ứng với **mọi loại asset** (Crypto, Forex, Commodity, Margin cao, Day trading), mọi con số bị gán cứng đều trở thành một cái bẫy vi cấu trúc:

| Vị trí File & Dòng Code | Giá trị Hardcode | Bản chất Hiện tượng / Ý định | Tác hại Vi cấu trúc khi gặp Asset khác |
| :--- | :--- | :--- | :--- |
| [`gatekeeper.rs:144`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs#L144) | `lookback_bars = 5usize` | Quét ngược 5 nến để xác định dải trễ Hysteresis | 5 nến 1m là 5 phút. Với Forex hay coin biến động chậm, bão có thể dư chấn kéo dài 15–30 phút; 5 nến là quá ngắn làm mở cửa bậy giữa cơn dư chấn. |
| [`matrix.rs:98`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L98) | `scale_atr * 0.20` | Chia thô cho 0.20 để chuẩn hóa KAMA slope | Làm bão hòa cứng tại $\pm 1.00$ trên mọi asset biến động mạnh, làm mất hoàn toàn tính phân biệt độ dốc của xu hướng. |
| [`matrix.rs:104,136`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L104) | `rolling_percentile(data, 20)` | Cửa sổ trượt 20 nến tính phân vị Squeeze & Chaos | **Bẫy suy biến về 0.50:** Nếu bão hoặc nén kéo dài $>20$ nến, phân vị tự tụt về 0.50 làm hệ thống tưởng hết bão/hết nén. |
| [`matrix.rs:142`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L142) | `0.35 + 0.65 * tanh(...)` | Độn sàn nhiễu giả 0.35 cho Chaos Index | Mọi tài sản dù phẳng lặng tuyệt đối như Forex ban đêm cũng bị ép mang chỉ số hỗn loạn 0.35, thu hẹp dải động lực của Gatekeeper. |
| [`trend_pb.rs:112,164`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L112) | `clv < 0.15 || close < open` (Long)<br>`clv > -0.15 || close > open` (Short) | Yêu cầu màu nến và lực nến CLV | **Không có trong Config!** Bóp chết các cây nến Pinbar/Hammer kinh điển (rút chân cực mạnh $CLV \ge 0.8$ nhưng thân nến đóng đỏ nhẹ). |
| [`trend_pb.rs:118,170`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L118) | `&[1.0, 1.0, 1.0]` | Trọng số chấm điểm Slope, Dist, Stoch | Bị gán cứng 1:1:1, không có gen DNA điều chỉnh tỷ trọng giữa độ dốc xu hướng và khoảng cách hồi quy. |
| [`trend_pb.rs:125,173`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L125) | `close * 0.002` (0.2%) | Sàn đệm SL tối thiểu | Với các tài sản biến động thấp như Forex (EUR/USD), $0.2\%$ là quá rộng (20 pips, gấp 5 lần biến động nến 1m thông thường). |
| [`momentum_ignition.rs:109`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L109) | `target_breakout = atr * 0.25` | Mục tiêu bứt phá qua dải BB | Gán cứng 0.25 ATR. Với Altcoin biến động cao, 0.25 ATR là quá nhỏ (nhiễu bình thường); với Forex, 0.25 ATR có thể là một cú breakout lớn. |
| [`momentum_ignition.rs:116`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L116) | `sat_unbounded(rsi_excess, 15.0)` | Thang đo độ dư thừa RSI | Gán cứng 15.0 điểm RSI. Không thể thích ứng giữa asset có biên độ RSI hẹp vs asset hay overbought kéo dài. |
| [`risk.rs:25`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L25) | `GLOBAL_HARD_MAX_ROE = 0.15` (15%) | Trần cắt lỗ tối đa toàn cục | Gán cứng 15% ROE. Tại 100x là $0.15\%$ giá, tại 20x là $0.75\%$ giá, tại 1x là $15\%$ giá. Khiến chiến lược hoàn toàn bị VETO nếu trade ở đòn bẩy thấp (Spot/Margin 1x–3x) với Altcoins. |
| [`risk.rs:473`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L473) | `spread_buffer = 0.5 * current_atr` | Khoảng cách an toàn giữa TP và SL | Gán cứng 0.5 ATR, không cho phép tối ưu hóa theo đặc tính biến động của asset. |

---

## 📐 3. KIỂM TOÁN ĐỘ BAO QUÁT [MIN, MAX] & ĐỘ MỊN (STEP) CHO MỌI LOẠI TÀI SẢN

### 🧪 3.1. Nghịch lý Sàn Biến Động `min_natr_threshold`: Forex vs Memecoins
- **Cấu hình hiện tại:** `#[dna(min = 0.0008, max = 0.0030, step = 0.0002)]`
  - Dải giá trị: **$0.08\%$ đến $0.30\%$**.
- **Thực tế vi cấu trúc các thị trường:**
  - **Thị trường Forex (EUR/USD, GBP/USD, USD/JPY):** Độ biến động nến 1m thông thường chỉ dao động từ **$0.015\%$ đến $0.040\%$** (1.5 đến 4 pips).
    $\implies$ **Gatekeeper sẽ VETO 100% mọi cơ hội trên Forex vì bị coi là `VOLATILITY_FLATLINE`!** Hệ thống bị tê liệt hoàn toàn khi áp dụng cho Forex.
  - **Thị trường Crypto Mid/Low-Cap & Memecoins (1000PEPE, DOGE, FLOKI):** Độ biến động nến 1m trung bình là **$0.40\%$ đến $1.20\%$**.
    $\implies$ Cận trên $0.30\%$ là quá thấp, GA không thể đặt một ngưỡng lọc biến động cao hơn để né các vùng nhiễu của Memecoin!
- **Đề xuất chuẩn hóa:**
  $$\text{min\_natr\_threshold} \in [0.0001,\ 0.0100],\ \text{step} = 0.0002\ (0.01\%\ \text{đến}\ 1.00\%,\ 50\ \text{nấc})$$

---

### 🧪 3.2. Trần Cắt Lỗ Cứng `GLOBAL_HARD_MAX_ROE = 15%` vs Đòn Bẩy Thấp
- **Thực tế:**
  - Tại $L = 20\text{x}$: Trần SL $= 15\% / 20 = 0.75\%$ giá. (Rất hợp lý cho ETH/SOL).
  - Tại $L = 2\text{x}$ (Spot Margin): Trần SL $= 15\% / 2 = 7.5\%$ giá.
  - Tại $L = 1\text{x}$ (Spot thuần): Trần SL $= 15\% / 1 = 15.0\%$ giá.
- **Vấn đề:** Khi giao dịch các cặp coin có biến động lớn trên khung 1m (hoặc khi người dùng muốn trade Day-trading đòn bẩy nhỏ 3x–5x), khoảng cách SL cấu trúc (dưới swing low / KAMA) thường rơi vào khoảng $1.5\% - 3.0\%$ giá.
  - Tại $L = 5\text{x}$: $2.0\%$ giá $\times 5 = 10\%$ ROE $\le 15\%$ $\implies$ Đạt.
  - Nhưng nếu $L = 20\text{x}$: $2.0\%$ giá $\times 20 = 40\%$ ROE $> 15\%$ $\implies$ **Bị VETO 100%!**
- **Giải pháp:** Trần ROE không nên bị gán cứng cố định một con số `0.15` duy nhất trong Rust code, mà nên là một tham số trần có thể cấu hình linh hoạt theo hồ sơ rủi ro (Risk Profile) của từng loại tài sản.

---

### 🧪 3.3. Nghịch lý Dải DNA Của Bộ 3 Phân Tầng Trailing Stop (`tsl_atr_tight / normal / loose`)
- **Hiện trạng code:**
  ```rust
  #[dna(min = 0.5, max = 4.5, step = 0.10)]
  pub tsl_atr_tight: f64,
  #[dna(min = 0.5, max = 4.5, step = 0.10)]
  pub tsl_atr_normal: f64,
  #[dna(min = 0.5, max = 4.5, step = 0.10)]
  pub tsl_atr_loose: f64,
  ```
- **Hệ quả logic:**
  - Cả 3 tầng đều có **cùng một dải tìm kiếm** $[0.5, 4.5]$!
  - Thuật toán GA tìm kiếm ngẫu nhiên hoàn toàn có thể chọn:
    $$\text{tsl\_tight} = 4.0\ \text{ATR},\quad \text{tsl\_normal} = 2.5\ \text{ATR},\quad \text{tsl\_loose} = 1.0\ \text{ATR}$$
  - **Nghịch lý toán học:** Phân tầng "Siết Chặt" (Tight) lại để SL xa tít mù (4.0 ATR), trong khi phân tầng "Nới Lỏng" (Loose) cho siêu sóng lại siết nghẹt thở (1.0 ATR)!
  - Điều này phá vỡ hoàn toàn nguyên lý phân tầng thứ bậc (Hierarchy Ordering).
- **Đề xuất chuẩn hóa:** Thiết lập các dải không chồng chéo hoặc ràng buộc tỷ lệ:
  - $\text{tsl\_atr\_tight} \in [0.4,\ 1.8],\ \text{step} = 0.05$ (29 nấc)
  - $\text{tsl\_atr\_normal} \in [1.2,\ 3.2],\ \text{step} = 0.10$ (21 nấc)
  - $\text{tsl\_atr\_loose} \in [2.5,\ 6.0],\ \text{step} = 0.15$ (24 nấc)

---

### 🧪 3.4. Ràng Buộc Sàn Binance `min_order_usd: 21.0`
- **Hiện trạng:** Gán cứng `default_min_order_usd() = 21.0`.
- **Hệ quả:**
  - Đúng chuẩn với Binance USDT-Margined Futures ($MIN\_NOTIONAL = 20.0$).
  - Nhưng nếu chạy trên sàn khác (Bybit $MIN\_NOTIONAL = 5.0$, OKX $1.0$), hoặc chạy Forex (Notional tính theo Micro-Lot $1,000$ đơn vị tiền tệ), thì con số $\$21.0$ trở nên hoàn toàn vô nghĩa và gây lỗi tính toán margin.
- **Giải pháp:** Chuyển `min_order_usd` thành tham số nạp động từ Exchange Information (`MarketFilter`) thay vì để tĩnh trong file config.

---

## 📊 4. BẢNG ĐỐI CHẤT: HIỆN TRẠNG VS HỆ QUẢ VS HƯỚNG CẢI TỔ DNA

| Tham số / Khối | Hiện trạng trong Code | Bẫy Thiếu Dynamic & Asset Thu Hẹp | Đề xuất Cải tổ Chuẩn Universal |
| :--- | :--- | :--- | :--- |
| **`min_natr_threshold`** | $[0.08\%, 0.30\%]$ | Veto 100% Forex; không lọc được nhiễu Memecoin | Mở rộng $[0.01\%, 1.00\%]$, step $0.02\%$ |
| **`clv` trong Trend_PB** | Hardcode $0.15$ và `close < open` | Giết chết nến rút chân đỏ $CLV \ge 0.8$ | Đưa `min_clv` vào `TacticalPullbackParams`, cho phép nến rút chân mạnh |
| **Trọng số Trend_PB** | Hardcode $[1.0, 1.0, 1.0]$ | Không cho phép GA tối ưu độ dốc vs khoảng cách | Thêm gen trọng số lồi `w_slope`, `w_dist` |
| **3 Tầng Trailing Stop** | Cả 3 tầng trùng dải $[0.5, 4.5]$ | GA sinh ra bộ tham số ngược đời (Tight > Loose) | Phân định dải độc lập có trật tự tăng dần |
| **Target Breakout MOM** | Hardcode $0.25\ \text{ATR}$ | Quá nhỏ với coin biến động mạnh, quá lớn với Forex | Đưa `target_breakout_atr` vào DNA $[0.10, 1.00]$ |
| **Cửa sổ Hysteresis** | Hardcode $5$ bars | Quá ngắn cho bão lớn, không kịp hạ nhiệt | Đưa `hysteresis_lookback_bars` vào config $[3, 20]$ |
| **Sàn Nhiễu Chaos** | Hardcode $0.35$ trong `matrix.rs` | Bóp méo thang đo Chaos của các asset êm đềm | Xóa bỏ độn sàn giả tạo, để $Chaos \in [0.0, 1.0]$ thuần khiết |
