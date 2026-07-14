# 🛡️ Architecture: Execution & Risk Management

Thiết kế các cơ chế đặt lệnh an toàn, chốt lời/lỗ nhanh và quản lý đa sàn.

---

## 1. Unified Exchange Interface (Rust Trait)
Để hỗ trợ đa sàn trong tương lai (Binance, MT5...), Bot sử dụng một lớp trừu tượng:
- **`ExchangeTrait`:** Định nghĩa các hàm chuẩn: `get_balance()`, `place_order()`, `cancel_all()`, `get_mark_price()`.
- **Adapters:** Mỗi sàn sẽ có một bản triển khai (Implementation) riêng để chuyển mã lệnh của App thành API sàn tương ứng.

## 2. Dedicated Observer Task (Thần hộ mệnh)
Để đạt tốc độ chốt lỗ cao nhất, hệ thống tách biệt luồng logic và luồng giám sát giá:
- **Nhiệm vụ:** Chỉ chuyên trách việc so sánh `Mark Price` với `Trailing Stoploss`.
- **Ưu tiên:** Có mức ưu tiên CPU cao nhất và chạy trong vòng lặp siêu ngắn (Tick-level).
- **Phản ứng:** Gửi lệnh đóng vị thế ngay lập tức nếu giá chạm ngưỡng, không cần thông qua Strategy Engine.

## 3. Cơ chế Hedge & Crash Response
Khi thị trường xảy ra biến động cực đoan (Flash Crash):
- **Lệnh `FLIP_AND_OPTIMIZE`:**
    1. Đóng vị thế hiện tại nếu đang lỗ nặng.
    2. Mở một vị thế ngược lại (Hedge) với khối lượng nhỏ để nương theo đà biến động.
    3. Thiết lập Trailing SL cực chặt cho lệnh Hedge này.
    4. Kích hoạt Optimize lại Asset đó với độ ưu tiên "Khẩn cấp".

## 4. Cơ chế dừng (Termination Mechanisms)
Hệ thống sử dụng các cấp độ dừng khác nhau để đảm bảo an toàn tài sản:

1.  **Terminate Immediately (Hard Stop / Panic):**
    - **Hành động:** 
        1. Đóng toàn bộ Position hiện tại ngay lập tức bằng lệnh Market.
        2. Hủy sạch mọi Order đang chờ trên sàn.
        3. Ngắt kết nối WebSocket và ngừng lấy mọi luồng dữ liệu (Data Feed) của Asset đó.
    - **Mục tiêu:** "Rút phích cắm" hoàn toàn khi có sự cố nghiêm trọng hoặc theo yêu cầu tức thì của người dùng.

2.  **Terminate Gracefully (Soft Stop):**
    - **Hành động:** 
        1. Chuyển trạng thái sang "Chặn vào lệnh" (`allow_entry = false`).
        2. Vẫn duy trì WebSocket để cập nhật giá.
        3. Để hệ thống Trailing SL tự động quản lý và đóng nốt các vị thế đang có theo chiến thuật.
        4. Tự động ngắt kết nối và đóng Actor khi số dư vị thế về 0.
    - **Mục tiêu:** Rút lui an toàn, tôn trọng các điểm chốt lời/lỗ đã được tối ưu.

3.  **Auto-Kill:** Tự động dừng Asset nếu lỗ lũy kế trong ngày vượt ngưỡng cho phép.

4.  **Auto-Panic on System Failure:** Tự động kích hoạt Hard Stop nếu App phát hiện lỗi hệ thống lặp lại nghiêm trọng.

## 6. Quản lý Đòn bẩy (Leverage Management)
Hệ thống không sử dụng đòn bẩy cố định cho mọi trường hợp:
- **Dynamic Leverage:** Mức đòn bẩy được chỉ định trực tiếp bởi Strategy Engine cho từng Asset dựa trên phân tích biến động (Volatility).
- **Hard-Cap:** Một ngưỡng đòn bẩy tối đa toàn cầu (ví dụ: 20x) luôn được áp dụng để ngăn chặn các sai sót logic.
