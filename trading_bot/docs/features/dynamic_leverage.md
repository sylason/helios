# Dynamic Leverage (Đòn bẩy động)

## 🎯 Mục tiêu
Loại bỏ việc hardcode Leverage (đòn bẩy) trong `bot_engine` hoặc file `.env`. Cho phép Strategy (Chiến lược) toàn quyền quyết định đòn bẩy một cách linh hoạt (Dynamic) tuỳ thuộc vào kịch bản thị trường (Playbook).

Tính năng này chuẩn bị cơ sở hạ tầng cho các Playbook nâng cao như **Nuke Playbook** (chỉ kích hoạt ở điều kiện chắc chắn thắng với đòn bẩy cực đại x100/x125, trong khi Playbook bình thường giữ ở x10 - x20).

## 🏛️ Kiến trúc & Thiết kế

### 1. Ferro Trade (Core Engine)
- **Bổ sung Action mới:** Cần thêm biến thể `ChangeLeverage` vào Enum `Action` trong `ferro_trade/src/broker/actions.rs`.
  ```rust
  pub enum Action {
      // Các action hiện có...
      ChangeLeverage { leverage: u32 },
  }
  ```
- **Xử lý tại Backtester:** Khi Backtest Engine nhận được `ChangeLeverage`, hệ thống mô phỏng (Broker) sẽ chỉ cập nhật state nội bộ (biến `current_leverage`) để tính toán chính xác ký quỹ (Margin) và thanh lý (Liquidation) cho các lệnh Buy/Sell tiếp theo.

### 2. Trading Bot (Bot Engine)
- **Account Manager:**
  - Xóa bỏ logic `self.rest_client.change_initial_leverage(...)` đang bị hardcode tại bước `initialize_account`.
  - Lúc khởi chạy, bot chỉ set Margin Type thành `ISOLATED`, bảo toàn mức đòn bẩy hiện tại của tài khoản trên sàn.
- **Order Executor:**
  - Lắng nghe Action từ Strategy gửi qua Event Bus.
  - Khi bắt được `Action::ChangeLeverage { leverage }`:
    - Gọi API `change_initial_leverage` của Binance (`RestApi`) để cập nhật đòn bẩy thực tế trên sàn.
    - Cập nhật state `current_leverage` nội bộ của `OrderExecutor` để hàm `calculate_order_size` tính đúng Qty của lệnh sắp tới.

### 3. Helios (Strategy Sandbox)
- **Tạo luồng Playbook Nuke (Tương lai):**
  - Khi thuật toán phân tích xác định xu hướng Nuke:
    1. Phát tín hiệu `Action::ClosePosition { portion: 1.0 }` để xóa sổ vị thế hiện tại.
    2. Kế tiếp phát `Action::ChangeLeverage { leverage: 100 }`.
    3. Cuối cùng phát `Action::Buy / Action::Sell` để vào lệnh với cấu hình mới.

## 🚧 Lộ trình triển khai (Trì hoãn)
*Tính năng này hiện tại được lưu trữ lại. Việc triển khai sẽ được bắt đầu khi Helios Strategy đã đạt đủ độ ổn định để chạy các chiến lược đa đòn bẩy (Multi-playbook / Nuke).*
