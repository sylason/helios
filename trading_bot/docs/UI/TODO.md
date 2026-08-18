# 📋 UI Development Task Tracker (TODO.md)

Danh sách nhiệm vụ phát triển UI cho Trading Bot Client. Cập nhật dấu `[x]` khi hoàn thành từng baby step.

---

## 🏗️ Giai đoạn 1: Khởi Tạo Kiến Trúc Shell & Styling Base
- [x] `[G1.1]` Tạo thư mục cấu trúc React Components (`src/components/layout`, `src/components/control`, `src/components/dashboard`, `src/pages`, `src/types`).
- [x] `[G1.2]` Tạo hệ thống Type Definitions trong `src/types/bot.ts` (AssetStatus, PositionState, AccountState, SystemHealth).
- [x] `[G1.3]` Xây dựng `Topbar.tsx` (Logo, Demo Mode Badge, Tauri IPC Status indicator).
- [x] `[G1.4]` Xây dựng `Sidebar.tsx` (Navigation giữa Dashboard & Bot Control, disable các trang chưa ưu tiên).
- [x] `[G1.5]` Cập nhật `App.tsx` với App Shell Layout & State-based Routing.

---

## ⚙️ Giai đoạn 2: Phát Triển Trang 2 - Active Watchlist & Bot Control Center (Ưu tiên #1)
- [x] `[G2.1]` Xây dựng `WatchlistTable.tsx` hiển thị danh sách các cặp tiền (ETHUSDT, BTCUSDT, SOLUSDT).
- [x] `[G2.2]` Tích hợp Status Badges (INACTIVE, SYNCING, OPTIMIZING, READY, GRACEFUL_EXIT, ERROR).
- [x] `[G2.3]` Hiển thị thông tin Champion DNA (Strategy Name, Fitness Return %, Max DD %).
- [x] `[G2.4]` Hiển thị Live Position State (Side, Entry Price, Qty, uPnL $ & %).
- [x] `[G2.5]` Hiển thị Giá Trailing SL Target hiện tại trên Binance.
- [x] `[G2.6]` Gắn sự kiện các Nút thao tác: `ACTIVATE BOT`, `TERMINATE GRACEFULLY`, `STOP IMMEDIATELY`.
- [x] `[G2.7]` Xây dựng `AssetLogPanel.tsx` hiển thị 5 log sự kiện mới nhất khi chọn 1 Asset.

---

## 📊 Giai đoạn 3: Phát Triển Trang 1 - Executive Dashboard (Ưu tiên #2)
- [x] `[G3.1]` Xây dựng `KpiSummaryCards.tsx` (4 thẻ: Wallet Balance, Available Balance, Total uPnL, Active Assets Count).
- [x] `[G3.2]` Xây dựng `SystemHealthBar.tsx` (Binance API Weight Limit, WebSocket Status, Latency).
- [x] `[G3.3]` Xây dựng `GlobalKillSwitch.tsx` (Nút Dừng Khẩn Cấp Toàn Cục - Emergency Stop All Bots).
- [x] `[G3.4]` Tích hợp stream log rút gọn từ Rust Backend về Console thu nhỏ.

---

## 🔗 Giai đoạn 4: Tích Hợp IPC Event Stream & Test Suite
- [x] `[G4.1]` Thiết lập Mock Data Service (`botService.ts`) để test UI độc lập trong môi trường Vite Browser (`npm run dev`).
- [x] `[G4.2]` Kết nối Tauri Command IPC (`invoke`) cho các nút Activate / Deactivate.
- [x] `[G4.3]` Viết Unit Test Suite với Vitest (`formatters.test.ts`) kiểm tra chức năng định dạng chuẩn 100% (`npx vitest run` PASS).
- [ ] `[G4.4]` Kiểm tra build desktop cuối cùng với `npm run build`.
