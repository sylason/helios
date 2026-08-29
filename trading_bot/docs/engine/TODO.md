# 📋 Bot Engine Development Task Tracker (TODO.md)

Danh sách nhiệm vụ phát triển Lõi `bot_engine` Multi-Asset Server theo phương pháp Baby Steps. Đánh dấu `[x]` khi hoàn thành từng bước.

---

## 🏗️ Giai đoạn 1: Chuẩn Hóa Database & Shared WsManager
- [ ] `[E1.1]` Chuẩn hóa `SqliteStore` dùng 1 file duy nhất `storage/trading_bot.db` kèm Index cho cột `symbol`.
- [ ] `[E1.2]` Nâng cấp `WsManager` hỗ trợ Dynamic Subscribe / Unsubscribe nhiều symbol trên 1 Combined Stream.
- [ ] `[E1.3]` Xây dựng Bảng Điều Hướng `routing_table: Arc<RwLock<HashMap<String, mpsc::Sender<Candle>>>>` trong `WsManager` để phân phối nến $O(1)$.
- [ ] `[E1.4]` Viết Unit Test kiểm tra tính năng Dynamic Subscribe và routing gói tin nến.

---

## ⚡ Giai đoạn 2: JIT Optimizer Scheduler & DNA Freshness Guard
- [ ] `[E2.1]` Xây dựng `OptimizerScheduler` sử dụng `tokio::sync::Semaphore(1)` để quản lý hàng đợi chạy GA chống nghẽn CPU.
- [ ] `[E2.2]` Tích hợp nguyên tắc Just-In-Time: Chỉ truy vấn nến 30 ngày từ SQLite ngay khi lấy được Semaphore lock.
- [ ] `[E2.3]` Bổ sung hàm kiểm tra tính tươi mới của DNA (`check_dna_freshness(symbol, step_interval_mins)`).
- [ ] `[E2.4]` Viết Unit Test cho `OptimizerScheduler` và `DNA Freshness Guard`.

---

## 👑 Giai đoạn 3: Xây Dựng EngineController & Máy Trạng Thái FSM
- [ ] `[E3.1]` Tạo struct `EngineController` làm Facade trung tâm quản lý danh sách các `AssetActor`.
- [ ] `[E3.2]` Triển khai các lệnh điều khiển: `start_asset`, `cancel_start`, `pause_trading`, `resume_trading`.
- [ ] `[E3.3]` Triển khai các lệnh ngắt: `graceful_stop`, `cancel_graceful_stop`, `force_stop_immediate`, `remove_asset`.
- [ ] `[E3.4]` Tích hợp công thức tính vốn lãi kép lũy tiến kèm vốn sàn (Compounding Capital Floor) và Luồng Điều Chuyển Vốn (Rebalancing).
- [ ] `[E3.5]` Cấu hình cố định `Mark Price` cho toàn bộ các lệnh Trailing SL trong `OrderExecutor`.
- [ ] `[E3.6]` Tích hợp cơ chế tự động cấu hình `ISOLATED Margin` và tiếp quản vị thế có sẵn (`Adopt & Protect`).
- [ ] `[E3.7]` Tích hợp cơ chế cô lập lỗi theo từng Asset (`Fault Isolation`).
- [ ] `[E3.8]` Viết Unit Test kiểm tra toàn bộ luồng chuyển đổi trạng thái FSM của `EngineController`.

---

## 🔌 Giai đoạn 4: Tích Hợp Tauri IPC Bridge & System Tray
- [ ] `[E4.1]` Khởi tạo các Tauri Commands (`cmd_start_asset`, `cmd_pause_asset`, `cmd_force_stop_asset`, `cmd_get_full_snapshot`, `cmd_rebalance_capital`).
- [ ] `[E4.2]` Thiết lập Tauri Event Emitter phát Delta Events (`event_candle_tick`, `event_sl_updated`, `event_order_filled`).
- [ ] `[E4.3]` Cấu hình System Tray trong `src-tauri` và chặn sự kiện đóng cửa sổ để hiện Bảng Thoát Tổng Hợp.
