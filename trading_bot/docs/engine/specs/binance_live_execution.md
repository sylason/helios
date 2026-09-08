# Thiết kế Kỹ thuật: Binance Live Execution

## Tổng quan Mục tiêu
Module này chịu trách nhiệm **duy nhất**: chuyển đổi tín hiệu mua/bán từ `HeliosStrategy` thành lệnh thực tế trên sàn Binance, đồng thời đảm bảo bot có thể **chạy liên tục 24/7 trong nhiều tuần** mà không cần can thiệp thủ công. Đây là xương sống của toàn bộ Demo Run.

---

## Khối 1: Bộ Lọc Quy chuẩn Sàn (Market Info & Filters)

### Mục đích
Binance sẽ từ chối lệnh (lỗi `LOT_SIZE` hoặc `PRICE_FILTER`) nếu giá trị không đúng quy chuẩn. Khối này là "Người Gác Cổng" (Gatekeeper) đầu tiên.

### Cơ chế hoạt động
- **Lấy cấu hình sàn:** Khi bot khởi động (và định kỳ làm mới mỗi 24h), gọi `GET /fapi/v1/exchangeInfo` để lấy metadata của `ETHUSDT`.
- **Các tham số quan trọng cần lưu lại:**
  - `LOT_SIZE.stepSize`: Bước nhảy tối thiểu của số lượng coin (ví dụ `0.001` ETH).
  - `LOT_SIZE.minQty`: Số lượng coin tối thiểu mỗi lệnh.
  - `PRICE_FILTER.tickSize`: Bước nhảy tối thiểu của giá (ví dụ `0.01` USDT).
- **Thuật toán làm tròn (Rounding)**:
  ```
  rounded_qty = floor(raw_qty / stepSize) * stepSize
  rounded_price = floor(raw_price / tickSize) * tickSize
  if rounded_qty < minQty → reject_order("Kích thước lệnh quá nhỏ")
  ```
- **Lưu trữ:** Dữ liệu cấu hình này lưu trong RAM (struct `SymbolFilter`), không cần DB.

### Quyết định thiết kế
- Cập nhật mỗi 24h thay vì mỗi lần đặt lệnh để tiết kiệm API weight.
- Nếu lần gọi `exchangeInfo` thất bại, dùng giá trị cũ (cached) thay vì crash.

---

## Khối 2: Quản lý Vốn & Đòn bẩy (Margin & Leverage)

### Mục đích
Tránh lỗi "Insufficient Margin" gây crash lệnh, và đảm bảo mọi lệnh chỉ dùng đúng lượng vốn được phép theo cấu hình.

### Cơ chế hoạt động
1. **Thiết lập Isolated Margin** (một lần khi khởi động):
   - Gọi `POST /fapi/v1/marginType` với `symbol=ETHUSDT, marginType=ISOLATED`.
   - Mục đích: Tách biệt rủi ro, lỗ của ETH không ảnh hưởng đến balance của các cặp khác.

2. **Cấu hình Đòn bẩy** (mỗi khi có thay đổi từ chiến lược):
   - Gọi `POST /fapi/v1/leverage` với `symbol=ETHUSDT, leverage=X`.

3. **Kiểm tra Số dư trước khi đặt lệnh**:
   - Gọi `GET /fapi/v2/account` để lấy `availableBalance`.
   - Tính: `required_margin = (rounded_qty * current_price) / leverage`
   - Điều kiện: `availableBalance >= required_margin * 1.05` (thêm 5% buffer để tránh trường hợp giá chênh).
   - Nếu không đủ: Ghi log cảnh báo, bỏ qua lệnh (không crash).

---

## Khối 3: Cỗ máy Thực thi (Order Executor)

### Mục đích
Chuyển hóa `Action` từ chiến lược thành lệnh Binance USD-M Futures.

### Cấu trúc Pure WebSocket
Thay vì dùng HTTP POST, ta sử dụng `DerivativesTradingUsdsFuturesWsApi` từ SDK.
1. Khởi tạo một `connection` vĩnh viễn với WebSocket API.
2. Gửi lệnh qua kết nối này. Độ trễ giảm từ 100-300ms xuống còn **10-30ms**.

### Ánh xạ Tín hiệu → Lệnh qua WS API

| Tín hiệu Helios | Hành động WS API |
|---|---|
| `Action::Buy` (chưa có vị thế) | `connection.new_order(side=BUY, positionSide=LONG)` |
| `Action::Sell` (chưa có vị thế) | `connection.new_order(side=SELL, positionSide=SHORT)` |
| `Action::Sell` (đang giữ Long) | `connection.new_order(side=SELL, positionSide=LONG, reduceOnly=true)` |
| `Action::Buy` (đang giữ Short) | `connection.new_order(side=BUY, positionSide=SHORT, reduceOnly=true)` |
| `Action::Hold` | Không làm gì |

### Loại lệnh
- **Phase Testnet**: Dùng **Market Order** (`type=MARKET`) để đơn giản hóa debug.
- **Phase Live** (tương lai): Nghiên cứu thêm Limit Order với giá chốt gần bid/ask.

### Xử lý Response
- Sau khi gọi API, parse `orderId` và ghi Log thành công.
- Nếu API trả về lỗi, gọi sang **Khối 6 (Error Handler)**.

---

