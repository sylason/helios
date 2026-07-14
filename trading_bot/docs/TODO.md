# 🗺️ Trading Bot Roadmap - Bản Demo & v0.1

Tài liệu này liệt kê chi tiết các chức năng cần hoàn thiện, sắp xếp theo thứ tự ưu tiên và phụ thuộc kỹ thuật.

---

## 🛡️ TẦNG 1: HỆ THỐNG PHÒNG NGỰ (The Shield)
*Mục tiêu: Đảm bảo Bot không bị Binance khóa IP và dữ liệu luôn liên tục.*

- [ ] **1. Weight Monitor (Giám sát tải API)**
    - **Nhiệm vụ:** Đọc header `x-mbx-used-weight-1m`, cung cấp hàm `is_safe()` cho các module khác.
    - **Phụ thuộc:** Không.
- [ ] **2. Gap Filler (Cỗ máy bù nến)**
    - **Nhiệm vụ:** So sánh nến cuối trong DB với nến Live, tự động gọi REST API lấy nến thiếu khi mất mạng.
    - **Phụ thuộc:** **Weight Monitor**.
- [ ] **3. User Stream Keep-alive**
    - **Nhiệm vụ:** Kết nối WebSocket private, tự động gửi `listenKey` mỗi 30-60 phút để không bị ngắt kết nối.
    - **Phụ thuộc:** Không.

---

## 🧠 TẦNG 2: BỘ NÃO DỮ LIỆU (The In-Memory Core)
*Mục tiêu: Xử lý dữ liệu tốc độ cực cao trong RAM bằng Polars.*

- [ ] **4. In-Memory DataFrame Manager (Polars Bridge)**
    - **Nhiệm vụ:** Tải **30 ngày OHLCV** từ DuckDB lên RAM khi khởi động. Duy trì cửa sổ dữ liệu này liên tục (Rolling Window).
    - **Phụ thuộc:** **DuckDB Storage**.
- [ ] **5. Recalculation Engine**
    - **Nhiệm vụ:** Xóa toàn bộ Indicator cũ và tính lại **30 ngày** chỉ trong vài ms khi nhận tham số mới từ Optimizer.
    - **Phụ thuộc:** **In-Memory DataFrame Manager**.
- [ ] **6. Streaming Risk Monitor (ATR & PNL)**
    - **Nhiệm vụ:** Tính toán ATR (biến động) và PNL (lời/lỗ tạm tính) theo từng nhịp nhảy của giá để quản lý Trailing Stoploss.
    - **Phụ thuộc:** **In-Memory DataFrame Manager**.

---

## 📈 TẦNG 3: CHIẾN LƯỢC & TỐI ƯU (The Strategy)
*Mục tiêu: Tự động hóa hoàn toàn việc tìm kiếm lợi nhuận.*

- [ ] **7. Background Optimizer (Prospector Engine)**
    - **Nhiệm vụ:** Tích hợp bộ `Prospector` từ `ferro_trade`. Tự động chạy GA trên **30 ngày dữ liệu nến 1m** gần nhất.
    - **Hành vi:** Chạy ẩn định kỳ (mỗi vài giờ) sau khi người dùng nhấn Activate.
    - **Phụ thuộc:** **DuckDB Storage**, **Module 11**.
- [ ] **8. Hot-swap Controller**
    - **Nhiệm vụ:** Nhận tham số (DNA) tốt nhất từ Optimizer và ra lệnh cho `Recalculation Engine` thực thi ngay lập tức (kể cả khi đang có lệnh).
    - **Phụ thuộc:** **Recalculation Engine**.
- [ ] **11. Helios Strategy Integration (v1)**
    - **Nhiệm vụ:** Port code Helios sang `trading_bot`. Xây dựng `LiveStrategyWrapper` để Helios đọc nến Real-time + 30 ngày lịch sử.
    - **Phụ thuộc:** Không.

---

## 💸 TẦNG 4: THỰC THI & HIỂN THỊ (Minimalist UI)
*Mục tiêu: Giao diện bảng tài sản "Một chạm".*

- [ ] **9. Trade Executor (Testnet)**
    - **Nhiệm vụ:** Đặt lệnh Mua/Bán, Stop-loss thực tế lên Binance Testnet.
    - **Phụ thuộc:** **Weight Monitor**, **User Stream**.
- [ ] **10. Minimal Asset Table (Tauri v2)**
    - **Nhiệm vụ:** Bảng quản lý đa tài sản (Futures, Forex...). Hiển thị Signal State (READY/WAIT).
    - **Controls:** Nút **Activate / Deactivate (Immediate)** và nút **Terminate Gracefully**.
    - **Phụ thuộc:** **Module 4, 6 & 11**.

---

## ⚡ Chiến lược Triển khai Song song (Parallel Strategy)

- **Luồng A (Data Pipeline):** Module 1 ➔ Module 4 ➔ Module 2. (Mục tiêu: Nến 30 ngày sạch).
- **Luồng B (Private Stream):** Module 3 ➔ Module 9. (Mục tiêu: Đặt lệnh và quản lý ví).
- **Luồng C (Strategy Brain):** Module 11 ➔ Module 5 (Recalc) ➔ Module 7 (Optimize ngầm) ➔ Module 8 (Hot-swap).

---

## 📝 Ghi chú Kỹ thuật (Thiết kế v2)
*   **One-tap Activation:** Nút Activate kích hoạt toàn bộ chuỗi: Sync 30 ngày ➔ Optimize ngầm ➔ Trade.
*   **Enum Signal State:** Sử dụng Enum thay vì Boolean để quản lý trạng thái (`ACTIVATING`, `TRADING`, `TERMINATING...`).
*   **DB Persistence:** `Prospector` lưu DNA vào DuckDB để phục hồi nhanh sau khi khởi động app.
