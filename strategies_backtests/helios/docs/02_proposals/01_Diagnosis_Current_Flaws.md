# 🩺 Chẩn đoán: 12 Lỗi Logic Khiến Helios Bị Tê Liệt

## Tổng quan Vấn đề
Helios hiện tại có kiến trúc 4 tầng (Sensor → Matrix → Regime → Tactical). Mỗi tầng đều có **bộ lọc riêng**. Kết quả: **xác suất một tín hiệu sống sót qua cả 4 tầng là cực thấp**.

Ví dụ thực tế:
- Tầng 1 (Environment Fuse): Chặn ~40% thời gian (stability_index < threshold)
- Tầng 2 (Regime Scoring): Chặn thêm ~30% (final_score < min_score_threshold)
- Tầng 3 (Playbook): Chặn thêm ~15% (các điều kiện phụ: trend_dir == Neutral, v.v.)
- Tầng 4 (Tactical Trigger): Chặn thêm ~10% (đòi hỏi 3-5 điều kiện đồng thời)
- **Kết quả: Chỉ ~5% thời gian thị trường đáp ứng đủ điều kiện để vào lệnh**

---

## Danh sách 12 Lỗi Theo Mức Nghiêm Trọng

### 🔴 CRITICAL (Phá hỏng chiến lược)

#### Bug #1: Environment Fuse quá Bảo thủ (sensor.rs)
```
stability_index = raw_index * persistence_factor
```
- `persistence_factor` đòi hỏi thị trường **ổn định liên tục N nến** (min_orderly_bars = 5-65).
- Mỗi khi có 1 cú dao động bất thường, counter bị trừ **3 đơn vị** (`saturating_sub(3)`).
- Trên thị trường crypto 1 phút, biến động bất thường xảy ra mỗi 5-10 phút.
- **Kết quả:** persistence_factor gần như luôn ở mức thấp (< 0.3), kéo final_score xuống dưới ngưỡng.

#### Bug #2: Ranging Score là "Phần thừa" (regime.rs dòng 86)
```rust
scores.ranging = (1.0 - scores.trend.max(scores.squeeze)
                      .max(scores.anomaly).max(scores.absorption))
                .max(0.01).min(1.0);
```
- Ranging **KHÔNG CÓ CÔNG THỨC RIÊNG**. Nó chỉ đơn giản bằng `1 - max(các score khác)`.
- Vấn đề: Trend Score gần như **luôn luôn > 0.3** (vì công thức dùng `tanh()` nên chỉ cần ADX > 10 là đã có điểm).
- Do đó: Ranging Score hiếm khi vượt 0.7, và luôn **THUA** Trend/Squeeze trong cuộc đua xếp hạng.
- **Ranging chỉ thắng khi TẤT CẢ các score khác đều < 0.01** → gần như không bao giờ xảy ra trên thực tế.

#### Bug #3: Parabolic Score dùng trend_angle_adj (regime.rs dòng 83)
```rust
scores.parabolic = (trend_angle.abs() / reg_cfg.parabolic.angle_threshold).tanh();
```
- `trend_angle` ở đây là `trend_angle_adj` — một Percentile [0, 1].
- `angle_threshold` là 50-85 (Degrees).
- Phép chia: `0.5 / 85 = 0.006` → `tanh(0.006) ≈ 0.006`
- **Parabolic Score LUÔN ≈ 0, không bao giờ được nhận diện.**

#### Bug #4: MomentumIgnition là Dead Code (playbook.rs)
- Không có nhánh `match` nào trong `select_playbook()` trả về `TacticalPlaybook::MomentumIgnition`.
- 50 dòng code trong `tactical.rs` (dòng 178-229) hoàn toàn vô dụng.

---

### 🟡 HIGH (Giảm hiệu suất nghiêm trọng)

