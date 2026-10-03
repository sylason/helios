# KIỂM TOÁN KHÔNG GIAN SIÊU THAM SỐ & DNA: TREND PULLBACK (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **File mã nguồn đối chiếu:** `strategies_backtests/helios/src/strategy/core_v3/config.rs` (Struct `TacticalPullbackParams`, dòng 411–460)  
> **Ngày lập:** 03/10/2026

---

## 📊 1. BẢNG TỔNG HỢP 10 THAM SỐ CỦA `TacticalPullbackParams`

| STT | Tên Tham Số | Kiểu | Default | Dải DNA [Min, Max, Step] | Số Nấc | Vai Trò & Hiện Trạng Mã Nguồn |
| :---: | :--- | :---: | :---: | :---: | :---: | :--- |
| 1 | `max_initial_sl_pct` | `f64` | `0.15` (15%) | `[0.04, 0.15, 0.01]` | 12 | Trần ROE rủi ro tối đa cho phép. Tiêu thụ tại `arbiter.rs:210, 220` & `risk.rs:146`. |
| 2 | `rsi_max_cap` | `f64` | `68.0` | `[50.0, 80.0, 2.0]` | 16 | Trần RSI an toàn chống mua đỉnh sóng. Cận 50.0 quá thấp (chặn uptrend). |
| 3 | `stoch_pullback_thresh`| `f64` | `30.0` | `[10.0, 50.0, 2.0]` | 21 | Ngưỡng Stochastic %K xác nhận hạ nhiệt. Cận 10.0 quá khắt khe trong siêu trend. |
| 4 | `kama_slope_threshold` | `f64` | `0.05` | `[0.01, 0.40, 0.02]` | 20 | 🚨 Cận 0.01 quá phẳng lì, khiến GA chọn nhầm sideway làm xu hướng. |
| 5 | `max_pullback_dist_atr`| `f64` | `2.00` | `[0.50, 3.50, 0.20]` | 16 | 🚨 Cận 3.50 ATR là mua đu đỉnh, vi phạm bản chất nhịp hồi quy. |
| 6 | `invalidation_buffer_atr`| `f64`| `0.50` | `[0.10, 1.50, 0.10]` | 15 | 🚨 Cận 1.50 ATR quá sâu, biến scanner thành kẻ bắt dao rơi khi trend đã gãy. |
| 7 | `min_activation_score` | `f64` | `0.05` | *Non-DNA* | 1 | Sàn mềm kỹ thuật 0.05, nhường quyền chọn lọc cho Arbiter. |
| 8 | `struct_sl_buffer_atr` | `f64` | `0.50` | `[0.10, 3.00, 0.15]` | 20 | 🚨 Cận 3.00 ATR gây xung đột với trần ROE 15%, bị Arbiter VETO 100% lệnh! |
| 9 | `max_boundary_z` | `f64` | `0.50` | `[0.20, 0.80, 0.05]` | 13 | 🚨 Cận 0.20 quá hẹp; kết hợp với Z MTF dài hạn làm khóa chết tín hiệu trong trend. |
| 10 | `max_squeeze_intensity`| `f64` | `0.75` | `[0.50, 0.90, 0.05]` | 9 | Chốt chặn Chop Zone BB/KC co thắt. Hoạt động an toàn. |

---

## ⚠️ 2. HAI BẪY TỐI ƯU HÓA DI TRUYỀN (GA OVERFITTING TRAPS)

1. **Bẫy Tự Sát VETO 100% Lệnh (Starvation Trap):**
   - Khi GA chọn `struct_sl_buffer_atr = 3.00` kết hợp với KAMA Suction `low.min(kama_mid)`, khoảng cách SL lên tới $4.5$ ATR (tương đương lỗ ROE $-60\% \sim -90\%$).
   - Arbiter kiểm tra `max_initial_sl_pct` ($\le 15\%$ ROE) và lập tức phủ quyết 100% lệnh của Trend_PB.
2. **Bẫy Cào Lệnh Trong Sideway (Overtrading Trap):**
   - Khi GA chọn `kama_slope_threshold = 0.01` (độ dốc chỉ 0.002 ATR/bar) và `max_pullback_dist_atr = 3.50`, Trend_PB biến thành một cỗ máy cào lệnh scalping trong vùng không có sóng, bị chi phí ma sát nuốt trọn tài khoản.

---

## 🎯 3. BẢNG ĐỀ XUẤT ĐIỀU CHỈNH DẢI DNA CHUẨN QUANT

| Tên Tham Số | Dải Cũ (V3.3) | Dải Đề Xuất Mới | Bước Nhảy (Step) | Số Nấc | Cơ Sở Khoa Học Định Lượng |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `kama_slope_threshold` | `[0.01, 0.40]` | **`[0.04, 0.35]`** | `0.01` | 32 | Loại bỏ hoàn toàn vùng flatline vô hướng; đảm bảo trend thực chất. |
| `max_pullback_dist_atr`| `[0.50, 3.50]` | **`[0.60, 2.40]`** | `0.10` | 19 | Ngăn chặn việc mua đu đỉnh khi giá đã bay quá 2.5 ATR so với KAMA. |
| `invalidation_buffer_atr`| `[0.10, 1.50]` | **`[0.15, 0.80]`** | `0.05` | 14 | Cắt lỗ cấu trúc sớm khi nến xuyên qua KAMA $> 0.8$ ATR (trend đã gãy). |
| `struct_sl_buffer_atr` | `[0.10, 3.00]` | **`[0.20, 1.20]`** | `0.05` | 21 | Khống chế tổng khoảng cách SL luôn nằm gọn trong trần 15% ROE. |
| `max_boundary_z` | `[0.20, 0.80]` | **`[0.50, 2.00]`** | `0.10` | 16 | Nới rộng biên độ để thích ứng với Z-score MTF dài hạn trong siêu sóng. |
| `stoch_pullback_thresh`| `[10.0, 50.0]` | **`[20.0, 50.0]`** | `2.00` | 16 | Tránh bắt dao rơi ở vùng Stoch $< 20$. |
| `rsi_max_cap` | `[50.0, 80.0]` | **`[60.0, 80.0]`** | `2.00` | 11 | Không bóp nghẹt các nhịp uptrend lành mạnh (RSI $50 \sim 60$). |
| `max_squeeze_intensity`| `[0.50, 0.90]` | **`[0.50, 0.85]`** | `0.05` | 8 | Tinh chỉnh chuẩn ranh giới Chop Zone. |
