# Ghi Chú Kiểm Toán Định Lượng: Rủi Ro Tầng Thực Thi & Lỗ Hổng Tiền Thật Trên Binance Futures
## TRADING BOT EXECUTION RISK, CONCURRENCY & LIVE VULNERABILITIES

> **Dự án:** Trading Bot Execution Platform (`trading_bot/bot_engine`)  
> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/notes/`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/)  
> **Mã tài liệu:** `NOTE-13-EXECUTION-RISK`  
> **Ngày lập:** 14/10/2026  
> **Trạng thái:** Báo cáo kiểm toán độc lập cấp cao (Line-by-line Verified)

---

## 🏛️ 1. TỔNG QUAN RỦI RO THỰC THI (THE EXECUTION PARADOX)

Nếu tầng chiến lược (`helios`) là bộ não ra quyết định lý thuyết trong môi trường nến đóng không có độ trễ, thì tầng thực thi (`trading_bot`) là cơ bắp tác chiến trực tiếp với sàn Binance Futures — nơi có độ trễ mạng (latency), trượt giá (slippage), lệnh algo chuyên biệt, và hiểm họa mất kết nối mạng.

> 💣 **KẾT QUẢ KIỂM TOÁN TỔNG THỂ:**  
> Toàn bộ các chốt chặn toán học nội bộ của nến đóng (Monotonic Sequence Counter, 2-Piece Assembler, NaN Starvation Guard) **đã được fix rất tốt**. Tuy nhiên, **5 tử huyệt giao thoa với sàn Binance Live vẫn tồn tại nguyên vẹn trong code**, trong đó có 2 lỗ hổng cấp bách có thể làm **cháy toàn bộ ví người dùng hoặc bán tháo sạch vị thế khi khởi động bot**.

---

## 🚨 2. TỬ HUYỆT 1: PHÌNH TO VỐN SÀN THỰC THI (CAPITAL BREACH VULNERABILITY)

* **Vị trí code:** 
  - [`trading_bot/bot_engine/src/execution/live_engine.rs:133-143`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L133-L143)
  - [`trading_bot/bot_engine/src/execution/order_executor.rs:284-285`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L284-L285)

### 2.1. Bản Chất Lỗ Hổng
Trong kiến trúc thiết kế, bot quản lý vốn độc lập từng cặp tiền với hạn mức được cấp cơ sở: `allocated_capital: 60.0` (\$60 USDT/cặp).
Tuy nhiên, khi luồng WebSocket nhận được bản tin cập nhật tài khoản `AccountUpdate`:
```rust
// live_engine.rs:133
let usable_capital = self.account_state.available_balance;
// ...
executor_clone.execute_action(&symbol, action_clone, current_price, usable_capital, position_amt, leverage).await
```
Và tại `order_executor.rs:284`:
```rust
let risk_usdt = available_balance * risk_percentage;
let target_qty = self.calculate_order_size(risk_usdt, current_price, leverage)?;
```

### 2.2. Hậu Quả Thực Chiến Ngoài Đời Thật
- Giả sử người dùng nạp vào ví Binance Futures tổng cộng **\$10,000 USDT** (để dự phòng hoặc chạy nhiều bot).
- Người dùng cấu hình cặp ETHUSDT chỉ được phép trade với ngân sách \$60.
- Khi Strategy duyệt mở lệnh với `percent_per_trade = 0.25` (25% margin):
  $$\text{Margin Sử Dụng} = 25\% \times \$10,000 = \mathbf{\$2,500\text{ USDT}} \quad (\text{thay vì } 25\% \times \$60 = \$15)!$$
  $$\text{Notional Value (20x)} = \$2,500 \times 20 = \mathbf{\$50,000\text{ USDT}}!$$
- Trường `pub allocated_capital: f64` trong `ExecutionCore` **hoàn toàn bị bỏ xó**.
- **Nếu chạy 4 cặp tiền song song:** Cả 4 cặp đều thi nhau rút cạn toàn bộ số dư ví \$10,000, phóng đại rủi ro lên hàng trăm lần so với thiết kế!

### 2.3. Giải Pháp Khắc Phục (Hard Cap Enforcement)
Tại `live_engine.rs:133`, bắt buộc phải áp dụng hàm chặn trần an toàn:
```rust
let usable_capital = self.account_state.available_balance.min(self.core.allocated_capital);
```
Không một cặp tiền nào được phép tính toán khối lượng vượt quá số vốn được phân bổ cho chính nó.

---

## ⚡ 3. TỬ HUYỆT 2: SPLIT-BRAIN RISK ENGINE & PHANTOM SL DESYNC

* **Vị trí code:** 
  - [`trading_bot/bot_engine/src/execution/live_engine.rs:117-160, 204`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L117-L160)
  - [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs:248-252`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L248-L252)

