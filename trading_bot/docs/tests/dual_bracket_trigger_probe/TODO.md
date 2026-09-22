# TODO: Kiểm Định Thực Nghiệm Dual Bracket Trigger Probe

> **Mục tiêu:** Kiểm chứng hành vi thực tế của sàn Binance Futures khi một trong hai lệnh (SL hoặc TP) bị kích hoạt cắn lệnh và vị thế đóng về 0.

---

## 📋 Danh Sách Nhiệm Vụ (Baby Steps)

- [x] **[T1] Thiết kế cấu trúc dữ liệu JSON Báo Cáo:**
  - [x] Tạo struct `DualBracketAuditReport` chứa metadata, raw response của từng pha và kết luận khoa học.
  - [x] Cấu hình xuất file JSON độc lập theo timestamp (`run_YYYYMMDD_HHMMSS.json`) và file `latest.json`.
- [x] **[T2] Xây dựng bài test `binance_dual_bracket_trigger_probe.rs`:**
  - [x] **Pha 0:** Pre-Clean triệt để (Hủy toàn bộ Algo Orders + Market Close mọi vị thế tồn đọng).
  - [x] **Pha 1:** Mở vị thế Long 0.01 ETH và lưu raw JSON.
  - [x] **Pha 2:** Đặt song song SL (-2%) và TP (+2%) với `closePosition = true`, lưu raw JSON cả 2 lệnh.
  - [x] **Pha 4:** Cơ chế Dynamic Tightening Loop: Ép cắn SL bám sát `MarkPrice - 0.05` và vòng lặp chờ cho tới khi vị thế đóng về 0.000 ETH (timeout 45s).
  - [x] **Pha 5:** Truy vấn trạng thái sàn sau khi vị thế = 0:
    - [x] `query_algo_order` cho SL -> Lấy `actualOrderId`.
    - [x] `query_order` lấy Full Raw JSON của lệnh Market thực tế đóng vị thế (`avgPrice`, `executedQty`, `realizedPnl`).
    - [x] `query_algo_order` cho TP -> Kiểm tra xem Binance tự hủy hay vẫn giữ `NEW`.
    - [x] `current_all_algo_open_orders` -> Kiểm tra danh sách lệnh còn mở.
    - [x] `position_information_v2` -> Kiểm tra xác nhận vị thế về 0.
  - [x] **Pha 6:** Xuất dữ liệu ra file JSON độc lập (`run_<TIMESTAMP>.json`) và file `latest.json`.
  - [x] **Pha 7:** Post-Clean an toàn.
- [x] **[T3] Chạy thực nghiệm & Phân tích kết quả:**
  - [x] Chạy `cargo run --example binance_dual_bracket_trigger_probe`.
  - [x] Đọc và phân tích file JSON thu thập được (`run_20260918_092319.json` & `latest.json`).
  - [x] Trả lời dứt khoát: **Binance TỰ ĐỘNG CHUYỂN TP SANG `EXPIRED` VÀ XÓA KHỎI SỔ LỆNH ALGO!**
- [x] **[T4] Cập nhật tài liệu kỹ thuật & Đề xuất kiến trúc cho `OrderExecutor`:**
  - [x] Báo cáo chi tiết kết quả thực nghiệm và trích xuất raw payload JSON.
  - [x] Đề xuất nâng cấp `OrderExecutor` với cơ chế Dual Cloud Bracket an toàn tuyệt đối.
  - [ ] Báo cáo kết quả và đối chiếu với thiết kế của `OrderExecutor`.
- [x] **[T5] Tích Hợp User Data Stream Vào `binance_dual_bracket_trigger_probe.rs`:**
  - [x] Khởi tạo `start_user_data_stream` qua REST lấy `listen_key`.
  - [x] Kết nối WebSocket User Data Stream (`DerivativesTradingUsdsFuturesWsStreams`).
  - [x] Đăng ký lắng nghe sự kiện: `ORDER_TRADE_UPDATE`, `ACCOUNT_UPDATE`, `ALGO_UPDATE`, `CONDITIONAL_ORDER_TRIGGER_REJECT`.
  - [x] Thu thập và ghi nhận mọi event theo thời gian thực kèm timestamp vào `Arc<Mutex<Vec<UserDataStreamEventAudit>>>`.
  - [x] Pha 5.5: Audit và phân loại các event bắn về từ User Data Stream (thu thập thành công 30 events).
  - [x] Pha 6: Xuất toàn bộ danh sách events vào file JSON `run_20260922_163003.json` và `latest.json`.
  - [x] Chạy thực nghiệm `cargo run --example binance_dual_bracket_trigger_probe` và phân tích kết quả.
  - [x] **Xác nhận thực chứng:** User Data Stream bắn sự kiện `ALGO_UPDATE` với trạng thái `EXPIRED` của lệnh Take Profit chỉ sau 0.7 mili-giây khi vị thế về 0!
