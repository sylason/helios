# Tiến độ Khối 3: Order Executor (Thực thi Lệnh qua WebSocket)

- [x] **1. Thiết kế Module Order Executor**
  - [x] Khởi tạo thư mục và file `src/execution/order_executor.rs`.
  - [x] Tạo struct `OrderExecutor` chứa các references đến `MarketFilter` và `WsManager` (kết nối WS).

- [x] **2. Tích hợp Market Filter (Khối 1) & Account State (Khối 2)**
  - [x] Viết hàm tính toán kích thước lệnh (Size Calculation) dựa trên `$ USDT` muốn cược và giá thị trường.
  - [x] Gọi hàm làm tròn `round_to_step_size` để cắt số lẻ của Coin.
  - [x] Chặn các lệnh nếu tổng giá trị `Qty * Price` nhỏ hơn `MinNotional` (thường là 5 USDT trên Binance).

- [x] **3. Chuyển đổi Action sang Lệnh Sàn**
  - [x] Map `GoLong` thành lệnh `BUY` mở vị thế (Position Side: LONG).
  - [x] Map `GoShort` thành lệnh `SELL` mở vị thế (Position Side: SHORT).
  - [x] Map `CloseAll` thành lệnh mua/bán ngược lại để đóng vị thế.

- [x] **4. Gửi Lệnh qua WS API**
  - [x] Áp dụng `binance_sdk::websocket_api::NewOrderParams`.
  - [x] Gọi qua `connection.new_order()`.
  - [x] Bắt `BotError` (Khối 6) nếu xảy ra lỗi đường truyền.

- [ ] **5. Unit & Integration Testing**
  - [ ] (Unit Test) Hàm tính size lệnh hoạt động đúng với các step_size khác nhau (VD: 0.001 BTC, 1 DOGE).
  - [ ] Cập nhật kết quả vào `TODO.md` Master.
