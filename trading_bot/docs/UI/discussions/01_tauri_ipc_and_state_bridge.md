# 💬 BIÊN BẢN THẢO LUẬN: KIẾN TRÚC CẦU NỐI TAURI IPC & STATE BRIDGE (DEMO DESKTOP)

- **Thời gian:** 2026-09-09
- **Chủ đề:** Lựa chọn cơ chế State Bridge trong `src-tauri` cho bản Demo Local-First.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Chúng ta cần phát triển nhanh giao diện Desktop UI và hệ thống điều khiển của `trading_bot` độc lập với quá trình debug chiến lược toán học và logic paper trade đang diễn ra ở luồng khác. Bản Demo cần chạy hoàn toàn trên máy cục bộ (Local Desktop App), đảm bảo tính riêng tư (Zero Network Port) và tốc độ phản hồi tức thì.

---

## ⚖️ 2. Các Phương Án Cân Nhắc

| Tiêu Chí | Phương Án 1 (Được Chọn): Dual-Mode IPC Bridge (Focus Desktop Local) | Phương Án 2: Pure Engine State Only |
| :--- | :--- | :--- |
| **Môi trường hoạt động** | Tối ưu 100% cho ứng dụng Desktop Tauri cục bộ, có fallback an toàn cho Browser Dev. | Chỉ chạy khi có kết nối mạng và API key Binance thật 100%. |
| **Tốc độ phát triển** | Rất nhanh: Có thể test toàn bộ UI/UX, modals, nút bấm, FSM transitions ngay trên Desktop. | Chậm hơn: Phụ thuộc vào tính sẵn sàng của Binance API và mạng Internet. |
| **Tính riêng tư** | Tuyệt đối: Chạy ngầm trong process nội bộ của máy tính, không mở cổng HTTP/WS ra ngoài. | Tuyệt đối. |

---

## 🎯 3. Quyết Định Kỹ Thuật (Decision Log)

1. **Thống nhất:** Triển khai **Dual-Mode IPC Bridge** nhưng **tập trung 100% vào trải nghiệm Local Desktop** trên Tauri v2.
2. **Cấu trúc `src-tauri`:**
   - Quản lý State trong Rust bằng `Arc<RwLock<DesktopAppState>>` trong `tauri::State`.
   - Cung cấp đầy đủ các command handlers (`cmd_...`) xử lý logic chuyển đổi FSM, cập nhật vị thế và quản trị rủi ro.
   - Sẵn sàng gắn `bot_engine` vào khi hoàn thành việc debug logic chiến lược.
