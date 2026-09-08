# Logic Workflow - Trading Bot (Thiết kế "Một chạm" v2)

Tài liệu này hệ thống hóa luồng thực thi logic đằng sau giao diện người dùng (UI) tối giản, tập trung vào cơ chế tự động hóa hoàn toàn.

## 1. Các Trạng thái của Asset (Asset States)
- `INACTIVE`: Trạng thái nghỉ, không lấy dữ liệu, không trade.
- `ACTIVATING`: Đang thực hiện chuỗi: Sync Data ➔ Initial Optimize.
- `TRADING`: Đang kết nối real-time và thực thi chiến lược (`Signal = READY`).
- `TERMINATING_GRACEFUL`: Đang dừng an toàn (Chỉ đóng lệnh bằng Trailing SL, không mở mới).
- `STOPPED`: Đã dừng hoàn toàn sau lệnh Panic hoặc Graceful.

---

## 2. Luồng thực thi chi tiết (Command Flows)

### 2.1. Lệnh [ACTIVATE] (Nút chính)
Kích hoạt chu trình tự động "Một chạm".

1.  **Data & Network Setup**: 
    - Mở kết nối WebSocket (Real-time).
    - Đồng thời gọi Binance REST API để bù đắp dữ liệu lịch sử (History Backfill) cho đủ window 30 ngày.
2.  **Background Optimization**: 
    - Ngay khi có đủ dữ liệu, hệ thống tự động kích hoạt `Prospector Engine` chạy ẩn.
    - Người dùng không cần nhấn nút Optimize riêng lẻ.
3.  **Signal Unlock**: 
    - Khi chu kỳ Optimize đầu tiên hoàn tất ➔ Cập nhật `Signal State = READY`.
    - Bot tự động chuyển từ quan sát sang thực thi lệnh (Trading).
4.  **Adaptive Loop**: 
    - Duy trì Optimize định kỳ (mỗi vài giờ) để cập nhật `best_params.json` (Hot-swap).

### 2.2. Lệnh [DEACTIVATE / TERMINATE IMMEDIATELY]
Dừng khẩn cấp ("Rút phích cắm").

1.  **Market Close**: Gửi lệnh đóng toàn bộ Position hiện tại bằng giá Market.
2.  **Purge Orders**: Hủy sạch mọi lệnh chờ (Limit/Stop) trên sàn.
3.  **Cut Connections**: Ngắt WebSocket, ngừng nhận Data, đóng Actor để giải phóng tài nguyên.
4.  *State*: `STOPPED`.

### 2.3. Lệnh [TERMINATE GRACEFULLY] (Nút phụ)
Dừng an toàn, bảo vệ lợi nhuận cuối cùng.

1.  **Lock Entries**: Chặn hoàn toàn việc mở vị thế mới hoặc lệnh mới.
2.  **Passive Management**: 
    - Vẫn duy trì WebSocket để cập nhật giá.
    - Để hệ thống Trailing SL tự động quản lý và đóng nốt các vị thế đang có.
3.  **Auto-Shutdown**: Khi số dư vị thế về 0 ➔ Tự động thực hiện quy trình ngắt kết nối và dừng Actor.
4.  *State*: `TERMINATING_GRACEFUL` ➔ `STOPPED`.

---

## 3. Cơ chế Signal State
- **Signal = WAIT**: Bot đang lấy dữ liệu hoặc đang Optimize lần đầu. Chưa được phép đặt lệnh.
- **Signal = READY**: Đã có bộ tham số tối ưu và dữ liệu ổn định. Sẵn sàng thực thi chiến lược.

---
---

🟢 1. Luồng Kích hoạt & Đồng bộ (Activation & Sync)
Mục tiêu: Đưa Asset từ trạng thái "Ngủ" sang "Sẵn sàng".

1. UI (React): Nhấn nút Activate ➔ Tauri Command: Nhận Symbol (ví dụ: ETHUSDT).
2. Tauri Bridge: Gửi lệnh qua Tokio Channel ➔ AssetActor: Thức dậy và khởi tạo.
3. AssetActor: Đổi trạng thái sang InitialSync ➔ Binance REST API: Gọi lấy nến lịch sử.
4. Data Flow: Binance ➔ InitialSync module ➔ DuckDB: Lưu bền vững ➔ Asset Memory: Nạp vào Rolling Buffer (43,200 nến).
5. Kết quả: Hoàn tất 30 ngày dữ liệu trong bộ nhớ và ổ cứng.

---

🧬 2. Luồng Tối ưu hóa ngầm (Background Optimization)
Mục tiêu: Tìm bộ thông số (DNA) tốt nhất cho chiến lược hiện tại.

1. AssetActor: Sau khi Sync xong ➔ Tra cứu DuckDB: "Đã có DNA cho ETHUSDT chưa?".
2. Nếu CHƯA có: Đổi trạng thái sang Optimizing.
    * Worker: Trích xuất 30 ngày dữ liệu từ DuckDB.
    * GA Optimizer (ferro_trade): Chạy tiến hóa để tìm DNA có lợi nhuận cao nhất.
    * Storage: Lưu DNA tìm được vào DuckDB.
3. Nếu ĐÃ có: Nạp DNA cũ lên ➔ Cập nhật vào cấu trúc Strategy.
4. Kết quả: Chiến lược Helios đã có "Não bộ" (tham số) tối ưu.

---

🚀 3. Luồng Phân tích & Giao dịch Live (Live Trading Loop)
Mục tiêu: Xử lý nến thời gian thực và ra quyết định.

1. Binance WebSocket: Bắn nến 1m vừa đóng (is_final) ➔ EventBus: Phát tán tín hiệu.
2. AssetActor: Nhận nến ➔ Rolling Buffer: Thêm nến mới, xóa nến cũ nhất (duy trì đúng 30 ngày).
3. Analysis Bridge:
    * Gửi 43,200 nến ➔ HeliosBridge.
    * Simulated Backtest: Chạy lại logic Helios trên 30 ngày để khớp chỉ báo.
4. Signal: Helios trả về Actions (Mua/Bán/Đóng) ➔ Execution Module: (Sắp làm) Gửi lệnh lên sàn.
5. UI Feedback: Phát Event ➔ UI cập nhật giá và trạng thái lệnh.

---

🔴 4. Luồng Dừng & Giải phóng (Termination)
Mục tiêu: Rút lui an toàn hoặc ngắt điện khẩn cấp.

1. Nút bấm UI: Chọn Graceful hoặc Immediate.
2. Emergency Channel: Gửi lệnh xuống AssetActor.
3. Xử lý logic:
    * Immediate: Hủy mọi lệnh chờ ➔ Đóng Market vị thế ngay ➔ Dừng Actor.
    * Graceful: Chặn không cho vào lệnh mới (Entry) ➔ Để Trailing SL tự xử lý nốt lệnh đang mở ➔ Đóng khi hết lệnh.
4. Dọn dẹp: Lưu trạng thái cuối ➔ Đóng kết nối Database ➔ Giải phóng RAM.

---

📝 Tóm tắt Kỹ thuật (Cheat Sheet)
* Data: REST (Quá khứ) ➔ WebSocket (Hiện tại) ➔ DuckDB (Bền vững).
* Logic: Actor (Điều phối) ➔ Bridge (Backtest mini) ➔ Strategy (Quyết định).
* IPC: Command (Xuống) ➔ Event (Lên) ➔ Channel (Ngang).