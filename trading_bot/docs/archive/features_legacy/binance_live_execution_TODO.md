# Tiến độ: Binance Live Execution

Theo dõi tiến độ triển khai từng khối chức năng theo thứ tự phụ thuộc kỹ thuật (Technical Dependency Order).

---

## 📦 Tài liệu Thiết kế
- [x] TECHNICAL_DESIGN.md - Mô tả chi tiết 6 khối chức năng và Chiến lược Testing (Khối 7)
- [x] EXECUTION_FLOW.md - Luồng logic thực thi từng bước (Dual-Branch)
- [x] TODO.md - File kiểm soát tiến độ này

---

## 🛡️ GIAI ĐOẠN 1: Nền tảng Hạ tầng (Infrastructure Layer)
*Mục tiêu: Đặt nền móng trước khi code bất kỳ logic giao dịch nào.*

### Khối 4: Rate Limit Monitor (Giám sát giới hạn API REST)
**Phụ thuộc:** Không có.

- [x] Tạo module execution/rate_limit_monitor.rs (Đã gộp vào models/weight_monitor.rs)
- [x] Struct WeightMonitor với biến đếm AtomicU32
- [x] Đọc header X-MBX-USED-WEIGHT-1M từ HTTP response
- [x] Hàm is_safe() và is_emergency() để tạm dừng các call REST không cần thiết.
- [x] **Viết Unit Test** cho các ngưỡng cảnh báo và thuật toán reset giới hạn.

### Khối 5: Hạ tầng Đa kết nối WebSocket (Multi-WS Manager)
**Phụ thuộc:** Không có.

- [x] Tạo module quản lý 3 kết nối: Market Data, User Data, Trade API.
- [x] Bọc các luồng WebSocket trong vòng lặp tự động Reconnect.
- [x] Cơ chế lấy listenKey qua REST và Keep-alive PUT /fapi/v1/listenKey mỗi 30 phút.
- [x] Tích hợp Gap Filler để gọi REST lấy nến bù khi WS Market Data bị ngắt.
- [ ] **Viết Integration Test** mô phỏng ngắt kết nối giả để kiểm tra logic Auto-Reconnect.

---

## 📐 GIAI ĐOẠN 2: Bộ lọc & State Event-Driven (Validation Layer)
*Mục tiêu: Đảm bảo dữ liệu từ Binance luôn cập nhật tự động vào RAM.*

### Khối 1: Market Info & Filters (Bộ lọc Quy chuẩn Sàn)
**Phụ thuộc:** Rate Limit Monitor (Khối 4).

- [x] Tạo module execution/market_info.rs
- [x] Hàm fetch_exchange_info(symbol) lưu step_size, tick_size, min_qty vào RAM.
- [x] Hàm apply_filter(raw_qty, raw_price) để làm tròn.
- [x] **Viết Unit Test** cho các hàm làm tròn số lượng (round stepSize) và giá trị (round tickSize).

### Khối 2: Account & Position State (Event-Driven State)
**Phụ thuộc:** Khối 5 (User Data Stream).

- [x] Lắng nghe sự kiện ACCOUNT_UPDATE từ User Stream để cập nhật Balance vào RAM.
- [x] Lắng nghe sự kiện ORDER_TRADE_UPDATE để cập nhật trạng thái Position (Long/Short) vào RAM.
- [x] Hàm setup_isolated_margin và set_leverage (gọi 1 lần lúc startup).
- [x] Hàm can_open_position() kiểm tra Margin hiện có từ RAM (thêm 5% buffer).
- [x] **Viết Unit Test** mô phỏng tin nhắn JSON trả về từ Binance để cập nhật Position State ảo.

---

## ⚡ GIAI ĐOẠN 3: Cỗ máy Thực thi (Execution Engine)
*Mục tiêu: Gửi lệnh thực tế lên Binance Testnet với độ trễ tối thiểu.*

### Khối 6: Error Handler (Quản lý Lỗi)
**Phụ thuộc:** Không có — cần có trước khi viết OrderExecutor.

- [x] Tạo enum BotError liệt kê các loại lỗi (API, Network, Margin, Filter, ...)
- [x] Mapper từ mã lỗi Binance sang BotError.
- [x] Tuyệt đối không dùng .unwrap() hay .expect().

### Khối 3: Order Executor (Pure WebSocket)
**Phụ thuộc:** Khối 1, 2, 5, 6.

- [x] Tạo module execution/order_executor.rs
- [x] Khởi tạo TickContext với K-line cuối cùng và DNA mới nhất (Nhánh 2).
- [x] Trích xuất Action từ giỏ actions_buffer sau khi gọi strategy.next().
- [x] Hàm determine_order_side chuyển Action thành OrderSide.
- [x] Gọi connection.new_order() thông qua Trade WS API.
- [x] Ghi Log lệnh (orderId, giá, size) vào SQLite làm lịch sử.
- [x] **Viết Integration Test (Realtime)** bắn thử 1 lệnh Market giả lập lên Testnet API và xác minh nhận được thông báo khớp lệnh (Đã chạy E2E bot thật).

---

## ✅ GIAI ĐOẠN 4: Kiểm thử Tổng hợp (End-to-End Verification)
*Mục tiêu: Xác nhận toàn bộ luồng hoạt động đúng trên Testnet trước khi chạy Demo dài hạn.*

- [x] Viết kịch bản End-to-End (E2E) Test mô phỏng toàn bộ luồng từ lúc nhận tick đến lúc gửi lệnh.
- [x] Chạy bot test trên **Binance Testnet API Realtime**.
- [ ] Bot chạy liên tục 24h trên Testnet không lỗi Panic.
- [x] Xác nhận các lệnh Long/Short được đặt thành công qua WS API.
- [x] Xác nhận State cập nhật tự động khi khớp lệnh.
- [ ] Đánh dấu Epic 3 trong docs/TODO.md (Master) là hoàn thành.

---

## 📝 Ghi chú Thêm
- API Key Testnet lưu trong .env: BINANCE_TESTNET_API_KEY, BINANCE_TESTNET_SECRET_KEY.
- Tận dụng tối đa bộ SDK binance-connector-rust/examples.
