# 🔄 Architecture: Recovery & Position Management

Tài liệu mô tả cách hệ thống tự phục hồi và tiếp quản vị thế sau sự cố.

---

## 1. Luồng khởi động "Takeover" (Boot-up Handshake)
Khi App khởi động, thay vì bắt đầu từ con số 0, nó thực hiện quy trình tiếp quản:
1. **Query Exchange:** Gọi API sàn để lấy danh sách vị thế (`get_open_positions`) và lệnh chờ (`get_open_orders`).
2. **State Reconstruction:**
    - Nếu có vị thế đang mở: Gán vị thế vào `AssetState` của Asset tương ứng.
    - Áp dụng ngay bộ **Trailing Stoploss** an toàn dựa trên thông tin chiến lược hiện có.
3. **Active Protection:** Kích hoạt ngay luồng **Observer Task** để giám sát vị thế đó.
4. **Re-analyze:** Chạy ngay một chu kỳ Backtest/Optimize khẩn cấp để cập nhật bộ tham số phù hợp nhất với thị trường hiện tại cho vị thế đang mở.

## 2. Quản lý Vị thế ngoại lai (Manual Trades)
Nếu phát hiện vị thế trên sàn không do Bot mở (ví dụ người dùng đặt tay):
- **Cơ chế:** Bot vẫn sẽ tiếp quản và áp dụng các quy tắc bảo vệ vốn (Trailing SL) như một vị thế bình thường để đảm bảo an toàn tối đa cho tài khoản.

## 3. Cơ chế Panic & Shutdown
Tùy thuộc vào tình huống sự cố, hệ thống áp dụng các mức độ dừng:
- **Level 1 (Soft Stop):** Ngừng vào lệnh mới, tìm điểm thoát tốt nhất cho lệnh cũ.
- **Level 2 (Hard Panic):** Đóng toàn bộ ngay lập tức (dùng khi mất điện/crash hệ thống lặp lại).
- **Recovery Option A:** Ưu tiên đóng tất cả vị thế ngay khi có lời hoặc đóng lập tức nếu rủi ro quá cao.