## Khối 4: Giám sát Sinh tồn - Event-Driven Rate Limit & User Data

### Mục đích
Binance cấm IP nếu gọi REST API quá nhiều. Bằng cách dùng WebSocket, ta gần như loại bỏ rủi ro này, nhưng vẫn cần theo dõi. Đồng thời, thay vì "hỏi" số dư liên tục, ta lắng nghe sự kiện trả về.

### User Data Stream (Event-Driven)
- Bot kết nối vào `user_data_stream` qua WebSocket.
- Khi lệnh Mua/Bán khớp, Binance đẩy về sự kiện `ORDER_TRADE_UPDATE`.
- Khi số dư thay đổi (do cắn phí, trả margin), Binance đẩy về `ACCOUNT_UPDATE`.
- Bot cập nhật trực tiếp `PositionState` trong RAM thông qua các event này mà không cần gọi API `GET /account`.

### Rate Limit Monitor (Dành cho các REST Call hãn hữu)
- Các hàm như lấy `exchangeInfo` hay bù nến (Gap Filler) vẫn phải gọi REST.
- Đọc header `X-MBX-USED-WEIGHT-1M`. Cảnh báo nếu > 1920, Đình chỉ REST nếu > 2200. Việc đặt lệnh qua WS API không bị ảnh hưởng bởi giới hạn REST này.

---

## Khối 5: Phục hồi Mạng lưới (WebSocket Auto-Reconnect)

### Mục đích
Demo chạy trong nhiều tuần, sẽ không thể tránh khỏi gián đoạn mạng ngắn. Khối này giúp bot tự "thức dậy" mà không cần bạn khởi động lại.

### Cơ chế hoạt động
Bot sẽ quản lý 3 kết nối WebSocket riêng biệt:
1. **Market Data Stream**: K-line 1m.
2. **WebSocket API**: Gửi lệnh Mua/Bán (`connection`).
3. **User Data Stream**: Lắng nghe `ORDER_TRADE_UPDATE`.

- Toàn bộ các kết nối này được bọc trong vòng lặp `loop { ... }`.
- Nếu kết nối 1 đứt: Gọi REST bù nến, rồi reconnect WS 1.
- Nếu kết nối 2 đứt: Reconnect lại WS API client, không ảnh hưởng đến dữ liệu.
- Giữ kết nối User Stream: Gửi `connection.keepalive_user_data_stream()` tự động.

---

## Khối 6: Quản lý Lỗi (Error Handling Bus)

### Mục đích
Không để bất kỳ lỗi đơn lẻ nào làm crash toàn bộ tiến trình Demo đang chạy.

### Phân loại lỗi & Chiến lược xử lý

| Mã lỗi Binance | Tên lỗi | Chiến lược |
|---|---|---|
| `-1003` | Too many requests | Sleep 60s, rồi thử lại |
| `-2010` | New order rejected (Insufficient margin) | Ghi Log, bỏ qua lệnh |
| `-1021` | Timestamp out of sync | Tự đồng bộ thời gian, thử lại |
| `-1130` | Invalid order quantity | Ghi Log chi tiết, skip |
| `Network Timeout` | Mất kết nối | Retry 3 lần (backoff), rồi Auto-Reconnect |
| Lỗi không xác định | Unexpected panic | `catch_unwind` hoặc `Result` chaining, ghi log, tiếp tục |

### Thiết kế: Không có lỗi Panic trong Demo
- Toàn bộ các hàm gọi API phải trả về `Result<T, BotError>`.
- Không được dùng `.unwrap()` hoặc `.expect()` trong code production.
- Mỗi lỗi phải được ghi đầy đủ: timestamp, loại lỗi, context (đang làm gì lúc bị lỗi).

---

## Khối 7: Chiến lược Kiểm thử (Testing & Verification Strategy)

### Mục đích
Hệ thống Live Trading liên quan trực tiếp đến tài chính, do đó mã nguồn phải được đảm bảo bằng các bài kiểm thử tự động thay vì chỉ kiểm tra thủ công. Quá trình kiểm thử sẽ diễn ra theo 3 lớp.

### Các Lớp Kiểm thử (Test Layers)
1. **Unit Tests (Kiểm thử Cục bộ):**
   - Viết test bằng #[test] bên trong từng module (VD: ate_limit_monitor.rs, market_info.rs).
   - Mục đích: Đảm bảo thuật toán làm tròn (Rounding), tính toán Margin, và logic xử lý State chạy đúng độc lập. Không cần mạng.
2. **Integration Tests (Kiểm thử Tích hợp):**
   - Tạo thư mục 	ests/ riêng biệt.
   - Mục đích: Mô phỏng chuỗi sự kiện. Gửi tín hiệu ảo (Mock Signal) từ strategy và xác minh OrderExecutor sinh ra đúng chuẩn lệnh trước khi gửi lên sàn.
3. **Live Testnet Execution (Kiểm thử Realtime trên Sàn Ảo):**
   - Chạy với biến môi trường BINANCE_USE_TESTNET=true.
   - Kết nối trực tiếp vào Binance Testnet qua WebSocket.
   - Bơm một cây nến thực (hoặc mock nến) để kích hoạt lệnh.
   - Yêu cầu sàn phải phản hồi thành công (Sự kiện ORDER_TRADE_UPDATE trả về khớp lệnh trên Testnet).
