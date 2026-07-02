# 👁️ GIÁC QUAN & MA TRẬN NHẬN THỨC (PERCEPTION)

Tầng này trả lời câu hỏi: **"Thị trường đang thực sự ở trạng thái nào?"** mà không dùng đến cảm tính của con người.

---

## 1. PHÂN LOẠI 6 THẾ TRẬN (REGIMES)
Helios quan sát thị trường qua 6 lăng kính độc lập:

1.  **Squeeze (Nén lò xo):** Đo sự nén chặt của Bollinger Bands bên trong Keltner Channels. Báo hiệu sự bùng nổ sắp tới.
2.  **Trending (Xu hướng):** Sử dụng KAMA Cross và ADX. Phân biệt rõ Bullish (+) và Bearish (-).
3.  **Ranging (Đi ngang):** Trạng thái mặc định khi không có lực đẩy. Ưu tiên các chiến thuật Mean Reversion.
4.  **Absorption (Hấp thụ/Gom hàng):** Giá đi ngang nhưng Volume tăng vọt. Dấu vết của "Cá mập".
5.  **Anomaly (Bất thường):** Z-Score của giá lệch quá 2.5 độ lệch chuẩn. Tai nạn/Flash Crash.
6.  **Parabolic (Hưng phấn):** Góc tấn công (LinReg Slope) bay thẳng đứng. Vùng cực kỳ nguy hiểm.

---

## 2. VŨ KHÍ BÍ MẬT: ROLLING PERCENTILE 💎
Thay vì dùng các ngưỡng cứng (Fixed Thresholds) như RSI > 70, Helios dùng **Bách phân vị trượt**:
*   **Logic:** "Giá trị RSI này cao hơn bao nhiêu % trong 200 nến vừa qua?"
*   **Ưu điểm:** Tự thích nghi với "tần số" của từng đồng Coin. RSI 60 ở đồng Coin này có thể là "quá nóng", nhưng ở đồng khác lại là "vừa mới khởi đầu".

---

## 3. 🏛️ 3 TRỤ CỘT CỦA SỰ THẬT (THE 3 PILLARS OF TRUTH)
Để loại bỏ hoàn toàn FOMO và sự chủ quan, Helios V2 không chỉ dựa vào một loại tín hiệu. Hệ thống đánh giá thị trường qua 3 chiều độc lập:

1.  **Góc độ Động học (Kinematics - Angle/Slope) 📐:**
    *   *Câu hỏi:* "Giá đang chạy nhanh hay chậm?"
    *   *Chỉ báo:* LinReg Slope & Angle.
2.  **Góc độ Hiệu suất (Efficiency - Entropy/ER) 📉:**
    *   *Câu hỏi:* "Giá đang chạy thẳng (Trend thật) hay đang chạy dích dắc (Nhiễu)?"
    *   *Chỉ báo:* KAMA Efficiency Ratio (ER).
3.  **Góc độ Thống kê (Statistics - Z-Score) 📊:**
    *   *Câu hỏi:* "Giá đang ở quá xa vùng giá trị trung bình (Overextended) hay chưa?"
    *   *Chỉ báo:* Price Z-Score.

**=> Mệnh lệnh Tác chiến:** Playbook chỉ được kích hoạt khi có sự **Hội tụ (Convergence)** của ít nhất 2 trong 3 trụ cột này.

---

## 4. LỖ HỔNG "VÙNG MÙ" (THE CHOPPY GAP) 🪚
*   **Vấn đề:** Hiện tại Helios chưa phân biệt rõ giữa **Ranging Trật tự** (Orderly) và **Ranging Hỗn loạn** (Choppy).
*   **Hậu quả:** Playbook Ranging sẽ bị "cưa chân bàn" trong vùng hỗn loạn.
*   **Giải pháp Đề xuất:** 
    *   Sử dụng **Efficiency Ratio (ER)** làm bộ lọc Entropy.
    *   Nếu ER < 0.3 => Thị trường quá zigzag => Kích hoạt trạng thái **Wait** bắt buộc.

---

## 4. SƠ ĐỒ LUỒNG NHẬN THỨC

```text
DỮ LIỆU THÔ (Price, Vol)
      |
      v
[ SENSORS ] (ADX, RSI, ATR...)
      |
      v
[ MATRIX ] (Percentile, Z-Score, ER)
      |
      v
[ REGIME SCORES ] (Bảng điểm 6 Squad)
      |
      v
[ ARBITRATOR ] (Phân xử & Chọn Playbook)
```
