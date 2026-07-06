# 🏓 Tại sao RangeOscillator Không Bao Giờ Kích Hoạt?

## Lộ trình của một Tín hiệu Range (Từ Sinh → Chết)

Để RangeOscillator LONG được kích hoạt, tín hiệu phải **sống sót qua 7 cửa ải liên tiếp**:

### Cửa ải 1: Environment Fuse (sensor.rs)
```
stability_index = raw_index * persistence_factor
```
- Trong sideway, volume thường yếu → `liq_score` thấp → `raw_index` thấp.
- `persistence_factor` cần tích lũy liên tục → Khó đạt > 0.5 trên 1m.
- **Xác suất sống sót: ~50%**

### Cửa ải 2: Regime Scoring - Ranging phải THẮNG (regime.rs)
```
scores.ranging = 1.0 - MAX(trend, squeeze, anomaly, absorption)
```
- Trong sideway thực tế, ADX vẫn dao động 15-25 → `trend_strength ≈ 0.2`
- Công thức trend: `confidence / (min_threshold * buffer)` → thường > 0.3
- Nếu trend = 0.3 → ranging = 0.7
- Nhưng `squeeze_intensity` cũng thường > 0.3 trong sideway (vì BB/KC thu hẹp)
- → ranging = 1.0 - max(0.3, 0.3) = 0.7. Có vẻ OK?
- **NHƯNG** `final_score = ranging * stability_index`. Nếu stability = 0.4 → final = 0.28.
- Ngưỡng `min_score_threshold` thường > 0.3 → **CHẶN**.
- **Xác suất sống sót: ~25%**

### Cửa ải 3: Playbook chọn RangeOscillator (playbook.rs)
- Nếu Ranging thắng → trả về `TacticalPlaybook::RangeOscillator`. OK, không có lọc thêm.
- **Xác suất sống sót: 100% (nếu qua được cửa 2)**

### Cửa ải 4: Biên độ đủ rộng (tactical.rs dòng 238)
```rust
if range_width / current_close < range_cfg.min_width_pct { return; }
```
- `min_width_pct` = 0.005 đến 0.05 (0.5% đến 5%)
- BB Width trong sideway crypto thường = 0.5-2%
- **Xác suất sống sót: ~60%**

### Cửa ải 5: Giá phải VƯỢT BB Lower (tactical.rs dòng 250)
```rust
if current_close < bb_lower ...
```
- Bollinger Band chứa ~95% giá. Close < Lower chỉ xảy ra ~2.5% thời gian.
- **Xác suất sống sót: ~2.5%** ← ĐÂY LÀ NÚT THẮT CỔ CHAI CHÍNH

### Cửa ải 6: Volume phải yếu (tactical.rs dòng 243)
```rust
let is_exhausted = volume_force < range_cfg.volume_max_threshold;
```
- Khi giá vượt ra ngoài BB (điều kiện 5), thường là do Volume MẠNH (Breakout).
- Nhưng ở đây lại đòi Volume YẾU (Exhaustion).
- **Hai điều kiện 5 và 6 MÂU THUẪN nhau.**
- **Xác suất sống sót: ~30%**

### Cửa ải 7: Stochastic oversold + MACD pivot (tactical.rs dòng 250)
```rust
stoch_k < range_cfg.stoch_oversold && is_rebounding_up
```
- **Xác suất sống sót: ~40%**

### 📊 Xác suất Tổng hợp
```
P = 0.50 × 0.25 × 1.0 × 0.60 × 0.025 × 0.30 × 0.40
P ≈ 0.000225 = 0.02%
```

**Trên 43200 nến (30 ngày × 1440 nến/ngày): Khoảng ~10 cơ hội Range trong 1 tháng.**
Nhưng do mâu thuẫn logic giữa Cửa 5 và Cửa 6, con số thực tế gần bằng **0**.

---

## 🔑 Giải pháp đề xuất cho Range Trading

Thay vì đòi giá vượt BB (tín hiệu cực đoan), hãy dùng logic **Mean Reversion bên trong BB**:

```
// THAY THẾ: Giá nằm trong 1/4 dưới của BB (gần Lower) thay vì vượt Lower
let bb_range = bb_upper - bb_lower;
let price_position = (current_close - bb_lower) / bb_range; // 0.0 = tại Lower, 1.0 = tại Upper

// LONG: Giá ở vùng 1/4 dưới (price_position < 0.25)
// SHORT: Giá ở vùng 1/4 trên (price_position > 0.75)
```

Cách này tăng xác suất trigger từ ~2.5% lên ~25%, tức gấp 10 lần.
