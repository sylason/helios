# Chiến lược Phân chia Môi trường (Environment Separation)

Tài liệu này quy định kiến trúc phân tách giữa môi trường Phát triển (R&D) và môi trường Chạy thực tế (Live/Demo).

## 1. Vấn đề hiện tại
- `bot_engine` đang được liên kết trực tiếp tới thư mục `helios` thông qua cấu hình `Cargo.toml`.
- Bất kỳ thay đổi mang tính thử nghiệm nào (như phát triển Helios v2, v3, hay thay đổi cấu trúc dữ liệu) đều có rủi ro làm gián đoạn hoặc phá hỏng tính ổn định của `bot_engine` nếu vô tình biên dịch lại.
- Quá trình chạy Demo cần duy trì sự ổn định 24/7 trong nhiều ngày hoặc nhiều tuần.

## 2. Giải pháp Kiến trúc (Copy & Isolate)

Thay vì dùng chung mã nguồn, ta sẽ áp dụng cơ chế **Vùng cách ly (Isolated Zone)** cho Demo.

1. **Khởi tạo Sandbox cho Bot**:
   - Nhân bản (copy) thư mục `strategies_backtests/helios` phiên bản ổn định (v2) sang một thư mục hoàn toàn mới tên là `v2_trading_bot_demo`.
   - Thư mục mới này sẽ đóng vai trò như "Trái tim" cố định của `bot_engine`.
   
2. **Cập nhật Liên kết**:
   - Sửa file `bot_engine/Cargo.toml`:
     Từ `helios = { path = "../../strategies_backtests/helios" }` 
     Thành `helios = { path = "../../strategies_backtests/v2_trading_bot_demo" }`.

3. **Lợi ích**:
   - **Tự do Sáng tạo**: Bạn có thể tiếp tục nhào nặn `helios` v2 và v3 trong thư mục gốc. Thêm bớt biến số, thay đổi thuật toán, tối ưu thông số tùy thích.
   - **Ổn định Live**: Tiến trình Demo chạy độc lập với thư viện `v2_trading_bot_demo`, không bị ảnh hưởng bởi những đoạn code lỗi trong quá trình nghiên cứu của bạn.
   - **Quy trình Deploy rõ ràng**: Khi `helios` phiên bản mới đã hoàn hảo, bạn chỉ việc ghi đè nội dung của nó sang thư mục `v2_trading_bot_demo`, biên dịch lại `bot_engine`, và bản Live sẽ được nâng cấp một cách an toàn nhất.

## 3. Lộ trình Triển khai
1. Chạy lệnh copy thư mục `helios` sang `v2_trading_bot_demo`.
2. Đổi tên/Cập nhật các tham chiếu bên trong (nếu cần thiết để tránh xung đột tên crate, hoặc giữ nguyên và chỉ sửa path ở `bot_engine`).
3. Khởi động lại `bot_engine` để nó lock vào thư viện mới.
