# 📋 UI Development Task Tracker (TODO.md)

Danh sách nhiệm vụ phát triển Giao diện Desktop Client (React 19 + Tauri v2) theo phương pháp Baby Steps. Cập nhật dấu `[x]` khi hoàn thành từng bước.

---

## 🏗️ Giai đoạn 1: Khởi Tạo Kiến Trúc Shell & Styling Base (✅ HOÀN THÀNH)
- [x] `[G1.1]` Tạo thư mục cấu trúc React Components (`src/components/layout`, `src/components/control`, `src/components/dashboard`, `src/pages`, `src/types`).
- [x] `[G1.2]` Tạo hệ thống Type Definitions trong `src/types/bot.ts` (AssetStatus, PositionState, AccountState, SystemHealth).
- [x] `[G1.3]` Xây dựng `Topbar.tsx` (Logo, Demo Mode Badge, Tauri IPC Status indicator).
- [x] `[G1.4]` Xây dựng `Sidebar.tsx` (Navigation giữa Dashboard & Bot Control, disable các trang chưa ưu tiên).
- [x] `[G1.5]` Cập nhật `App.tsx` với App Shell Layout & State-based Routing.

---

## ⚙️ Giai đoạn 2: Phát Triển Khung Trang 2 - Bot Control Center (✅ HOÀN THÀNH)
- [x] `[G2.1]` Xây dựng `WatchlistTable.tsx` hiển thị danh sách các cặp tiền (ETHUSDT, BTCUSDT, SOLUSDT).
- [x] `[G2.2]` Tích hợp Status Badges (INACTIVE, SYNCING, OPTIMIZING, READY, GRACEFUL_EXIT, ERROR).
- [x] `[G2.3]` Hiển thị thông tin Champion DNA (Strategy Name, Fitness Return %, Max DD %).
- [x] `[G2.4]` Hiển thị Live Position State (Side, Entry Price, Qty, uPnL $ & %).
- [x] `[G2.5]` Hiển thị Giá Trailing SL Target hiện tại trên Binance (Mark Price).
- [x] `[G2.6]` Gắn sự kiện các Nút thao tác: `ACTIVATE BOT`, `TERMINATE GRACEFULLY`, `STOP IMMEDIATELY`.
- [x] `[G2.7]` Xây dựng `AssetLogPanel.tsx` hiển thị 5 log sự kiện mới nhất khi chọn 1 Asset.

---

## 📊 Giai đoạn 3: Phát Triển Trang 1 - Executive Dashboard (✅ HOÀN THÀNH)
- [x] `[G3.1]` Xây dựng `KpiSummaryCards.tsx` (4 thẻ: Wallet Balance, Available Balance, Total uPnL, Active Assets Count).
- [x] `[G3.2]` Xây dựng `SystemHealthBar.tsx` (Binance API Weight Limit, WebSocket Status, Latency).
- [x] `[G3.3]` Xây dựng `GlobalKillSwitch.tsx` (Nút Dừng Khẩn Cấp Toàn Cục kèm modal 2 bước).
- [x] `[G3.4]` Tích hợp stream log rút gọn từ Rust Backend về Console thu nhỏ trên Dashboard.

---

## 🔗 Giai đoạn 4: Bộ Công Cụ Định Dạng & Mock Service (✅ HOÀN THÀNH)
- [x] `[G4.1]` Thiết lập Mock Data Service (`botService.ts`) để test UI độc lập trong môi trường Vite Browser (`npm run dev`).
- [x] `[G4.2]` Kết nối khung Tauri Command IPC (`invoke`) cho các nút điều khiển.
- [x] `[G4.3]` Viết Unit Test Suite với Vitest (`formatters.test.ts`) kiểm tra chức năng định dạng chuẩn 100% (`npx vitest run` PASS).

---

## 🚀 Giai đoạn 5: TÍCH HỢP TOÀN DIỆN LOCAL-FIRST TAURI IPC & CÁC MODAL AN TOÀN (TIẾP THEO)
- [x] `[G5.1]` **Đồng Bộ Chuẩn Hóa Type FSM:** Cập nhật `src/types/bot.ts` khớp chuẩn 1:1 với Rust FSM (`INACTIVE`, `STARTING`, `RUNNING_ACTIVE`, `PAUSED`, `GRACEFUL_STOPPING`, `STOPPED`, `ERROR`). Đã kiểm chứng Vitest và `tsc --noEmit` PASS 100%.
- [x] `[G5.2]` **Thanh Tìm Kiếm & Lọc Trực Tiếp (Inline Search):** Bổ sung Search Bar và Safe Liquidity Badge `💧` ngay phía trên `WatchlistTable.tsx`, tích hợp bộ nút FSM ngữ cảnh và cột vốn lãi kép. Đã kiểm chứng PASS 100%.
- [x] `[G5.3]` **Hộp Thoại Điều Chuyển Vốn An Toàn (RebalanceModal):** Xây dựng modal 2 lựa chọn (`📌 Áp dụng từ lệnh kế tiếp` vs `⚡ Đóng lệnh ngay`) khi cặp đang mở vị thế. Đã kiểm chứng PASS 100%.
- [x] `[G5.4]` **Bảng Thoát Tổng Hợp (UnifiedExitModal):** Xây dựng modal hiển thị 6 cột vị thế kèm 3 nút dứt khoát (`Flat All`, `Leave Cloud SL`, `Cancel`); hỗ trợ Fast Exit khi toàn bộ vị thế đang FLAT. Đã kiểm chứng Vitest và `npm run build` PASS 100%.
- [ ] `[G5.5]` **Hệ Thống Toast Cảnh Báo Lỗi & Safety Position Close:** Hiển thị Floating Toast đỏ tự tắt 5s khi lỗi cô lập; tự đóng lệnh an toàn và ngắt bot của cặp tiền gặp sự cố.
- [ ] `[G5.6]` **Lõi State Bridge Trong `src-tauri`:** Xây dựng `DesktopAppState` (Dual-Mode Local-First Desktop State), hỗ trợ `cmd_get_full_snapshot` và Delta Event Emitters (`event_asset_state_updated`, `event_position_updated`, `event_toast_notification`).
- [ ] `[G5.7]` **Chạy Ẩn System Tray & Chặn Đóng Cửa Sổ:** Cấu hình System Tray 4 menu item (`Open Desk`, `Pause All`, `Emergency Close All`, `Exit Application`) và hook ẩn xuống khay khi bấm `[X]`.
