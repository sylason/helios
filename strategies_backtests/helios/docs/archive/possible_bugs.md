# 🔬 HELIOS — BÁO CÁO CHẨN ĐOÁN LỖI LOGIC

> **Mục tiêu:** Tại sao Optimizer không tìm ra bộ parameters tốt?
> **Phương pháp:** Đọc toàn bộ code, đối chiếu output thực tế từ run diagnostic.
> **Kết quả test:** `0 trades` — Optimizer không bao giờ có cơ hội để đánh giá strategy.

---

## 🚨 LỖI CẤP ĐỘ 1: TỬ HUYỆT (Khóa toàn bộ mọi trades)

---

### 💀 BUG #1 — `chaos` dùng đơn vị SAI → Environment Fuse luôn = Chaotic

**Nơi xảy ra:** `matrix.rs` dòng 86 → `sensor.rs` dòng 28

**Vấn đề:**

- NATR (Normalized ATR) trả về giá trị **theo đơn vị %**, ví dụ: `1.17` = 1.17%
- `chaos_z_threshold` trong JSON optimizer range: `0.005` đến `0.065` (đơn vị **ratio** thập phân)
- Code gán trực tiếp: `chaos = sensors.natr.as_ref().clone()` → **chaos = 1.17**
- Điều kiện check: `chaos < chaos_z_threshold` → `1.17 < 0.054` → **LUÔN FALSE**

**Hệ quả:**
```
Fuse Inputs → Chaos: 1.1768  (Thresh: < 0.0544)  ← SAI đơn vị, không bao giờ pass
```
→ **100% bars bị kết luận là `Chaotic`** → `next()` return sớm → **0 trades mãi mãi**

**Cần làm:** `chaos = natr / 100.0` → range sẽ là `0.0 ~ 0.05` khớp với threshold

---

### 💀 BUG #2 — `entropy` (Efficiency Ratio) quá thấp → Fuse bị chặn

**Nơi xảy ra:** `sensor.rs` dòng 30 + `matrix.rs` dòng 92

**Vấn đề:**

- `er_threshold` của Rank 1 DNA = **0.3405**
- Entropy value tại bar cuối = **0.2723**
- Điều kiện: `entropy > er_threshold` → `0.27 > 0.34` → **FALSE**

**Nguyên nhân gốc:** Efficiency Ratio (ER) đo lường "độ thẳng" của giá. Trên thị trường crypto 1m, ER thường rất thấp vì giá có noise lớn. Threshold `0.34` quá cao cho timeframe này.

**Hệ quả:** Ngay cả khi bug #1 được sửa, fuse vẫn có thể bị block bởi entropy.

---

## ⚠️ LỖI CẤP ĐỘ 2: SUY YẾU OPTIMIZER (Làm tín hiệu méo mó)

---

### 🔴 BUG #3 — `sqz_raw` toàn bộ = 0.0 → Squeeze Score vô dụng

**Nơi xảy ra:** `matrix.rs` → `normalization::calculate_squeeze_intensity()`

**Output thực tế từ log:**
```
sqz_raw: last10: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
SqueezeIntensity: last10: [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]
```

**Nguyên nhân:** BB width > KC width nên `1 - ratio` âm → bị `.max(0.0)` → = 0.0. `rolling_percentile` nhận toàn bộ 0.0 → "mọi giá trị bằng nhau" → trả về **0.5 cứng**.

**Hệ quả:** Squeeze regime score luôn = `tanh(0.5 / threshold)` — một hằng số cố định, không phản ánh thực tế. **Optimizer không thể học được gì từ Squeeze.**

---

### 🔴 BUG #4 — `TrendDir` bão hòa hoàn toàn → không phân biệt được hướng

**Nơi xảy ra:** `matrix.rs` dòng 50

**Vấn đề:**
```
trend_dir = tanh_scale(linreg_slope, sensitivity=0.001)
```
- LinReg Slope thực tế: `-3.15` hoặc `+3.9` 
- Với divisor `0.001`: `tanh(-3150)` = **-1.0 cứng**; `tanh(3900)` = **1.0 cứng**
- → Mọi bar đều là `-1.0` hoặc `+1.0`, không bao giờ là trung gian.

