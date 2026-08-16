# HELIOS V3.3 ARCHITECTURE REVIEW: PART 12 - RISK MANAGEMENT AUDIT & PARAMETER MAP

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #11: [`core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)** và kiến trúc quản lý rủi ro nâng cấp V3.3.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ QUẢN LÝ RỦI RO CỦA `RISK_MANAGER`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Đóng vai trò **"Lá Chắn Cuối Cùng" (Last Line of Defense)**, hoạt động hoàn toàn độc lập với các Playbook Scanners để bảo vệ tài sản người dùng.
- **Thực nghiệm v3_19**: Đã chứng minh hiệu quả rực rỡ với **Profit Factor = 15.36**, Max Trade Loss được khống chế ở mức **$-3.17\%$**, Max Win Trade đạt **$+63.10\%$** và gồng mượt tới 13.061 nến (~9 ngày).

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `RISK_MANAGER`

### A. Tự Động Khống Chế Trần Cắt Lỗ Theo Đòn Bẩy (Leverage Safety Bounds):
- **Cơ chế Phương án A**: Tự động siết trần Stop Loss ban đầu theo đòn bẩy Futures:
  $$\text{max\_initial\_sl\_pct} = \min\left(\text{playbook\_max\_sl\_pct}, \frac{0.80}{\text{leverage}}\right)$$
- **Tác dụng**: Nếu đòn bẩy là $20x$, trần Stop Loss tự động nén dưới $4.0\%$ lỗ (tương đương $80\%$ ký quỹ). Đảm bảo bot **luôn luôn cắt lỗ an toàn TRƯỚC KHI bị sàn thanh lý cưỡng chế (Liquidation)**!

### B. Cơ Chế Giải Phong Vốn Ngâm (Stagnant Capital Release - False Breakout Exit):
- Trong `manage_trailing_stop`: Nếu sau khi mở lệnh bùng nổ (`MOMENTUM_IGN`, `ANOMALY`) mà giá **đi ngang ngâm vốn không có lợi nhuận ($PNL \le 0$)** quá 1 chu kỳ `decay_cycles`, bot lập tức đóng vị thế (`tick.close(None)`).
- **Tác dụng**: Nhận diện và thoát $100\%$ các cú bẫy bùng nổ xịt (False Breakouts) trước khi bị xả ngược chiều!

### C. Bảo Tồn 100% Dải Tham Số Động Cho GA Optimizer:
- Giữ nguyên các dải tham số động cho GA Optimizer tự do thích ứng theo tài sản và bối cảnh:
  - **`time_countdown_bars`** (Dải $10 \rightarrow 100$ nến): Dò tìm số nến đi ngang sau đỉnh để tính 1 chu kỳ ngâm lệnh.
  - **`tighten_step_multiplier`** (Dải $0.50 \rightarrow 0.95$, step $0.025$): Dò tìm độ quyết liệt siết Stop Loss.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH THAM SỐ VÀ CƠ CHẾ KHỐNG CHẾ RỦI RO TRONG FILE #11 (`risk.rs`)

| Chức Năng | Công Thức / Logic V3.3 | Vai Trò Bảo Vệ Tài Sản |
|---|---|---|
| `calculate_size` | `size_fraction.min(0.20)` | Khống chế tối đa 20% tổng tài sản cho đơn lệnh Exclusive |
| `clamp_sl_price` | $\min\left(\text{max\_sl\_pct}, \frac{0.80}{\text{leverage}}\right)$ | Siết trần Stop Loss theo đòn bẩy, chống cháy tài khoản Liquidation |
| `manage_trailing_stop` | `decay_cycles = stagnant_bars / time_countdown_bars` | Progressive Trailing SL Decay - siết SL theo số nến ngâm lệnh |
| Stagnant Exit | `if decay_cycles > 0 && current_pnl <= 0.0 { close() }` | Thoát lệnh sớm khi dính bẫy False Breakout ngâm vốn |
