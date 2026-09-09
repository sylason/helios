# ⚙️ Trang 2: Active Watchlist & Bot Control Center (Thiết Kế Chi Tiết)

Trang ưu tiên số 1 trong giai đoạn Demo, dùng để kích hoạt, theo dõi vị thế, quản lý vốn và can thiệp điều khiển bot từng cặp giao dịch.

---

## 🎯 1. Nhiệm Vụ & Luồng Hoạt Động

- Quản lý danh sách các cặp tiền giao dịch (Active Watchlist: ETHUSDT, BTCUSDT,...).
- Kích hoạt chu kỳ tự động của Engine cho từng cặp: **Check Nến Lịch Sử -> Check Hạn Dùng DNA (JIT) -> GA Prospector Optimize -> Active Live Trading**.
- Hỗ trợ đầy đủ các thao tác điều khiển đối xứng: `START`, `CANCEL_START`, `PAUSE`, `RESUME`, `GRACEFUL_STOP`, `CANCEL_GRACEFUL_STOP`, `FORCE_STOP_IMMEDIATE`, `REMOVE_ASSET`.

---

## 🔍 2. Hệ Thống Thanh Lọc Nhanh (Filter Tab System)

Thay vì dùng Dropdown đơn điệu, bảng Watchlist trang bị thanh tab lọc phân loại trạng thái:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  [ ALL (5) ]   [ 🟢 ACTIVE (2) ]   [ ⏸️ PAUSED (1) ]   [ ⚪ INACTIVE (2) ]   [ 💧 LIQUIDITY PASSED ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

1. **Tab `ALL` (Mặc định Demo):** Hiển thị toàn bộ các cặp tiền đang theo dõi trong hệ thống.
2. **Tab `🟢 ACTIVE TRADING`:** Chỉ hiển thị các cặp đang chạy thực chiến hoặc đang có vị thế mở (`IN_POSITION`).
3. **Tab `⏸️ PAUSED`:** Các cặp đang tạm dừng tìm tín hiệu mới (vẫn giữ Trailing SL trên sàn).
4. **Tab `⚪ INACTIVE`:** Các cặp đang tắt hoặc ở chế độ chờ.
5. **Tab `💧 LIQUIDITY PASSED` (Tầm nhìn tương lai):** Tự động lọc các cặp tiền đạt chuẩn thanh khoản cao ($\text{Volume 24h} > \$50\text{M}$, $\text{Spread} < 0.02\%$, Orderbook sâu) để chống trượt giá (Slippage).

---

## 📊 3. Cấu Trúc Bảng Watchlist Table (`WatchlistTable.tsx`)

### Các cột thông tin (Columns Breakdown):
1. **Asset Identity:** Symbol (`ETHUSDT`), Thị trường (`FUTURES 20x`).
2. **Status Badge (Trạng thái Vận hành):**
   - `INACTIVE` (Xám) - Bot đang tắt.
   - `STARTING (SYNCING)` (Vàng chớp) - Đang nạp nến lịch sử từ Binance REST.
   - `STARTING (GA_OPTIMIZING)` (Tím chớp) - Đang chạy GA Just-In-Time lượt đầu.
   - `RUNNING_ACTIVE` (Xanh lục) - Sẵn sàng nhận signal nến real-time.
   - `PAUSED` (Vàng) - Tạm dừng mở lệnh mới, vẫn giữ Trailing SL.
   - `GRACEFUL_STOPPING` (Cam) - Đang chờ Trailing SL chốt lệnh cũ trước khi ngắt.
   - `ERROR` (Đỏ) - Gặp lỗi API/Mạng.
3. **Vốn Cấp Phát & Lãi Kép (Allocated Capital):**
   - Hiển thị mức vốn hiện tại (VD: `$80.00` = Vốn gốc `$60.00` + Lãi `$20.00`).
   - Nút nhỏ `[⚙️ Rebalance]` để mở hộp thoại điều chuyển vốn giữa các Asset.
4. **Champion DNA Metrics:** Tên chiến lược (`Helios V3`) + In-Sample Return % & Max Drawdown %.
5. **Live Position State:** `FLAT` hoặc `LONG` / `SHORT` (`Qty @ EntryPrice`) + Realtime Un-realized PnL ($ và %).
6. **Target Trailing SL:** Mức giá Trailing StopLoss thực tế đang treo trên Binance (theo **Mark Price**).
7. **Action Controls (Nút Thao Tác Ngữ Cảnh):**
   - Khi `INACTIVE`: 🟢 `[▶ START BOT]` | 🗑️ `[REMOVE]`
   - Khi `STARTING`: ❌ `[✖ CANCEL START]`
   - Khi `RUNNING_ACTIVE`: ⏸️ `[PAUSE]` | 🛑 `[STOP]` (Hiện menu chọn Graceful hoặc Force Close)
   - Khi `PAUSED`: ▶️ `[RESUME]` | 🛑 `[STOP]`
   - Khi `GRACEFUL_STOPPING`: ↩️ `[CANCEL & RESUME]` | ⚡ `[FORCE CLOSE NOW]`

---

## 📋 4. Nhật Ký Sự Kiện Nhanh (`AssetLogPanel.tsx`)

- Khi chọn một dòng Asset trong bảng, Panel bên dưới sẽ hiển thị 5 dòng log mới nhất của riêng cặp tiền đó.
- Giúp người dùng biết bot đang làm gì (VD: *22:15:00 - [ETHUSDT] Trailing SL updated to 1929.54 (Mark Price)*).

---

## 📜 5. Ghi Chú Tính Năng Tương Lai (Post-Demo Features)

1. **Bộ Lọc Thanh Khoản Tự Động (Liquidity Screener):** Quét toàn bộ sàn Binance Futures, tự động đề xuất các cặp tiền có thanh khoản tốt nhất, biến động ATR phù hợp với chiến lược Helios.
2. **Mô-Đun Kê Khai & Tính Thuế (Tax & Financial Reporting):**
   - Tự động thống kê toàn bộ lịch sử khớp lệnh và tính toán Realized PnL ròng sau khi trừ phí giao dịch Binance.
   - Xuất báo cáo tài chính / thuế theo năm tài chính (CSV/Excel) phục vụ kê khai thuế đầu tư.
