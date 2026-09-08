# 🚨 MASTER TODO: 40 LỖ HỔNG CRITICAL (NGUY KỊCH) TOÀN HỆ SINH THÁI

> **Mục tiêu tài liệu:** Quản trị tập trung toàn bộ 40 lỗi mức độ **CRITICAL** (nguy cơ cháy tài khoản, sập bot, hoặc sai lệch PnL nghiêm trọng).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/TODO_CRITICAL.md`  
> **Trạng thái tổng quan:** **11/40 lỗi đã được sửa & kiểm thử thành công**, **29/40 lỗi đang chờ xử lý**.

---

## 📊 Bảng Tiến Độ Tổng Quan Theo Cụm Chức Năng

| Cụm chức năng | Tổng số Critical | Đã sửa | Còn lại | Mức độ rủi ro chính |
| :--- | :---: | :---: | :---: | :--- |
| **Nhóm 1: Khớp Lệnh, Quản Trị Rủi Ro & Sàn Binance** | 8 | 1 | 7 | Naked Position trên sàn, mất Stop Loss, kẹt lệnh |
| **Nhóm 2: Vòng Đời Actor, Cooldown & Hot-Swap DNA** | 6 | 2 | 4 | Đóng băng bot, cào lệnh trả thù, mất dữ liệu RAM |
| **Nhóm 3: Chiến Lược Helios V3, Scanners & Arbiter** | 9 | 5 | 4 | DNA Dead Zone, dìm điểm scanner, đóng nhầm chiều |
| **Nhóm 4: Toán Học Sensors, Chỉ Báo & MTF Buffer** | 6 | 0 | 6 | Lấy nhầm giá thành volume, sập Simplex NaN |
| **Nhóm 5: Động Cơ GA Optimizer & Hạ Tầng Mạng** | 11 | 3 | 8 | Vô hiệu hóa LHS, WebSocket ngủ quên, sập số thập phân |
| **TỔNG CỘNG** | **40** | **11** | **29** |  |

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

- [ ] **FLAW-4.1:** **"DNA Dead Zone" Trong Arbiter: >50% Quần Thể GA Ra 0 Trades**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:102-140`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L102-L140) & [`config.rs:245`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L245)
  - *Bản chất:* Công thức `confidence = score * regime_weight * multiplier` nhân 3 phân số $\le 1.0$ nên hầu như không bao giờ vượt qua 0.45. Khi GA chọn `min_confidence >= 0.50` (>50% không gian DNA), cá thể bị bất khả thi toán học và ra 0 trades!
  - *Giải pháp đề xuất:* Chuẩn hóa lại công thức confidence về thang đo chuẩn hoặc thu hẹp dải DNA `min_confidence` về miền khả thi $[0.15 \dots 0.45]$.

- [x] **FLAW-4.2:** **Take Profit Không Validate Dẫn Đến Đóng Nhầm Chiều (Wrong-Side TP)** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs)
  - *Đã xử lý:* Đã chuẩn hóa và validate bắt buộc cho Take Profit trong `range_osc.rs`: Long TP $\ge \text{Close} \times 1.002$ và Short TP $\le \text{Close} \times 0.998$, triệt tiêu hoàn toàn rủi ro đảo chiều ăn non hoặc vừa vào lệnh đã cắn TP lỗ.

- [x] **FLAW-4.3:** **Lỗi Trị Tuyệt Đối `.abs()` Làm Bot Bắt Dao Rơi Khi Giá Sập Hầm** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Đã xử lý:* Đã xóa bỏ hoàn toàn `.abs()`, bổ sung kiểm tra định hướng bứt phá có lợi `is_favorable_breakout`: Chỉ mở khóa Cooldown Long nếu giá bứt phá TĂNG NGƯỢC LÊN $\ge 1.5$ ATR (và Short nếu giá sập sâu $\ge 1.5$ ATR). Kèm unit test xác minh `test_cooldown_falling_knife_blocked`.