### 3.1. Bản Chất Lỗi Lệch Pha Trạng Thái
1. Ở môi trường Paper (`PaperExecutionEngine`), khi nhận `Action::UpdateTradeSlTp`, `PaperTrader` cập nhật ngay `pos.current_sl_price = new_sl`.
2. **Ở Môi Trường Live (`BinanceLiveExecutionEngine`):**
   - Khi nhận `Action::UpdateTradeSlTp`, lệnh gửi sang Binance REST API qua `tokio::spawn`, **nhưng KHÔNG BAO GIỜ cập nhật biến `self.live_position.current_sl_price`**!
   - Biến này chỉ xuất hiện duy nhất 1 lần khi nhận `PositionUpdate` từ WebSocket: `current_sl_price: None` (dòng 204).
3. **Vòng Lặp Phân Liệt Não (Split-Brain Feedback Loop):**
   - Ở mọi cây nến tiếp theo, `HeliosBridge::evaluate_live_tick` đọc `live_position.current_sl_price` (luôn là `None`) và nạp vào Strategy struct `Trade` với `trade.sl_price = None`.
   - Trong `RiskManager::manage_trailing_stop`: Vì `trade.sl_price` là `None`, điều kiện `calculated_sl > trade.sl_price.unwrap_or(NEG_INFINITY)` **luôn luôn bằng TRUE**!
   - Strategy liên tục tưởng rằng vị thế chưa có Stop Loss, nên **mỗi cây nến đóng đều phát lệnh dời SL**.
   - `OrderExecutor` dùng cache `active_sl_prices` chặn lại bằng log `🛡️ [MONOTONIC RATCHET BLOCKED]`. Nhưng Strategy bên trong hoàn toàn bị mù, tiếp tục spam API ở các nến sau và tính toán sai lệch High Watermark!

### 3.2. Giải Pháp Khắc Phục
Override hàm `execute_action_ctx` trong `BinanceLiveExecutionEngine`: Ngay khi phát lệnh dời SL, cập nhật ngay lập tức `self.live_position.current_sl_price = Some(new_sl)` để Strategy nhận biết được trạng thái SL thực tế của sàn.

---

## 🕳️ 4. TỬ HUYỆT 3: KHOẢNG TRỐNG NAKED SL 150MS & SILENT FAILURE TRAP

* **Vị trí code:** [`trading_bot/bot_engine/src/execution/order_executor.rs:190-236`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L190-L236)

### 4.1. Quy Trình Dời SL Hiện Tại
```rust
// Bước 1: Hủy sạch toàn bộ Algo SL cũ trên sàn
self.rest_client.cancel_all_algo_open_orders(p_cancel).await;

// Bước 2: Ngủ 150ms chờ Binance cập nhật
tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

// Bước 3: Đặt SL mới (thử tối đa 3 lần)
for attempt in 1..=3 { ... }

// Bước 4: BẪY NGUY HIỂM: Silent Failure
if !success {
    error!("❌ [{}] Đã thử 3 lần nhưng không thể đặt Trailing SL mới", symbol);
}
return Ok(()); // <--- Trả về Ok(()) như bình thường!
```

