# 🧭 ĐẶC TẢ KỸ THUẬT: BỘ LỌC VĨ MÔ CHỐNG ĐÁNH NGƯỢC CHIỀU (DIRECTIONAL MACRO VETO)

---

## 🎯 1. Mục Tiêu Cốt Lõi
- Triệt tiêu hoàn toàn thảm họa "bắt dao rơi" khi thị trường sập thác đổ (như chuỗi 12 lệnh Long của Anomaly trong đợt ETH rơi từ 2426 về 2371).
- Ngăn chặn tình trạng cản đầu xe tăng khi thị trường bơm dốc đứng (Super Uptrend).
- **Tính Dynamic 100%:** Đối xứng toán học 2 chiều, phi thứ nguyên, áp dụng bình đẳng cho mọi loại tài sản bất kể thị giá.

---

## 📐 2. Các Trục Đo Dữ Liệu Đầu Vào

Bộ lọc đọc trực tiếp từ `MarketMatrixV3` đã được de-overengineer:
1. **$\delta_{\text{KAMA}} = \text{matrix.kama\_slope\_atr}[i] \in [-1.0, 1.0]$:** 
   Độ dốc KAMA chuẩn hóa theo ATR và hệ số scale DNA.
2. **$\text{trend\_axis} = \text{matrix.trend\_axis}[i] \in [0.0, 1.0]$:** 
   Sức mạnh xu hướng ADX chuẩn hóa theo gen `adx_trend_threshold`.

---

## ⚖️ 3. Quy Tắc Veto Định Hướng Hai Chiều Đối Xứng

$$\text{is\_macro\_downtrend} = (\delta_{\text{KAMA}} < -\text{thresh}) \land (\text{trend\_axis} \ge 0.40)$$
$$\text{is\_macro\_uptrend} = (\delta_{\text{KAMA}} > +\text{thresh}) \land (\text{trend\_axis} \ge 0.40)$$

Trong đó $\text{thresh}$ được quy định bởi cấu hình hoặc gen DNA (mặc định $\ge 0.10$).

### 1. Chiều Thị Trường SẬP THÁC ĐỔ (`is_macro_downtrend == true`):
* 🚫 **CẤM TUYỆT ĐỐI MỞ LỆNH LONG** đối với các Playbook bắt đáy/ngược hướng (`REVERSAL`, `ANOMALY`) hoặc nhịp hồi Long của `TREND_PB`.
* ✅ **MỞ RỘNG CỬA CHO LỆNH SHORT:** Cho phép `TREND_PB Short` và `MOMENTUM Breakdown Short` tự do tham gia để ăn trọn con sóng sập lớn (như Trade #005).

### 2. Chiều Thị Trường BƠM DỰNG CỘT (`is_macro_uptrend == true`):
* 🚫 **CẤM TUYỆT ĐỐI MỞ LỆNH SHORT** đối với các Playbook bắt đỉnh/ngược hướng.
* ✅ **MỞ RỘNG CỬA CHO LỆNH LONG:** Cho phép `TREND_PB Long` và `MOMENTUM Breakout Long` đón đầu dòng tiền.

### 3. Vùng Thị Trường TRUNG HÒA / SIDEWAY:
* Khi $|\delta_{\text{KAMA}}| \le \text{thresh}$ hoặc $\text{trend\_axis} < 0.40$:
  * Không có xu hướng vĩ mô một chiều cực đoan.
  * Cả Long và Short đều được tự do đấu thầu tại tầng Arbiter.
