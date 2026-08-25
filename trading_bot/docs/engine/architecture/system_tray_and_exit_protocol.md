# 🪟 System Tray & Unified Exit Protocol Architecture

Tài liệu thiết kế cơ chế Chạy nền dưới Khay Hệ Thống (System Tray) và Bảng Xác Nhận Thoát Tổng Hợp (Unified Exit Summary Modal).

---

## 🪟 1. Cơ Chế Chạy Nền Khay Hệ Thống (System Tray)

1. **Bấm dấu `[X]` trên cửa sổ giao diện:**
   - Cửa sổ Desktop UI tự động ẩn xuống Khay Hệ Thống (System Tray) ở góc phải Taskbar.
   - Toàn bộ các tiến trình `AssetActor`, `WsManager` và `ProspectorRunner` **vẫn tiếp tục chạy nền 24/7 bình thường**.
2. **Menu Chuột Phải Tại System Tray Icon:**
   - 🟢 **Mở Lại Cửa Sổ (Open Desk)**
   - ⏸️ **Tạm Dừng Tất Cả Bot (Pause All)**
   - 🚨 **Dừng Khẩn Cấp (Emergency Close All)**
   - ❌ **Thoát Hoàn Toàn Ứng Dụng (Exit Application)**

---

## 🛑 2. Bảng Xác Nhận Thoát Tổng Hợp (Unified Exit Summary Modal)

Khi người dùng chọn **Thoát Hoàn Toàn Ứng Dụng (Exit)** từ Tray Menu:

Hệ thống sẽ **không hiện nhiều popup rời rạc gây phiền hà**, mà hiển thị **Duy Nhất 1 Bảng Tổng Hợp**:

```text
+---------------------------------------------------------------------------------------+
|  ⚠️ CẢNH BÁO THOÁT ỨNG DỤNG - DANH SÁCH VỊ THẾ ĐANG MỞ                              |
+---------------------------------------------------------------------------------------+
|  Symbol    | Side   | Size        | Entry Price | Live uPnL  | Binance Trailing SL    |
+------------+--------+-------------+-------------+------------+------------------------+
|  ETHUSDT   | LONG   | 0.500 ETH   | $3,500.00   | +$25.40    | $3,480.00 (Protected)  |
|  BTCUSDT   | SHORT  | 0.020 BTC   | $64,200.00  | -$3.20     | $64,800.00 (Protected) |
+---------------------------------------------------------------------------------------+
|  Vui lòng chọn hành động trước khi đóng phần mềm:                                     |
|                                                                                       |
|  [⚡ ĐÓNG SẠCH TẤT CẢ MARKET]   [🛡️ GIỮ NGUYÊN STOP LOSS TRÊN SÀN]   [↩ HỦY BỎ]       |
+---------------------------------------------------------------------------------------+
```

### 🎯 Chi tiết 3 Lựa Chọn:
1. **`[⚡ ĐÓNG SẠCH TẤT CẢ MARKET]` (Flat All & Exit):**
   - Hủy toàn bộ lệnh Algo trên Binance.
   - Gửi lệnh Market đóng sạch mọi vị thế mở ngay lập tức $\rightarrow$ Tắt an toàn $100\%$ không còn rủi ro trên thị trường.
2. **`[🛡️ GIỮ NGUYÊN STOP LOSS TRÊN SÀN]` (Leave Cloud SL & Exit):**
   - Giữ nguyên các lệnh Algo `STOP_MARKET` đã treo trên Server Đám Mây của Binance.
   - Đóng phần mềm trên máy tính. Binance sẽ tự động kích hoạt cắt lỗ/chốt lời khi giá chạm ngưỡng.
3. **`[↩ HỦY BỎ]` (Cancel):**
   - Đóng hộp thoại, quay trở lại ứng dụng bình thường.
