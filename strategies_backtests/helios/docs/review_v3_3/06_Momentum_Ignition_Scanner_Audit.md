# HELIOS V3.3 ARCHITECTURE REVIEW: PART 6 - MOMENTUM IGNITION SCANNER AUDIT & REDUNDANT PARAMETERS LIST

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #6: [`core_v3/scanners/momentum_ignition.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs)** và danh sách tinh giảm các biến dư thừa.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `MOMENTUM_IGN`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Săn sóng bùng nổ động lượng khi thị trường bứt phá khỏi giai đoạn tích lũy nén dải (Squeeze Breakout).
- **Mục tiêu**: Bắt trọn nến bứt phá đầu tiên có sự bùng nổ của khối lượng (Volume Expansion) và giá bục khỏi dải Bollinger Band Upper/Lower khi dải BB đang bung mở rộng.

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `MOMENTUM_IGN`

### A. Trang Bị Bộ Cảm Biến Độc Lập Per-Playbook:
- Thay vì dùng chung chỉ báo `sensors.rsi` với các Playbook khác, `MomentumIgnitionScanner` sở hữu bộ cảm biến riêng:
  - **`rsi_momentum_period`**: Chu kỳ RSI chuyên biệt cho bùng nổ động lượng.
  - **`rsi_momentum_tf`**: Khung thời gian (Timeframe) chuyên biệt cho bùng nổ động lượng.

### B. Loại Bỏ Các Biến Dư Thừa & Gây Overfit:
1. **Xóa bỏ `rsi_climax` (Dư thừa gây Overfit)**:
   - Trong mã nguồn cũ: `rsi_pct >= mom_cfg.rsi_threshold && rsi_pct <= mom_cfg.rsi_climax`.
   - `MOMENTUM_IGN` sinh ra để săn sóng bùng nổ mạnh nhất. Việc kẹp trần `rsi_climax` (cấm RSI $> 80$) vô tinh cản trở thuật toán mở lệnh ở những cây nến tăng đẹp nhất và tạo bẫy Overfit trên dải DNA.
   - **Giải pháp**: Xóa bỏ hoàn toàn `rsi_climax`, chỉ cần `rsi_pct >= rsi_threshold`.
2. **Xóa bỏ `vol_max_multiplier` và `vol_raw = v / ma` rác**:
   - Thay thế bằng Volume Z-Score chuẩn hóa (`vol_z_score`). Chỉ dùng 1 biến ngưỡng duy nhất: `vol_z_threshold` ($1.5 \rightarrow 3.0$ StdDev).
3. **Xóa bỏ `vol_vs_break_weight`**:
   - Loại bỏ phép cộng trọng số dư thừa giữa Volume và Breakout distance.

### C. Mốc Bùng Nổ Động Lượng Tối Ưu (Phương Án A):
- **Bứt Phá Tăng (Bullish Breakout)**: Giá đóng cửa bục khỏi dải Bollinger Band Upper (`close > bb_upper`) KHI dải BB đang bung rộng (Volatile Expansion) AND `rsi_momentum >= rsi_threshold` AND `vol_z_score >= vol_z_threshold`.
- **Bứt Phá Giảm (Bearish Breakout)**: Giá đóng cửa bục khỏi dải Bollinger Band Lower (`close < bb_lower`) AND `(1.0 - rsi_momentum) >= rsi_threshold` AND `vol_z_score >= vol_z_threshold`.
- **Thoát Lệnh (Exit Strategy)**: KHÔNG TP cố định (`suggested_tp = None`), nhường cho Progressive Trailing SL tự động siết theo giá và gồng bão động lượng.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH BIẾN DƯ THỪA ĐÃ TINH GIẢM TRONG FILE #6 (`momentum_ignition.rs` & `MomentumParams`)

| Tên Biến Cũ | Lý Do Tinh Giảm / Loại Bỏ | Giải Pháp Thay Thế V3 |
|---|---|---|
| `rsi_climax` | Kẹp trần RSI vô lý, gây Overfit cho GA Optimizer | Xóa bỏ. Chỉ cần `rsi_momentum >= rsi_threshold` |
| `vol_max_multiplier` | Phụ thuộc vào `vol_ratio` và phép chia volume thô | Thay bằng `vol_z_threshold` ($1.5 \rightarrow 3.0$ StdDev) |
| `volume_spike_min` | Dư thừa khi đã có `vol_z_threshold` | Gộp vào `vol_z_threshold` |
| `vol_vs_break_weight` | Phép cộng trọng số gượng ép làm dài DNA | Xóa bỏ |
| `kc_u` / `kc_l` (Keltner Breakout) | Đường Keltner quá rộng, bứt phá chậm | Thay bằng mốc `close > bb_upper` khi dải BB bung mở |
| `rsi.period` / `rsi.timeframe` dùng chung | Gây nhiễu chồng chéo tín hiệu với các Playbook khác | Thay bằng bộ cảm biến riêng **`rsi_momentum_period`** & **`rsi_momentum_tf`** |