### 4.2. Hai Kịch Bản Thảm Họa Thực Chiến
1. **Khoảng trống Naked 150ms:** Trong 150ms (hoặc kéo dài hàng chục giây nếu mạng lag, Binance nghẽn API 429/504), vị thế trên sàn **hoàn toàn không có lệnh cắt lỗ nào bảo vệ**. Nếu một cây nến Flash Crash quét qua đúng thời điểm này, tài khoản sẽ bị thanh lý sạch.
2. **Bẫy Im Lặng Khi Đặt Lệnh Thất Bại (Silent Failure Trap):** Nếu sau 3 lần retry đặt SL mới thất bại (ví dụ do margin thay đổi hoặc Binance bảo trì): Hàm ghi 1 dòng log `error!`, **sau đó trả về `Ok(())`**! Bot Engine ngỡ rằng lệnh SL cũ vẫn còn, nhưng thực tế **SÀN ĐÃ BỊ HỦY SẠCH LỆNH BẢO VỆ TỪ BƯỚC 1**. Vị thế hoàn toàn mồ côi ngoài thị trường!

### 4.3. Giải Pháp Khắc Phục
- **Chốt chặn Emergency Panic Close:** Nếu sau 3 lần retry đặt SL mới thất bại $\to$ lập tức kích hoạt lệnh khẩn cấp Market Close toàn bộ vị thế (`reduce_only: true`) ngay lập tức để bảo toàn vốn, tuyệt đối không để vị thế trôi nổi mồ côi.
- **Atomic SL Replacement:** Đặt lệnh SL mới với `clientAlgoId` riêng, xác nhận Binance phản hồi trạng thái `NEW` thành công rồi mới gửi lệnh hủy SL cũ.

---

## 🧹 5. TỬ HUYỆT 4: BẪY BÁN THÁO KHI KHỞI ĐỘNG (CLEAN SLATE TRAP) & BẪY TUỘT SL

