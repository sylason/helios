# MỔ XẺ CHUYÊN SÂU CLIMAX REVERSAL SCANNER (REVERSAL)

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `scanners/reversal.rs`, `core_v3/config.rs:582-630`, `core_v3/arbiter.rs:168-175`.  
> **Dữ liệu thực tế:** Run 1 (0 lệnh) & Run 2 (5 lệnh: 3 thắng, 2 thua, Net ROE: −2.9%, Avg Win: +7.9%, Avg Loss: −13.3%, R:R ≈ 0.6).

---

## 1. BẢN CHẤT VẬT LÝ THỊ TRƯỜNG VS HIỆN TRẠNG MÃ NGUỒN

| Tiêu chí | Bản chất vật lý thị trường | Code hiện tại (`reversal.rs`) | Đánh giá First-Principles |
| :--- | :--- | :--- | :--- |
| **Bảo vệ Xu hướng Vĩ mô** | Không bao giờ bắt dao rơi khi thác lũ xả hàng cực mạnh chưa dừng. | Tham số `_matrix` bị bỏ trống. **Không kiểm tra độ dốc KAMA (`kama_slope_atr`)**, không kiểm tra ADX. | ❌ **Tử huyệt bắt dao rơi:** Cản đầu xe tăng khi thị trường đang crash dốc đứng. |
| **Nhận diện Khối lượng Kiệt sức** | Cú Climax đảo chiều phải có nến bùng nổ thanh khoản cực đại (Selling Climax), tiếp theo là nến cạn cung. | Dùng `sat_proximity(vol_ratio, exhaust_drop)`. Thưởng điểm khi Volume **cạn kiệt (< 0.5× SMA)**. | ❌ **Mâu thuẫn danh xưng & bản chất:** Tên là Climax (cao trào) nhưng code lại bắt nến Low Volume. |
| **Cổng tỷ lệ R:R (Reward:Risk)** | Lệnh đảo chiều bắt đáy/đỉnh bắt buộc phải có tiềm năng R:R $\ge 1.5 - 2.0$ để bù đắp Win Rate thấp. | **Hoàn toàn không có cổng kiểm tra R:R.** TP chốt cứng tại `kama_mid`, SL = nến + buffer. | ❌ **R:R âm ngay từ thiết kế:** TP tiềm năng 0.5 ATR vs SL 1.0–1.5 ATR $\to$ R:R chỉ đạt ~0.5–0.6. |
| **Mục tiêu chốt lời (TP)** | Sóng hồi quy động lượng thường vượt qua hoặc tiệm cận vùng mất cân bằng thanh khoản (Fair Value). | TP đặt cố định tại `Some(kama_mid)`. | 🟡 KAMA Midline là đường trễ, khi giá hồi thì KAMA dốc xuống đón đầu $\to$ bóp nghẹt thêm biên lợi nhuận. |

---

## 2. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành phần | Ưu điểm | Khuyết điểm | Edge Cases nguy hiểm |
| :--- | :--- | :--- | :--- |
| **Bộ lọc Z-Score + Displacement** (`z_score.abs() > thresh` & `disp >= min_disp`) | Đo đạc trực quan mức độ kéo căng của giá ra khỏi trục cân bằng KAMA. | Không phân biệt được "giá kéo căng trong bão hỗn loạn" với "giá kéo căng trong xu hướng bậc thang mạnh mẽ". | Xu hướng siêu bền vững (Super Trend): Giá liên tục bám biên BB và duy trì Z-Score $\ge 2.5$ suốt 50 nến $\to$ Scanner liên tục báo tín hiệu bắt đỉnh/đáy sai. |
| **Bộ lọc CLV & MFI** (`clv >= min_clv && mfi <= mfi_thresh`) | Lọc bỏ nến tiếp diễn xu hướng, chỉ chọn nến có rút chân/đè đầu và dòng tiền hạ nhiệt. | MFI là chỉ báo trễ, trong cú sập parabol MFI có thể nằm ở vùng quá bán (<20) rất lâu trước khi giá tạo đáy thật. | Nến cờ đuôi nheo (Pennant) tiếp diễn: Nến hồi rút chân nhẹ nhưng sau đó thủng đáy tiếp. |
| **Hàm chấm điểm Volume** (`sat_proximity(vol_current / vol_sma, exhaust_drop)`) | Cố gắng bắt tín hiệu cạn kiệt thanh khoản của bên đẩy giá. | So sánh `vol_current` (nến 1m) với `vol_sma` (khung khác) gây lệch đơn vị scale. Thất bại khi gặp cú nảy thanh khoản ảo. | Khi thị trường đóng băng (Flatline/Chết thanh khoản), volume cạn khiến điểm số vọt lên cực đại. |
| **Thiết lập TP = `kama_mid`** | Neo mục tiêu vào vùng giá trị hợp lý (Mean-Reversion). | KAMA dốc trượt theo giá làm TP dịch lại gần Entry $\to$ Thắng chỉ được +5% đến +8% ROE, trong khi 1 lệnh thua dính Hard SL mất -13% đến -15% ROE. | Giá chưa kịp chạm KAMA đã quay đầu tạo đáy mới $\to$ TSL không kịp kích hoạt bảo vệ. |

---

## 3. NGUYÊN NHÂN TOÁN HỌC KHIẾN REVERSAL THUA LỖ TRONG RUN 2

Trong Run 2, Reversal đạt **Win Rate 60%** (3 thắng / 2 thua), nhưng **Tổng ROE vẫn âm (-2.9%)**:
$$\mathbb{E}[ROE] = (60\% \times (+7.9\%)) + (40\% \times (-13.3\%)) = 4.74\% - 5.32\% = -0.58\% / \text{trade}$$
Sau khi trừ thêm phí sàn 2 chiều ($\approx 2.6\%$ ROE tại 20x), kỳ vọng toán học rơi xuống mức âm sâu.

---

## 4. ĐỀ XUẤT CẢI TIẾN TOÀN DIỆN CHO REVERSAL

1. **Bổ sung Hard Gate R:R tối thiểu:**
   $$\text{Expected\_RR} = \frac{|\text{close} - \text{kama\_mid}|}{|\text{close} - \text{suggested\_sl}|} \ge 1.20$$
   Nếu $\text{Expected\_RR} < 1.20 \implies$ VETO ngay từ Scanner, không gửi Proposal sang Arbiter.
2. **Bổ sung Macro Trend Alignment (Chặn bắt dao rơi):**
   - Đọc `matrix.kama_slope_atr`.
   - Long: Cấm khi `kama_slope_atr < -max_adverse_slope` (ví dụ: cấm khi dốc âm quá -0.25).
   - Short: Cấm khi `kama_slope_atr > max_adverse_slope` (cấm khi dốc dương quá +0.25).
3. **Cải tổ logic Volume Climax:**
   - Hoặc đổi tên và logic sang đúng nghĩa **Climax Volume**: Đòi hỏi `vol_z_score >= 2.0` ở nến trước đó và nến hiện tại cạn volume.
   - Hoặc chuẩn hóa đúng volume 1m so với SMA 1m.
