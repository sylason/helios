# Kế hoạch & Đặc tả Kỹ thuật: Thực thi Lệnh Live trên Binance (Binance Executor)

Tài liệu này mô tả chi tiết quy trình, kiến trúc, và các ràng buộc khi chuyển đổi từ Paper Trader (Giao dịch ảo cục bộ) sang **Binance Executor** (Giao dịch trực tiếp trên sàn thật hoặc Testnet).

## 1. Khái quát (Overview)
`BinanceExecutor` là module thay thế (hoặc nâng cấp) cho `PaperTrader`. Nhiệm vụ của nó là nhận tín hiệu (`Action`) từ `HeliosStrategy` và chuyển đổi thành lệnh REST API hợp lệ để gửi lên Binance USD-M Futures.

Mục tiêu chính:
- **Độ chính xác cao**: Tuân thủ nghiêm ngặt các quy tắc làm tròn kích thước lệnh (`stepSize`, `minQty`) và mức giá (`tickSize`) do sàn Binance quy định.
- **Tính toán Ký quỹ (Margin) linh hoạt**: Hỗ trợ chuẩn xác mô hình Isolated Margin, tính toán chi phí trước khi vào lệnh.
- **An toàn (Fail-safe)**: Có cơ chế theo dõi Rate Limit và đối chiếu số dư để không bao giờ "cháy" tài khoản vì lỗi lập trình.

## 2. Quản lý Ràng buộc & Cỡ Lệnh (Asset Constraints)

Binance có các quy tắc khắt khe về việc đặt lệnh. Nếu lệnh bị lẻ (ví dụ 0.12345 ETH) nhưng sàn chỉ cho phép 3 số thập phân (`stepSize = 0.001`), API sẽ trả về lỗi `LOT_SIZE`. 

### Quy trình Tích hợp Cấu hình Sàn (`ExchangeInfo`):
1. **Khởi tạo**: Khi bot bắt đầu chạy (hoặc định kỳ mỗi 24h), gọi API `GET /fapi/v1/exchangeInfo`.
2. **Trích xuất Data**: Tìm object chứa `symbol == "ETHUSDT"`. Tìm trong mảng `filters`:
   - `PRICE_FILTER`: Lấy `tickSize` (ví dụ `0.01`).
   - `LOT_SIZE`: Lấy `stepSize` (ví dụ `0.001`) và `minQty`.
3. **Bộ lọc Kích thước (Size Filter)**:
   Mọi `size` tính ra từ Helios phải đi qua hàm làm tròn xuống:
   ```rust
   // Ví dụ minh họa thuật toán làm tròn
   let rounded_size = (raw_size / step_size).floor() * step_size;
   if rounded_size < min_qty { return Err("Size quá nhỏ"); }
   ```

## 3. Ước tính Chi phí & Quản lý Đòn bẩy (Leverage & Margin)

Do Helios v2/v3 có thể sử dụng cơ chế Dynamic Leverage (ví dụ Nuke Mode đẩy lên 100x), bot cần phải:
1. **Thiết lập Đòn bẩy qua API**: Gọi `POST /fapi/v1/leverage` với `leverage = X` để sàn Binance cập nhật tỷ lệ ký quỹ trước khi mở vị thế.
2. **Cấu hình Isolated Margin**: Gọi `POST /fapi/v1/marginType` truyền `marginType = ISOLATED`.
3. **Ước tính Chi phí (Cost Estimation)**: 
   Trước khi gửi lệnh Market/Limit, bot phải tính toán:
   `Cost = (Size * Current_Price) / Leverage`
   Nếu `Cost > Available_Balance`, bot sẽ từ chối gửi lệnh để tránh Spam API gây lỗi rác.

## 4. Quản lý Sinh tồn & Rate Limit (Survival Management)

Module `BinanceExecutor` không thể hoạt động mù quáng. Cần bổ sung các lớp Fail-safes:
1. **Giám sát Rate Limit (Weight Monitor)**: 
   - Đọc các Headers từ HTTP Response của Binance (ví dụ `X-MBX-USED-WEIGHT-1M`).
   - Nếu `USED-WEIGHT > 2000` (Binance giới hạn 2400/phút), lập tức đình chỉ luồng thực thi (sleep) 30-60 giây để tránh bị cấm IP.
2. **Tự động Phục hồi WebSocket (Auto-Reconnect)**:
   - Module `binance_stream.rs` cần được bọc trong một vòng lặp vĩnh cửu. Nếu stream bị đóng (Disconnect) hoặc không có bản tin PING/PONG hợp lệ, tiến trình sẽ tự động thiết lập lại kết nối mới sau 5 giây.
3. **Báo cáo sự cố**: Khi có lỗi mạng hoặc API từ chối, `AssetActor` sẽ cập nhật trạng thái lỗi lên Event Bus để lưu Log hoặc gửi cảnh báo.

## 5. Lộ trình Triển khai (Roadmap)
- **Phase 1**: Xây dựng module `MarketInfo` để tự động kéo và parse `exchangeInfo` + `stepSize`.
- **Phase 2**: Viết unit test cho các bộ lọc (Filter) làm tròn size và price.
- **Phase 3**: Khởi tạo `BinanceExecutor`, cắm API Key thật (của hệ thống Testnet) và thử bắn lệnh Market `LONG/SHORT` nhỏ nhất.
- **Phase 4**: Tích hợp cơ chế theo dõi Rate Limit và Auto-Reconnect mạng.
