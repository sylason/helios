# Client UI Specification (Tauri + React)

## 1. Mục tiêu
Tauri đóng vai trò như một màng bọc nhẹ cho web-app React. React sẽ hiển thị giao diện, còn Core Rust (Backend) xử lý tính toán. Giữa 2 bên giao tiếp qua **IPC (Inter-Process Communication)**.

## 2. Trách nhiệm của UI (Không có Logic Trading)
UI hoàn toàn là một **Giao diện Câm (Dumb UI)**. Nó không quyết định khi nào vào lệnh. Nhiệm vụ của nó:
- Render HTML/CSS thật mượt mà và đẹp (Premium Aesthetics).
- Gửi lệnh (Commands) của user qua Backend (VD: "Khởi động Bot", "Dừng Khẩn Cấp").
- Lắng nghe các Sự Kiện (Events) từ Backend đẩy lên (VD: "Đã vào lệnh Long", "Giá đang thay đổi") để render ra màn hình.

## 3. Danh sách IPC Commands (React -> Rust)
- `start_engine(symbol: String)`: Bật bot cho cặp chỉ định.
- `stop_engine(symbol: String)`: Tắt bot cặp chỉ định.
- `panic_sell_all()`: Dừng khẩn cấp, cắt mọi vị thế (Kill Switch).

## 4. Danh sách IPC Events (Rust -> React)
- `on_system_log(msg: String, level: String)`: Nhận dòng text để in ra hộp thoại Log.
- `on_position_update(data: PositionInfo)`: Cập nhật Lãi/Lỗ lên màn hình.
- `on_connection_status(status: Enum)`: Đèn báo trạng thái Binance WS (Xanh/Đỏ).

## 5. Yêu cầu Giai đoạn Demo (MVP)
Giao diện cực kỳ đơn giản:
- Cột bên trái: Danh sách nút điều khiển (Start, Stop).
- Cột bên phải: Một cửa sổ Terminal màu đen (in Logs) giống như Console của Hacker.
- Thanh Status Bar: Hiển thị Ping tới Binance, trạng thái DB.

*Không cần chart phức tạp ở giai đoạn Demo.*
