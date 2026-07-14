# Khối 6: Error Handler (Quản lý Lỗi)

## 1. Tổng quan
Hệ thống giao dịch tự động không thể tránh khỏi các lỗi liên quan đến kết nối mạng, API, hay dữ liệu không hợp lệ. Mục tiêu của Khối 6 là cung cấp một bộ máy quản lý lỗi tập trung, giúp:
- Bao bọc toàn bộ các lỗi từ bên thứ ba (đặc biệt là Binance SDK) thành một cấu trúc `BotError` thống nhất.
- Đảm bảo hệ thống không bao giờ bị Crash (Panic) giữa chừng do `unwrap()` hay `expect()`.
- Hỗ trợ log lỗi chi tiết để thuận tiện trong việc gỡ lỗi (Debugging) và báo động khẩn cấp (Emergency Cmd).

## 2. Thiết kế Enum `BotError`
`BotError` sẽ là một enum implements `std::error::Error` (có thể tận dụng crate `thiserror` để làm code sạch hơn) với các phân nhóm sau:

- **ApiError(code: i32, msg: String)**: Các lỗi trả về trực tiếp từ Binance REST/WS API (ví dụ: `-2011` Unknown order).
- **NetworkError(String)**: Lỗi liên quan đến mất kết nối mạng, timeout, hoặc đứt WebSocket.
- **MarginError(String)**: Lỗi phát sinh trước/trong khi đặt lệnh do không đủ số dư ví (Balance) hoặc bị giới hạn bởi đòn bẩy.
- **MarketFilterError(String)**: Lệnh bị từ chối do vi phạm quy tắc sàn (ví dụ: kích thước quá nhỏ `MIN_NOTIONAL`, giá nằm ngoài khoảng cho phép `PRICE_FILTER`).
- **ParseError(String)**: Lỗi trong quá trình parse/deserialize JSON từ luồng dữ liệu hoặc REST response.
- **StateError(String)**: Lỗi liên quan đến trạng thái cục bộ của Bot chưa sẵn sàng (ví dụ: cố gắng đặt lệnh khi InitialSync chưa hoàn tất).
- **Internal(String)**: Các lỗi nội bộ khác.

## 3. Kiến trúc Mapping Lỗi từ Binance
Binance SDK có thể ném ra nhiều loại lỗi (ví dụ như `ConnectorError` trong `binance-connector-rust`). Khối Error Handler sẽ có các hàm phụ trợ để `map_err` chuyển các lỗi gốc này sang `BotError`. Cụ thể là bóc tách mã lỗi (error_code) và tin nhắn (error_msg) từ Binance để đưa vào `ApiError(code, msg)`.

## 4. Quy định viết code
- **TUYỆT ĐỐI KHÔNG** dùng `.unwrap()` hay `.expect()` trong luồng chạy chính của bot.
- Bất kì hàm nào có khả năng thất bại đều phải trả về `Result<T, BotError>`.
- Các nhánh lỗi phải được catch và in ra qua hệ thống `tracing::error!` hoặc `warn!`, và có thể kích hoạt tín hiệu `EmergencyCmd::PanicCloseAll` nếu là lỗi nghiêm trọng.
