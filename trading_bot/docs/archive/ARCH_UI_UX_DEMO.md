# 🎨 Architecture: UI/UX Design (Minimalist Asset Table)

Thiết kế giao diện người dùng tối giản, tập trung vào hiệu suất và khả năng điều khiển "Một chạm" cho đa tài sản.

---

## 🛠️ 1. Công nghệ Stack
- **Core Engine:** Rust (Logic trading, Data engine, Risk management).
- **UI Framework:** **Tauri v2** (Rust Backend + React/TypeScript Frontend).
- **State Management:** Tauri IPC để truyền dữ liệu real-time và trạng thái `signal_ready`.

## 🖼️ 2. Bố cục Giao diện (Asset Table)
Thay vì các Widget riêng lẻ, App sử dụng một bảng tổng quát để quản lý hàng loạt Asset:

| Category | Exchange | Asset | Action 1 (Main) | Action 2 (Graceful) | Signal |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **FUTURES** | **BINANCE** | **ETHUSDT** | `Activate` / `Deactivate` | `Terminate Gracefully` | `READY/WAIT` |

### Các thành phần hiển thị:
- **Category:** Phân loại tài sản (Futures, Crypto, Forex, Stocks).
- **Exchange:** Sàn giao dịch tương ứng.
- **Signal:** Trạng thái bộ máy tối ưu (`READY` nếu đã có tham số và sẵn sàng trade, `WAIT` nếu đang optimize).

## 🛑 3. Hệ thống Điều khiển & An toàn
- **Nút [Activate / Deactivate (Immediate)]:** 
    - **Activate:** Kích hoạt chuỗi tự động: Lấy dữ liệu (Realtime + History) ➔ Chạy Optimize ngầm ➔ Tự động Trade khi Signal Ready.
    - **Deactivate (Immediate):** Lệnh dừng khẩn cấp. Đóng ngay lập tức mọi Position (Market), hủy Order và ngắt toàn bộ luồng dữ liệu (WebSocket).
- **Nút [Terminate Gracefully]:** Dừng thông minh. Ngừng mở lệnh mới nhưng vẫn duy trì WebSocket để Trailing SL tự đóng các vị thế đang mở.

## 📂 4. Nhật ký & Giám sát
- Người dùng chủ yếu theo dõi PnL và lệnh trên app của sàn (ví dụ Binance App) để kiểm chứng độ chính xác.
- UI chỉ cung cấp log hệ thống tối giản để biết Bot đang ở trạng thái nào.
