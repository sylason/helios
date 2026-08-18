# 📊 Trang 1: Executive Dashboard (Thiết Kế Chi Tiết)

Trang ưu tiên số 2 trong giai đoạn Demo, tổng hợp bức tranh tài sản và sức khỏe hệ thống theo thời gian thực.

---

## 🎯 1. Nhiệm Vụ & Luồng Hoạt Động

- Hiển thị tổng quát số dư tài khoản, lời/lỗ biến động trên Binance Testnet.
- Giám sát độ khỏe của Rust Engine Backend (API Rate Limit Weight, WebSocket Latency).
- Cung cấp Nút Dừng Khẩn Cấp Toàn Cục (Global Emergency Stop).

---

## 🧩 2. Cấu Trúc Các Component

### 2.1. KPI Summary Cards (`KpiSummaryCards.tsx`)
Hiển thị 4 thẻ thông số quan trọng:
1. 💳 **Wallet Balance:** Tổng số dư ví ($ USDT).
2. 💵 **Available Balance:** Số dư khả dụng có thể ký quỹ ($ USDT).
3. 📈 **Total uPnL:** Tổng Lời/Lỗ tạm tính của mọi vị thế đang mở ($ & %).
4. ⚡ **Active Assets:** Số cặp bot đang chạy trực tiếp (VD: `1 / 5 Active`).

### 2.2. System Health & Rate Limit Bar (`SystemHealthBar.tsx`)
- **Binance API Weight Bar:** Hiển thị mức tiêu tốn Weight Limit của REST API (VD: `45 / 2400`). Đổi màu cam/đỏ khi quá 80%.
- **WebSocket Latency Indicator:** Độ trễ dòng nến/tài khoản (VD: `45ms`).

### 2.3. Global Emergency Kill-Switch (`GlobalKillSwitch.tsx`)
- Nút bấm to nổi bật màu đỏ: **🚨 EMERGENCY STOP ALL BOTS**.
- Nhấn nút này gửi lệnh IPC hủy toàn bộ lệnh và đóng toàn bộ vị thế của mọi cặp tiền lập tức.
