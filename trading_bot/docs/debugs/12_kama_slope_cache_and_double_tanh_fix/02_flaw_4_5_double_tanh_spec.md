# ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-4.5 (DOUBLE TANH DÌM ĐIỂM REVERSAL & ANOMALY)

> **Mã định danh:** `FLAW-4.5`  
> **Mức độ nghiêm trọng:** `CRITICAL` (Dìm điểm tự tin của 2 Tactical Scanners xuống dưới $0.32$, gây tê liệt hoàn toàn, không bao giờ vào lệnh Reversal và Anomaly)  
> **Tập tin liên quan:**  
> - `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs` (Dòng 20, 35)  
> - `strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs` (Dòng 20, 39)  
> - `strategies_backtests/helios/src/strategy/core_v3/matrix.rs` (Dòng 93)  

---

## 1. Bản Chất Toán Học Của Lỗi "Double Tanh"

### 1.1. Chuỗi Biến Đổi Tín Hiệu Bị Bóp Méo
1. **Giai đoạn 1 (Tính Z-Score thô):**
   Trong `ferro_trade::calculate::bulk::z_score`:
   $$Z_{\text{raw}} = \frac{X - \mu}{\sigma} \in (-\infty, +\infty)$$
   Đây là giá trị độ lệch chuẩn tuyến tính (ví dụ $Z = -3.0$ nghĩa là giá lệch 3 độ lệch chuẩn).

2. **Giai đoạn 2 (Tanh Lần 1 trong Matrix):**
   Trong `MarketMatrixV3::from_sensors` (`matrix.rs:93`):
   $$\text{price\_z\_score} = \tanh\left(\frac{Z_{\text{raw}}}{z_{\text{sens}}}\right)$$
   Với $z_{\text{sens}} = 2.0$, toàn bộ giá trị Z-Score bị nén phi tuyến tính vào dải:
   $$\text{price\_z\_score} \in (-1.0, 1.0)$$

3. **Giai đoạn 3 (Tanh Lần 2 trong Scanners — LỖI CHÍNH):**
   Trong cả `reversal.rs:35` và `anomaly.rs:39`:
   ```rust
   let z_score = matrix.price_z_score[i]; // <--- Đã bị nén về (-1.0, 1.0)
   let z_thresh = rev_cfg.z_score_threshold.max(0.1); // DNA: [0.5, 5.0] sigma
   let f_z = (z_score.abs() / z_thresh).tanh(); // <--- TANH LẦN 2!
   ```

### 1.2. Hậu Quả Toán Học: Vùng Bão Hòa Thấp Tê Liệt
Vì $| \text{price\_z\_score} | < 1.0$, nên với ngưỡng DNA trung bình $z_{\text{thresh}} = 2.5 \dots 3.0$:
$$\frac{| \text{price\_z\_score} |}{z_{\text{thresh}}} \le \frac{1.0}{3.0} \approx 0.3333$$
Khi đó, điểm thành phần $f_z$ sau hàm $\tanh$:
$$f_z = \tanh(0.3333) \approx 0.3215$$

**Bảng so sánh hành vi điểm số trước và sau khi sửa:**

| Độ lệch thị trường $Z_{\text{raw}}$ | Ngưỡng DNA $z_{\text{thresh}}$ | Điểm $f_z$ hiện tại (Bị Double Tanh) | Điểm $f_z$ kỳ vọng chuẩn ($Z_{\text{raw}}$) |
| :---: | :---: | :---: | :---: |
| $0.0\sigma$ (Giá bình thường) | $2.5\sigma$ | $0.000$ | **$0.000$** |
| $1.0\sigma$ (Lệch nhẹ) | $2.5\sigma$ | $0.182$ | **$0.380$** |
| $2.5\sigma$ (Chạm ngưỡng đảo chiều) | $2.5\sigma$ | $0.297$ | **$0.762$** |
| $5.0\sigma$ (Cực đại biến động) | $2.5\sigma$ | $0.375$ | **$0.964$** |
| $10.0\sigma$ (Thiên nga đen / Sập hầm) | $2.5\sigma$ | $\le 0.380$ (Chạm trần) | **$0.999$** |

### 1.3. Hệ Quả Tê Liệt Chiến Lược
- Trong `reversal.rs`: $\text{score} = 0.40 \times f_z + 0.40 \times f_{\text{disp}} + 0.20 \times f_{\text{vol}}$.  
  Vì $f_z \le 0.380$, thành phần $0.40 \times f_z \le 0.152$.  
  Dù các yếu tố khác hoàn hảo ($f_{\text{disp}} = 1.0, f_{\text{vol}} = 1.0$), điểm tối đa chỉ đạt $0.752$.  
  Nếu DNA tối ưu `min_activation_score > 0.75`, **scanner 100% không bao giờ bắn lệnh**!
- Trong `anomaly.rs`: $\text{score} = 0.60 \times f_{\text{wick}} + 0.40 \times f_z$.  
  Tương tự, thành phần $0.40 \times f_z$ bị triệt tiêu sức mạnh.

---

## 2. Giải Pháp Triển Khai

Trong cả `reversal.rs` và `anomaly.rs`:
- Chuyển sang đọc Z-Score tuyến tính trực tiếp từ `sensors.z_score[i]`:
  ```rust
  let z_score = sensors.z_score[i];
  ```
- Giữ nguyên công thức:
  ```rust
  let f_z = (z_score.abs() / z_thresh).tanh();
  ```
- **Lợi ích:**
  1. Khi $|Z_{\text{raw}}| = z_{\text{thresh}}$, $f_z = \tanh(1.0) \approx 0.762$ (đúng ngưỡng cảnh báo chuẩn Quant).
  2. Khi $|Z_{\text{raw}}| \gg z_{\text{thresh}}$, $f_z \to 1.0$ (phản ánh trung thực độ lệch giá cực đoan).
  3. Giữ nguyên hướng dấu của Z-Score cho việc phân loại Long/Short.
  4. Không ảnh hưởng đến `matrix.price_z_score` (vốn phục vụ các Multiplier vĩ mô trong Arbiter).
