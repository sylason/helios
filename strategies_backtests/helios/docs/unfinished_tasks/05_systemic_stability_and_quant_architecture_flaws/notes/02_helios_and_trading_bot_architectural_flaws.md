# BẢN PHÂN TÍCH CHUYÊN SÂU: CÁC ĐIỂM YẾU KIẾN TRÚC GIAO THOA GIỮA HELIOS VÀ TRADING BOT
## HELIOS & TRADING BOT ARCHITECTURAL FLIEWS & BOUNDARY MISMATCH

> **Dự án:** Helios V3 (Strategy Core) & Trading Bot (Execution Platform)  
> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/)  
> **Ngày lập:** 11/10/2026  
> **Mục tiêu:** Mổ xẻ các lỗ hổng kiến trúc, xung đột logic và vùng mù vận hành giữa phân hệ Chiến lược (`helios`) và phân hệ Thực thi (`trading_bot`).

---

## 🏛️ TỔNG QUAN RANH GIỚI HỆ THỐNG

`ferro_trade` đóng vai trò là động cơ toán học backtest và bộ tối ưu hóa GA. Mọi quyết định logic vào lệnh, quản trị rủi ro và bảo vệ tài khoản trong thực chiến (Paper & Live) thuộc về sự phối hợp giữa:
1. **`helios`**: Bộ não phân tích định lượng (Sensors, Matrix, Scanners, Arbiter, Risk Manager). Được thiết kế theo triết lý **Stateless** để phục vụ chạy backtest / GA parallel tốc độ cao.
2. **`trading_bot`**: Nền tảng thực thi thời gian thực (Actor Model, Binance WebSocket/REST, SQLite Store, PaperTrader, LiveEngine). Mang bản chất **Stateful** sâu sắc.

Chính sự lệch pha giữa **"Stateless Strategy"** và **"Stateful Execution"** đã tạo ra 5 điểm yếu kiến trúc cốt tử sau:

---

## 🚨 1. XUNG ĐỘT THẨM QUYỀN QUẢN TRỊ RỦI RO (SPLIT-BRAIN RISK ENGINE)
* **Vị trí code:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) vs [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs) & [`order_executor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs).
* **Bản chất nghiệp vụ:** Hiện tại cả hai phân hệ đều tự ý cài đặt logic tính toán Trailing Stop Loss độc lập:
  - `helios::risk::RiskManager` tính TSL dựa trên vòng lặp nến `highs[entry_bar..=i]` và hệ số ATR.
  - `PaperTrader` và `OrderExecutor` lại tự quản lý biến `current_sl_price` với bộ lọc Monotonic Ratchet Guard riêng.
* **Hậu quả:** Gây ra hiện tượng "Phân liệt bộ não" (Split-Brain). Khi chạy Live hoặc Paper, mức Stop Loss thực tế được dời có thể lệch so với cách `RiskManager` tính toán lúc Backtest, làm phá vỡ tính đúng đắn 1:1 giữa mô phỏng và đời thực.
* **Định hướng giải pháp:** Độc quyền hóa thẩm quyền quyết định mức giá SL/TP về cho `helios::risk`. Tầng Bot Engine chỉ là cơ quan thừa hành, tuyệt đối không tự ý đẻ thêm logic tính toán mức giá dừng lỗ/chốt lời.

---

## 🚨 2. TÊ LIỆT PHẢN XẠ TRONG KHUNG NẾN (INTRA-CANDLE BLINDNESS)
* **Vị trí code:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs) (`on_new_candle`).
* **Bản chất nghiệp vụ:** Cả `helios` và `AssetActor` hiện tại chỉ "thức giấc" và đánh giá tín hiệu khi **nến 1m ĐÃ ĐÓNG** (`is_closed == true`).
* **Hậu quả:** 
  - Trong suốt 60 giây của cây nến, nếu xảy ra sự kiện thiên nga đen (Flash Crash, cá mập xả lũ volume đột biến, gãy cấu trúc hỗ trợ nghiêm trọng) ở giây thứ 10:
  - Hệ thống hoàn toàn bị **"mù" và "điếc"** trong 50 giây còn lại. Vị thế phải trơ mắt chịu đựng lỗ nở rộng cho đến hết giây thứ 60 mới bắt đầu xử lý.
* **Định hướng giải pháp:** Xây dựng cơ chế **Tick-Level Emergency Circuit Breaker**:
  - `AssetActor` lắng nghe luồng WebSocket MarkPrice/TradePrice thời gian thực.
  - Nếu phát hiện giá vi phạm ngưỡng cấm khẩn cấp hoặc biến động cực đoan $\to$ gửi lệnh khẩn cấp Market Close ngay lập tức (Zero-delay), không chờ nến 1m đóng.

---

## 🚨 3. THIẾU TẦNG QUẢN TRỊ RỦI RO DANH MỤC ĐA CẶP (ZERO PORTFOLIO RISK GUARDIAN)
* **Vị trí code:** Kiến trúc tổng thể `trading_bot/bot_engine/src/controller/`.
* **Bản chất nghiệp vụ:** Mỗi `AssetActor` vận hành độc lập theo mô hình "Ốc đảo biệt lập" (Siloed Actor). Actor của ETHUSDT không hề biết Actor của BTCUSDT hay SOLUSDT đang làm gì.
* **Hậu quả:**
  - Khi thị trường toàn ngành biến động mạnh (Systemic Market Shock), tất cả các Actor đều đồng loạt nhận diện cơ hội và mở lệnh full margin $20\text{x}$.
  - Nếu bot chạy 4 cặp tiền, tổng đòn bẩy danh mục bị phóng đại lên **$80\text{x}$**, chỉ cần thị trường giật ngược $1.2\%$ là toàn bộ tài khoản bị Margin Call thanh lý sạch.
* **Định hướng giải pháp:** Xây dựng **`PortfolioRiskManager` (Thống Đốc Quản Trị Danh Mục)** ở tầng Bot Engine:
  - Khống chế trần đòn bẩy danh mục: $\sum \text{Allocated Margin} \le \text{Max Portfolio Leverage}$.
  - Giới hạn số lượng vị thế mở đồng thời (ví dụ: tối đa 2 vị thế trên toàn sàn).
  - Phân tích tương quan danh mục (Correlation Exposure): Cấm mở thêm vị thế Long Altcoin nếu BTC đang cắm đầu giảm.

---

## 🚨 4. KHOẢNG TRỐNG "MỒ CÔI LỆNH" TRÊN BINANCE FUTURES (NAKED SL GAP)
* **Vị trí code:** [`trading_bot/bot_engine/src/execution/order_executor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L192-L235).
* **Bản chất nghiệp vụ:** Quy trình dời Trailing SL hiện tại trên sàn Binance Live là:
  $$\text{Hủy lệnh Algo SL cũ} \longrightarrow \text{Chờ 150ms} \longrightarrow \text{Đặt lệnh Algo SL mới}$$
