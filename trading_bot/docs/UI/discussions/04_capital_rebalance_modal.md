# 💬 BIÊN BẢN THẢO LUẬN: HỘP THOẠI ĐIỀU CHUYỂN VỐN AN TOÀN (REBALANCE MODAL)

- **Thời gian:** 2026-09-09
- **Chủ đề:** Quy trình và giao diện điều chuyển vốn khi cặp tiền đang có vị thế mở.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Khi một cặp tiền (ví dụ `ETHUSDT`) giao dịch có lãi thực tế ($Realized PnL > 0$), người dùng có nhu cầu rút bớt phần lãi này để cấp cho cặp tiền khác (`BTCUSDT`). Tuy nhiên, nếu rút vốn trong lúc lệnh ETH vẫn đang mở thì tỷ lệ đòn bẩy thực tế của vị thế đó sẽ bị đẩy lên cao, tiềm ẩn rủi ro nếu thị trường rung lắc mạnh.

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Trường hợp 1 (Vị thế đang FLAT):** Cập nhật ngay tức thì ($<0.1$s) số vốn mới cho cả 2 cặp tiền mà không cần hiện cảnh báo.
2. **Trường hợp 2 (Cặp tiền đang CÓ VỊ THẾ MỞ):** Hiển thị Modal Cảnh Báo An Toàn với 2 lựa chọn rõ ràng:
   - 🟢 **Lựa chọn A (Khuyến nghị - Áp Dụng Từ Lệnh Kế Tiếp):** Hệ thống ghi nhận số vốn mới và chờ đến khi lệnh hiện tại chốt xong SL/TP thì mới chính thức điều chuyển vốn. Vị thế đang chạy được bảo toàn $100\%$ an toàn.
   - ⚡ **Lựa chọn B (Đóng Lệnh Ngay Lập Tức):** Gửi lệnh Market đóng vị thế ngay tức khắc và thực hiện điều chuyển vốn ngay cho cặp tiền đích.
