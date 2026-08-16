# HELIOS V3.3 ARCHITECTURE REVIEW: PART 19 - STEP 4 RISK MANAGER & TRAILING SL PROGRESSIVE DECAY DESIGN

Tài liệu này ghi nhận chi tiết thiết kế nâng cấp cho **Bước 4: Quản Trị Rủi Ro & Trailing Stop Progressive Decay (RiskManager)** trong Helios V3.3. Thiết kế này thiết lập cơ chế bảo vệ vốn an toàn tuyệt đối trên đòn bẩy $20x$, triệt tiêu $100\%$ hiện tượng **Lệnh Zombie Ngâm Vốn (Zombie Trades)** bất kể $PNL \le 0$ hay $PNL > 0$, đồng thời tối đa hóa Vòng quay Vốn (Capital Velocity) để nâng tần suất giao dịch lên $30 - 60$ lệnh/tháng.

---

## 🏛️ 1. MỤC TIÊU THIẾT KẾ CỦA BƯỚC 4

1. **Bảo vệ tài khoản tuyệt đối trước nguy cơ Thanh Lý (Liquidation)** trên mức đòn bẩy cố định $20x$.
2. **Triệt tiêu 100% hiện tượng "Lệnh Zombie Ngâm Vốn"**:
   - Dù lệnh có $PNL \le 0$ hay $PNL > 0$, nếu sau một khoảng thời gian ($N$ nến) mà **không tạo được đỉnh lợi nhuận mới (High Watermark Peak)**, Stop Loss sẽ tự động bị siết chặt dần theo hàm lũy thừa để đóng vị thế, giải phóng vốn cho các cơ hội mới.
3. **Tối đa hóa Vòng quay Vốn (Capital Velocity)**: Nâng tần suất giao dịch từ 32 lệnh/tháng lên mức lý tưởng $30 - 60$ lệnh/tháng phân bổ đều cho 6 Playbooks.

---

## 📐 2. CƠ CHẾ TOÁN HỌC & CÁC CÔNG THỨC LÕI

### 2.1. Kẹp Trần Stop Loss Khởi Tạo An Toàn (Leverage Safety Bounds)
Mọi lệnh khi mở vị thế bắt buộc phải tuân thủ mức kẹp trần:
$$\text{Max\_Allowed\_SL\_Pct} = \min\left(\text{playbook\_max\_sl\_pct}, \frac{0.80}{\text{leverage}}\right)$$
- Với $\text{leverage} = 20.0 \implies \text{Mức Cắt Lỗ Trần} = \frac{0.80}{20} = \mathbf{4.0\%}$.
- Đảm bảo mọi lệnh bị cắt lỗ an toàn **TRƯỚC KHI** sàn Binance kích hoạt thanh lý cưỡng bức ($80\% - 90\%$ Margin).

### 2.2. High-Watermark Peak Tracking (Truy Vết Đỉnh Lợi Nhuận)
- Bắt đầu theo dõi đỉnh cao nhất (`peak_val` và `peak_bar`) kể từ nến thứ 2 sau khi vào lệnh (loại bỏ nhiễu râu nến lúc khớp).
- Mọi phép tính khoảng cách Trailing SL đều lấy mốc từ `peak_val` này thay vì lấy từ giá đóng cửa hiện tại.

### 2.3. Progressive Time Decay & Siết Chặt Trailing SL Đa Chu Kỳ
- Đếm số nến đi ngang kể từ đỉnh: $\text{stagnant\_bars} = i - \text{peak\_bar}$.
- Số chu kỳ suy giảm: $\text{decay\_cycles} = \frac{\text{stagnant\_bars}}{\text{time\_countdown\_bars}}$.
- Sau mỗi chu kỳ decay ($\text{decay\_cycles} \ge 1$):
  $$\text{current\_multiplier} = \text{base\_multiplier} \times (\text{tighten\_step\_multiplier})^{\text{decay\_cycles}}$$
  *(Ví dụ với $\text{tighten\_step\_multiplier} = 0.85$: Hệ số khoảng cách SL sẽ rút ngắn từ $2.0 \times ATR \rightarrow 1.70 \rightarrow 1.44 \rightarrow 1.22$)*.

### 2.4. Khắc Phục Triệt Để Bẫy "Lệnh Zombie Lãi Nhẹ" (Stagnant Capital Release Mở Rộng)
- **Trường hợp 1 ($PNL \le 0.0$)**: Nếu $\text{decay\_cycles} \ge 1 \implies$ Đóng chủ động (`tick.close()`) ngay lập tức để cắt lỗ nhỏ.
- **Trường hợp 2 ($PNL > 0.0$ - Lãi nhẹ đi ngang)**:
  - Khoảng cách SL bị siết chặt liên tục $\implies$ Mức giá $\text{New\_SL}$ sẽ được đẩy vượt lên trên giá vào lệnh ($Entry$).
  - Khi giá tiếp tục đi ngang hoặc nhấp nháy nhẹ, nó sẽ lập tức chạm $\text{New\_SL}$ và thoát lệnh ở điểm **Hòa vốn hoặc Lãi nhẹ**, giải phóng ngay tiền ký quỹ cho hệ thống!
- **Trường hợp 3 (Ngâm quá giới hạn tối đa)**: Nếu $\text{decay\_cycles} \ge \text{max\_stagnant\_cycles}$ (ví dụ ngâm quá 4 chu kỳ $= 480$ nến $\approx 8$ tiếng) $\implies$ Đóng chủ động không điều kiện.

### 2.5. Bảo Vệ Siêu Xu Hướng (Mega Trend Protection)
- Nếu lệnh thuộc nhóm bám sóng (`TREND_PB`, `MOMENTUM_IGN`) và đang có lãi lớn ($PNL > \text{mega\_trend\_pnl\_trigger}$, ví dụ $> 5\%$), hệ thống chặn cận dưới của mức siết tại `tsl_atr_normal`, không bóp nghẹt lệnh để cho phép gồng trọn siêu sóng $15\% - 30\%$.

---

## 🔍 3. ƯU ĐIỂM, KHUYẾT ĐIỂM TIỀM ẨN & CƠ CHẾ PHÒNG VỆ CỦA BƯỚC 4

### 3.1. Ưu Điểm (Pros)
1. **Chống Cháy Tài Khoản 100%**: Bảo vệ tài khoản tuyệt đối với mức trần $4.0\%$ trên đòn bẩy $20x$.
2. **Triệt tiêu Zombie Trades**: Không còn lệnh nào ngâm $45$ tiếng làm đóng băng hệ thống.
3. **Tối ưu Vòng quay Vốn**: Dòng tiền ký quỹ liên tục được giải phóng để mở các cơ hội mới.

### 3.2. Khuyết Điểm Tiềm Ẩn (Cons)
- **Rủi ro Bị Quét Lệnh Sớm (Premature Shakeout)**:
  - Nếu một con sóng bùng nổ cần $3 - 4$ tiếng tích lũy đi ngang trước khi bay mạnh, việc siết TSL có thể khiến bot bị thoát lệnh hòa vốn ngay trước khi con sóng khủng diễn ra.

### 3.3. Cơ Chế Đối Phó (Counter-Measures)
- **Mega Trend Protection**: Bảo vệ mức thở cho các lệnh đã có lãi lớn.
- **Tự do hóa `time_countdown_bars`**: GA Optimizer sẽ tự động dò tìm thời gian đếm nến tối ưu ($60 - 240$ nến) phù hợp với chu kỳ tích lũy của từng tài sản.
