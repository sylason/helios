# HELIOS V3.3 ARCHITECTURE REVIEW: PART 7 - RANGE OSCILLATOR SCANNER AUDIT & REDUNDANT PARAMETERS LIST

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #7: [`core_v3/scanners/range_osc.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs)** và danh sách tinh giảm các biến dư thừa.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `RANGE_OSC`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Săn sóng đi ngang đảo chiều hai biên trong khung thị trường Sideway (ADX Tanh Scale $< 0.35$).
- **Mục tiêu**: Mua (Long) tại dải biên dưới Bollinger Band (`bb_lower`), Bán (Short) tại dải biên trên Bollinger Band (`bb_upper`).
- **Target Take Profit**: Đặt chốt lời cố định tại dải biên đối diện (`suggested_tp = Some(bb_upper)` cho Long và `Some(bb_lower)` cho Short).

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `RANGE_OSC`

### A. Trang Bị Bộ Cảm Biến Stochastic Độc Lập Per-Playbook:
- Để tránh bẫy giằng co tham số với `TREND_PB`, `RangeOscScanner` sở hữu bộ cảm biến Stochastic hoàn toàn độc lập:
  - **`stoch_range_period`**: Chu kỳ Stochastic riêng cho Sideway (thường ngắn hơn để phản ứng nhanh).
  - **`stoch_range_k` & `stoch_range_d`**: Chu kỳ mịn K/D riêng.
  - **`stoch_range_tf`**: Khung thời gian riêng.

### B. Cơ Chế Kích Hoạt Lệnh Sideway Tối Ưu (Phương Án A):
- **Điều kiện Mua (Long)**: Giá gần biên dưới `bb_lower` AND `stoch_range_k > stoch_range_d` **KHI CẢ HAI NẰM Ở VÙNG OVERSOLD ($K < 25$)** AND `volume_force <= volume_max_threshold` AND `bb_width_pct >= min_width_pct`.
- **Điều kiện Bán (Short)**: Giá gần biên trên `bb_upper` AND `stoch_range_k < stoch_range_d` **KHI CẢ HAI NẰM Ở VÙNG OVERBOUGHT ($K > 75$)** AND `volume_force <= volume_max_threshold` AND `bb_width_pct >= min_width_pct`.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH BIẾN DƯ THỪA ĐÃ TINH GIẢM TRONG FILE #7 (`range_osc.rs` & `RangeParams`)

| Tên Biến Cũ | Lý Do Tinh Giảm / Loại Bỏ | Giải Pháp Thay Thế V3 |
|---|---|---|
| `bb_vs_stoch_weight` | Phép cộng trọng số gượng ép làm dài dải DNA | Xóa bỏ. Kiểm tra trực tiếp Stoch $K < 25$ tại biên BB |
| `stochastic.period` dùng chung | Gây giằng co tham số với `TREND_PB` | Thay bằng bộ cảm biến riêng **`stoch_range_period`** & **`stoch_range_tf`** |
| `range_bb_tolerance_pct` | Nhập nhằng với độ lệch biên | Đơn giản hóa mốc chạm biên Bollinger Bands |
