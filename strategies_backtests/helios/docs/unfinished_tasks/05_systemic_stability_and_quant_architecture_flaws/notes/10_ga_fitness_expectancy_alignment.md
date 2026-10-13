# Ghi Chú Nghiên Cứu Định Lượng: Tái Cấu Trúc Hàm Mục Tiêu Fitness GA — Đồng Trục Kỳ Vọng Toán Học (EV Alignment)

> [!WARNING]
> **TÀI LIỆU ĐỐI CHẤT HỌC THUẬT & NGHIÊN CỨU TOÁN HỌC (LAB RESEARCH NOTE - UNVERIFIED)**  
> Tài liệu này mổ xẻ sự thất bại của hàm Fitness cũ trong GA Optimizer (`Prospector`), phân tích 4 tử huyệt toán học dẫn đến sự sụp đổ hiệu suất OOS, và thiết lập công thức mục tiêu chuẩn Quant thể chế.

---

## 🔬 1. HIỆN TRẠNG TOÁN HỌC CỦA HÀM FITNESS CŨ

Trong `trading_bot/bot_engine/src/optimizer/prospector_runner.rs` (dòng 502–558), hàm `calculate_enhanced_fitness` đang được tính theo công thức:

$$\text{Fitness} = \underbrace{\left(\frac{\text{Return\%}}{\text{MaxDD\%} + 3.0}\right)}_{\text{Base Calmar}} \times \text{PF\_Mult} \times \text{SQN\_Mult} \times \text{Freq\_Weight} \times \text{WinRate\_Mult}$$

### Các hệ số thành phần:
1. `Base Calmar`: $\frac{ret}{dd + 3.0}$.
2. `Freq_Weight`: 
   - Nếu $trades < 8$: $\left(\frac{trades}{8}\right)^2$.
   - Nếu $8 \le trades \le 55$: $1.0$.
   - Nếu $trades > 55$: $\max\left(0.70,\ 1.0 - (trades - 55) \times 0.008\right)$.
3. `WinRate_Mult`: $1.0 + \tanh\left(\frac{wr - 50.0}{15.0}\right) \times 0.45$.

---

## 🚨 2. BỐN TỬ HUYỆT KHIẾN GA BỊ "LỆCH TÂM"

Đối chiếu trực tiếp với dữ liệu thực nghiệm ngày 11/10/2026 (In-Sample $+63.60\%$, Winrate $76.8\%$ $\longrightarrow$ OOS Forward âm $-4.09\%$, Winrate $31.8\%$):

### 2.1. Tử Huyệt 1: Bẫy Ép Win Rate Cao $\longrightarrow$ Tiến Hóa Ra Quái Thai "Thắng Nhỏ, Gồng Lỗ Đậm"
- **Bản chất lỗi:** Thuật toán Di truyền (GA) được thưởng thêm tới $+45\%$ điểm số nếu đạt Win Rate cao.
- **Hành vi ma mãnh (Gaming the Metric):** Cách dễ nhất để tối đa hóa Win Rate trên tập dữ liệu quá khứ là **NỚI RỘNG STOP LOSS VÀ CHỐT LỜI CỰC NON**. Giá biến động ngẫu nhiên sẽ ít khi quét trúng SL rộng và sớm muộn cũng hồi để chốt lời non $\longrightarrow$ Win Rate vọt lên $70\% - 80\%$.
- **Hệ quả thực tế:**
  - 7 lệnh thắng: Lãi trung bình chỉ $+5.2\%$ ROE ($+\$0.61$/lệnh).
  - Lệnh thua #0014: Cắn Hard SL **$-12.9\%$ ROE ($-\$1.52$)**!
  - Lệnh thua #0020: Cắn Hard SL **$-9.1\%$ ROE ($-\$1.06$)**!
  - Tỷ lệ Payoff Ratio bị nghịch đảo: $\frac{\text{Avg Win}}{\text{Avg Loss}} \approx 0.4 \dots 0.5$. Một lệnh thua xóa sạch 3 lệnh thắng.

---

