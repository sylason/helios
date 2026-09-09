# 📋 MASTER PROJECT ROADMAP & DEVELOPMENT TRACKER: TRADING BOT

Tài liệu quản trị vòng đời phát triển toàn diện cho dự án `trading_bot` (Execution Platform + Desktop UI).

---

## 🏛️ GIAI ĐOẠN 0: HỆ THỐNG HÓA TÀI LIỆU & QUY CHUẨN TÍNH NĂNG (SPEC CONSOLIDATION) - (✅ HOÀN THÀNH)

- [x] `[G0.1]` Đọc quét 100% tài liệu trong `trading_bot/docs/` và tạo `FEATURE_INVENTORY.md`.
- [x] `[G0.2]` Phân loại toàn bộ tính năng: 🟢 Active, 🟡 Proposed, 🔴 Deprecated, ⚠️ Conflict.
- [x] `[G0.3]` Thảo luận, làm rõ và chốt quyết định kỹ thuật cho từng tính năng mâu thuẫn / chưa hợp lý (7 biên bản thảo luận trong `docs/UI/discussions/`).
- [x] `[G0.4]` Dọn dẹp cấu trúc thư mục `docs/`: Chuyển tài liệu cũ vào `archive/`, chuẩn hóa tài liệu Active theo cấu trúc `specs/` và `discussions/`.

---

## ⚙️ GIAI ĐOẠN 1: HẠ TẦNG DỮ LIỆU & WEBSOCKET ROUTER (BOT_ENGINE)

- [ ] `[G1.1]` Chuẩn hóa `SqliteStore` dùng 1 file duy nhất `storage/trading_bot.db` kèm Index cho `symbol`.
- [ ] `[G1.2]` Xây dựng `Shared WsManager` kết nối 1 Combined Stream duy nhất cho mọi cặp tiền.
- [ ] `[G1.3]` Thiết lập Bảng điều hướng `routing_table` phân phối gói nến $O(1)$ tới từng `AssetActor`.
- [ ] `[G1.4]` Viết Unit Test kiểm tra tính năng Dynamic Subscribe / Unsubscribe và định tuyến nến.

---

## ⚡ GIAI ĐOẠN 2: JIT OPTIMIZER SCHEDULER & DNA FRESHNESS GUARD

- [ ] `[G2.1]` Xây dựng `OptimizerScheduler` sử dụng `tokio::sync::Semaphore(1)` xếp hàng chạy GA chống nghẽn CPU.
- [ ] `[G2.2]` Tích hợp nguyên tắc Just-In-Time (JIT): Chỉ đọc nến 30 ngày từ SQLite ngay khi lấy được Semaphore lock.
- [ ] `[G2.3]` Triển khai `DNA Freshness Guard`: Kiểm tra hạn DNA ($<15$p $\rightarrow$ trade ngay, hết hạn $\rightarrow$ chạy GA JIT).
- [ ] `[G2.4]` Viết Unit Test cho `OptimizerScheduler` và `DNA Freshness Guard`.

---

## 👑 GIAI ĐOẠN 3: LÕI QUẢN TRỊ ĐA TÀI SẢN (ENGINECONTROLLER FACADE & FSM)

- [ ] `[G3.1]` Xây dựng struct `EngineController` làm Facade trung tâm quản lý danh bạ `AssetActorRegistry`.
- [ ] `[G3.2]` Triển khai bộ 8 lệnh FSM đối xứng: `start`, `cancel_start`, `pause`, `resume`, `graceful_stop`, `cancel_graceful`, `force_stop`, `remove_asset`.
- [ ] `[G3.3]` Tích hợp công thức Quản trị vốn Lũy tiến (Lãi kép) kèm Vốn sàn (Base Floor).
- [ ] `[G3.4]` Triển khai cơ chế Tiếp quản vị thế mở sẵn (`Adopt & Protect`) và tự động cấu hình `ISOLATED Margin 20x`.
- [ ] `[G3.5]` Triển khai cơ chế Cô lập lỗi (`Fault Isolation`) theo từng Asset.
- [ ] `[G3.6]` Viết Unit Test kiểm tra toàn bộ luồng chuyển đổi trạng thái FSM của `EngineController`.

---

## 🔌 GIAI ĐOẠN 4: CẦU NỐI TAURI IPC & SYSTEM TRAY (SRC-TAURI)

- [ ] `[G4.1]` Import crate `bot_engine` vào `src-tauri/Cargo.toml` và quản lý `Arc<EngineController>` trong `tauri::State`.
- [ ] `[G4.2]` Triển khai các Tauri Command Handlers (`cmd_start_asset`, `cmd_pause_asset`, `cmd_force_stop_asset`, `cmd_rebalance_capital`, `cmd_emergency_stop_all`, `cmd_get_full_snapshot`).
- [ ] `[G4.3]` Xây dựng Tauri Event Emitter phát Delta Events (`event_asset_state_updated`, `event_candle_tick`, `event_toast_notification`).
- [ ] `[G4.4]` Cấu hình System Tray Menu và chặn sự kiện đóng cửa sổ `[X]` để hiện Bảng Thoát Tổng Hợp.

---

## 🖥️ GIAI ĐOẠN 5: HOÀN THIỆN GIAO DIỆN CLIENT & NỐI DÂY REAL-TIME (REACT UI)

- [ ] `[G5.1]` Nối dây `botService.ts` sang gọi Tauri `invoke()` và `listen()` cập nhật state thời gian thực.
- [ ] `[G5.2]` Xây dựng Hộp thoại Điều chuyển Vốn An Toàn (Rebalancing Modal).
- [ ] `[G5.3]` Xây dựng Bảng Thoát Tổng Hợp Duy Nhất (Unified Exit Summary Modal).
- [ ] `[G5.4]` Hoàn thiện bộ nút thao tác FSM trên `WatchlistTable`, nút thêm cặp tiền và Toast thông báo lỗi.
- [ ] `[G5.5]` Chạy kiểm thử End-to-End toàn diện trên Desktop Client.
