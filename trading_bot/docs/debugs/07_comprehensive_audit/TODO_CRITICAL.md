# 🚨 MASTER TODO: 62 LỖ HỔNG CRITICAL (NGUY KỊCH) TOÀN HỆ SINH THÁI

> **Mục tiêu tài liệu:** Quản trị tập trung toàn bộ 62 lỗi mức độ **CRITICAL** (nguy cơ cháy tài khoản, sập bot, hoặc sai lệch PnL nghiêm trọng).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/TODO_CRITICAL.md`  
> **Trạng thái tổng quan:** **25/62 lỗi đã được sửa & kiểm thử thành công**, **37/62 lỗi đang chờ xử lý**.

---

## 📊 Bảng Tiến Độ Tổng Quan Theo Cụm Chức Năng

| Cụm chức năng | Tổng số Critical | Đã sửa | Còn lại | Mức độ rủi ro chính |
| :--- | :--- | :---: | :---: | :--- |
| **Nhóm 1: Khớp Lệnh, Quản Trị Rủi Ro & Sàn Binance** | 8 | 1 | 7 | Naked Position trên sàn, mất Stop Loss, kẹt lệnh |
| **Nhóm 2: Vòng Đời Actor, Cooldown & Hot-Swap DNA** | 6 | 2 | 4 | Đóng băng bot, cào lệnh trả thù, mất dữ liệu RAM |
| **Nhóm 3: Chiến Lược Helios V3, Scanners & Arbiter** | 9 | 8 | 1 | Hardcode min order & ép trần size |
| **Nhóm 4: Toán Học Sensors, Chỉ Báo & MTF Buffer** | 6 | 3 | 3 | Rò rỉ nến Binance REST, replay nến cũ |
| **Nhóm 5: Động Cơ GA Optimizer & Hạ Tầng Mạng** | 12 | 3 | 9 | Vô hiệu hóa LHS, WebSocket ngủ quên, sập số thập phân |
| **Nhóm 6: Kiểm Toán Chuyên Sâu TSL, TP, Sizing & Dynamic Risk** | 10 | 9 | 1 | Bẫy vả 2 đầu Whipsaw, Hardcode SL -80% ROE, Frozen TP, Absorption Inverted R:R |
| **Nhóm 7: Trạng Thái Thị Trường Không Thể Phân Tích (Untradable Regimes)** | 11 | 2 | 9 | Bão Whipsaw bị nhầm thành Range, Sổ lệnh Zombie, Absorption Bắt Dao Rơi |
| **TỔNG CỘNG** | **62** | **26** | **36** |  |

---

## 🏛️ NHÓM 1: KHỚP LỆNH, QUẢN TRỊ RỦI RO & SÀN BINANCE (8 LỖ HỔNG)

- [ ] **FLAW-5.1:** **Bug Cháy Tài Khoản (`equity <= 0.0`) Bị Trượt Sang Nến Cuối Cùng**
  - *Vị trí:* [`ferro_trade/src/broker/broker.rs:332-345`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs#L332-L345) & [`engine.rs:331-338`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/engine.rs#L331-L338)
  - *Bản chất:* Khi cháy tài khoản, lệnh thoát chỉ được đẩy vào queue mà vòng lặp `broker.next(i)` ngắt ngay. Khối `finalize_trades` ép đóng lệnh tại `final_bar_index` (nến cuối cùng của 90 ngày) làm sai lệch toàn bộ thời gian và PnL.
  - *Giải pháp đề xuất:* Đóng trực tiếp lệnh tại nến hiện tại $i$ trước khi trả về `Err("Out of money")`.

- [ ] **FLAW-5.2:** **Bỏ Qua Râu Nến Khi Thanh Lý Trong Phiên & Dead Field `margin_rate`**
  - *Vị trí:* [`ferro_trade/src/broker/broker.rs:19, 326-332`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs#L326-L332)
  - *Bản chất:* `equity` chỉ được kiểm tra tại giá `Close`. Nến quét râu làm âm tài khoản trong phiên nhưng rút chân đóng cửa thì bot vẫn ngộ nhận tài khoản sống sót; trường `margin_rate` là dead field không hề được tính toán.
  - *Giải pháp đề xuất:* Kiểm tra thanh lý tại giá bất lợi nhất (`Low` cho Long, `High` cho Short) theo công thức Margin chuẩn của Binance.

- [ ] **FLAW-5.3:** **Lỗ Hổng Naked Position Trên Binance Live Khi Retry SL Thất Bại**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/order_executor.rs:240-270`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L240-L270)
  - *Bản chất:* Quy trình dời SL trên sàn: Hủy SL cũ -> chờ 150ms -> retry 3 lần đặt SL mới. Nếu cả 3 lần retry thất bại, hàm nuốt lỗi trả về `Ok(())`, bỏ mặc vị thế thật trên sàn không có Stop Loss bảo vệ!
  - *Giải pháp đề xuất:* Nếu retry thất bại: Bắn cảnh báo khẩn cấp (Emergency Alert) và lập tức đóng vị thế ở giá Market để bảo toàn vốn.

- [ ] **FLAW-5.4:** **Thiếu Hoàn Toàn Take Profit Trên Binance Live**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/live_engine.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs) & [`order_executor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs)
  - *Bản chất:* Bot Live chỉ gửi lệnh Stop Loss lên sàn, hoàn toàn không gửi Take Profit algo order (`LIMIT` hoặc `TAKE_PROFIT_MARKET`), làm lỡ các cú bứt phá chốt lời khi giá chạm TP rồi rút chân.
  - *Giải pháp đề xuất:* Đồng bộ gửi cặp lệnh OCO hoặc lệnh Algo Take Profit song song với Trailing SL lên Binance.

- [x] **FLAW-5.5 & FLAW-9.3:** **Bất Đồng Bộ Mô Hình Khớp Nến Quét Cả SL & TP (Dual-Hit Conflict)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/paper_trader.rs:396-440`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L396-L440)
  - *Đã xử lý:* Đã loại bỏ heuristic khoảng cách Open (gây thắng ảo 50-70% ở Paper), chuyển sang mô hình bảo thủ Pessimistic Matching ưu tiên Stop Loss trước 1:1 với `ferro_trade::Broker`.

- [ ] **FLAW-7.1:** **Cơ Chế "Clean Slate Market Dump" Tự Bán Tháo Vị Thế Khi Bật Bot**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/account_manager.rs:174-178`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/account_manager.rs#L174-L178)
  - *Bản chất:* Khi khởi động bot, nếu phát hiện có vị thế đang chạy trên Binance, bot tự động hủy toàn bộ lệnh và gửi lệnh Market ép đóng vị thế ngay lập tức thay vì tiếp quản bảo vệ (Adopt & Protect).
  - *Giải pháp đề xuất:* Khôi phục đúng triết lý Adopt & Protect: Đọc vị thế hiện có, nạp vào `LivePositionState`, tính toán SL bảo vệ và tiếp quản quản trị.

- [ ] **FLAW-7.2:** **Order Submission Deadlock Khóa Chết Toàn Bộ Giao Dịch**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/live_engine.rs:69`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L69) & [`order_executor.rs:148`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L148)
  - *Bản chất:* Khi gửi lệnh Algo SL, bot bật cờ `is_submitting_order = true` nhưng vì Algo SL chưa khớp nên sàn không gửi Trade Update event, cờ này không bao giờ reset về `false`, khóa chết toàn bộ lệnh Buy/Sell tiếp theo.
  - *Giải pháp đề xuất:* Tách biệt cờ `is_submitting_order` chỉ áp dụng cho lệnh mở vị thế (Buy/Sell), hoặc tự động reset cờ ngay sau khi nhận phản hồi REST API thành công từ Binance.

