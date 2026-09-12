# 🍞 ĐẶC TẢ KỸ THUẬT: HỆ THỐNG TOAST CÔ LẬP LỖI & ĐÓNG VỊ THẾ KHẨN CẤP (FAULT ISOLATION TOAST SPEC)

- **Vị trí Component:** `src/components/layout/ToastContainer.tsx` & `src/types/bot.ts`
- **Mã Tính Năng:** `UI-09` / `IPC-06`
- **Trạng thái:** ✅ **SPEC ĐÃ CHỐT**

---

## 🎯 1. Mục Đích & Nguyên Tắc Vận Hành

1. **Hiển Thị Thông Báo Nổi (Floating Toast):**
   - Đặt cố định ở góc dưới bên phải màn hình (`bottom: 24px`, `right: 24px`, `z-index: 1050`).
   - Tự động mờ dần và biến mất sau **5 giây** (`duration: 5000ms`).
   - Hỗ trợ 4 cấp độ thông báo:
     - 🚨 **`error` (Đỏ):** Lỗi nghiêm trọng từ sàn Binance (thiếu margin, lỗi API, đứt kết nối).
     - ⚠️ **`warning` (Vàng):** Cảnh báo API Weight tiệm cận ngưỡng trần (>1800/2400) hoặc trượt giá lớn.
     - ℹ️ **`info` (Xanh dương):** Hoàn tất JIT GA Optimization, tải Champion DNA mới.
     - ✅ **`success` (Xanh lá):** Điều chuyển vốn thành công, chuyển đổi chế độ bot thành công.

2. **Cơ Chế Cô Lập Lỗi (Fault Isolation - Error Containment):**
   - Khi nhận sự kiện lỗi nghiêm trọng (`event_asset_error`) từ một cặp tiền (ví dụ `ETHUSDT`):
     - Toast thông báo lỗi hiển thị: `[ETHUSDT] Sự cố sàn: ... Đã tự động đóng vị thế Market để bảo vệ vốn.`
     - Cặp tiền gặp sự cố chuyển trạng thái sang `ERROR` trên Watchlist Table.
     - 5 dòng log chi tiết về lỗi được đẩy vào `AssetLogPanel` của cặp tiền đó.
     - **Tất cả các cặp tiền khác (`BTCUSDT`, `SOLUSDT`...) vẫn tiếp tục vận hành và giao dịch 100% bình thường, không bị ảnh hưởng.**

3. **Cơ Chế Đóng Vị Thế Khẩn Cấp (Safety Position Close):**
   - Nếu cặp tiền gặp lỗi đang có vị thế mở $\implies$ Lõi Rust tự động bắn lệnh `MARKET` đóng toàn bộ khối lượng ngay lập tức để cắt đứt rủi ro biến động thị trường.
   - Trạng thái vị thế cập nhật về `FLAT` trên giao diện người dùng.

---

## 📐 2. Cấu Trúc Dữ Liệu Toast Notification

```typescript
export type ToastLevel = 'info' | 'success' | 'warning' | 'error';

export interface ToastMessage {
  id: string;
  symbol?: string;
  level: ToastLevel;
  title: string;
  message: string;
  timestamp: string;
}
```

---

## 🔘 3. Tương Tác & Hành Vi Giao Diện

- **Nút Đóng Thủ Công `[✕]`:** Cho phép người dùng tắt ngay toast trước khi hết thời gian 5s.
- **Xếp Chồng Tối Đa (Max Stacking):** Hiển thị tối đa 4 toasts cùng lúc; toast mới nhất nằm ở trên cùng, các toast cũ tự động dồn xuống dưới.
