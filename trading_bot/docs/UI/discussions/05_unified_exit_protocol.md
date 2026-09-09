# 💬 BIÊN BẢN THẢO LUẬN: GIAO THỨC THOÁT TỔNG HỢP (UNIFIED EXIT PROTOCOL)

- **Thời gian:** 2026-09-09
- **Chủ đề:** Quy trình xác nhận và đóng ứng dụng khi có/không có vị thế mở.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Khi tắt phần mềm, người dùng cần được kiểm soát tuyệt đối các vị thế tiền thật đang mở trên sàn Binance Futures, tránh tình trạng ứng dụng bị tắt đột ngột nhưng các vị thế trôi nổi không được bảo vệ.

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Trường hợp A (Tất cả vị thế đều FLAT - Không có lệnh mở):**
   - Áp dụng cơ chế **Thoát Nhanh (Fast Exit)**: Ứng dụng đóng ngay lập tức mà không hiển thị popup thừa thãi, mang lại trải nghiệm tinh gọn, mượt mà.
2. **Trường hợp B (Đang CÓ VỊ THẾ MỞ trên bất kỳ cặp tiền nào):**
   - Tự động bật **Bảng Xác Nhận Thoát Tổng Hợp Duy Nhất (Unified Exit Modal)**.
   - Hiển thị bảng chi tiết gồm 6 cột: `Symbol`, `Side`, `Size`, `Entry Price`, `Live uPnL`, `Binance Trailing SL Target`.
   - Cung cấp 3 nút hành động dứt khoát:
     - ⚡ **`[ĐÓNG SẠCH TẤT CẢ MARKET]` (Flat All & Exit):** Gửi lệnh Market đóng toàn bộ vị thế $\rightarrow$ Thoát ứng dụng an toàn 100%.
     - 🛡️ **`[GIỮ NGUYÊN STOP LOSS TRÊN SÀN]` (Leave Cloud SL & Exit):** Giữ nguyên lệnh Algo Trailing SL treo trên server Binance $\rightarrow$ Thoát ứng dụng.
     - ↩️ **`[HỦY BỎ]` (Cancel):** Đóng popup, tiếp tục sử dụng ứng dụng.
