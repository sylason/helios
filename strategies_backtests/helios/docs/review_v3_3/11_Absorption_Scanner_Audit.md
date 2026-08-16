# HELIOS V3.3 ARCHITECTURE REVIEW: PART 11 - ABSORPTION SCANNER AUDIT & REDUNDANT PARAMETERS LIST

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #10: [`core_v3/scanners/absorption.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs)** và danh sách tinh giảm các biến dư thừa.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `ABSORPTION`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Săn sóng bùng nổ khi cá mập Mua Gom (Bullish Accumulation) hoặc Xả Hàng (Bearish Distribution) trong vùng giá nén dải hẹp (`squeeze_intensity` cao) trước khi hình thành xu hướng bứt phá mới.
- **Thoát Lệnh (Exit Strategy)**: KHÔNG TP cố định (`suggested_tp = None`), nhường 100% cho Progressive Trailing SL gồng Siêu Trend kéo dài.

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `ABSORPTION`

### A. Triệt Tiêu 100% Bẫy Overfit & Suy Thoái DNA Trên Realtime Market:
1. **Thay thế bẫy MACD trễ bằng `VWMA vs SMA`**:
   - Mã nguồn cũ đoán hướng ngẫu nhiên bằng `macd > macd_signal` (gây Overfit nặng trên dữ liệu lịch sử và ngắt tín hiệu sai $50/50$ trên realtime).
   - **Nâng cấp V3.3**: Dùng `VWMA > SMA` (Xác nhận volume lớn tập trung ở nến Xanh Mua $\implies$ LONG) và `VWMA < SMA` (Xác nhận volume lớn tập trung ở nến Đỏ Bán $\implies$ SHORT).
2. **Xóa bỏ `vol_ratio` rác, dùng Volume Z-Score (`vol_z_score`)**:
   - Loại bỏ hoàn toàn phép nhân x60, x240 volume nến 1m gượng ép. Chỉ kích hoạt khi `vol_z_score >= vol_z_threshold` ($1.5 \rightarrow 3.0$ StdDev).
3. **Khóa Lãnh Thổ Bằng Mutually Exclusive Regime Router**:
   - `ABSORPTION` chỉ hoạt động độc quyền trong **`VOLATILE / ANOMALY REGIME`**. Khi thị trường Sideway hoặc Trend dốc, công tắc Arbiter tự động KHÓA `ABSORPTION` để nhường cho `RANGE_OSC` và `TREND_PB`.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH BIẾN DƯ THỪA ĐÃ TINH GIẢM TRONG FILE #10 (`absorption.rs` & `AbsorptionParams`)

| Tên Biến Cũ | Lý Do Tinh Giảm / Loại Bỏ | Giải Pháp Thay Thế V3 |
|---|---|---|
| `macd_main` / `macd_signal` | Bẫy đoán hướng nén giá trễ, gây Overfit | Thay bằng **`VWMA vs SMA`** |
| `vol_vs_squeeze_weight` | Phép cộng trọng số gượng ép làm dài DNA | Xóa bỏ |
| `trend_filter_threshold` | Dư thừa khi Arbiter đã chuyển mạch bối cảnh | Xóa bỏ |
| `vol_raw` nhân `vol_ratio` | Tạo nhiễu volume nổ giả | Thay bằng **`vol_z_threshold`** |
