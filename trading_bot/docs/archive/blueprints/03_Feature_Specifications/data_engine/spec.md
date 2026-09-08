# Data Engine Specification

## 1. Trách nhiệm (Responsibilities)
- Kết nối và thu thập dữ liệu Nến 1 phút (K-Line 1m) từ sàn Binance.
- Tự động lấp đầy các khoảng trống (Gap) do mất kết nối hoặc khi vừa mở app.
- Lắng nghe `markPrice` để phục vụ Trailing Stoploss.

## 2. Loại bỏ `aggTrade`
Khác với thiết kế cũ, hệ thống sẽ **KHÔNG** subscribe `aggTrade` và **KHÔNG** tự tổng hợp nến.
- **Lý do**: Tiết kiệm tài nguyên (CPU/RAM) và thời gian phát triển logic gộp nến. Việc sử dụng trực tiếp WebSocket K-line 1m của Binance tuy có độ trễ cực nhỏ so với tự tính, nhưng hoàn toàn đáp ứng được nhu cầu chiến lược hiện tại.

## 3. Luồng khởi động (Trình tự chống Gap)
Để đảm bảo toàn vẹn dữ liệu (không có khoảng trống thời gian giữa nến lịch sử và nến real-time), luồng khởi động bắt buộc phải tuân theo thứ tự sau:

1. **Bước 1**: Mở ngay lập tức kết nối WebSocket K-line 1m (`wss://fstream.binance.com/ws/{symbol}@kline_1m`) và đệm dữ liệu nhận được vào RAM.
2. **Bước 2**: Truy vấn DuckDB xem timestamp của cây nến cuối cùng được lưu là lúc nào.
3. **Bước 3**: Gọi REST API (`GET /fapi/v1/klines`) yêu cầu dữ liệu lịch sử từ timestamp cuối cùng đó cho đến thời điểm hiện tại.
4. **Bước 4**: Ghi dữ liệu REST vào DuckDB.
5. **Bước 5**: Lấy dữ liệu đệm từ RAM (của WebSocket) ghi nối tiếp vào DuckDB. Xóa buffer và bắt đầu đẩy thẳng nến từ WS vào DB/Strategy.

## 4. Quản trị vòng đời dữ liệu
- Khi khởi động, kích hoạt một Task dọn dẹp (Cleanup Task): `DELETE FROM kline_1m WHERE timestamp < (now - 1 tháng 3 ngày)`.
- Điều này giữ cho dung lượng file `.duckdb` nhỏ gọn do tần suất tắt/mở app của user cá nhân rất cao.
