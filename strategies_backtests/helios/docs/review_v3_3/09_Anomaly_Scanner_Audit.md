# HELIOS V3.3 ARCHITECTURE REVIEW: PART 9 - ANOMALY SCANNER AUDIT & REDUNDANT PARAMETERS LIST

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #9: [`core_v3/scanners/anomaly.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs)** và danh sách tinh giảm các biến dư thừa.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `ANOMALY`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Săn sóng bão tố quét thanh khoản (Flash Sweeps / Pinbar Rejection) trong môi trường thị trường biến động bão tố (`Volatile Regime`).
- **Mục tiêu**: Bắt các cây nến rút chân từ chối giá mạnh mẽ (Pinbar) khi cá mập ép quét Stop Loss cừu nhỏ lẻ trước khi giật ngược giá cực mạnh.
- **Thoát Lệnh (Exit Strategy)**: KHÔNG TP cố định (`suggested_tp = None`), nhường 100% cho Progressive Trailing SL gồng Siêu Trend Bão Tố.

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `ANOMALY`

### A. Khắc Phục Bẫy Nến Đơn Lẻ (Single Candle Wick Trap):
- **Vấn đề cũ**: `anomaly.rs` tính tỷ lệ râu nến `upper_rejection` chỉ trên duy nhất nến $i$ đơn lẻ, dễ dính bẫy nến rút chân giả có biên độ cực nhỏ khi thị trường không có sóng bão.
- **Giải pháp nâng cấp**: Bắt buộc phải kết hợp với chỉ số biến động **`chaos_ratio` (NATR)** từ `matrix.rs`. Chỉ cho phép `ANOMALY` hoạt động khi `chaos_ratio` đạt ngưỡng biến động bão tố thực sự (`Volatile Regime`).

### B. Giải Quyết Nguy Cơ Chồng Chéo Với `REVERSAL`:
- **Mutually Exclusive Regime Router**: 
  - `REVERSAL` hoạt động trong **`RANGING REGIME`** (khi khối lượng cạn kiệt `volume_force` thấp).
  - `ANOMALY` hoạt động trong **`VOLATILE REGIME`** (khi biến động bão tố `chaos_ratio` cao).
  - Hai Playbook được phân chia lãnh thổ hoàn toàn độc lập, không còn chồng chéo tín hiệu.

### C. Trang Bị Bộ Cảm Biến RSI Độc Lập Per-Playbook:
- Thay vì dùng chung `sensors.rsi`, `AnomalyScanner` sở hữu bộ cảm biến riêng:
  - **`rsi_anomaly_period`**: Chu kỳ RSI riêng cho bão tố quét thanh khoản.
  - **`rsi_anomaly_tf`**: Khung thời gian riêng.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH BIẾN DƯ THỪA ĐÃ TINH GIẢM TRONG FILE #9 (`anomaly.rs` & `AnomalyParams`)

| Tên Biến Cũ | Lý Do Tinh Giảm / Loại Bỏ | Giải Pháp Thay Thế V3 |
|---|---|---|
| `pattern_vs_wick_weight` | Phép cộng trọng số gượng ép làm phình dải DNA | Xóa bỏ. Kiểm tra trực tiếp `upper_rejection >= min_rej` trong Volatile Regime |
| `z_vs_rsi_weight` | Trọng số nhân phân số phức tạp dư thừa | Xóa bỏ |
| `rsi.period` dùng chung | Gây giằng co tham số với các Playbook khác | Thay bằng bộ cảm biến riêng **`rsi_anomaly_period`** & **`rsi_anomaly_tf`** |