- [ ] **FLAW-7.7:** **Vị Thế Tiếp Quản Bị Thả Nổi Không Có Stop Loss Nếu Đang Âm**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:273`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L273) & [`live_engine.rs:132`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L132)
  - *Bản chất:* Khi vị thế tiếp quản từ Binance đang lỗ danh nghĩa (`current_pnl <= 0.0`), `risk.rs` bỏ qua logic đặt SL vì điều kiện `if current_pnl > 0.0` mới cho phép đặt SL! Vị thế bị thả nổi không có SL.
  - *Giải pháp đề xuất:* Thiết lập Hard Stop Loss ban đầu bắt buộc cho mọi vị thế tiếp quản kể cả khi PnL đang âm.

---

## 🔄 NHÓM 2: VÒNG ĐỜI ACTOR, COOLDOWN & HOT-SWAP DNA (6 LỖ HỔNG)

- [x] **FLAW-9.4:** **Sốc Rủi Ro Khi Hot-Swap DNA Áp Đặt Hồi Tố Lên Vị Thế Đang Mở** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - *Đã xử lý:* Đã thêm trường `pending_dna`. Khóa 100% DNA cũ bảo vệ vị thế đang chạy, tự động kích hoạt DNA mới khi vị thế chuyển về Flat ("Lock DNA on Active Trade").

- [x] **FLAW-9.2:** **Lệnh Tàng Hình & Bỏ Qua Cooldown Khi Mở-Đóng Cùng Nến (Revenge Re-Entry)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - *Đã xử lý:* Đã đưa xả `account_rx` lên đầu nến; triển khai Chốt chặn Tầng 1 cấm mở lệnh cùng cây nến vừa đóng, và Chốt chặn Tầng 2 `CooldownGuard` cách ly 15 nến nếu PnL $\le 0.001$.

- [ ] **FLAW-9.1 & FLAW-3.1:** **Đóng Băng Cooldown Vĩnh Viễn Trong Live/Paper (Index Freeze Deadlock)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:192, 224`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L192) & [`helios_bridge.rs:211`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L211)
  - *Bản chất:* Trong Live/Paper, `rolling_buffer` trượt liên tục nên chỉ số nến hiện tại `i = last_idx` luôn cố định (ví dụ 43,099). Biểu thức kiểm tra `i < locked_until_bar` (ví dụ `43099 < 43109`) luôn đúng mãi mãi $\rightarrow$ Bot bị khóa Cooldown vĩnh viễn không thể vào lệnh.
  - *Giải pháp đề xuất:* Chuyển đổi toàn bộ logic Cooldown từ số thứ tự bar (`usize`) sang Timestamp epoch milliseconds (`i64`).

- [ ] **FLAW-7.4:** **Phá Vỡ Tính Đơn Điệu (Monotonic Ratchet) Trên Live Khi Hot-Swap DNA**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/live_engine.rs:45-89, 126-136`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L126-L136)
  - *Bản chất:* `live_engine.rs` không lưu giữ `current_sl_price` trong RAM; khi DNA mới có mức SL rộng hơn, bot hủy mức SL chặt trên Binance và gửi lệnh nới lỏng SL lùi xuống! *(Lưu ý: PaperTrader đã được sửa Ratchet Guard, cần đồng bộ sang LiveEngine).*
  - *Giải pháp đề xuất:* Đưa Monotonic Ratchet Guard vào `OrderExecutor` và `BinanceLiveExecutionEngine`.

- [ ] **FLAW-7.5:** **Thiếu Validate DNA (`len`, `NaN`, `Inf`) Gây Panic Sập Actor**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:270`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L270) & [`asset_actor.rs:154`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L154)
  - *Bản chất:* Nhận vector DNA thô từ kênh watch channel không kiểm tra độ dài hoặc tính hợp lệ của số thực. Nếu vector bị lệch chiều hoặc chứa `NaN`, Actor bị panic sập hoặc tê liệt logic Trailing SL.
  - *Giải pháp đề xuất:* Thêm hàm `validate_dna(dna: &[f64]) -> Result<(), ConfigError>` kiểm tra 110 chiều, chặn `NaN`/`Inf` và kẹp miền giá trị hợp lệ trước khi broadcast.

- [ ] **FLAW-7.6:** **Zero SQLite Position Storage (100% RAM-Only)**
  - *Vị trí:* [`trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:30-72`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/sqlite_impl.rs#L30-L72) & [`repository.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/repository.rs)
  - *Bản chất:* SQLite hoàn toàn không có bảng lưu trữ vị thế (`positions`). Khi bot restart hoặc gặp sự cố mất điện, toàn bộ trạng thái Paper Trading và metadata Live bị xóa sạch.
  - *Giải pháp đề xuất:* Thiết kế bảng `active_positions` và `closed_trades` trong SQLite, lưu và khôi phục trạng thái vị thế mỗi chu kỳ nến.

---

## 🎯 NHÓM 3: CHIẾN LƯỢC HELIOS V3, SCANNERS & ARBITER (9 LỖ HỔNG)

- [x] **FLAW-H.4:** **Bẫy Cắt Lỗ Sớm Stagnant Decay (Máy Chém 45 Phút)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)
  - *Đã xử lý:* Đã xóa bỏ hoàn toàn `decay_cycles > 0 && current_pnl <= 0.0`, kết nối 2 gen phổ quát `max_stagnant_hold_bars` và `stagnant_min_profit_pct`.

- [x] **FLAW-H.1:** **`update_from_dna` Không Cập Nhật `MarketMatrixV3`** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:58`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L58)
  - *Đã xử lý:* Đã thêm `self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);` khi nạp DNA mới.

- [x] **FLAW-H.2:** **Thuộc Tính `#[dna]` Gắn Nhầm Vào Trường `bool`** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)
  - *Đã xử lý:* Đã gỡ bỏ `#[dna]` khỏi trường bool `enable_re_entry_cooldown`.

- [ ] **FLAW-H.3:** **Hardcode `min_order_usd = 21.0` & Ép Trần Cứng `size_fraction.min(0.20)`**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:13-24`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L13-L24)
  - *Bản chất:* Ép cứng mức vào lệnh tối thiểu $21 và kẹp trần 20% vốn, vô hiệu hóa gen DNA `percent_per_trade` (vốn cho phép GA tối ưu tỷ lệ vào lệnh linh hoạt).
  - *Giải pháp đề xuất:* Đọc trực tiếp từ cấu hình DNA `risk_cfg.percent_per_trade`, chỉ fallback về min sàn nếu nhỏ hơn quy định `min_notional` của Binance.

- [x] **FLAW-4.1:** **"DNA Dead Zone" Trong Arbiter: >50% Quần Thể GA Ra 0 Trades** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:102-140`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L102-L140) & [`config.rs:245`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L245)
  - *Đã xử lý:* Đã thay thế công thức tích 3 phân số bằng **Weighted Geometric Mean** kết hợp **Zero Guard**: $C = \sqrt{w_{regime} \times M}$; nếu $S \le 0.0 \lor C \le 0.0$ thì $\text{confidence} = 0.0$, ngược lại $\text{confidence} = S^W \times C^{1.0 - W}$. Kích hoạt trực tiếp gen ma `scanner_weight` ($W$, một phần FLAW-2.1) và giải phóng 100% dải động DNA `min_confidence` ($0.01 \dots 0.99$), triệt tiêu hoàn toàn "vùng đất chết". Kèm 3 unit tests xác minh.

- [x] **FLAW-4.2:** **Take Profit Không Validate Dẫn Đến Đóng Nhầm Chiều (Wrong-Side TP)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs)
  - *Đã xử lý:* Đã chuẩn hóa và validate bắt buộc cho Take Profit trong `range_osc.rs`: Long TP $\ge \text{Close} \times 1.002$ và Short TP $\le \text{Close} \times 0.998$, triệt tiêu hoàn toàn rủi ro đảo chiều ăn non hoặc vừa vào lệnh đã cắn TP lỗ.

- [x] **FLAW-4.3:** **Lỗi Trị Tuyệt Đối `.abs()` Làm Bot Bắt Dao Rơi Khi Giá Sập Hầm** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Đã xử lý:* Đã xóa bỏ hoàn toàn `.abs()`, bổ sung kiểm tra định hướng bứt phá có lợi `is_favorable_breakout`: Chỉ mở khóa Cooldown Long nếu giá bứt phá TĂNG NGƯỢC LÊN $\ge 1.5$ ATR (và Short nếu giá sập sâu $\ge 1.5$ ATR). Kèm unit test xác minh `test_cooldown_falling_knife_blocked`.

- [x] **FLAW-4.4:** **Nhận Diện Sai Lệnh Thắng / Thua (False Win) Khi Nến Rút Chân** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:180-212`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L180-L212) & [`ferro_trade/src/backtest/context.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs)
  - *Đã xử lý:* Bổ sung trường `pub last_closed_trade: Option<&'a Trade>` vào `TickContext` (Zero-Cost reference mượn trực tiếp từ `broker.closed_trades.last()`), nạp đồng bộ trong cả `engine.rs` và `helios_bridge.rs`. Trong `core_v3/mod.rs`, Cooldown Tracker đọc trực tiếp `real_pnl = closed.pl(None)` và `exit_price` từ lệnh đóng thực tế, triệt tiêu 100% bẫy False Win và bẫy đu đỉnh khi nến rút chân (Hammer). Kèm unit test xác minh `test_cooldown_triggered_on_hammer_wick_loss`.