**Output thực tế:**
```
TrendDir last10: [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]
```

**Hệ quả:** `detect_trend_direction()` dùng buffer `0.15` — nhưng giá trị lúc nào cũng là ±1.0 nên luôn là Bullish hoặc Bearish, không bao giờ Neutral. **Optimizer không thể tinh chỉnh tín hiệu xu hướng.**

---

### 🔴 BUG #5 — `momentum` (RSI → Percentile) bị đông đặc

**Nơi xảy ra:** `matrix.rs` dòng 80

**Output thực tế:**
```
Bar 4324:  RSI 42.8 → Mom_Norm: 0.7688
Bar 21621: RSI 63.2 → Mom_Norm: 0.7688  ← RSI KHÁC NHAU nhưng kết quả GIỐNG NHAU
Bar 43238: RSI 47.7 → Mom_Norm: 0.2237
```

**Vấn đề:** `rolling_percentile` với `momentum_lookback = 333 bars` trên dữ liệu MTF bị step-fill (nhiều nến liên tiếp cùng giá trị RSI). Khi nhiều giá trị bằng nhau → `count_equal` rất lớn → công thức `(count_smaller + 0.5 * count_equal) / total` bị kéo về 0.5.

**Hệ quả:** Momentum signal bị "flatten" — khó phân biệt oversold/overbought thực sự.

---

### 🟡 BUG #6 — `volume_force` tính sai khi dùng MTF vol_ma

**Nơi xảy ra:** `matrix.rs` dòng 95-104

**Vấn đề:**
```rust
let vol_raw = (volume_1m * vol_ratio) / vol_ma_timeframe_higher
```
- `vol_ratio` = 5.0 (khi vol_ma timeframe = 5m)
- Nhân volume 1m với 5 để "chuẩn hóa" sang 5m — **đây là xấp xỉ thô**
- Vol MA của timeframe 5m được tính từ tổng 5 nến 1m, không phải trung bình đơn giản

**Hệ quả:** `volume_force` bị lệch về phía dương → Filter thanh khoản trong Environment Fuse bị sai → quyết định vào lệnh bị nhiễu.

---

## ⚠️ LỖI CẤP ĐỘ 3: SAI LOGIC CHIẾN THUẬT

---

### 🟠 BUG #7 — `trend_angle_adj` trong Playbook dùng sai

**Nơi xảy ra:** `playbook.rs` dòng 68

**Vấn đề:**
```
if trend_angle.abs() > config.market_regime.trend.angle_parabolic { return Wait; }
```
- `trend_angle` ở đây là **trend_angle_adj** (0.0 đến 1.0 — percentile đã chuẩn hóa)
- Nhưng `angle_parabolic` trong config là **60.0 đến 85.0 degrees** (từ JSON)
- → So sánh `0.035 > 60.0` luôn FALSE → block không bao giờ kích hoạt

**Hệ quả:** Playbook Trending không bao giờ bị chặn bởi Parabolic Filter. Nhưng vì Environment Fuse đã block từ trước nên bug này chưa có tác động thực tế.

---

### 🟠 BUG #8 — `min_score_threshold` được áp dụng lên **weighted score**, không phải raw score

**Nơi xảy ra:** `playbook.rs` dòng 50

**Vấn đề:**
- `min_score_threshold` trong JSON range: `0.1 - 0.95`
- Nhưng `max_weighted_score` = `raw_score * priority_weight`, có thể lên tới `1.0 * 3.5 = 3.5`
- → Threshold `0.65` so với score `2.58` → không bao giờ là rào cản thực sự

**Hệ quả:** `min_score_threshold` gần như mất tác dụng lọc. Optimizer tối ưu hóa một biến **vô nghĩa**.

---

### 🟠 BUG #9 — `Anomaly Score` dùng Tanh của price_z_score đã là Tanh

**Nơi xảy ra:** `regime.rs` dòng 30-32

**Vấn đề:**
```
price_z_score = tanh(z_raw / sensitivity)  → range [-1, 1]
z_score_norm  = price_z_score.abs() * z_score_threshold  → range [0, threshold]
anomaly_score = tanh(z_score_norm * 0.8 + chaos_boost * 0.2)
```
- Double Tanh → signal bị nén hai lần → **cực kỳ khó bão hòa**
- `z_score_threshold` trong config hiện tại đóng vai trò **multiplier**, không phải threshold
- Nhưng tên gọi trong JSON là "threshold" → **Optimizer bị nhầm semantic**

