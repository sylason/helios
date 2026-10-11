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

---

### ❌ Lỗi #004: Thiếu Định Lượng Telemetry & Heartbeat khi Gatekeeper Veto Flatline/Chaos 65.4 Giờ
- **Hiện tượng:** ETHUSDT trong bài test Walk-Forward 5 ngày OOS chỉ thực hiện giao dịch trong 2 ngày, còn 65.4 giờ bị rơi vào trạng thái Volatility Flatline. Log trước đó chỉ in một dòng chuyển trạng thái ban đầu, thiếu định lượng cụ thể tại sao Flatline (giá trị NATR thực tế vs Fee Floor vs DNA Alpha) và thiếu nhịp đập Heartbeat khiến việc audit OOS thiếu dữ kiện.
- **Nguyên nhân gốc rễ:**
  1. `GateEvaluation` không lưu lại giá trị `required_min_natr_pct`, `fee_floor_pct`, `min_alpha_pct`, `chaos_threshold`.
  2. `DefenseLedger` chỉ ghi nhận edge-trigger (vào/ra bão), khi bão kéo dài nhiều ngày thì hoàn toàn im lặng (silence trap).
  3. Thiếu công cụ trực quan hóa (Visual Report) để soi ~7,200 nến OOS, vị trí vào/thoát lệnh, dải bảo vệ Flatline/Chaos và đường cong vốn.
- **Giải pháp đã thực hiện (Phương án C):**
  1. Nâng cấp `GateEvaluation` mang đầy đủ 4 trường định lượng của Gatekeeper (`gatekeeper.rs`).
  2. Bổ sung `check_hourly_heartbeat` (phát log mỗi 60 nến = 1 giờ) hiển thị `[MARKET HEARTBEAT]` kèm giá và thông số NATR/Chaos thực tế (`defense_ledger.rs`).
  3. Xây dựng module `visual_report.rs` xuất file `report_oos.html` tương tác (TradingView Lightweight Charts v4.2 canvas 60fps) cắt lát đúng ~7,200 nến OOS, hiển thị candlestick, markers trade, dải shading Flatline/Chaos, và đường cong vốn Equity Subplot, tự động sinh vào thư mục log trước khi dọn dẹp storage trong `walk_forward_smart_seed.rs`.

---

### ❌ Lỗi #005: Bẫy Break-Even Tự Sát & Khối Lệnh Bán Tháo Hoảng Loạn `Emergency BE Rejection Close` Trong `risk.rs` (TODO-SEM-03)
- **Hiện tượng:** Khi giá chỉ tăng nhẹ $+0.02\%$, cơ chế Break-Even đã kích hoạt, dời SL lên Entry; sau đó khi nến dao động nhẹ chạm dưới Entry, bot thực hiện `tick.close(None)` bán tháo Market, khiến vị thế phải trả $0.10\%$ phí sàn 2 chiều (âm $-2.0\%$ ROE tại đòn bẩy 20x).
- **Nguyên nhân gốc rễ:**
  1. *Kích hoạt sớm dưới sàn phí:* Điều kiện `peak_profit >= current_atr * be_trigger_mult` kích hoạt khi giá chỉ tăng $+0.02\%$ trên nến 1m, trong khi phí sàn 2 chiều tối thiểu là $0.10\%$ ($2.0\%$ ROE tại đòn bẩy 20x).
  2. *Panic Market Close:* Khối lệnh `if current_close <= min_be { tick.close(None); }` thực thi lệnh Market đóng vị thế khẩn cấp ngay giữa thanh nến, cướp quyền khớp lệnh Limit/Stop Market trên sàn và ép vị thế chịu trượt giá cùng phí Taker.
  3. *Lệch ngữ nghĩa (Semantic Mismatch):* So sánh khập khiễng giữa biến số khoảng cách giá tuyệt đối và ROE đòn bẩy, bỏ qua chi phí sàn thực tế.
  4. *Ký sinh TTP:* Ghép nối điều kiện `is_be_triggered || peak_roe >= ttp_act_roe` khiến Trailing Take Profit bị kích hoạt sớm khi lợi nhuận chưa đủ lớn, vô hiệu hóa khả năng gồng lãi sóng dài.
  5. *Xung đột với Monotonic Ratchet Guard (`TODO-RSK-02`):* Khi giá retest về Entry khiến `current_pnl <= 0.0`, Guard chặn không cho cập nhật SL hòa vốn lên sàn Binance vì nhầm lẫn với vị thế đang chịu lỗ ngay từ đầu.
- **Giải pháp đã thực hiện:**
  1. *Sàn kinh tế kép (Dual Economic Floor):* Bắt buộc $\text{peak\_roe} \ge \max(2.0 \times \text{round\_trip\_fee\_roe},\ 6.0\%)$ VÀ $\text{peak\_dist} \ge \max(\text{Entry} \times 2.0 \times \text{round\_trip\_fee},\ \text{ATR} \times \text{be\_trigger\_mult})$.
  2. *Tách biệt hoàn toàn TTP:* `is_ttp_active = peak_roe >= ttp_act_roe;` (loại bỏ cờ BE khỏi TTP).
  3. *Triệt tiêu Panic Close & Đệm phí an toàn:* Xóa bỏ hoàn toàn `tick.close(None)`; tính toán mức SL hòa vốn `min_be` có đệm bù phí sàn và vi mô lợi nhuận $+0.03\%$ giá (`micro_cushion`), bảo vệ vị thế qua cơ chế Algo Stop Market đám mây.
  4. *Khai thông Ratchet Guard:* Cho phép dời SL khi `current_pnl > 0.0 || is_be_triggered`, bảo toàn tuyệt đối chốt chặn vị thế âm ban đầu nhưng không cản trở việc khóa hòa vốn.
  5. *Kiểm thử 100% Zero-Regression:* 3 unit tests chuyên sâu bảo vệ logic sàn kinh tế, tính toán SL và tính độc lập của TTP.