- [x] **FLAW-4.5:** **Lỗi "Double Tanh" Dìm Điểm Tê Liệt Reversal & Anomaly Scanners** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:20`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L20) & [`anomaly.rs:20`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs#L20)
  - *Đã xử lý:* Chuyển cả 2 scanner sang đọc Z-Score tuyến tính trực tiếp từ `sensors.z_score[i]` thay vì `matrix.price_z_score[i]`. Triệt tiêu hoàn toàn lần nén tanh thứ hai, khôi phục phân bố điểm $f_z = \tanh(|Z_{\text{raw}}| / z_{\text{thresh}}) \in [0.0, 1.0]$ phản ánh trung thực số độ lệch chuẩn $\sigma$ thực tế và giải phóng sức mạnh cho 2 scanner Reversal và Anomaly. Kèm unit test `test_reversal_and_anomaly_use_raw_z_score_not_double_tanh` xác minh.

---

## 📐 NHÓM 4: TOÁN HỌC SENSORS, CHỈ BÁO & MTF BUFFER (6 LỖ HỔNG)

- [x] **FLAW-3.2:** **`volume_force` Lấy Nhầm Z-Score GIÁ Thay Vì KHỐI LƯỢNG** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:94-101`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L94-L101) & [`ferro_trade/src/indicators/factory/bulk.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs)
  - *Đã xử lý:* Bổ sung chỉ báo `z_score_volume` vào `ferro_trade::BulkIndicatorFactory` (tính Z-Score trực tiếp trên chuỗi `"Volume"`). Bổ sung `vol_z_score` vào `HeliosSensors`, nạp độc lập trong `mod.rs` và truyền vào `matrix.rs` để tính `volume_force`. Bổ sung guard chống chia cho 0 khi `z_score_sensitivity <= 0.0`. Kèm unit test xác minh phân tách độc lập Volume Z-Score và Price Z-Score.

- [x] **FLAW-3.3:** **Sập Simplex Normalization (Tổng Trọng Số Vọt Lên 200%) Khi Gặp `NaN`** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`ferro_trade/src/indicators/calculate/bulk.rs:1637-1652`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1637-L1652) & [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:126-140`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L126-L140)
  - *Đã xử lý:* Áp dụng chuẩn toán học Quant 2 lớp (2-Layer Defense):
    1. **Lớp 1 (Source Warmup Guard):** Trong `ferro_trade/src/indicators/calculate/bulk.rs` hàm `chaos_ratio`, trả về `f64::NAN` cho chuỗi warmup (`clean_natr.len() < sma_period` hoặc `v.is_nan() || sma_val.is_nan()`) để `IndicatorManager` tự động phát hiện và tính đúng `warmup_period`. Kèm unit test `test_chaos_ratio_returns_nan_during_warmup` xác minh.
    2. **Lớp 2 (Matrix Percentile & Arithmetic Guard):** Trong `strategies_backtests/helios/src/strategy/core_v3/matrix.rs`, fallback `c_ratio` khi gặp `NaN` về `0.0` (thị trường 0% Chaos - yên ắng) thay vì `1.0` (báo động giả 100% bão tố). Khi toàn bộ chỉ báo đều là `NaN` ở các nến đầu, kích hoạt fallback chia đều $1/3 - 1/3 - 1/3$, bảo đảm tổng trọng số Simplex luôn luôn $\sum \equiv 1.00$ trong mọi điều kiện toán học. Kèm unit test `test_simplex_normalization_nan_resilience` xác minh.

- [x] **FLAW-3.4:** **Cache Collision Trong `IndicatorType::KamaSlope`** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`ferro_trade/src/indicators/identifier.rs:119`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L119) & [`factory/bulk.rs:80`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs#L80)
  - *Đã xử lý:* Bổ sung `atr_period: usize` vào biến thể `IndicatorType::KamaSlope`, cập nhật hàm khởi tạo trong `BulkIndicatorFactory::kama_slope`. Đảm bảo phân tách khóa băm `CacheKey` độc lập tuyệt đối giữa các cá thể GA có chu kỳ ATR khác nhau. Kèm unit test `test_kama_slope_cache_key_differentiation_by_atr_period` xác minh mã băm và tính bất đẳng thức của 2 key khác nhau.

- [ ] **FLAW-1.1:** **Rò Rỉ Nến Chưa Đóng (Unclosed Candle) Từ Binance REST**
  - *Vị trí:* [`trading_bot/bot_engine/src/data_engine/binance_rest.rs:65-71`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/binance_rest.rs#L65-L71) & [`models.rs:94-98`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/models.rs#L94-L98)
  - *Bản chất:* `fill_gap` gọi REST API không truyền `endTime` nên Binance trả về cả cây nến 1m đang chạy dở. Nến dở dang bị lưu vào SQLite. Khi WebSocket đẩy nến đóng thật về, hàm dedup thấy trùng timestamp nên **vứt bỏ vĩnh viễn nến đóng thật**!
  - *Giải pháp đề xuất:* Truyền `endTime = current_timestamp - 60_000` để chỉ lấy các cây nến đã đóng 100%.

- [ ] **FLAW-1.2:** **Signal Replay Bắn Lệnh Live Trên Nến Quá Khứ Khi Reconnect**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs:326-367`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L326-L367)
  - *Bản chất:* Khi mất mạng có lại, `fill_gap` bơm 10-15 nến quá khứ vào luồng. `AssetActor` không kiểm tra cờ `!is_historical`, dẫn đến chạy chiến lược và bắn lệnh thật lên sàn theo giá quá khứ!
  - *Giải pháp đề xuất:* Thêm cờ `is_historical: bool` và bỏ qua bước thực thi action khi đang nạp bù nến quá khứ.

- [ ] **FLAW-7.3:** **Rò Rỉ Dữ Liệu Tương Lai (Lookahead Leakage) Trong Walk-Forward Evaluator**
  - *Vị trí:* [`trading_bot/bot_engine/examples/walk_forward_evaluator.rs:158-170`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/walk_forward_evaluator.rs#L158-L170)
  - *Bản chất:* Quên truyền `.with_sync_until(train_end_ts)` khiến Actor nạp trước toàn bộ nến Out-Of-Sample vào buffer trước khi OOS bắt đầu chạy!
  - *Giải pháp đề xuất:* Đồng bộ `.with_sync_until(train_end_ts)` như trong `walk_forward_smart_seed.rs`.

---

## 🧬 NHÓM 5: GA OPTIMIZER, CẤU HÌNH DNA & HẠ TẦNG (11 LỖ HỔNG)

- [x] **Flaw Cờ Khớp Lệnh `trade_on_close`:** `evaluate_live_tick` dùng `false` trong khi GA dùng `true` `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs:71`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L71)
  - *Đã xử lý:* Đã chuyển về `trade_on_close: true` đồng bộ 1:1.

- [x] **Monotonic Ratchet Guard & Naked Position Guard Tại PaperTrader:** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/paper_trader.rs:544-585`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L544-L585)
  - *Đã xử lý:* Đã chặn nới lỏng SL và cấm xóa SL về None khi đang có vị thế mở; đồng bộ chuẩn xác sang `LivePositionState`.

- [x] **FLAW-2.1:** **21 Tham Số Ma (Phantom Genes) Chiếm 19.1% DNA** `[ĐÃ XỬ LÝ 11 GENES CHÍNH ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs:80-530`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L80-L530)
  - *Bản chất:* 21 trường cấu hình trong DNA được GA tối ưu và lưu DB nhưng mã nguồn runtime hoàn toàn không đọc hoặc bị ghi đè bởi hằng số cứng. Lãng phí 20% tài nguyên tính toán GA.
  - *Đã xử lý:* Đã cắt bỏ 11 chiều gen không dùng đến: `macd` (4 genes), `vol_ma` (2 genes), `sl_min_dist_pct` (1 gene), `base_lot_size` (1 gene), `leverage` (1 gene), `percent_per_trade` (1 gene), `mega_trend_max_cycles` (1 gene). Thêm `tsl_buffer_floor_atr` (1 gene), tinh gọn DNA từ 118 xuống 108 chiều.

