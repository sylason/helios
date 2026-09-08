# Khối 3: Order Executor (Thực thi Lệnh)

## 1. Mục tiêu
`OrderExecutor` là trái tim thực thi của hệ thống giao dịch trực tiếp (Live Execution). Nó có nhiệm vụ lấy Tín hiệu Giao dịch (Action) từ `Strategy Engine`, tính toán lại Khối lượng (Position Size) dựa trên Số dư hiện tại (Account Balance) & Risk Management, và gửi lệnh Mua/Bán trực tiếp lên Binance thông qua giao thức WebSocket (WS Trade API) để đạt tốc độ thực thi nhanh nhất.

## 2. Kiến trúc & Dependency
Khối này nằm tại `src/execution/order_executor.rs` và phụ thuộc vào:
- **Khối 1 (Market Filter):** Để làm tròn số thập phân của Giá (Tick Size) và Số lượng (Step Size).
- **Khối 2 (Account State):** Để lấy `available_balance` và `position_amt` hiện tại nhằm tính toán kích thước lệnh an toàn.
- **Khối 5 (WS Manager):** Cung cấp kết nối `Websocket API` (cụ thể là đối tượng connection) để bắn lệnh `new_order`.
- **Khối 6 (Error Handler):** Để xử lý các rủi ro văng lỗi khi gửi lệnh thất bại và trả về `BotError`.

## 3. Luồng hoạt động (Execution Flow)
1. **Tiếp nhận Tín hiệu:** Nhận `Action` (VD: `GoLong`, `GoShort`, `CloseAll`) từ `strategy.next()`.
2. **Kiểm tra An toàn (Pre-flight Check):**
   - Lệnh có bị MarketFilter chặn không? (VD: Không được mua dưới 5 USDT).
   - Account có đủ Margin không?
3. **Tính toán Khối lượng:** 
   - Size = `(Available Balance * Risk % * Leverage) / Current Price`.
   - Cắt gọt bằng `Step Size` của MarketFilter.
4. **Gửi lệnh qua WS:**
   - Dùng `NewOrderParams::builder()` của SDK.
   - Định dạng lệnh là `MARKET` (Mua/Bán ngay lập tức với giá thị trường).
   - Đẩy lệnh qua WebSocket API thay vì REST API để tối ưu mili-giây.
5. **Ghi Log Lịch sử:**
   - Đẩy sự kiện khớp lệnh vào SQLite (`TradeHistory` table) để lưu trữ.

## 4. Rủi ro & Cách khắc phục (Graceful Degradation)
- **Lỗi Mạng lúc Gửi:** Retry tối đa 3 lần. Nếu vẫn hỏng, thông báo Telegram và hủy bỏ.
- **Vượt quá Rate Limit (Khối 4 báo động):** Cấm gửi lệnh, vào trạng thái `Safety Mode`.
- **Không dùng `.unwrap()`:** Mọi hàm tính toán rủi ro (chia cho 0) hoặc gọi API đều trả về `Result<(), BotError>`.
