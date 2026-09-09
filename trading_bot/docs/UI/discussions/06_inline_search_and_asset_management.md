# 💬 BIÊN BẢN THẢO LUẬN: TÌM KIẾM & QUẢN TRỊ CẶP TIỀN TRỰC TIẾP (INLINE SEARCH & ASSET FILTER)

- **Thời gian:** 2026-09-09
- **Chủ đề:** Tinh giản trải nghiệm tìm kiếm, hiển thị thanh khoản và kích hoạt bot trên Bảng Tài Sản.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Thay vì phải mở một cửa sổ popup/modal riêng biệt để "Thêm cặp tiền" gây phức tạp thao tác, người dùng cần một giao diện trực quan, tinh gọn ngay trên Bảng Quản Trị Cặp Tiền (Watchlist Table).

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Thiết Kế Trực Tiếp Trên Bảng Tài Sản (Inline Table Design):**
   - Bố trí một **Thanh Tìm Kiếm (Search & Filter Bar)** ngay phía trên Bảng Asset.
   - Người dùng chỉ cần gõ tên cặp tiền (ví dụ: `ETH`, `BTC`, `SOL`...) $\rightarrow$ Bảng tự động lọc và hiển thị ngay dòng cặp tiền tương ứng.
2. **Các Trường Thông Tin Cốt Lõi Trên Mỗi Dòng Asset:**
   - **Tên Cặp & Loại Thị Trường:** `ETHUSDT` (FUTURES 20x).
   - **Chỉ Báo Thanh Khoản An Toàn (Safe Liquidity Badge):** Huy hiệu xanh `💧 Safe Liquidity` xác nhận cặp tiền đạt chuẩn khối lượng và spread thấp.
   - **Trạng Thái Vận Hành:** Badges màu trực quan (`INACTIVE`, `RUNNING_ACTIVE`...).
   - **Nút Hành Động:** Nút `[▶ START BOT]` kích hoạt ngay lập tức chuỗi nạp DNA và giao dịch.
3. **Lợi ích:** Tinh giản tối đa trải nghiệm người dùng (Zero-Modal Friction), phản hồi nhanh chóng và dễ quan sát.