* **Vị trí code:** 
  - [`trading_bot/bot_engine/src/execution/account_manager.rs:175-178`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/account_manager.rs#L175-L178)
  - [`trading_bot/bot_engine/src/execution/live_engine.rs:204`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L204)
  - [`trading_bot/bot_engine/src/execution/order_executor.rs:31, 122`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L31)

### 5.1. Bẫy Bán Tháo Sạch Vị Thế Khi Khởi Động (`account_manager.rs:175-178`)
```rust
// CLEAN SLATE: Dọn dẹp bàn cờ (hủy lệnh + đóng vị thế)
let _ = self.cancel_all_orders(symbol).await;
let _ = self.force_close_all_positions(symbol, position_amt).await;
```
Khi bot khởi động và đồng bộ trạng thái ban đầu, đoạn code này tự động gửi lệnh Market Close bán tháo toàn bộ vị thế đang mở trên sàn! Người dùng đang có vị thế gồng lãi lớn chỉ vì khởi động lại bot mà bị cắt ngang xương!

### 5.2. Bẫy Tuột Stop Loss (SL Retraction Khi Khởi Động Lại)
- Khi bot restart:
  - Cache `active_sl_prices` trong `OrderExecutor` là `HashMap` rỗng trong RAM.
  - Bot **không hề query Open Algo Orders trên Binance** để biết SL trên sàn đang là bao nhiêu.
  - Cây nến đầu tiên chạy: Strategy thấy giá hiện tại và tính mức SL ban đầu (thấp hơn nhiều so với mức SL Trailing đã khóa lãi trên sàn).
  - Vì trong RAM `old_sl` là `None`, hàm `evaluate_ratchet_sl` chấp thuận mức SL mới này!
  - `OrderExecutor` gửi lệnh hủy SL cũ và đặt lại mức SL thấp tịt ban đầu, làm mất trắng toàn bộ thành quả khóa lãi trước đó!

### 5.3. Giải Pháp Khắc Phục (True Adopt & Protect)
1. Xóa bỏ hoàn toàn 2 dòng `cancel_all_orders` và `force_close_all_positions` trong `account_manager.rs`.
2. Khi bot khởi động: Gọi REST API Binance `GET /fapi/v1/openOrders` (hoặc `openAlgoOrders`), trích xuất mức giá Stop Market đang chạy và nạp vào `active_sl_prices` của `OrderExecutor` và `live_position.current_sl_price`.

---

## ⚖️ 6. NGHỊCH LÝ CẦU DAO COOLDOWN: SAI LỆCH 1:1 GIỮA BACKTEST VÀ LIVE

* **Vị trí code:** [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:272-274`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L272-L274) vs [`trading_bot/bot_engine/src/controller/asset_actor.rs:231-262, 646-657`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L231-L262).

### 6.1. Bản Chất Phân Kỳ Giữa Hai Môi Trường
- Trong `core_v3/mod.rs:272`: Ghi chú *"Cầu Dao Cooldown VETO được độc quyền sở hữu bởi Bot Engine. Helios Strategy không tự ý can thiệp Cooldown"*.
- Do đó, trong **Backtest Broker và GA Optimizer**: Sau một lệnh thua ở nến $k$, nếu nến $k+1$ có tín hiệu đẹp, hệ thống lập tức mở lệnh ngay (0 nến phạt).
- Nhưng trong **Live / Paper của Bot Engine (`AssetActor`)**: Lệnh thua kích hoạt phạt $15 \sim 45$ nến. Trong 15 nến tiếp theo, `AssetActor` **VETO 100% TÍN HIỆU**, bất kể tín hiệu đó đẹp đến đâu!

### 6.2. Hậu Quả Toán Học
- Thuật toán di truyền GA tìm ra DNA tối ưu trên giả định **không có Cooldown**.
- Khi mang DNA này ra chạy thực chiến, bot bị Cầu Dao Engine trói chân, bỏ lỡ các lệnh thắng hồi phục hoặc né được các lệnh thua tiếp theo.
- **Đường cong vốn thực tế và số lượng lệnh ngoài đời bị lệch pha hoàn toàn so với phòng thí nghiệm Walk-Forward.**

---

## 📋 7. KẾ HOẠCH HÀNH ĐỘNG KHẮC PHỤC TẦNG THỰC THI (ACTION ITEMS)

| Ưu Tiên | Nhiệm Vụ | File Mục Tiêu | Giải Pháp Kỹ Thuật |
| :---: | :--- | :--- | :--- |
| **P0** | **Khóa Chết Phình To Vốn Sàn** | `live_engine.rs:133` | `usable_capital = available_balance.min(allocated_capital)`. |
| **P0** | **Xóa Bẫy Bán Tháo Khởi Động** | `account_manager.rs:175-178` | Xóa bỏ `cancel_all_orders` và `force_close_all_positions`. |
| **P0** | **Vá Silent Failure & Naked SL** | `order_executor.rs:233-236` | Kích hoạt Market Close khẩn cấp nếu 3 lần retry đặt SL mới thất bại. |
| **P1** | **Khử Lỗi Mù SL Của Strategy** | `live_engine.rs:117-160` | Cập nhật `live_position.current_sl_price` ngay khi phát lệnh dời SL. |
| **P1** | **True Adopt & Protect Khi Restart** | `account_manager.rs`, `live_engine.rs` | Query Open Algo Orders từ Binance, kế thừa mức SL đang sống trên sàn. |
| **P1** | **Marked-to-Market Paper Engine** | `paper_engine.rs:114` | Trả về `paper_trader.get_total_equity(current_price)` bao gồm Floating PnL. |
| **P2** | **Đồng Bộ Mô Hình Cooldown** | `broker.rs` & `core_v3/mod.rs` | Đưa logic Cooldown vào Backtest Broker để GA tối ưu hóa chuẩn xác 1:1. |