* **Hậu quả:** 
  - Trong 150ms đó (hoặc kéo dài hàng chục giây nếu mạng lag, Binance nghẽn API HTTP 429/504): Vị thế trên sàn hoàn toàn **KHÔNG CÓ LỆNH BẢO VỆ NÀO (Naked Position)**!
  - Nếu Flash Crash quét qua đúng thời khắc này, tài khoản sẽ cháy không có phanh cứu hộ.
* **Định hướng giải pháp:**
  - Áp dụng cơ chế **Atomic SL Replacement (Đặt Mới Trước - Hủy Cũ Sau)**: Đặt thành công lệnh SL mới trên Binance (nhận mã `orderId` và trạng thái `NEW`) rồi mới gửi lệnh hủy SL cũ.
  - Kèm chốt chặn an toàn: Nếu đặt SL mới thất bại sau 3 lần retry $\to$ kích hoạt **Emergency Panic Close** ngay lập tức.

---

## 🚨 5. BẪY TIẾP QUẢN VỊ THẾ KHI BOT RESTART ("ADOPT & PROTECT" SL RETRACTION)
* **Vị trí code:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs), [`live_engine.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs).
* **Bản chất nghiệp vụ:** Khi bot bị crash hoặc restart giữa lúc trên sàn Binance đang có vị thế gồng lãi lớn (ví dụ Long ETH giá \$2400, giá hiện tại \$2500, SL sàn đã dời lên \$2480):
* **Hậu quả:**
  - Bot khởi động lại với RAM rỗng, không nhớ đỉnh cũ `peak_val`.
  - Nếu khởi tạo lại với `peak_val = entry_price` (\$2400) $\to$ Bot sẽ hủy lệnh SL \$2480 trên sàn và đặt lại SL ban đầu xa tít ở \$2380 (**Bẫy Tuột Stop Loss - SL Retraction**), biến vị thế đã khóa chắc lãi thành vị thế chịu rủi ro lỗ.
* **Định hướng giải pháp:** Cơ chế **True Adopt & Protect**:
  - Khi khởi động: Query trực tiếp toàn bộ Open Algo Orders trên Binance.
  - Kế thừa và bảo lưu mức giá SL đang sống trên sàn, tuân thủ nguyên tắc Monotonic Ratchet Guard: SL chỉ tiến chứ không bao giờ được lùi.
