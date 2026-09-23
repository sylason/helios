# 📋 DANH MỤC CÔNG VIỆC: NÂNG CẤP HẠ TẦNG LOGGING & CÔ LẬP HAI LUỒNG (DUAL-STREAM TELEMETRY)

Tài liệu quản lý tiến độ triển khai hệ thống Logging & Telemetry chuyên biệt cho toàn bộ hệ sinh thái (`ferro_trade`, `helios`, `bot_engine`).

---

## 🎯 CÁC TÁC VỤ ĐÃ HOÀN THÀNH

- [x] **TASK-LOG-01: Triệt tiêu Console Spam và Disk I/O trong `ferro_trade` (GA Optimizer Core)**
  - Bổ sung trường `pub verbose: bool` (mặc định `false`) vào `ProspectorConfig` và `Default` implementation.
  - Bọc 100% các câu lệnh `println!` trong `prospector.rs` (start, workers, scout progress, pruning/promotion, random restarts, cleanup) dưới điều kiện `verbose`.
  - Thay thế `println!` thành `eprintln!` trong `ga_worker.rs` khi xảy ra lỗi nghiêm trọng.
  - Đảm bảo GA Optimizer chạy ở chế độ im lặng tuyệt đối (Silent Mode), tiết kiệm CPU và chống nghẽn I/O terminal.
  - *Kiểm chứng:* `cargo check --package ferro_trade` passed 100%.

- [x] **TASK-LOG-02: Bổ sung Auction Telemetry trong `helios` Arbiter**
  - Kích hoạt telemetry khi `debug_mode == true` trong `process_tick` của `arbiter.rs`.
  - In chi tiết biên bản đấu thầu mỗi khi có ít nhất 1 proposal: Tên Scanner, Side, Raw Score, Điểm Confidence sau khi trừ Soft Chaos Penalty, và kết quả phân định Thắng / Hòa / Triệt tiêu (Flat).
  - Không spam log khi không có proposal nào được sinh ra.
  - *Kiểm chứng:* `cargo test --lib strategy::core_v3::arbiter` passed 8/8 unit tests.

- [x] **TASK-LOG-03: Nâng cấp `PaperTrader` (Post-Trade Autopsy & Chống Quadratic Log Spam)**
  - Bổ sung trường `highest_price_seen` và `lowest_price_seen` để theo dõi biên độ giá cực hạn trong suốt vòng đời lệnh.
  - Bổ sung trường `mfe_pct` (Maximum Favorable Excursion) và `mae_pct` (Maximum Adverse Excursion) vào `PaperTradeRecord`.
  - In sự kiện **Post-Trade Autopsy** ngay khi lệnh đóng (Entry, Exit, Lý do thoát, Net PnL, ROE, MFE, MAE, Số nến giữ, Phí sàn, Số dư vốn).
  - Khắc phục lỗi in lặp quadratic spam: Sử dụng `AtomicUsize` để chỉ in delta các giao dịch mới đóng trong báo cáo định kỳ và không in khung rỗng khi chưa có lệnh.
  - Thêm phương thức `log_final_summary` xuất 1 lần duy nhất toàn bộ 100% lịch sử giao dịch khi kết thúc bài test.
  - *Kiểm chứng:* `cargo test --lib execution::paper_trader` passed 9/9 unit tests (bao gồm test mới `test_mfe_mae_and_delta_logging`).

- [x] **TASK-LOG-04: Chuẩn hóa Veto Log & DNA Milestones trong `asset_actor.rs`**
  - Sửa câu lệnh log dòng 678 trong `asset_actor.rs` hiển thị đúng giá trị thực tế `(Required >= {:.2}*ATR = {:.2})`, xóa bỏ ký tự `<` hardcode gây hiểu nhầm.
  - Gọi `log_final_summary` khi `AssetActor` dừng để xuất biên bản tổng kết toàn diện.
  - *Kiểm chứng:* `cargo check --examples` passed 100%.

- [x] **TASK-LOG-05: Cấu hình Dual-Appender trong `trading_bot/bot_engine/examples/common/mod.rs`**
  - Triển khai `LoggingGuards` quản lý đồng thời 2 WorkerGuard không đồng bộ.
  - Tách thành 2 file log riêng biệt:
    + `{strategy_id}_system.log`: Ghi nhận sự kiện hệ thống, database, đồng bộ, chu kỳ Re-Opt GA.
    + `{strategy_id}_trades.log`: Ghi nhận chuyên biệt OOS Telemetry (Auction, Veto, Lệnh mở/đóng, Trailing SL/TP, Autopsy, Milestones DNA).
  - Console (Terminal): Chỉ xuất hiện các mốc quan trọng cấp cao, sạch sẽ và thân thiện.
  - *Kiểm chứng:* `cargo check --examples` và toàn bộ các caller (`walk_forward_smart_seed`, `paper_forward_test`, ...) biên dịch sạch sẽ.