#### Bug #5: Ranging Tactical đòi "giá phải vượt BB" (tactical.rs dòng 250)
```rust
if current_close < bb_lower && is_exhausted && stoch_k < range_cfg.stoch_oversold ...
```
- Trong thị trường sideway, giá **hầu như không bao giờ vượt ra ngoài** Bollinger Band (vì BB được thiết kế để chứa ~95% giá).
- Điều kiện `close < bb_lower` chỉ đúng ~2.5% thời gian, kết hợp với 3 điều kiện khác → xác suất trigger ≈ 0.1%.

#### Bug #6: Playbook nhân đôi bộ lọc (playbook.rs dòng 36 + tactical.rs)
```rust
// playbook.rs:
let final_score = max_raw_score * env.stability_index;
if final_score <= min_thresh { return TacticalPlaybook::Wait; }

// tactical.rs (mỗi playbook):
if is_expanding && is_volume_strong && current_close > kama_mid && macd_hist > -0.01 && !tick.position().is_open() {
```
- Tầng 3 (Playbook) đã lọc bằng `final_score * stability_index`.
- Tầng 4 (Tactical) lọc lại bằng 4-5 điều kiện AND cứng.
- **Hai tầng lọc chồng lên nhau nhưng KHÔNG LIÊN QUAN** → Xác suất tổng hợp = P1 × P2 → cực nhỏ.

#### Bug #7: SL/TP cứng nhắc kết hợp Trailing → Mâu thuẫn (tactical.rs + risk.rs)
- Tactical đặt cả `sl_price` VÀ `tp_price` cố định khi vào lệnh.
- Đồng thời, `manage_trailing_stop()` liên tục dời SL.
- Nếu Trailing dời SL về Breakeven nhưng TP cố định chưa chạm → Lệnh bị kẹt giữa SL breakeven và TP xa → Thường kết thúc ở Breakeven (lãi 0).

#### Bug #8: Size cố định 20% vốn (tactical.rs dòng 44-50)
- Đòn bẩy 20x: Vị thế gấp 4x tài khoản, chịu nổi giật 25%.
- Đòn bẩy 100x: Vị thế gấp 20x tài khoản, chịu nổi giật chỉ 5%.
- Với ATR ~2% trên crypto 1m, SL thường > 5% → **Bị thanh lý trước khi chạm SL**.

---

### 🟢 MEDIUM (Giảm hiệu quả Optimizer)

#### Bug #9: ~100 genes quá nhiều cho GA (config.rs)
- Với ~100 tham số, không gian tìm kiếm (Search Space) ước tính > 10^80 tổ hợp.
- GA với population 50, 100 thế hệ chỉ thăm dò ~5000 điểm → **bao phủ ≈ 0%**.
- Nhiều genes là **ký sinh** (không ảnh hưởng kết quả vì playbook không bao giờ kích hoạt).

#### Bug #10: Các lookback percentile quá nhạy cảm (matrix.rs)
- `rolling_percentile()` trả về 0.5 (trung tính) khi gặp giá trị lặp lại (Forward Fill).
- Trên khung H1 resample xuống 1m, 60 nến liên tiếp có cùng giá trị → Percentile = 0.5 suốt 60 phút.
- **Regime Scoring đọc vào giá trị 0.5 liên tục → không thể phân biệt trạng thái.**

#### Bug #11: `volume_force > 5.0` hardcoded (sensor.rs dòng 24)
- Ngưỡng Volume Spike cố định = 5.0, không phải gene.
- Trên một số asset biến động cao (Doge, Shib), volume thường xuyên spike > 5x → **Fuse liên tục chặn 100%**.

#### Bug #12: Absorption đòi BB hẹp + Volume mạnh cùng lúc (regime.rs dòng 78)
```rust
if bb_width_norm < reg_cfg.absorption.range_width_max {
    scores.absorption = (volume_force / ...)...
}
```
- Khi BB hẹp (nén), Volume thường THẤP (vì thị trường yên tĩnh).
- Khi Volume mạnh, BB thường ĐÃ NỞ RỘNG.
- **Hai điều kiện này mâu thuẫn nhau** → Absorption Score hiếm khi > 0.

