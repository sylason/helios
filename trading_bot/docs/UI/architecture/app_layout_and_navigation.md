# 🏛️ App Layout & Navigation Architecture

Tài liệu thiết kế cấu trúc khung ứng dụng (App Shell), Thanh tiêu đề (Topbar), Thanh điều hướng (Sidebar) và cơ chế Chuyển trang (Routing).

---

## 📐 1. Cấu Trúc Tổng Thể (App Shell Grid)

Khung ứng dụng sử dụng CSS Grid/Flexbox với màu nền tối chuẩn Cyberpunk / Quant Trading Desk (`#0F172A`).

```text
+-----------------------------------------------------------------------+
|  TOPBAR: Logo | Demo Mode Badge | Rust IPC Connection Indicator       |
+------------------+----------------------------------------------------+
| SIDEBAR          | MAIN CONTENT AREA                                  |
|                  |                                                    |
|  [📊 Dashboard]   |  Render active page component:                     |
|  [⚙️ Bot Control] |  - Active Watchlist & Control Center (Default)     |
|  [🔒 Settings]   |  - Executive Dashboard                             |
|  [🔒 Journal]    |                                                    |
|  [🔒 Live Chart] |                                                    |
|                  |                                                    |
+------------------+----------------------------------------------------+
```

---

## 🧩 2. Các Component Khung (Layout Components)

### 2.1. Topbar (`src/components/layout/Topbar.tsx`)
- **Nhiệm vụ:** Hiển thị thương hiệu ứng dụng, chế độ hoạt động và trạng thái kết nối IPC với Rust Engine.
- **Thành phần:**
  - `Brand`: 🚀 **FERRO-BOT QUANT DESK** (Phông chữ đậm, màu Cyan/Info).
  - `Environment Badge`: `DEMO MODE (Binance Testnet)` (Badge viền vàng).
  - `IPC Status Indicator`: Đèn chấm tròn sáng Xanh (`CONNECTED`) hoặc Đỏ (`DISCONNECTED`) thể hiện kết nối giữa UI và Backend Rust.

### 2.2. Sidebar (`src/components/layout/Sidebar.tsx`)
- **Nhiệm vụ:** Chuyển đổi giữa các trang ứng dụng.
- **Danh sách Menu Items:**
  1. ⚙️ **Bot Control** (Giai đoạn Demo - Active, Mặc định).
  2. 📊 **Dashboard** (Giai đoạn Demo - Active).
  3. 🔒 **Settings** (Tạm khóa / Disabled trong bản Demo).
  4. 🔒 **Trade Journal** (Tạm khóa / Disabled trong bản Demo).
  5. 🔒 **Live Charts** (Tạm khóa / Disabled trong bản Demo).

---

## 🔀 3. Cơ Chế Chuyển Trang (State-Based Routing)

Do ứng dụng chạy trên Desktop Tauri nhẹ nhàng, chúng ta sử dụng **State-based Page Switch** đơn giản (`currentPage` state trong `App.tsx`) thay vì kéo theo React Router nặng nề:

```typescript
type PageId = 'control' | 'dashboard';
const [activePage, setActivePage] = useState<PageId>('control');
```