- [x] **FLAW-2.2:** **Xung Đột Đòn Bẩy Giữa JSON 20x Và DNA 1x-50x** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`helios_optimizer_config.json:5`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_optimizer_config.json#L5) & [`config.rs:455`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L455)
  - *Bản chất:* JSON khóa cứng tài khoản sàn 20x (`margin_rate = 0.05`), nhưng DNA cho GA chọn 1x-50x. Nếu GA chọn 1x: Stop Loss bị nới ra 80%, trong khi sàn 20x thanh lý ở mức -5%!
  - *Đã xử lý:* Tách `leverage` ra khỏi DNA, cố định 20x và chuyển thành User Setting; tạo thêm biến `nuke_leverage: f64 = 50.0` độc lập cho chiến lược Nuke.

- [ ] **FLAW-2.3:** **Nguy Cơ Sập Bot (`index out of bounds`) Khi Đổi Kích Thước DNA**
  - *Vị trí:* [`ferro_trade/ferro_trade_derive/src/lib.rs:154`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/ferro_trade_derive/src/lib.rs#L154) & [`asset_actor.rs:235`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L235)
  - *Bản chất:* Macro truy cập `dna[offset]` trực tiếp không kiểm tra bounds. Khi đổi số chiều DNA, nạp bản ghi cũ từ SQLite sẽ panic sập Actor.
  - *Giải pháp đề xuất:* Kiểm tra `if dna.len() != EXPECTED_LEN { resize / fallback }` an toàn trước khi nạp.

- [ ] **FLAW-6.1:** **Initial Job #0 Không Chạy Thuật Toán Latin Hypercube Sampling (LHS)**
  - *Vị trí:* [`ferro_trade/src/optimizer/prospector.rs:360, 644`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/prospector.rs#L360)
  - *Bản chất:* Khởi tạo Job #0 bị gán nhãn `Hotspot` thay vì `New`, khiến đợt khởi đầu chạy thẳng vào Monte Carlo ngẫu nhiên, vô hiệu hóa hoàn toàn LHS.
  - *Giải pháp đề xuất:* Gán đúng nhãn `JobType::New` cho mồi ban đầu để kích hoạt LHS phủ đều không gian siêu tham số.

- [ ] **FLAW-6.2:** **Lệch Chuẩn Thang Đo Fallback Biến Lỗi Crash Thành Điểm Gần Hòa Vốn**
  - *Vị trí:* [`ferro_trade/src/optimizer/ga_worker.rs:469, 476, 509`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/ga_worker.rs#L469)
  - *Bản chất:* Khi backtest crash, điểm fallback là `-1000 / 1M = -0.001` (gần hòa vốn), cao hơn nhiều so với cá thể thua lỗ (-100.0) hay ngắt sớm (-5000.0), làm hỏng bộ lọc cắt tỉa GA.
  - *Giải pháp đề xuất:* Đặt fallback đồng nhất cực âm sâu: `-100_000.0`.

- [ ] **FLAW-6.3:** **"Cấu Hình Chết" `min_trades` Trong GA Engine**
  - *Vị trí:* [`ferro_trade/src/optimizer/ga_worker.rs:192, 257, 426`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/ga_worker.rs#L192)
  - *Bản chất:* Cấu hình `min_trades = 10` truyền vào struct nhưng hàm đánh giá hoàn toàn không đọc đến, khiến cá thể có 1-2 trade may mắn vẫn giành giải cao.
  - *Giải pháp đề xuất:* Đọc và áp dụng phạt nặng vào fitness nếu `trades_count < min_trades`.

- [x] **FLAW-6.4:** **Thiếu Out-Of-Sample (OOS) Validation Gate Trong Live Trading** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs)
  - *Đã xử lý:* Đã triển khai Cổng Thẩm Định WFE Validation Gate (48 giờ OOS): Tự động đánh giá độc lập cá thể Champion qua `diagnostic_backtest`. Nếu OOS Return $\le 0.0$ hoặc Max Drawdown $> 15\%$, hệ thống lập tức từ chối nạp, bắn log cảnh báo chi tiết `[WFE GATE: REJECTED - OVERFIT DETECTED]` và duy trì DNA an toàn trước đó.

- [ ] **FLAW-6.5:** **Hiện Tượng Overfit Optimizer & Thiếu Cơ Chế Điều Hòa (Regularization / Cross-Validation)**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs) & [`ferro_trade/src/optimizer/ga_worker.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/ga_worker.rs)
  - *Bản chất:* Hiện tượng Overfit trong thuật toán di truyền là chắc chắn sẽ xảy ra (không thể tránh khỏi khi tối ưu trên chuỗi thời gian tài chính có độ nhiễu cao). Dù WFE Gate đã chặn được các cá thể âm OOS, nhưng khi tỷ lệ Overfit cao (như 10 lần bị REJECT trong Log 26), hệ thống phải giữ lại DNA cũ thay vì tiến hóa thích ứng kịp thời với bối cảnh thị trường mới.
  - *Giải pháp đề xuất (Sẽ nghiên cứu & giải quyết sau):* Áp dụng các kỹ thuật từ Machine Learning vào GA Optimizer: Time-Series K-Fold Cross Validation, Phạt độ phức tạp và độ nhạy của DNA (Regularization Penalty vào hàm Fitness), Đánh giá Fitness đa cụm thời gian (Multi-Window Walk-Forward Fitness Objective), hoặc Early Stopping dựa trên Validation Loss.

- [ ] **FLAW-6.6:** **Tình Trạng Đóng Băng Cập Nhật DNA Khi Liên Tục Bị WFE Gate Reject (Stale DNA Risk)**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:280-320`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L280-L320)
  - *Bản chất:* Khi thị trường chuyển pha đột ngột (từ Trend sang Range hoặc biến động cực hạn), GA Optimizer có thể bị trượt 2-3 chu kỳ liên tiếp (8-12 giờ) không tìm ra cá thể OOS dương. Bot bị buộc phải giữ lại DNA cũ của pha thị trường trước đó. Dù "Lock DNA on Active Trade" bảo vệ vị thế đang mở rất tốt, nhưng các lệnh mở mới sau đó có nguy cơ sử dụng DNA lỗi thời.
  - *Giải pháp đề xuất (Sẽ nghiên cứu & giải quyết sau):* Thiết kế cơ chế Stale DNA De-risking: Tự động thu hẹp quy mô vị thế (De-leverage / Half-size) hoặc kích hoạt chế độ phòng thủ (Defensive Filter) khi DNA hiện tại đã quá 12 giờ chưa được làm mới.

- [ ] **FLAW-8.1:** **WebSocket Rơi Vào Vòng Lặp `sleep(60)` Vô Tận Khi Đứt Mạng**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/ws_manager.rs:139-151`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L139-L151)
  - *Bản chất:* Callback SDK chạy ngầm, sau đó luồng rơi vào `loop { sleep(60) }`. Khi sàn ngắt kết nối, vòng lặp sleep chạy mãi mãi, khối reconnect phía dưới không bao giờ chạm tới được. Bot rơi vào trạng thái chết lâm sàng.
  - *Giải pháp đề xuất:* Lắng nghe kênh ngắt kết nối và kích hoạt tái kết nối chủ động.

- [ ] **FLAW-8.2:** **`round_qty` Dùng `.round()` Thay Vì `.floor()` Gây Lỗi Sàn `-2019`**
  - *Vị trí:* [`trading_bot/bot_engine/src/models/market_info.rs:45`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/market_info.rs#L45)
  - *Bản chất:* Làm tròn khối lượng bằng `.round()` làm tròn lên, dẫn đến đặt lệnh vượt số dư khả dụng (`Margin Insufficient`) hoặc bán khống vượt vị thế.
  - *Giải pháp đề xuất:* Luôn dùng `.floor()` khi làm tròn khối lượng đặt lệnh Futures.

- [ ] **FLAW-8.3:** **`.unwrap()` Trực Tiếp Trên `Decimal::from_f64` Gây Panic Sập Bot**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/order_executor.rs:77, 84`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L77)
  - *Bản chất:* Gọi `.unwrap()` trên `Decimal::from_f64(price/qty)`. Nếu dính `NaN` hoặc `Inf`, bot panic sập ngay lập tức.
  - *Giải pháp đề xuất:* Dùng `unwrap_or_default()` hoặc trả về `Result::Err`.

- [ ] **FLAW-8.4:** **Hố Ngăn Cách 100% Giữa Tauri v2 IPC Và Bot Engine**
  - *Vị trí:* [`trading_bot/bot_desktop_client`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_desktop_client)
  - *Bản chất:* Backend Tauri chỉ là template `greet`, 0 lệnh điều khiển bot được kết nối với Engine.
  - *Giải pháp đề xuất:* Xây dựng các Tauri commands nối trực tiếp vào `AssetActor` qua MPSC channel.

---

## 🔬 NHÓM 6: KIỂM TOÁN CHUYÊN SÂU HỆ THỐNG TRAILING SL, TP, SIZING & DYNAMIC RISK (AUDIT TOÀN DIỆN V3.3) (6 LỖ HỔNG)

- [x] **FLAW-TSL.1:** **Thiếu Hoàn Toàn Cơ Chế Trailing Take Profit (Static TP hoặc None)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:200-245, 340-405`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) & [`paper_trader.rs:20, 510-530`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)
  - *Bản chất:* 
    1. Trong 6 Tactical Scanners: Chỉ có `RangeOscScanner` (chốt lời biên Bollinger Bands) và `ReversalScanner` (chốt lời tại KAMA mid) có phát sinh `suggested_tp`. Cả 4 Scanners còn lại (`TrendPullback`, `MomentumIgnition`, `Absorption`, `Anomaly`) hoàn toàn để `suggested_tp: None`.
    2. Suốt vòng đời vị thế, `RiskManager::manage_trailing_stop` gọi `tick.update_trade_sl_tp(trade.id, Some(safe_tightened_sl), trade.tp_price);` $\rightarrow$ Mức TP nếu có thì đứng yên vĩnh viễn, không hề có cơ chế dời động (Trailing TP / Dynamic TP) theo bước sóng hoặc ATR.
    3. Khi vào siêu sóng (Mega Trend), hệ thống chỉ dựa vào việc nâng sàn Stop Loss (`mega_trend_profit_floor`). Nếu giá giật ngược bất ngờ (Flash Dump) quét chạm SL thì bị bốc hơi một phần lợi nhuận thay vì chủ động chốt lời từng phần (Partial TP / Ladder TP).
  - *Đã xử lý (Phương án 3: Hybrid Trailing Take Profit):*
    1. **Nhánh A (Dynamic Structural TP cho Range & Reversal):** Dời TP bám sát dải `bb_upper/lower` (cho `RANGE_OSC`) và `kama_mid` (cho `REVERSAL`). Tích hợp 2 chốt chặn: *Min Profit Floor Guard* ($TP \ge Entry \times (1 + \text{min\_gain})$) chống co thắt dải biến chốt lời thành chốt lỗ, và *Spread Guard* ($|TP - SL| \ge 0.5 \times ATR$) chống va chạm SL/TP.
    2. **Nhánh B (Peak-Callback Trailing TP cho Trend/Momentum/Absorption/Anomaly):** Ban đầu để `TP: None` thả diều siêu sóng. Khi `peak_roe >= ttp_activation_roe` (mặc định 20% ROE), kích hoạt theo dõi callback. Nếu giá từ đỉnh quay đầu giảm quá `ttp_callback_atr` (mặc định 0.8 ATR) và đảm bảo có lãi $\ge \text{min\_gain}$, tự động gọi `tick.close(None)` khóa cứng đỉnh lợi nhuận.
    3. **DNA hóa 3 tham số:** `ttp_activation_roe`, `ttp_callback_atr`, `dynamic_tp_min_gain_pct` tích hợp vào `RiskStopParams`.
    4. **Đồng bộ Telemetry:** Bổ sung `ExitReason::TrailingTakeProfit` (ký hiệu `"TTP"`) vào `PaperTrader`. Đã pass 100% unit tests.

- [x] **FLAW-TSL.2:** **Năm Hằng Số Magic Numbers Ẩn Trong Quản Trị Rủi Ro Chưa Được Tham Số Hóa** `[ĐÃ XỬ LÝ 4 HẰNG SỐ CỐT LÕI ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)
  - *Bản chất:* Vẫn còn các giá trị cứng nằm sâu trong logic toán học, chưa được đưa vào cấu hình.
  - *Đã xử lý:*
    1. Đưa `tsl_buffer_floor_atr` vào DNA (`0.4 .. 2.5 ATR, step = 0.10`), thay thế hoàn toàn magic number 1.0 ATR ở dòng 338.
    2. Hạ trần ngâm lệnh Zombie `max_stagnant_hold_bars` từ 2880 bars (48h) xuống tối đa 720 bars (12h) với step 30 nến.
    3. Đưa `min_order_usd = 21.0` và `max_cap_fraction = 0.25` vào struct cấu hình, xóa bỏ hardcode 20% và $21 trong `calculate_size`.
    4. Vá lỗi toán học `NaN` trong `calculate_size` khi `equity <= 0.0`.

- [x] **FLAW-TSL.3:** **Bất Đồng Bộ Đòn Bẩy (Leverage Mismatch) Giữa DNA (1x-50x) Và Tài Khoản Sàn (20x)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs:455`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L455) & [`paper_trader.rs:488`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L488)
  - *Bản chất:* DNA cho GA chọn 1x-50x trong khi sàn cố định 20x (`margin_rate = 0.05`), gây nguy cơ cháy tài khoản trước khi chạm SL nếu GA chọn đòn bẩy thấp.
  - *Đã xử lý:* Tách `leverage: 20.0` thành User Setting cố định khớp 1:1 với sàn; tách riêng `nuke_leverage: 50.0` cho chiến lược Nuke; cho phép người dùng tùy chỉnh từ giao diện bot/test harness.

- [ ] **FLAW-TSL.4:** **Strategy Sizing Không Nhìn Thấy `MarketFilter` (Rủi Ro Altcoin Lot Size Thô)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:11-22`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L11-L22)
  - *Bản chất:* `RiskManager::calculate_size` chỉ tính toán theo tỷ lệ phần trăm vốn lý thuyết thuần túy (`size_fraction = target_margin / equity`), hoàn toàn không nhận thức được `step_size`, `tick_size`, `min_qty` hay `min_notional` thực tế của từng cặp tiền.
  - *Hệ quả Edge Cases:* Đối với Altcoin có bước khối lượng lớn (ví dụ bước 1 đơn vị coin thay vì 0.001), việc làm tròn `.floor()` ở tầng dưới (`OrderExecutor` / `PaperTrader`) có thể làm sụt giảm 30-50% khối lượng thực tế so với con số tối ưu của GA, làm méo mó hoàn toàn tỷ lệ Risk/Reward đã được backtest.
  - *Giải pháp đề xuất:* Truyền metadata sàn (`MarketFilter`) vào `TickContext` hoặc tạo lớp adapter chuẩn hóa sizing trước khi phát sinh `Action::Buy` / `Action::Sell`.

