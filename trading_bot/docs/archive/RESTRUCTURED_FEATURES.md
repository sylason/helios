# 🤖 Trading Bot - Cấu Trúc Chức Năng & Trang (Thiết kế Tối giản v2)

Tài liệu này định hình lại toàn bộ ứng dụng theo triết lý "Kích hoạt Một chạm" và quản lý tài sản theo dạng Bảng.

---

## 🏗️ 1. Giao Diện Người Dùng (Simplified UI)

### Trang chính: Asset Dashboard
Giao diện tập trung vào một bảng duy nhất hiển thị danh sách các tài sản.

| Phân loại | Sàn | Tài sản | Hành động 1 (Chính) | Hành động 2 (Phụ) | Trạng thái (Signal) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **FUTURES** | **BINANCE** | **ETHUSDT** | `Activate` / `Deactivate` | `Graceful Terminate` | `READY/WAIT` |

#### Chi tiết các nút điều khiển:
1.  **Nút: [Activate]** (Màu Xanh)
    - **Logic Một chạm:** Tự động thực hiện chuỗi:
        1.  Kết nối WebSocket (Real-time) & Sync dữ liệu (History).
        2.  Kích hoạt Optimize chạy ẩn (Background).
        3.  Khi có kết quả Optimize đầu tiên ➔ Trạng thái Signal chuyển sang `READY` ➔ Tự động vào trạng thái `Trading`.
    - **Khi đang chạy:** Nút này chuyển thành nút **[Deactivate (Immediate)]** (Màu Đỏ).

2.  **Nút: [Deactivate (Terminate Immediately)]**
    - **Nhiệm vụ:** Lệnh "Rút phích cắm" khẩn cấp. 
    - **Hành động:** Đóng mọi Position ngay lập tức (Market Close), hủy toàn bộ lệnh chờ (Order), ngắt kết nối WebSocket và dừng lấy dữ liệu cho Asset đó.

3.  **Nút: [Terminate Gracefully]** (Màu Vàng)
    - **Nhiệm vụ:** Lệnh "Rút lui an toàn". 
    - **Hành động:** Chuyển trạng thái Asset sang "Chỉ đóng, không mở". Ngừng mở trade/order mới. Giữ WebSocket để Trailing SL tự động đóng các vị thế hiện tại theo chiến thuật.

---

## ⚙️ 2. Chức Năng Nền (Background Logic)

- **Optimize Chạy ẩn:** Đây là trái tim của hệ thống. Nó tự động chạy khi nhấn Activate và lặp lại định kỳ (ví dụ sau mỗi vài giờ) để thích nghi với thị trường. Người dùng không cần can thiệp.
- **Signal State:** Biến trạng thái quan trọng nhất. Chỉ khi `Signal = READY`, các module Trade mới được phép gửi lệnh lên sàn.
- **Phân loại Asset:** Kiến trúc sẵn sàng cho đa sàn (Binance, Forex Brokers) và đa tài sản (Futures, Crypto Spot, Forex).

---

## 📝 3. Ghi chú Ý tưởng & Thảo luận
- **Giám sát:** Người dùng sẽ quan sát biểu đồ và lệnh trực tiếp trên App của sàn (Binance) để kiểm tra tính ổn định của Bot.
- **Tối giản:** UI chỉ hiển thị những gì thực sự cần để điều khiển, nhường CPU/RAM cho bộ máy Optimize và Trade Engine.
