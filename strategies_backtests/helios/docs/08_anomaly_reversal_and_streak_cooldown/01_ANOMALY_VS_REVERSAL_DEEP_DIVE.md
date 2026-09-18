# 🔬 BÁO CÁO KỸ THUẬT: PHÂN TÍCH BẢN CHẤT ANOMALY VS REVERSAL & BẪY ĂN MAY TRADE #005

---

## 🏛️ 1. Bản Chất Toán Học & Sự Khác Biệt Giữa Reversal và Anomaly

Hai Playbook này cùng mang tư tưởng giao dịch ngược hướng (Counter-Trend / Mean Reversion) nhưng có **động cơ vật lý và điều kiện kích hoạt hoàn toàn khác biệt**:

| Tiêu chí | `Reversal` (`scanners/reversal.rs`) | `Anomaly` (`scanners/anomaly.rs`) |
| :--- | :--- | :--- |
| **Mục tiêu chiến thuật** | Bắt đỉnh/đáy khi **ĐÀ XUNG LỰC ĐÃ KIỆT SỨC** (Exhaustion Climax). | Bắt các cây nến **RÚT RÂU ĐỘT BIẾN TẠI VÙNG GIÁ CỰC ĐOAN** (Pinbar Rejection Fade). |
| **Tín hiệu đầu vào** | 1. $Z\text{-score}$ giá lệch chuẩn ($f_z$).<br>2. Độ lệch xa đường trung vị KAMA ($f_{\text{disp}}$).<br>3. **Khối lượng phải kiệt quệ** ($f_{\text{vol}} = 1.0 - \tanh(\text{vol\_force} / \text{drop})$). | 1. $Z\text{-score}$ giá lệch chuẩn ($f_z$).<br>2. **Độ dài râu nến đơn lẻ** ($f_{\text{wick}} = \text{wick} / (\text{thresh} \times \text{ATR})$). |
| **Vai trò của Volume** | **BẮT BUỘC KHỐI LƯỢNG PHẢI CẠN KIỆT:** Nếu Volume đang bùng nổ, $f_{\text{vol}} \to 0 \implies$ Veto không vào lệnh. | **HOÀN TOÀN MÙ KHỐI LƯỢNG:** Không đo lường thanh khoản hay dòng tiền. |
| **Vai trò của Trend/KAMA** | Đặt mục tiêu Take Profit neo tại `kama_mid` (hồi quy về đường trung vị). | **HOÀN TOÀN MÙ KAMA & XU HƯỚNG:** Không biết KAMA đang dốc đứng hay đi ngang. |
| **Điểm yếu chí mạng** | Dễ bỏ lỡ các cú đảo chiều mà Volume vẫn còn dư âm lớn. | **BẪY RÂU NẾN GIẢ (FALSE PINBAR):** Trong sóng sập mạnh, nến 1m rút chân nhẹ 5-10 phút bị ngộ nhận thành đáy, nhảy vào Long bắt dao rơi liên tục. |

---

## 🎯 2. Tại Sao Trade #005 Là ANOMALY Vào Lệnh Mà Không Phải REVERSAL?

- **Bối cảnh nến (2026-09-14 20:51 UTC):**
  - ETH sau nhịp tăng dốc đứng chạm đỉnh $2579.01$.
  - Cây nến tại đỉnh này là một cây nến có **râu trên rất dài** (Upper Wick do phe bán đập xuống) và $Z\text{-score} > +2.0$.
  - Đồng thời, nến này đi kèm khối lượng giao dịch rất lớn (Volume bùng nổ của phe bò đu đỉnh và phe gấu dội lệnh).
- **Phản ứng của 2 Scanner:**
  1. **`ReversalScanner`:** Nhìn thấy `vol_force` cao chót vót $\implies$ $f_{\text{vol}} = 1.0 - \tanh(\text{vol\_force} / \text{drop}) \approx 0.05 \implies$ Tổng điểm `score < min_activation_score` $\implies$ **TỪ CHỐI RA LỆNH**.
  2. **`AnomalyScanner`:** Không quan tâm volume, chỉ thấy $Z > 0$ và râu trên dài vượt ngưỡng `wick_atr_threshold` $\implies$ $f_{\text{wick\_short}}$ vọt lên $> 0.80 \implies$ **BẮN TÍN HIỆU SHORT NGAY LẬP TỨC!**
- **Hệ quả thực tế:**
  - `Anomaly` đã "ăn may" bắt trúng đỉnh chóp của ETH, ăn trọn cú sập 153 giá (từ $2579$ về $2426$), gồng 1.114 cây nến và mang lại lợi nhuận kỷ lục **+$15.19 (+116.9% ROE)**.

---

## 🩸 3. Tại Sao Các Trade Sau Đó (`#006 ➔ #017`) Lại Là Thảm Họa Do `ANOMALY`?

Sau khi đóng lệnh Short #005 ở đáy $2426.21$, thị trường bước vào một con sóng giảm tiếp diễn (Cascading Sell-off) từ $2426$ sập thẳng xuống $2371$:

1. **Bẫy Z-Score liên tục âm:** Vì giá rơi tự do, $Z\text{-score}$ luôn duy trì ở mức âm sâu ($-2.0$ đến $-4.0$), khiến thành phần $f_z$ trong Anomaly luôn đạt điểm tối đa ($f_z \approx 1.0$).
2. **Bẫy râu nến vi mô (Micro Bounce Trap):** Trên khung thời gian 1 phút, trong một con sóng sập 50 giá, luôn có những cây nến rút râu dưới nhẹ 1-2 giá do bot chốt lời ngắn hạn. `Anomaly` thấy râu dưới là tưởng nhầm "thị trường tạo đáy" $\implies$ bắn tín hiệu LONG!
3. **Cắt lỗ liên hoàn:** Phe gấu chỉ nghỉ ngơi 5-10 phút rồi xả tiếp thủng đáy $\implies$ Lệnh Long bị cán Hard SL $-16\%$ ROE.
4. **Vòng lặp tự sát 12 lần liên tiếp:**
   - Cứ mỗi lần cắt lỗ xong, 10-20 phút sau xuất hiện 1 cây nến rút râu dưới mới $\implies$ `Anomaly` lại tiếp tục mở Long!
   - Thua 6 lệnh Cắt Lỗ Cứng (Trades #006, #007, #008, #012, #013, #014), hòa 4 lệnh BE, chỉ ăn non được 2 lệnh (+1.20$, +1.00$).
   - **Tổng thiệt hại chuỗi này: -$12.16$**, xóa sạch 80% lợi nhuận của Trade #005!

---

## 💡 4. Kết Luận Khách Quan
- `ANOMALY` là con dao hai lưỡi cực kỳ nguy hiểm: Nó có thể tình cờ ăn may một cú rơi lớn khi bắt được Pinbar đỉnh, nhưng trong xu hướng một chiều (Super Trend), nó sẽ liên tục "cào tài khoản" và tự sát vì cố chấp bắt dao rơi.
- Nếu không có cơ chế chặn xu hướng vĩ mô (Macro Trend Veto) hoặc kiểm soát chuỗi thua (Losing Streak Memory), `ANOMALY` sẽ biến mọi thành quả của bot thành tro bụi.
