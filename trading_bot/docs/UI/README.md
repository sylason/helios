# 🚀 Trading Bot Client UI Documentation

Tài liệu thiết kế kiến trúc và giao diện người dùng (UI) cho hệ thống **Trading Bot Desktop Client**.

---

## 🏛️ Cấu trúc Thư mục Tài liệu (UI Documentation Tree)

```text
trading_bot/docs/UI/
├── README.md                               # Document tổng quan này
├── TODO.md                                 # Danh sách công việc triển khai UI
├── architecture/
│   └── app_layout_and_navigation.md        # Cấu trúc App Shell, Topbar, Sidebar & Routing
├── pages/
│   ├── bot_control_center.md               # Thiết kế chi tiết Trang 2 (Bot Control Center)
│   └── executive_dashboard.md              # Thiết kế chi tiết Trang 1 (Executive Dashboard)
└── components/
    └── design_system_and_state.md          # Design Tokens, Component System & Tauri IPC State Flow
```

---

## 🎯 Mục Tiêu & Triết Lý Thiết Kế

1. **Local-First & Minimalist:** Tối giản giao diện, tập trung 100% vào thông tin sinh tồn (Information-First) và khả năng kiểm soát bot tức thì.
2. **React 19 + TypeScript + Tauri v2:** Phát triển nhanh bằng React Components, styling nhất quán với React-Bootstrap & Custom Dark Theme CSS, giao tiếpIPC trực tiếp với Rust `bot_engine`.
3. **Thứ tự Ưu tiên Giai đoạn Demo:**
   - **Ưu tiên #1:** Trang 2 - Active Watchlist & Bot Control Center (Quản lý và điều khiển từng Asset).
   - **Ưu tiên #2:** Trang 1 - Executive Dashboard (Tổng quan PnL, sức khỏe hệ thống, Nút dừng khẩn cấp).
   - *(Trang Settings, Trade Journal, Live Charts được tạm hoãn lại sau khi chạy demo có lời).*