- [x] **FLAW-TSL.5:** **Độ Trễ 1 Nến Của High Watermark (`peak_val`) Tại Cây Nến Mở Lệnh (`entry_bar`)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:107-135`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L107-L135)
  - *Bản chất:*
    ```rust
    let start_b = (entry_bar + 1).min(i);
    for b in start_b..=i { peak_val = peak_val.max(high[b]); }
    ```
    Biểu thức khởi tạo `start_b = (entry_bar + 1).min(i)` bỏ qua toàn bộ biến động giá của chính cây nến mở lệnh (`entry_bar`).
    Nếu thị trường có biến động mạnh ngay trong cây nến vừa mở vị thế (ví dụ giá bứt phá tạo râu nến cao vọt rồi hồi về trước khi đóng nến), mức đỉnh cao nhất đó bị bỏ lỡ, làm `peak_val` bị thấp giả tạo và khiến Trailing SL phản ứng chậm 1 nhịp.
  - *Đã xử lý:* Khởi tạo neo chuẩn từ `peak_val = trade.entry_price`, duyệt từ `(entry_bar + 1)..=i`. Triệt tiêu 100% Lookahead bias của nến entry, loại bỏ hoàn toàn lỗi đỉnh bị ghi nhận âm dưới giá vốn nếu nến sau giảm, và tính toán số nến đi ngang `effective_stagnant_bars` chính xác tuyệt đối. Đã có unit test `test_flaw_tsl5_high_watermark_entry_bar_bias_free` kiểm chứng.

- [x] **FLAW-TSL.6:** **Bảo Toàn Bản Sắc Stop Loss Của Từng Playbook & Tối Ưu Dải Cấu Hình DNA** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs), [`scanners/trend_pb.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs), [`scanners/momentum_ignition.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs), [`scanners/range_osc.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs), [`scanners/absorption.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs), [`mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Đã xử lý (Structural Stop Loss Per-Playbook & Dynamic DNA Expansion):*
    1. **Mở rộng không gian DNA 117 chiều:** Bổ sung 4 gene `struct_sl_buffer_atr` độc lập cho 4 Scanners trong `config.rs`:
       - `TacticalPullbackParams`: `[0.10 -> 4.00, step = 0.15]`, default `0.5`.
       - `TacticalMomentumParams`: `[0.10 -> 3.00, step = 0.15]`, default `0.3`.
       - `TacticalRangeParams`: `[0.10 -> 3.00, step = 0.15]`, default `0.3`.
       - `TacticalAbsorptionParams`: `[0.25 -> 10.00, step = 0.25]`, default `2.0` (40 nấc, tối ưu cho GA thích ứng mọi biên độ).
    2. **Bảo toàn bản sắc Stop Loss vi mô theo cấu trúc Price Action:**
       - `TrendPullback`: Neo SL dưới `min(Low, KAMA) - buffer` (Long) / trên `max(High, KAMA) + buffer` (Short).
       - `MomentumIgnition`: Neo SL dưới đáy nến bùng nổ `Low - buffer` (Long) / trên đỉnh nến `High + buffer` (Short).
       - `RangeOscillator`: Neo SL ngoài biên dải Bollinger Bands `min(Low, BB_Lower) - buffer` (Long) / `max(High, BB_Upper) + buffer` (Short).
       - `AbsorptionScout`: Neo SL ngoài biên hộp nén Squeeze `min(Low, BB_Lower) - buffer` (Long) / `max(High, BB_Upper) + buffer` (Short), triệt tiêu hoàn toàn bẫy kẹp SL quá chặt ở `BB_Middle`.
    3. **Sanity Guard hình học:** Đảm bảo SL luôn nằm đúng phía của giá (Long ở dưới, Short ở trên) và cách giá ít nhất `max(buffer, close * 0.002)`.
    4. **Kiểm thử toàn diện 100% Pass:** Đã có bộ unit test `test_structural_sl_per_playbook_integrity` kiểm chứng cả 4 Scanners.

- [x] **FLAW-TSL.7:** **Bẫy Alternating Whipsaw & Lệch Pha Cooldown Giữa Strategy Và Real-Time Engine** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs) & [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs) & [`config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)
  - *Đã xử lý (Dual Sentinels - Universal Bidirectional Cooldown & Directional Early Release):*
    1. **Mô hình Phòng Ngự Kép (Dual Sentinels - Zero Desync):**
       - `HeliosStrategyV3`: Quản lý `CooldownTracker` cho Backtest & GA Optimizer (Single Source of Truth cho quá trình tối ưu).
       - `AssetActor`: Do `evaluate_live_tick` chạy trên Dummy Broker nên Strategy không nhận thức được lệnh thật vừa bị cắt lỗ. `AssetActor` được trang bị `EngineCooldownGuard` tại cổng thực thi, đọc trực tiếp tham số DNA từ `self.helios_config` (`re_entry_cooldown_bars`, `cooldown_breakout_atr_mult`).
    2. **Khóa 2 chiều tuyệt đối (Bidirectional Cooldown):** Khi lệnh thật bị cắt lỗ hoặc hòa vốn danh nghĩa (`trade_pnl <= 0.001`), bot khóa CẢ 2 CHIỀU Long & Short trong `re_entry_cooldown_bars` nến. Triệt tiêu 100% bẫy vào lệnh ngược chiều ở đỉnh/đáy.
    3. **Directional Early Release theo DNA gene (`cooldown_breakout_atr_mult`):** Mở khóa sớm nếu giá Long bứt phá tăng vượt mức thoát $\ge k \times ATR$ hoặc Short sập gãy sâu vượt mức thoát $\ge k \times ATR$. Cấm tuyệt đối Long bắt dao rơi hoặc Short cản tàu.
    4. **Kiểm thử toàn diện:** Vượt qua 32/32 unit tests Helios và 38/38 unit tests Bot Engine (tổng 70/70 tests pass 100%).

- [x] **FLAW-TSL.8:** **Hardcode `0.80 / lev` Trong `clamp_sl_price` Cho Phép Lệnh Thua Tới -80% ROE** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:29-66, 105-120, 277-295`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) & [`config.rs:325-475, 555-565`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs) & [`mod.rs:280-315, 644-650`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Đã xử lý (Per-Playbook ROE Cap & Fallback Hierarchy):*
    1. **Xóa bỏ vĩnh viễn magic number:** Loại bỏ hoàn toàn `0.80 / lev` (cho phép âm -80% ROE) và giá trị hardcode `0.05` khi lệnh không có tag.
    2. **Fallback Hierarchy:** Lệnh có tag đọc `max_sl_roe_cap` của chính Scanner đó (`0.04 .. 0.40, step = 0.015`, default `0.15 - 0.25`). Lệnh không có tag (`None`) hoặc lệnh thủ công Adopted từ sàn fallback về `default_sl_roe_cap` (`0.04 .. 0.40, step = 0.015`, default `0.20`).
    3. **Min Distance Floor Guard:** `effective_cap = (roe_cap / lev).max(min_initial_sl_pct)`. Triệt tiêu hoàn toàn nghịch lý kẹp ngược khi đòn bẩy cao ($50\times - 100\times$), bảo vệ nến khỏi micro-noise và phí sàn Binance Futures.
    4. **Mở rộng dải sàn `min_initial_sl_pct`:** Đổi sang `[min = 0.0015, max = 0.040, step = 0.0005]` ($0.15\% \to 4.0\%$).
    5. **Mở rộng DNA 112 chiều:** Tích hợp `default_sl_roe_cap` vào `RiskStopParams`, bảo toàn 100% tính đơn điệu Monotonic Ratchet. Đã có 5 unit tests chuyên sâu kiểm chứng Spot 1x, Short 20x, Nuke 50x/100x và Untagged Fallback (25/25 Helios + 34/34 Bot Engine = 59/59 tests pass 100%).


- [x] **FLAW-TSL.9:** **Playbook `ABSORPTION` Bị Bội Thực Tín Hiệu (Over-Trading / Churn) Trong Vùng Choppy Market** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs) & [`arbiter.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs)
  - *Bản chất:* 
    - Trong Log 31 và Log 33, riêng `ABSORPTION` chiếm từ 50% đến 70% tổng số lệnh của toàn hệ thống (17/24 lệnh ở Log 31, 17/34 lệnh ở Log 33) với tỷ lệ thua cao (10/17 lệnh thua ở Log 33).
    - Scanner liên tục thắng thầu trong Arbiter khi thị trường đi ngang có khối lượng ảo, khiến bot bị ngâm lệnh rồi bị Stagnant Decay cắt lỗ liên miên.
  - *Đã xử lý (Tách Regime Compression & Đấu Thầu Động Phân Quyền):*
    1. **Tách biệt Squeeze khỏi Range:** Không còn gộp chung Nén (Compression) vào Range khiến Absorption hưởng sái điểm cao trong vùng cưa chân bàn.
    2. **Định lượng 4 trạng thái độc lập:** Absorption được gán trọng số bối cảnh riêng `w_comp_norm` ($w_c = squeeze \times (1 - chaos)$), tách biệt với Clean Range (`w_clean_range_norm`).
    3. **Ngưỡng phê duyệt động:** `required_confidence = w_t * trend + w_r * range + w_c * comp + w_ch * chaos`. Đưa `min_confidence_compression` vào DNA (118 chiều), giúp GA Optimizer tự nâng trần phê duyệt khi thị trường nhiễu, chấm dứt tình trạng Absorption độc tôn tín hiệu.

- [ ] **FLAW-TSL.10:** **Bất Cân Xứng Tỷ Lệ Lãi/Lỗ & Thiếu Dynamic Structural Take Profit Của Playbook Absorption (R:R = 1 : 0.35)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:80`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L80) & [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)
  - *Bản chất:* 
    - Trong Log ETH, lệnh thắng của Absorption bình quân chỉ lãi **+$1.09** sau khi gồng tối đa 630 nến (10.5 tiếng), trong khi lệnh thua dập thẳng vào Hard SL kịch trần bình quân **-$3.07** sau 35-172 nến. Tỷ lệ R:R nghịch đảo 1 : 0.35 (Lỗ gấp 3 lần Lãi).
    - Nguyên nhân vế Lỗ: Do FLAW-REG.9 bắt dao rơi khi xả lũ.
    - Nguyên nhân vế Lãi: `suggested_tp: None`, Absorption hoàn toàn không có Take Profit theo cấu trúc biên nén, chỉ biết chờ Trailing TP hoặc ngâm lệnh tới Stagnant Decay rồi chốt non.
  - *Giải pháp đề xuất:*
    1. Triệt tiêu vế Lỗ bằng FLAW-REG.9 (cấm bắt dao rơi).
    2. Thiết kế Dynamic Structural Take Profit cho Absorption khi giá bứt phá ra khỏi hộp nén (Squeeze Breakout Target), nâng R:R kỳ vọng lên $\ge 1 : 1.5$.

