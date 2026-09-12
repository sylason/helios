# 🖥️ ĐẶC TẢ KỸ THUẬT: BẢNG THOÁT TỔNG HỢP (UNIFIED EXIT MODAL SPEC)

- **Vị trí Component:** `src/components/layout/UnifiedExitModal.tsx`
- **Mã Tính Năng:** `UI-08` / `IPC-05`
- **Trạng thái:** ✅ **SPEC ĐÃ CHỐT**

---

## 🎯 1. Mục Đích & Nguyên Tắc Vận Hành

Khi người dùng chọn **`Thoát Hoàn Toàn Ứng Dụng (Exit Application)`** từ System Tray Menu hoặc kích hoạt lệnh thoát ứng dụng:
1. **Kiểm Tra Trạng Thái Vị Thế (In-Position Check):**
   - Nếu **TẤT CẢ các cặp tiền đều FLAT (0 lệnh mở)** $\implies$ Kích hoạt **Fast Exit**: Đóng phần mềm ngay lập tức mà không hiện popup thừa.
   - Nếu **CÓ ÍT NHẤT 1 VỊ THẾ ĐANG MỞ** $\implies$ Tự động bật **Bảng Thoát Tổng Hợp Duy Nhất (Unified Exit Modal)**.

---

## 📐 2. Giao Diện & Bố Cục Bảng Tổng Hợp

Bảng Modal sử dụng Dark Theme (`#0F172A`), hiển thị danh sách tất cả các cặp tiền đang mở lệnh với 6 cột chi tiết:

| Cột | Ý Nghĩa Hiển Thị |
| :--- | :--- |
| **1. Symbol** | Tên cặp tiền (vd: `ETHUSDT`, `BTCUSDT`). |
| **2. Phía Vị Thế (Side)** | Badge Xanh cho `LONG`, Badge Đỏ cho `SHORT`. |
| **3. Khối Lượng (Size)** | Khối lượng hợp đồng đang nắm giữ (vd: `1.500 ETH`). |
| **4. Giá Vào (Entry Price)** | Mức giá khớp ban đầu (vd: `$1,905.40`). |
| **5. Lãi/Lỗ Tạm Tính (uPnL)** | Số tiền và % lãi/lỗ thời gian thực (vd: `+$45.20 (+2.37%)`). |
| **6. Trailing SL (Binance)** | Mức giá cắt lỗ/chốt lời Algo đã treo trên máy chủ Binance (vd: `🛡️ $1,929.54`). |

---

## 🔘 3. Ba Nút Hành Động Quyết Định

1. ⚡ **`[ĐÓNG SẠCH TẤT CẢ MARKET]` (`FLAT_ALL`):**
   - Gửi lệnh Market đóng toàn bộ vị thế đang mở trên Binance ngay lập tức.
   - Sau khi các lệnh đóng hoàn tất $\rightarrow$ Tắt phần mềm an toàn 100%.
2. 🛡️ **`[GIỮ NGUYÊN STOP LOSS TRÊN SÀN]` (`LEAVE_CLOUD_SL`):**
   - Giữ nguyên các lệnh Algo `STOP_MARKET` đã treo trên server đám mây của Binance.
   - Tắt phần mềm. Binance sẽ tự động chốt lời/cắt lỗ khi giá chạm ngưỡng.
3. ↩️ **`[HỦY BỎ]` (`CANCEL`):**
   - Đóng hộp thoại Modal, quay trở lại ứng dụng bình thường.