- [ ] **FLAW-4.4:** **Nhận Diện Sai Lệnh Thắng / Thua (False Win) Khi Nến Rút Chân**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:180-189`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L180-L189)
  - *Bản chất:* Tính PnL bằng `current_close - entry_price` thay vì giá cắt lỗ thực tế. Nếu Long bị cắn SL ở râu nến dưới nhưng cuối nến rút chân tăng, code nhận diện SAI là lệnh thắng $\rightarrow$ Không kích hoạt Cooldown và đu đỉnh mua lại ngay đỉnh rút chân!
  - *Giải pháp đề xuất:* Nhận diện PnL dựa trên `last_trade.exit_price` (hoặc sự kiện đóng lệnh thực tế từ Engine).

- [ ] **FLAW-4.5:** **Lỗi "Double Tanh" Dìm Điểm Tê Liệt Reversal & Anomaly Scanners**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:35`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L35) & [`anomaly.rs:39`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs#L39)
  - *Bản chất:* `price_z_score` vốn đã qua `tanh` nén về $(-1.0, 1.0)$. Scanner lại chia tiếp cho $z_{thresh} (2.5 \dots 3.0)$ rồi `tanh` lần 2. Điểm $f_z$ tối đa chỉ đạt $0.32$ dù thị trường sập 10-sigma, làm tê liệt 2 scanner này.
  - *Giải pháp đề xuất:* Giữ Z-Score gốc dạng tuyến tính trước khi đưa vào scanner hoặc bỏ lần nén `tanh` thứ hai.

---

## 📐 NHÓM 4: TOÁN HỌC SENSORS, CHỈ BÁO & MTF BUFFER (6 LỖ HỔNG)

- [ ] **FLAW-3.2:** **`volume_force` Lấy Nhầm Z-Score GIÁ Thay Vì KHỐI LƯỢNG**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:94-101`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L94-L101)
  - *Bản chất:* `tanh_scale(&sensors.z_score, 0.5)` truyền vào Z-Score của GIÁ thay vì Khối lượng. Biến `volume_lookback` bị bỏ hoang.
  - *Giải pháp đề xuất:* Bổ sung mảng `vol_z_score` vào `HeliosSensors`, tính Z-Score trên Volume và truyền đúng vào ma trận.

- [ ] **FLAW-3.3:** **Sập Simplex Normalization (Tổng Trọng Số Vọt Lên 200%) Khi Gặp `NaN`**
  - *Vị trí:* [`ferro_trade/src/indicators/calculate/bulk.rs:1647`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1647) & [`matrix.rs:143-151`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L143-L151)
  - *Bản chất:* `chaos_ratio` sinh ra `NaN` làm mẫu số `sum` thành `NaN`. Biểu thức `NaN.max(1e-5)` ép mẫu số về `1e-5`, khiến $w_{trend} = 1.0, w_{range} = 1.0$ (tổng = 200%), làm tê liệt Arbiter.
  - *Giải pháp đề xuất:* Xử lý an toàn NaN trong `chaos_ratio` và kẹp trọng số `w_volatile_raw = 0.0` nếu là NaN.

- [ ] **FLAW-3.4:** **Cache Collision Trong `IndicatorType::KamaSlope`**
  - *Vị trí:* [`ferro_trade/src/indicators/identifier.rs:119`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L119) & [`factory/bulk.rs:74-87`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs#L74-L87)
  - *Bản chất:* Định danh `KamaSlope` hoàn toàn thiếu trường `atr_period`. Khi GA hoặc các scanner gọi với các `atr_period` khác nhau, lần gọi sau bị trúng cache sai của lần trước.
  - *Giải pháp đề xuất:* Bổ sung `atr_period: usize` vào enum `IndicatorType::KamaSlope`.

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

- [ ] **FLAW-2.1:** **21 Tham Số Ma (Phantom Genes) Chiếm 19.1% DNA**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs:80-530`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L80-L530)
  - *Bản chất:* 21 trường cấu hình trong DNA được GA tối ưu và lưu DB nhưng mã nguồn runtime hoàn toàn không đọc hoặc bị ghi đè bởi hằng số cứng. Lãng phí 20% tài nguyên tính toán GA.
  - *Giải pháp đề xuất:* Kết nối các gen ma vào runtime hoặc cắt bỏ khỏi vector DNA để giảm không gian tìm kiếm.

- [ ] **FLAW-2.2:** **Xung Đột Đòn Bẩy Giữa JSON 20x Và DNA 1x-50x**
  - *Vị trí:* [`helios_optimizer_config.json:5`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_optimizer_config.json#L5) & [`config.rs:455`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L455)
  - *Bản chất:* JSON khóa cứng tài khoản sàn 20x (`margin_rate = 0.05`), nhưng DNA cho GA chọn 1x-50x. Nếu GA chọn 1x: Stop Loss bị nới ra 80%, trong khi sàn 20x thanh lý ở mức -5%!
  - *Giải pháp đề xuất:* Khóa cứng `leverage = 20.0` trong DNA hoặc đồng bộ đòn bẩy sàn theo DNA.

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