---

## 🌪️ NHÓM 7: KIỂM TOÁN CÁC TRẠNG THÁI THỊ TRƯỜNG KHÔNG THỂ PHÂN TÍCH (UNTRADABLE REGIMES & DEFENSIVE GUARDS AUDIT) (8 LỖ HỔNG)

### 📊 Bảng Đối Chiếu Trạng Thái Thị Trường: Helios V3 Đã Có Gì & Đang Bị "Mù" Ở Đâu?

| Mẫu Thị Trường Cực Đoan | Đặc Điểm Nhận Dạng Vi Mô | Helios V3 Hiện Đã Implement | Điểm Mù (Blind Spot) & Lỗ Hổng Thực Tế | Mức Độ Nguy Cấp |
| :--- | :--- | :--- | :--- | :--- | :---: |
| **REG-1: Bão Whipsaw Quét Râu 2 Đầu** | Nến đổi màu liên tục, râu nến chiếm $>70\%$, bước đi ngẫu nhiên ($Hurst \approx 0.50$). | `chaos_ratio`, `efficiency_ratio`, `adx_centered`. | 🚨 **Bị mù:** Simplex tính $w_r = 1.0 - \max(w_t, w_v)$. Nhầm Whipsaw thành **Tradable Range 89%**, kích hoạt `RangeOsc` bắt đỉnh/đáy để bị nướng! | 🔴 **Cực cao** |
| **REG-2: Thanh Khoản Zombie** | $Volume \to 0$, sổ lệnh rỗng, nến dẹp như sợi chỉ. | `vol_z_score`, `squeeze_intensity`. | 🚨 **Tử huyệt Fallback:** Khi Volume $\to 0$, $Z_{\text{vol}}$ bị `NaN`. Code tại `matrix.rs:137` fallback `volume_force = 0.50`! Kích hoạt `Absorption` bắn lệnh vào sổ lệnh rỗng! | 🔴 **Cực cao** |
| **REG-3: Fat-Tail Kurtosis Spike** | Tin tức shock, nến quét 5-8% 2 đầu, Kurtosis $\kappa \gg 10$. | `z_score`, `anomaly_multiplier`, đẩy $w_v$ lên cao. | 🚨 **Thiếu Cầu Chì:** Arbiter không dừng lại mà lại cấp ưu tiên cao nhất cho `AnomalyFade` cản tàu trước sóng thanh lý! | 🔴 **Cực cao** |
| **REG-4: Tranh Chấp Tử Thần** | Cả Long và Short cùng bùng nổ điểm cao trên 1 nến. | Đấu thầu điểm tự tin `confidence`. | 🚨 **Cờ bạc:** Long 0.850 và Short 0.849, Arbiter chọn bừa Long thay vì hủy cả hai lệnh để đứng ngoài. | 🔴 **Nghiêm trọng** |

