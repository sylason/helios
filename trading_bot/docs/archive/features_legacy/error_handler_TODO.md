# Tiến độ Khối 6: Error Handler (Quản lý Lỗi)

- [x] **1. Định nghĩa Cấu trúc Error**
  - [x] Khởi tạo thư mục `src/error/` và file `mod.rs`.
  - [x] Khai báo Enum `BotError` chứa các phân nhóm lỗi: `ApiError`, `NetworkError`, `MarginError`, `MarketFilterError`, `ParseError`, `StateError`, `Internal`.
  - [x] Thêm crate `thiserror` vào `bot_engine/Cargo.toml` (nếu chưa có) và áp dụng macro `#[derive(Error, Debug)]` cho `BotError`.

- [x] **2. Error Mapping Function**
  - [x] Triển khai hàm chuyển đổi `From<binance_sdk::common::errors::ConnectorError>` sang `BotError`.
  - [x] Triển khai hàm chuyển đổi `From<serde_json::Error>` sang `BotError::ParseError`.

- [x] **3. Cập nhật Code Hiện Tại**
  - [x] Thay thế các kiểu trả về lỗi thô (VD: `Result<(), String>` hoặc `Result<(), Box<dyn Error>>`) trong `AccountManager`, `AssetActor` sang dùng `Result<(), BotError>`.
  - [x] Đảm bảo xóa mọi lệnh `unwrap()` hoặc `expect()` còn sót lại có rủi ro gây panic (ví dụ lúc parse websocket JSON).

- [x] **4. Unit Tests**
  - [x] Viết test mô phỏng map lỗi từ `ConnectorError` sang `BotError`.
  - [x] Viết test xác nhận lỗi Parse JSON được map đúng chuẩn.

- [x] **5. Đóng gói & Báo cáo**
  - [x] Cập nhật kết quả vào Checkbox ở Master `TODO.md`.
  - [x] Commit code.
