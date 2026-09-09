# 💬 BIÊN BẢN THẢO LUẬN: VÒNG ĐỜI CỬA SỔ & CHẠY ẨN SYSTEM TRAY 24/7

- **Thời gian:** 2026-09-09
- **Chủ đề:** Hành vi nút đóng cửa sổ `[X]` và Menu chuột phải của System Tray.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Giao dịch tự động Binance Futures đòi hỏi bot phải hoạt động liên tục 24/7 để nhận nến và quản lý Trailing Stop Loss. Khi người dùng vô tình bấm nút `[X]` đóng cửa sổ, hệ thống không được dừng bot đột ngột làm mất khả năng bảo vệ vị thế trên thị trường.

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Hành vi khi bấm `[X]`:**
   - Cửa sổ giao diện React tự động ẩn xuống Khay Hệ Thống (System Tray) ở góc phải Taskbar Windows.
   - Toàn bộ các tiến trình nhận nến WebSocket, tính toán chỉ báo và dời Trailing SL trong Rust tiếp tục chạy ngầm bình thường 100%.
2. **Menu Chuột Phải tại Icon System Tray (4 mục):**
   - 🟢 **Mở Lại Cửa Sổ (Open Desk):** Hiện lại giao diện chính của phần mềm.
   - ⏸️ **Tạm Dừng Tất Cả Bot (Pause All):** Tạm khóa quyền vào lệnh mới cho toàn bộ các cặp tiền.
   - 🚨 **Dừng Khẩn Cấp (Emergency Close All):** Đóng sạch toàn bộ vị thế trên sàn ngay lập tức.
   - ❌ **Thoát Hoàn Toàn (Exit Application):** Bật **Bảng Thoát Tổng Hợp Duy Nhất** để người dùng chọn hành động cuối cùng trước khi tắt phần mềm.