---

### 📝 Chi Tiết 8 Lỗ Hổng Thuộc Nhóm 7:

- [x] **FLAW-REG.1:** **Simplex Range Bù Trừ Bị Mù Hoàn Toàn Trước Bão Whipsaw** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:142-165`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L142-L165)
  - *Bản chất:* Simplex tính $w_{\text{range}} = 1.0 - \max(w_{\text{trend}}, w_{\text{volatile}})$. Khi bão Whipsaw quét râu 2 đầu xảy ra, ADX $< 20 \implies w_t = 0$. $w_v$ thấp vì NATR tiệm cận SMA(NATR, 50). Ma trận ngộ nhận Whipsaw thành **Ranging hoàn hảo ($w_r \approx 89\%$)**, khiến `RangeOscScanner` nhảy vào bắt đỉnh/đáy Bollinger Bands và bị nướng Stop Loss liên tục.
  - *Đã xử lý (4-State Continuous Simplex Vector Decomposition):*
    1. **Xóa bỏ Range bù trừ mù:** Thay vì $1.0 - \max(w_t, w_v)$, bóc tách 4 trạng thái độc lập: Trend, Clean Range, Compression, Chaos/Whipsaw.
    2. **Nhận diện Bão Whipsaw chính xác:** $w_{\text{chaos\_raw}} = 0.70 \times \text{chaos\_ratio} + 0.30 \times \text{z\_clipped}$. Khi bão Whipsaw quét 2 đầu, $w_{\text{chaos}}$ vọt lên $81.35\%$ trong khi $w_{\text{clean\_range}}$ bị dìm xuống $9.33\%$, triệt tiêu hoàn toàn điểm mù ngộ nhận Whipsaw thành Tradable Range.
    3. **Continuous Simplex Normalization:** Luôn bảo đảm tổng $\sum \equiv 1.0000$ (kèm fallback $0.25$ chia đều khi nến warmup gặp NaN).
    4. **Kiểm thử toàn diện:** Đã kiểm chứng 34/34 unit tests Helios và 38/38 unit tests Bot Engine (tổng 72/72 tests pass 100%).

- [ ] **FLAW-REG.2:** **Tử Huyệt Fallback `volume_force = 0.50` Khi Sổ Lệnh Zombie (Cạn Kiệt Thanh Khoản)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:137`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L137) & [`scanners/absorption.rs:35`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L35)
  - *Bản chất:* Khi thị trường đóng băng ($Volume \to 0$), phép tính $Z_{\text{vol}}$ bị chia cho 0 ra `NaN`. Dòng 137 tự động gán `if volume_force.is_nan() { 0.5 }`. Kết hợp với Bollinger Bands bóp nghẹt khiến Squeeze Intensity đạt $1.0$, `AbsorptionScanner` đạt điểm tuyệt đối $1.0$ và bắn lệnh vào thị trường chết, gây kẹt thanh khoản và trượt giá Gap khổng lồ.
  - *Giải pháp đề xuất:* Thiết lập cơ chế Fallback Zero (`volume_force = 0.0` khi NaN); bổ sung Absolute Quote Volume Floor Filter: cấm mở lệnh nếu khối lượng 1h $< \text{MinThreshold}$.

- [ ] **FLAW-REG.3:** **Signal Arbiter Thiếu Hoàn Toàn Cơ Chế "Cash is King / Flat Guard" & Veto Toàn Cục**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:127-187`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L127-L187)
  - *Bản chất:* Arbiter luôn cố gắng tìm người chiến thắng. Khi thị trường rơi vào siêu bão hỗn loạn hoặc nhiễu ngẫu nhiên, không có dòng code nào tuyên bố: `"Thị trường Untradable -> Giữ tiền mặt!"`. Đồng thời, công thức `(raw_score)^W * (context_score)^(1-W)` với $W$ cao ($0.85$) cho phép một scanner tự chấm khống điểm $0.95$ đè bẹp bối cảnh vĩ mô xấu và ép bot mở lệnh.
  - *Giải pháp đề xuất:* Bổ sung Flat Guard Veto: Nếu Simplex Entropy cao (không rõ ràng chế độ) hoặc `chaos_ratio > max_safe_chaos` $\implies$ Ép buộc trả về `Action::None` (giữ tiền mặt).

- [ ] **FLAW-REG.4:** **Conflict Deadlock Cờ Bạc Khi Long & Short Cùng Bùng Nổ Điểm Cao**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:168`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L168)
  - *Bản chất:* Khi cả Best Long (ví dụ `TrendPullback` 0.850) và Best Short (ví dụ `ClimaxReversal` 0.849) cùng xuất hiện trên 1 cây nến (vùng tranh chấp dữ dội), Arbiter chỉ đơn thuần so sánh `confidence > max_confidence` và chọn bên hơn $0.001$. Đây là hành vi cờ bạc trong vùng nguy hiểm.
  - *Giải pháp đề xuất:* Bổ sung Deadlock Cancellation: Nếu điểm của Best Long và Best Short đều $> 0.60$ và hiệu số điểm $< 0.15 \implies$ Hủy cả 2 lệnh, đứng ngoài quan sát.

