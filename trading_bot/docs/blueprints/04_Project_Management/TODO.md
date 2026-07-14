# Project TODO List (Agile/Scrum Format)

Tài liệu này theo dõi toàn bộ tiến độ của dự án theo dạng Epic và Task.

---

## 🟢 EPIC 1: Refactor Skeleton & The Data Engine - [Hoàn thành]
Tái cấu trúc thư mục, tích hợp Helios và xây dựng luồng nhận dữ liệu K-line 1m không bị Gap.

- [x] Bước 1: Tái cấu trúc thư mục `bot_engine/src` (models, data_engine, optimizer, execution, strategy_runner, interface).
- [x] Bước 2: Tích hợp `helios` crate qua Cargo path dependency.
- [x] Bước 3: Cài đặt DuckDB storage (lưu `kline_1m`, `strategy_configs`, cleanup).
- [x] Bước 4: Cài đặt WebSocket kline_1m và REST API Gap Filler. Khởi chạy thử nghiệm.

## 🟢 EPIC 1.5: Automated Tests cho Data Engine - [Hoàn thành]
Viết Unit Test để đảm bảo tính toàn vẹn dữ liệu và an toàn của hệ thống.
- [x] Unit Test cho `duckdb_impl.rs` với `:memory:` database.
- [x] Unit Test cho Circuit Breaker `weight_monitor.rs`.
- [x] Unit Test cho thuật toán tính toán Gap Filler `binance_rest.rs`.

## 🔵 EPIC 2: Động cơ Chiến lược & Optimizer Liên tục (The Strategy Engine)
Xây dựng nền tảng để chạy Optimizer ngầm và Hot-reload JSON params.

- [x] Lấy 30 ngày data từ DuckDB để cung cấp cho Optimizer.
- [x] Khởi chạy vòng lặp `ferro_trade` Genetic Algorithm chạy ngầm (liên tục).
- [x] Ghi kết quả JSON params tối ưu vào DuckDB `strategy_configs`.
- [ ] Strategy Runner chạy Live (Mỗi phút lấy 3-7 ngày dữ liệu tính lại Indicator từ đầu, đọc JSON params mới nhất).

## 🟡 EPIC 3: Thực thi Môi trường Kép (Demo & Sandbox) & Binance Live Execution
Chuyển đổi bot từ chế độ log kết quả sang việc mô phỏng và thực thi lệnh chuẩn xác trên Binance.

- [ ] Phân tách môi trường: Clone thư viện `helios` sang `v2_trading_bot_demo` và liên kết với `bot_engine` để bảo vệ tiến trình Demo.
- [ ] Module `MarketInfo`: Lấy thông tin `ExchangeInfo` để khởi tạo luật làm tròn `stepSize`, `minQty` và `tickSize`.
- [ ] Module `MarginManager`: Ước tính chi phí Margin dựa trên Đòn bẩy (Leverage) và cấu hình Isolated Margin.
- [ ] Module `BinanceExecutor`: Kết nối SDK bắn lệnh Market/Limit an toàn lên sàn Binance Testnet.
- [ ] Fail-safes: Tích hợp Rate Limit `WeightMonitor` (đỉnh 2400/phút).
- [ ] Mạng lưới: Bổ sung vòng lặp Auto-Reconnect cho WebSocket `BinanceStream`.

## 🟠 EPIC 4: Tích hợp Frontend (Tauri Client)
Giao diện hiển thị tối giản.

- [ ] Setup IPC Channel giữa React và Rust.
- [ ] Giao diện hiển thị Vị thế đang mở (Positions).
- [ ] Nút Bật/Tắt Bot theo Cặp.

---

## 🚀 EPIC 5: Sản xuất (Production Readiness) - Tương lai xa
- [ ] Cải tiến `ferro_trade` sang chế độ Live (Incremental Resampling) để tối ưu O(1).
- [ ] Đánh Live (Real Money) với API Key mã hóa.
- [ ] Phục hồi trạng thái toàn diện.
