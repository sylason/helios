# Ghi Chú Nghiên Cứu Định Lượng: Chi Phí Ma Sát Binance Futures & Tái Cấu Trúc Hàm Fitness GA Tinh Gọn

> [!WARNING]
> **TÀI LIỆU ĐỐI CHẤT HỌC THUẬT & NGHIÊN CỨU TOÁN HỌC (LAB RESEARCH NOTE - UNVERIFIED)**  
> Tài liệu này ghi nhận phân tích định lượng về cơ chế tính phí Futures trên Binance, tác động khuếch đại của đòn bẩy đến chi phí ròng (ROE), và cuộc phản biện nhận thức luận nhằm tinh giản tối đa hàm mục tiêu Fitness của thuật toán di truyền (GA Optimizer).

---

## 🔬 1. BẢN CHẤT VẬT LÝ & CƠ CHẾ PHÍ BINANCE FUTURES

Đối chiếu với 2 tài liệu chính thức từ Binance Square (`post/9431332250177`) và Binance Academy:

### 1.1. Công thức Phí Chuẩn Sàn
$$\text{Fee (USDT)} = \text{Notional Value} \times \text{Fee Rate} = (\text{Quantity} \times \text{Price}) \times \text{Fee Rate}$$

### 1.2. Hiệu Ứng Khuếch Đại Đòn Bẩy Đến Vốn Ký Quỹ (ROE Drag)
Khi đo lường hiệu suất trên vốn ký quỹ ($\text{Margin} = \frac{\text{Notional Value}}{L}$), tỷ lệ phí thực tế bị nhân lên gấp $L$ lần:

$$\text{Fee}_{\text{ROE}} = \frac{\text{Fee (USDT)}}{\text{Margin}} = L \times \text{Fee Rate}$$

### 1.3. Bảng Ma Sát Thực Tế Theo Cấp Đòn Bẩy (VIP 0)
*Giả định: Taker Fee = $0.04\%$, Maker Fee = $0.02\%$, Trượt giá ước tính = $0.02\%$/lệnh Market.*

| Đòn bẩy ($L$) | Phí Taker 1 Chiều ($\text{ROE}$) | Phí Taker 2 Chiều (Round-Trip) | Trượt giá 2 Chiều ($\text{ROE}$) | **Tổng Ma Sát Tối Thiểu ($\text{Friction Drag}$)** |
| :---: | :---: | :---: | :---: | :---: |
| **$10\text{x}$** | $0.40\%$ | $0.80\%$ | $0.40\%$ | **$1.20\%$ ROE** |
| **$20\text{x}$** | $0.80\%$ | $1.60\%$ | $0.80\%$ | **$2.40\%$ ROE** |
| **$50\text{x}$** | $2.00\%$ | $4.00\%$ | $2.00\%$ | **$6.00\%$ ROE** |

> 💣 **Kết luận:** Tại đòn bẩy $20\text{x}$, một giao dịch mở bằng Market và đóng bằng Stop Market tự động bốc hơi **$\ge 2.40\%$ ROE** bất kể thị trường đi về đâu. Mọi lệnh thắng có biên độ $< 2.40\%$ ROE thực chất là **lệnh thua ròng**.

---

## 🚨 2. TẠI SAO HÀM FITNESS CŨ SỤP ĐỔ NGOÀI OOS?

Hàm Fitness cũ trong `prospector_runner.rs`:
$$\text{Fitness} = \left(\frac{\text{Return\%}}{\text{MaxDD\%} + 3.0}\right) \times \text{PF\_Mult} \times \text{SQN\_Mult} \times \text{Freq\_Weight} \times \text{WinRate\_Mult}$$

1. **Mù Ma Sát Đòn Bẩy:** Chỉ trừ phí $0.04\%$ trên giá thay vì trừ $2.40\%$ trên Margin $\longrightarrow$ GA tưởng các lệnh ăn $0.1\%$ giá là có lời, tiến hóa ra các bộ DNA chuyên "cào lệnh scalping".
2. **Bẫy Ép Win Rate:** Thưởng $+45\%$ điểm số cho Win Rate cao $\longrightarrow$ GA chọn nới rộng SL và chốt non để tối đa hóa Win Rate trong In-Sample, tạo ra tỷ lệ $\frac{\text{Avg Win}}{\text{Avg Loss}} < 0.4$.
3. Ra ngoài OOS: Không còn sóng bơm thổi bù lỗ, các lệnh thua $-10\% \to -13\%$ ROE nuốt trọn toàn bộ các lệnh thắng nhỏ, tài khoản sụp đổ.

---

## ⚖️ 3. ĐỐI CHẤT PHẢN BIỆN: OVER-ENGINEER VS. UNDER-ENGINEER

### 3.1. Có Phải Ta Đang Over-Engineer?
- **Đề xuất sơ bộ trước đó:** Nhân 4 số hạng rời rạc:
  $$\text{Fitness} = EV_{\text{Net}} \times \text{Sortino} \times \text{Turnover\_Penalty} \times \text{Payoff\_Mult}$$