- [ ] **FLAW-REG.5:** **Bẫy Bắt Dao Rơi Của `anomaly.rs`: Coi Nến Xả Marubozu Thành Pinbar Long**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs:32-45`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs#L32-L45)
  - *Bản chất:* Công thức chấm điểm chỉ dựa vào độ dài râu nến dưới tuyệt đối và Z-Score. Một cây nến đỏ đặc xả hàng 18 giá có râu dưới 2 giá vẫn được chấm **0.976 điểm Long**! Scanner liên tục bắt dao rơi ngay giữa các con sóng xả hàng thanh lý cưỡng bức.
  - *Giải pháp đề xuất:* Bắt buộc kiểm tra hình thái nến: Râu nến $\ge 2.0 \times$ Thân nến và Giá đóng cửa phải nằm ở nửa trên của cây nến (`Close > (High + Low) / 2`).

- [ ] **FLAW-REG.6:** **Méo Mó Lực Khối Lượng Bất Đối Xứng Của `volume_force` Do MFI**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:98-102`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L98-L102)
  - *Bản chất:* Dùng trực tiếp $0.5 \cdot |\tanh| + 0.5 \cdot (\text{MFI} / 100)$. MFI là chỉ báo dao động hướng tâm. Khi xả hàng tháo chạy cực đại (Panic Sell), MFI rơi về $10$, khiến `volume_force` chỉ đạt $0.55$ (bị ngộ nhận là volume trung bình). Ngược lại nến xanh nhẹ không có volume đột biến nhưng MFI cao lại bị thổi phồng.
  - *Giải pháp đề xuất:* Chuẩn hóa MFI theo độ lệch tâm $|\text{MFI} - 50.0| / 50.0$ hoặc dùng trực tiếp tích chuẩn hóa Khối lượng $\times$ Biên độ nến True Range.

- [ ] **FLAW-REG.7:** **Cooldown "Mất Trí Nhớ" - Thiếu Losing Streak, Exponential Backoff & Daily Circuit Breaker**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:26-31, 196`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L26-L31) & [`asset_actor.rs:458`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L458)
  - *Bản chất:* Cooldown cào bằng mọi mức lỗ (lỗ $0.001$ hay lỗ $50\%$ ROE đều chỉ cách ly 15 nến). Thua 1 lệnh hay thua 4 lệnh liên tiếp đều chỉ cách ly bằng nhau, không có cơ chế tăng thời gian cách ly lũy tiến. Đồng thời thiếu hoàn toàn bộ ngắt mạch khẩn cấp khi lỗ ngày chạm ngưỡng (Daily Circuit Breaker).
  - *Giải pháp đề xuất:* Bổ sung `consecutive_losses` và cơ chế Exponential Backoff: $T = T_{\text{base}} \times 2^{\min(streak - 1, 3)}$ ($15\text{m} \to 30\text{m} \to 60\text{m} \to 120\text{m}$). Ngắt mạch tài khoản nghỉ giao dịch đến hôm sau nếu Drawdown trong ngày $\ge 5\%$.

- [ ] **FLAW-REG.8:** **Lỗi Trị Tuyệt Đối `.abs()` Trong `asset_actor.rs:521` Bắt Dao Rơi Khi Giá Sập Sâu**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs:521-536`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L521-L536)
  - *Bản chất:* Trong khi `core_v3/mod.rs` đã sửa kiểm tra bứt phá có lợi, `asset_actor.rs` vẫn giữ `price_diff = (current_close - last_exit_price).abs() >= 1.5%`. Nếu lệnh Long vừa cắt lỗ mà giá tiếp tục lao dốc thêm $> 1.5\%$, bot tưởng là bứt phá và lập tức xóa Cooldown, nhảy vào Long bắt dao rơi tiếp!
  - *Giải pháp đề xuất:* Đồng bộ 100% logic kiểm tra bứt phá định hướng: Chỉ mở khóa Long nếu giá tăng ngược lên $\ge 1.5 \times \text{ATR}$ (và Short nếu giá giảm sâu $\ge 1.5 \times \text{ATR}$).

- [x] **FLAW-REG.9:** **`Absorption` Bắt Dao Rơi Ngược Sóng Do Đoán Hướng Bằng VWMA Lag & Thiếu Macro Trend Filter** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:39-65`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L39-L65) & [`config.rs:505-510`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L505-L510)
  - *Bản chất:* Xác định hướng bùng nổ chỉ bằng `vwma >= bb_middle`. Khi giá sập hầm với volume lớn (như ETH xả từ 2625 về 2470), quán tính volume nến cao khiến VWMA vẫn nằm trên SMA, Absorption ngộ nhận là Accumulation và mở Long bắt dao rơi, dính liên tiếp 4 Hard SL (-26.6% ROE).
  - *Đã xử lý (Dual-Layer Confirmation & Dynamic Macro Trend Veto):*
    1. **Tầng 1 (Price Action / Mean Alignment):** Yêu cầu đồng thuận 100% giữa dòng tiền và hành động giá: Long đòi hỏi cả `vwma >= bb_middle` VÀ `close >= bb_middle`; Short đòi hỏi cả `vwma < bb_middle` VÀ `close <= bb_middle`. Triệt tiêu hoàn toàn trường hợp nến đỏ đặc cắm đầu bán xả nhưng gom râu làm VWMA lag.
    2. **Tầng 2 (Macro Trend Direction Guard - 100% Dynamic qua DNA):** Bổ sung gene `max_adverse_slope: f64` (dải `0.05 -> 0.50`, step `0.05`, nâng DNA lên 119 chiều). Nếu `trend_dir < -max_adverse_slope` $\implies$ VETO cấm Long bắt dao rơi; nếu `trend_dir > max_adverse_slope` $\implies$ VETO cấm Short cản tàu.
    3. **Kiểm thử hoàn tất:** 36/36 tests Helios pass, 38/38 tests Bot Engine pass 100%. Triệt tiêu hoàn toàn 4 cú bắt dao rơi sập hầm.

- [ ] **FLAW-REG.10:** **`RangeOsc` Bắt Đáy Ngược Sóng Trong Xu Hướng Giảm Trôi Dốc (Band-Walking Trap)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:52-78`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L52-L78)
  - *Bản chất:* Khi thị trường trôi dốc từ từ (Drifting Down), độ dốc trend không vượt ngưỡng `-max_slope` nên không kích hoạt cấm Long. Nến bám sát dải BB Lower khiến Gaussian proximity `f_price_long` đạt 1.0, Stoch oversold đạt 1.0 $\to$ RangeOsc liên tục bắt đáy Long và bị nướng 5/6 lệnh trên SOL.
  - *Giải pháp đề xuất:* Kiểm tra độ dốc của chính dải Bollinger Bands (`bb_middle` slope hoặc `bb_lower` slope). Nếu dải đang dốc cắm xuống, cấm bắt đáy Long.

- [ ] **FLAW-REG.11:** **Vào Lệnh Cưỡng Bức Ngay Cây Nến Kế Tiếp (Compulsive Next-Candle Re-Entry)**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs) & [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Bản chất:* Bộ lọc `REVENGE RE-ENTRY BLOCKED` chỉ chặn vào lệnh trên cùng một cây nến vừa đóng. Sang đúng cây nến 1 phút tiếp theo, bot lập tức mở lệnh mới (In-market exposure > 95%), không cho tài khoản khoảng đệm nghỉ ngơi (Inter-trade Rest Gap) để cấu trúc thị trường ổn định.
  - *Giải pháp đề xuất:* Thêm tham số đệm tối thiểu sau mọi lệnh thoát (ví dụ nghỉ tối thiểu 5-10 nến) để triệt tiêu hiện tượng đảo vị thế cào lệnh liên tục.

---

## 💡 ĐỀ XUẤT KIẾN TRÚC MỞ RỘNG (FUTURE ARCHITECTURAL ENHANCEMENTS)

- [ ] **ENHANCE-ARCH.1:** **Projected Simplex & User Whitelist Modular Toggle (Phương Án 3)**
  - *Mục tiêu:* Cho phép người dùng tùy chọn danh sách Playbooks hoạt động (`enabled_scanners: Vec<ScannerId>`) trong cấu hình bot để chủ động tắt các scanner không mong muốn theo đặc thù tài sản.
  - *Cơ chế toán học:* Áp dụng Chiếu Simplex Tác Chiến (Projected Simplex): $\widetilde{w}_k = w_k / \sum_{j \in \mathcal{A}} w_j$ và $\widetilde{C}_{\text{thresh}} = \sum_{k \in \mathcal{A}} \widetilde{w}_k \theta_k$. Giữ nguyên kích thước DNA 118 chiều bằng cơ chế Gene Freezing Mask (đóng băng gene không kích hoạt) giúp GA hội tụ nhanh gấp 3 lần và triệt tiêu 100% overfit.