### 2.2. Tử Huyệt 2: Trục Xoay Dựa Vào `Return%` Tích Lũy Thay Vì Kỳ Vọng Toán Học Ròng ($EV/\text{Trade}$)
- **Bản chất lỗi:** Tổng lợi nhuận tích lũy $\text{Return\%}$ có thể bị bóp méo bởi đòn bẩy $20\text{x}$ và lãi kép khi bắt trúng 1-2 con sóng bơm thổi may mắn trên In-Sample.
- **Sự thiếu vắng $EV$:** Hàm fitness không hề đo lường **Kỳ vọng toán học ròng trên từng giao dịch ($EV/\text{Trade}$)**. Một hệ thống cào 56 lệnh với đa phần là lệnh mỏng quanh hòa vốn (bị bào mòn phí) vẫn được điểm số cao ngất ngưởng nếu có vài lệnh ăn đậm đột biến. Khi ra ngoài OOS không còn sóng bơm thổi, hệ thống chỉ còn lại các lệnh cào lỗ phí.

---

### 2.3. Tử Huyệt 3: Phạt Tần Suất Giao Dịch (Frequency Penalty) Quá Yếu
- **Bản chất lỗi:** Mức phạt số lệnh nhiều chỉ giảm tuyến tính $0.8\%$ mỗi lệnh và bị kẹp sàn tối đa chỉ phạt $-30\%$ (`max(0.70)`).
- **Lách luật:** Một cá thể cào 80 lệnh có thể tăng gấp đôi $\text{Return\%}$ từ $35\%$ lên $70\%$ ($+100\%$). Mức tăng này dễ dàng bù đắp mức phạt nhỏ nhoi $30\%$, khiến GA tiếp tục thiên vị các bộ DNA cào lệnh dày đặc.

---

### 2.4. Tử Huyệt 4: Mù Chi Phí Ma Sát Sàn Thực Tế (Friction Drag Blindness)
- Trong môi trường phái sinh đòn bẩy $20\text{x}$:
  - Phí mở lệnh: $0.04\% \times 20 = 0.80\%$ ROE.
  - Phí đóng lệnh: $0.04\% \times 20 = 0.80\%$ ROE.
  - Trượt giá (Slippage 2 chiều): Tối thiểu $0.02\% \times 20 \times 2 = 0.80\%$ ROE.
  - $\longrightarrow$ **Chi phí ma sát tối thiểu mỗi vòng lệnh $\ge 2.40\%$ ROE!**
- Hàm fitness không phạt các lệnh có biên lợi nhuận mỏng dưới $2.4\%$ ROE. Trên sổ sách backtest chúng được tính là lệnh thắng, nhưng ngoài đời thực chúng là **lệnh thua ròng**.

---

## 📐 3. THIẾT KẾ ĐỀ XUẤT: HÀM FITNESS CHUẨN QUANT THỂ CHẾ

$$\text{Fitness}_{\text{Proposed}} = \mathbf{EV}_{\text{Net}} \times \text{Sortino}_{\text{Clipped}} \times \text{Turnover\_Penalty} \times \text{Payoff\_Ratio\_Mult}$$

1. **$\mathbf{EV}_{\text{Net}}$ (Trục Xoay Lợi Thế Toán Học):**
   $$\mathbf{EV}_{\text{Net}} = \frac{\sum_{i=1}^{N} \text{Net PnL}_i}{N} \quad (\text{Tính bằng ROE \% sau khi trừ 2.4\% phí + trượt giá})$$
2. **$\text{Sortino Ratio}$:** Đo lường biến động của riêng các lệnh lỗ (Downside Deviation), loại bỏ việc phạt các lệnh ăn đậm.
3. **$\text{Payoff Ratio Multiplier}$:** Thưởng khi $\frac{\text{Avg Win}}{\text{Avg Loss}} \ge 1.5$. Phạt nặng các bộ DNA có tỷ lệ Payoff $< 1.0$ (chống bẫy gồng lỗ ăn non).
4. **$\text{Exponential Turnover Penalty}$:** Phạt hàm mũ nếu số lệnh vượt quá $3$ lệnh/ngày.