- **Vạch trần Over-engineering:**
  - `Payoff_Mult` là **thừa thãi** vì bản thân công thức $EV = W \times \overline{\text{Win}} - (1-W) \times \overline{\text{Loss}}$ đã chứa đựng trọn vẹn tương quan Winrate và Payoff.
  - `Turnover_Penalty` là **xung đột tự mâu thuẫn**: Nếu mỗi lệnh đã bị trừ sòng phẳng phí sàn $2.4\%$ ROE mà vẫn tạo ra $EV_{\text{Net}} > 0$, việc phạt thêm số lệnh là hành vi phi lý trí (phạt một hệ thống sinh lời bền vững).
  - Nhân quá nhiều hệ số $(\times A \times B \times C \times D)$ sẽ tạo ra các điểm dị thường (mathematical singularities) để GA lách luật.

### 3.2. Có Phải Ta Đang Under-Engineer?
- Việc tính đúng chi phí ma sát $L \times \text{FeeRate}$ **không phải là over-engineering**, mà là sửa chữa một lỗi toán học sơ đẳng.
- Một hàm Fitness chỉ nhìn vào `Return%` thuần túy là **under-engineering** nghiêm trọng, vì không phân biệt được giữa lợi nhuận ngẫu nhiên (may mắn) và lợi thế thống kê bền vững.

---

## ⚡ 4. BA XUNG ĐỘT LOGIC NỘI TẠI CẦN HÓA GIẢI

1. **Xung đột Mẫu số nhỏ (Small Sample Singularity):**
   - Nếu chỉ tối đa hóa $EV_{\text{Net}}/\text{Trade}$, một cá thể chỉ đánh đúng 1 lệnh trong 30 ngày và ăn may $+15\%$ ROE sẽ có $EV = +15\%$, đánh bại cá thể đánh 30 lệnh chất lượng cao mỗi lệnh lãi đều $+3\%$ ROE.
   - *Hóa giải:* Bắt buộc phải nhân trọng số ý nghĩa thống kê theo định lý giới hạn trung tâm: $\sqrt{N}$ hoặc ngưỡng phạt mẫu tối thiểu $N < 15$.
2. **Xung đột Tự mâu thuẫn Phạt Kép (Double Penalization Conflict):**
   - Đã trừ phí sàn thực tế trong từng trade lại vừa phạt tần suất giao dịch.
   - *Hóa giải:* Xóa bỏ `Turnover_Penalty` dạng heuristic. Thay bằng điều kiện biên số lệnh hợp lý cho cửa sổ rolling.
3. **Xung đột Định kiến Giáo điều (Dogmatic Bias Conflict):**
   - Ép buộc $\text{Payoff} \ge 1.5$ sẽ bóp chết các bộ DNA thuộc trường phái Mean-Reversion / Range-Bound (vốn có Winrate $65\% - 70\%$ và Payoff $\approx 0.9 - 1.1$).
   - *Hóa giải:* Để $EV_{\text{Net}}$ tự do quyết định sự cân bằng giữa Winrate và Payoff.

---

## 🎯 5. HÀM MỤC TIÊU TINH GỌN TUYỆT ĐỐI (OCCAM'S RAZOR MASTER FORMULA)

Áp dụng tiêu chuẩn thống kê suy diễn tài chính (Financial Econometrics), hàm Fitness được tối giản về đúng **1 thước đo duy nhất: Chỉ số Thống kê Kiểm định T-Score Ròng ($t\text{-Stat}_{\text{Net}}$)** kết hợp phạt đuôi rủi ro giảm (Downside Deviation):

$$\mathbf{Fitness} = \frac{EV_{\text{Net}}}{\text{Downside Deviation}} \times \sqrt{N} \times \text{Sample\_Gate}(N)$$

Trong đó:
1. **$EV_{\text{Net}}$ (Kỳ vọng ròng trên từng lệnh):**
   $$EV_{\text{Net}} = \frac{1}{N}\sum_{i=1}^{N} \left( \text{Gross PnL}_i - \text{Friction}_{\text{ROE}} \right)$$
   Với $\text{Friction}_{\text{ROE}} = L \times (\text{Fee}_{\text{Open}} + \text{Fee}_{\text{Close}}) + \text{Slippage}_{\text{Est}}$.
2. **$\text{Downside Deviation}$ ($\sigma_{\text{down}}$):**
   $$\sigma_{\text{down}} = \sqrt{\frac{1}{N}\sum_{i=1}^{N} \min(0,\ \text{Net PnL}_i)^2}$$
   *(Chỉ đo rủi ro của các lệnh lỗ, không phạt các lệnh thắng lớn).*
3. **$\sqrt{N}$ (Hệ số ý nghĩa thống kê):**
   Đại diện cho định lý giới hạn trung tâm ($t$-statistic). Thưởng cho hệ thống duy trì được lợi thế ròng qua nhiều mẫu thử nghiệm độc lập.
4. **$\text{Sample\_Gate}(N)$ (Chốt chặn mẫu tối thiểu):**
   $$\text{Sample\_Gate}(N) = \begin{cases} \left(\frac{N}{N_{\min}}\right)^2 & \text{nếu } N < N_{\min} \ (N_{\min} = 12) \\ 1.0 & \text{nếu } N \ge N_{\min} \end{cases}$$
