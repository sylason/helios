# 💬 BIÊN BẢN THẢO LUẬN: THÔNG BÁO LỖI CÔ LẬP & XỬ LÝ VỊ THẾ AN TOÀN KHI GẶP SỰ CỐ

- **Thời gian:** 2026-09-09
- **Chủ đề:** Trải nghiệm Toast thông báo lỗi và quy tắc bảo vệ vốn khi xảy ra sự cố sàn.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Khi một cặp tiền gặp sự cố nghiêm trọng từ sàn Binance (ví dụ: thiếu tiền ký quỹ `-2019`, bị ngắt mạng cục bộ, hoặc lỗi trạng thái), hệ thống cần cô lập lỗi tại đúng cặp tiền đó, không làm sập các cặp tiền khác và bảo vệ an toàn cho vốn của người dùng.

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Trải Nghiệm Thông Báo Trên Giao Diện (UI/UX):**
   - Hiển thị **Floating Toast Notification** màu đỏ ở góc dưới bên phải màn hình (tự động biến mất sau 5 giây).
   - Dòng của cặp tiền gặp sự cố trên Watchlist Table tự động đổi sang Badge đỏ **`[🚨 ERROR]`**.
   - Chi tiết nguyên nhân lỗi được ghi nhận đầy đủ vào **`AssetLogPanel`** (5 dòng log gần nhất).
2. **Quy Tắc Bảo Vệ Vị Thế Khẩn Cấp (Safety Position Close):**
   - Nếu tại thời điểm phát sinh lỗi nghiêm trọng mà cặp tiền đó **đang có vị thế mở**:
     - Lõi Rust sẽ tự động gửi lệnh Market đóng vị thế ngay lập tức để cắt đứt mọi rủi ro thị trường.
     - Tắt bot của cặp tiền đó (chuyển trạng thái về `INACTIVE` / `ERROR`).
   - **Bảo toàn các cặp tiền khác (Fault Isolation):** Tất cả các cặp tiền còn lại (`BTCUSDT`, `SOLUSDT`...) **vẫn tiếp tục vận hành và giao dịch 100% bình thường**, hoàn toàn không bị gián đoạn.