**Hệ quả:** Optimizer không hiểu `z_score_threshold` đang làm gì → tối ưu hóa mù.

---

### 🟠 BUG #10 — `stoch_k` chuẩn hóa `/100` nhưng thresholds trong JSON vẫn là raw [0,1]

**Nơi xảy ra:** `tactical.rs` dòng 32 vs `playbook.rs` dòng 59

**Vấn đề:**
```rust
let stoch_k = sensors.stoch_k[i] / 100.0;  // [0, 1]
// Nhưng trong playbook:
if matrix_rsi > config.market_regime.phase.accel_min  // accel_min range: 0.1 - 0.9
```
Tactical dùng `stoch_k` [0,1] so sánh với `stoch_pullback_thresh` range [0.05, 0.45] — **Nhất quán**.
Nhưng trong `RangeOscillator`, dùng `stoch_oversold` range [0.05, 0.35] cho stoch đã chia 100 — **Nhất quán**.

→ Đây không phải bug, nhưng **Absorption bias** trong `regime.rs` dùng `matrix_rsi` (percentile [0,1]) so sánh với `bias_threshold` [0.55, 0.95] — Nhất quán. ✅

---

## 📊 TỔNG KẾT — MA TRẬN LỖI

| Cấp | Bug | File | Mức độ ảnh hưởng | Ảnh hưởng đến Optimizer |
|-----|-----|------|-----------------|------------------------|
| 💀 **TỬ HUYỆT** | #1 NATR đơn vị sai | matrix.rs | 100% trades bị chặn | Optimizer hoàn toàn mù |
| 💀 **TỬ HUYỆT** | #2 ER threshold quá cao | sensor.rs | Block thứ 2 sau #1 | Optimizer hoàn toàn mù |
| 🔴 **NGHIÊM TRỌNG** | #3 Squeeze = 0.5 cứng | matrix.rs | Squeeze signal vô dụng | Optimizer không học Squeeze |
| 🔴 **NGHIÊM TRỌNG** | #4 TrendDir bão hòa | matrix.rs | Không có gradient | Optimizer không tinh chỉnh trend |
| 🔴 **NGHIÊM TRỌNG** | #5 Momentum flatten | matrix.rs | RSI mất signal | Optimizer không học momentum |
| 🟡 **TRUNG BÌNH** | #6 Volume sai xấp xỉ | matrix.rs | Signal nhiễu | Optimizer học sai |
| 🟠 **LOGIC** | #7 Angle so sánh sai unit | playbook.rs | Parabolic filter vô dụng | Optimizer tối ưu biến vô nghĩa |
| 🟠 **LOGIC** | #8 min_score_threshold sai | playbook.rs | Threshold mất tác dụng | Optimizer tối ưu biến vô nghĩa |
| 🟠 **LOGIC** | #9 Double Tanh Anomaly | regime.rs | Signal bị nén quá mức | Optimizer học sai semantic |

---

## 🎯 KẾT LUẬN CỐT LÕI

**Tại sao Optimizer không tìm được kết quả?**

> Vì **Bug #1** khiến toàn bộ 43,243 bars bị kết luận là `Chaotic` → `next()` return ngay lập tức → **không một giao dịch nào được thực hiện** → Optimizer chỉ nhận về `0 trades` cho mọi bộ parameters → **fitness function không có tín hiệu để tối ưu**.

**Thứ tự ưu tiên sửa:**
1. 🚑 Bug #1 → Sửa ngay: `chaos = natr / 100.0`
2. 🚑 Bug #2 → Xem xét lại range của `er_threshold` hoặc dùng percentile cho ER
3. 🔧 Bug #3 → Cần thay thế `sqz_raw` logic — BB/KC overlap thay vì width ratio
4. 🔧 Bug #4 → Tăng `divisor` trong `tanh_scale` cho trend_dir (từ 0.001 → dùng std của slope)
5. 🔧 Bug #7, #8 → Sửa semantic mismatch trong config (unit của angle_adj vs degrees)
