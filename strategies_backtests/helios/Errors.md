# 🚨 SỔ TAY GHI NHẬN LỖI THIẾT KẾ & LOGIC (ERRORS.MD) - HELIOS

---

## 📅 Lỗi Ngày 2026-09-17 / 2026-09-18

### ❌ Lỗi #001: Scanner `ANOMALY` Cố Chấp Bắt Dao Rơi Do Hoàn Toàn Mù Xu Hướng (Trend-Blindness)
- **Hiện tượng:** Trong đợt sập mạnh của ETH từ $2426 về $2371, `AnomalyScanner` spam 12 lệnh Long liên tiếp, gây ra 6 lệnh Hard SL (-16% ROE mỗi lệnh), hòa 4 lệnh BE và làm bốc hơi -$12.16.
- **Nguyên nhân gốc rễ:** 
  1. `AnomalyScanner` chỉ nhìn vào $Z\text{-score}$ giá lệch chuẩn và độ dài râu nến đơn lẻ ($f_{\text{wick}}$). Trong sóng sập mạnh, $Z\text{-score}$ luôn âm sâu, khiến điểm số luôn đạt điều kiện vào lệnh.
  2. Scanner này hoàn toàn không kiểm tra độ dốc KAMA hay xu hướng vĩ mô (Macro Trend), ngộ nhận nhịp rút chân vi mô 5 phút thành đáy đảo chiều.
- **Hệ quả:** Biến một con sóng Downtrend lớn thành "nghĩa địa" cho tài khoản vì liên tục cản đầu xe tăng.
- **Giải pháp khắc phục:** 
  - Hoặc tắt hẳn `ANOMALY` khỏi Arbiter và chuyển sự tập trung sang `REVERSAL` (vốn có đo lường Volume kiệt sức).
  - Hoặc bắt buộc bổ sung bộ lọc xu hướng vĩ mô: Nếu $\text{kama\_slope\_atr} < -\text{thresh}$, cấm tuyệt đối mở Long.

---

### ❌ Lỗi #002: Cooldown Tracker Bị "Mất Trí Nhớ Chuỗi Thua" (Losing Streak Amnesia)
- **Hiện tượng:** Cùng một Scanner sau khi bị dính Hard SL chỉ bị cách ly một khoảng thời gian cố định ngắn (~30 nến). Ngay khi hết 30 nến, Scanner đó lại lập tức nhảy vào mở lệnh cùng chiều và tiếp tục bị cắt lỗ.
- **Nguyên nhân gốc rễ:**
  1. `CooldownTracker` chỉ có thời gian khóa cố định tuyến tính (`locked_until_bar = i + re_entry_cooldown_bars`), không có bộ nhớ đếm số lần thua liên tiếp (`consecutive_losses`).
  2. Không có cơ chế phạt lũy tiến (Exponential Backoff) để kéo dài thời gian cách ly khi thị trường đang rơi vào pha xấu đối với chiến thuật đó.
- **Giải pháp khắc phục:** Bổ sung `consecutive_losses` và công thức phạt lũy tiến $2^{(\text{losses}-1)}$, tự động reset về 0 khi có lệnh thắng và có trần `max_cooldown_cap` chống khóa vô hạn.

---

### ❌ Lỗi #003: `ANOMALY` và `RANGE_OSC` Chưa Được Tắt Khỏi Arbiter
- **Hiện tượng:** Dù đã có định hướng chỉ tập trung vào 4 Playbook chủ lực, nhưng trong `core_v3/mod.rs`, cả 6 Scanner vẫn được gọi `arbiter.add_scanner(...)`.
- **Hệ quả:** `ANOMALY` với điều kiện vào lệnh dễ dãi đã chiếm tới 77.3% tổng số lệnh, làm sai lệch bức tranh kiểm thử thực tế của các Playbook khác.
- **Giải pháp khắc phục:** Điều chỉnh đăng ký scanner trong `core_v3/mod.rs` đồng bộ với định hướng kiến trúc.
