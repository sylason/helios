# ⚙️ Trang 2: Active Watchlist & Bot Control Center (Thiết Kế Chi Tiết)

Trang ưu tiên số 1 trong giai đoạn Demo, dùng để kích hoạt, theo dõi vị thế và can thiệp điều khiển bot từng cặp giao dịch.

---

## 🎯 1. Nhiệm Vụ & Luồng Hoạt Động

- Quản lý danh sách các cặp tiền giao dịch (Active Watchlist: ETHUSDT, BTCUSDT,...).
- Kích hoạt chu kỳ tự động của Engine cho từng cặp: **Sync Data (60D) -> GA Prospector Optimize -> Active Live Trading**.
- Can thiệp dừng an toàn (`GRACEFUL_STOP`) hoặc dừng khẩn cấp (`STOP_IMMEDIATELY`).

---

## 📊 2. Cấu Trúc Bảng Watchlist Table (`WatchlistTable.tsx`)

### Các cột thông tin (Columns Breakdown):
1. **Asset Identity:** Symbol (`ETHUSDT`), Thị trường (`FUTURES 20x`).
2. **Status Badge (Trạng thái Vận hành):**
   - `INACTIVE` (Xám) - Bot đang tắt.
   - `SYNCING_DATA` (Vàng chớp) - Đang nạp nến lịch sử từ Binance REST.
   - `GA_OPTIMIZING` (Tím chớp) - Prospector đang tiến hóa ngầm bộ DNA.
   - `READY_TRADING` (Xanh lục) - Sẵn sàng nhận signal nến real-time.
   - `GRACEFUL_EXIT` (Cam) - Đang chờ dọn dẹp vị thế cũ trước khi ngắt.
   - `ERROR` (Đỏ) - Gặp lỗi API/Mạng.
3. **Champion DNA Metrics:** Tên chiến lược (`Helios V3`) + In-Sample Return % & Max Drawdown %.
4. **Live Position State:** `FLAT` hoặc `LONG` / `SHORT` (`Qty @ EntryPrice`) + Realtime Un-realized PnL ($ và %).
5. **Target Trailing SL:** Mức giá Trailing StopLoss đang cài trên sàn Binance.
6. **Action Controls (Nút Thao Tác):**
   - 🟢 **ACTIVATE BOT**: Bật bot cho cặp tiền.
   - 🟡 **GRACEFUL STOP**: Dừng an toàn (chờ Trailing SL chốt lệnh).
   - 🔴 **STOP IMMEDIATELY**: Ngắt khẩn cấp (Hủy lệnh + Đóng vị thế Market).

---

## 📋 3. Nhật Ký Sự Kiện Nhanh (`AssetLogPanel.tsx`)

- Khi chọn một dòng Asset trong bảng, Panel bên dưới sẽ hiển thị 5 dòng log mới nhất của riêng cặp tiền đó.
- Giúp người dùng biết bot đang làm gì (VD: *22:15:00 - Trailing SL updated to 1929.54*).
