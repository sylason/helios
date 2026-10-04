# MỔ XẺ CHUYÊN SÂU TREND PULLBACK SCANNER (TREND_PB)

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `scanners/trend_pb.rs`, `core_v3/config.rs:420-478`, `core_v3/matrix.rs:86-90`.  
> **Dữ liệu thực tế:** Run 1 (+72.8% ROE, n=12, 7 thắng) & Run 2 (+21.6% ROE, n=14, 6 thắng). Scanner duy nhất có Net ROE dương ở cả 2 lần chạy.

---

## 1. BẢN CHẤT VẬT LÝ THỊ TRƯỜNG VS HIỆN TRẠNG MÃ NGUỒN

| Tiêu chí | Bản chất vật lý thị trường | Code hiện tại (`trend_pb.rs`) | Đánh giá First-Principles |
| :--- | :--- | :--- | :--- |
| **Bảo vệ Vùng Biên (Chống Đu Đỉnh)** | Không mua khi giá đã dãn nở quá xa vùng cân bằng hoặc ở đỉnh phân phối. | Có `max_boundary_z`: Long cấm khi $Z > \text{max\_z}$, Short cấm khi $Z < -\text{max\_z}$. | ✅ **Điểm sáng lớn nhất:** Đây là chốt chặn then chốt giúp Trend_PB miễn nhiễm với FOMO đu đỉnh/bán đáy. |
| **Nhận diện Xu hướng** | Xu hướng bền vững cần độ dốc định hướng sạch (Directionality) và sức mạnh động lượng (Momentum). | Chỉ dựa vào 1 chỉ báo duy nhất: `kama_slope_atr`. Bỏ trống `trend_strength` (ADX) và `efficiency_ratio`. | 🟡 **Bão hòa ±1.00:** Khi trend dốc, `kama_slope_atr` bị kẹp chạm trần liên tục, mất độ nhạy phân biệt xu hướng. |
| **Đo lường Nhịp Hồi (Retracement)** | Nhịp hồi lành mạnh phải có động lượng hồi yếu (volume giảm, biên độ nến hẹp, tốc độ chậm). | Chỉ đo snapshot 1 nến qua `delta_kama = (close - kama_mid)/atr`. Không đo volume hoặc tốc độ nhịp hồi. | 🟡 **Mù quỹ đạo hồi:** Không phân biệt được nến Flash Crash xả hàng mạnh với nhịp điều chỉnh nhẹ nhàng. |
| **Xác nhận Lực Nến Đảo chiều** | Cần lực cầu từ chối giá thấp (Rejection) đẩy giá đóng cửa về nửa trên nến. | `clv >= 0.15 && close > open` (với Long). | 🟡 **Bẫy nến rút chân thân đỏ:** Nến Hammer/Pinbar rút chân cực mạnh ($CLV \ge 0.8$) nhưng đóng đỏ nhẹ $0.01\%$ bị loại bỏ oan. |
| **Hình học Stop Loss** | Neo theo đáy swing thực tế của nhịp hồi + đệm ATR. | Dùng `low.min(kama_mid) - buffer` (với Long). | ❌ **Bẫy KAMA Suction Trap:** Khi giá ở trên KAMA, SL bị kéo tụt xuống KAMA $\to$ SL xa $20\%-35\%$ ROE $\to$ Bị Sniper Guard VETO hàng loạt. |

---

## 2. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành phần | Ưu điểm | Khuyết điểm | Edge Cases nguy hiểm |
| :--- | :--- | :--- | :--- |
| **Boundary Guard (`max_boundary_z`)** | Ngăn chặn triệt để việc vào lệnh khi giá đã bay xa khỏi vùng tích lũy. | Có thể bỏ lỡ các con sóng parabol siêu mạnh (nhưng đây là nhiệm vụ của Momentum chứ không phải Pullback). | GA có thể tối ưu `max_boundary_z` quá hẹp (ví dụ 0.20) khiến scanner bị tê liệt. |
| **Hộp Không gian KAMA (`delta_kama`)** | Khống chế biên độ hồi quy quanh KAMA trong khoảng `[-inval_floor, max_dist]`. | Không đo được động lượng của nhịp hồi (Trajectory-blind). | Nến tin tức xả sập đột ngột dừng đúng tại KAMA: Scanner tưởng là Pullback đẹp và nhảy vào mua bắt dao rơi. |
| **Điều kiện `close > open`** | Lọc bỏ nến đỏ giảm điểm. | Loại bỏ các nến rút chân từ chối giá kinh điển (Hammer thân đỏ nhẹ). | Thị trường nảy giá rất mạnh từ hỗ trợ nhưng nến đóng đỏ do nến mở gap cao trước đó. |
| **Hình học SL `low.min(kama_mid) - buffer`** | Cố gắng tìm vùng hỗ trợ sâu nhất để tránh bị quét râu nến. | **Kéo giãn khoảng cách SL cực đại**: Kéo SL xa hơn 2–3 lần so với đáy nến thực tế $\to$ Xung đột với trần rủi ro ROE. | Trong Run 2 (Bar 1621–2330), hàng chục lệnh có điểm số raw = 0.96 bị VETO oan vì SL 10.4%–14% vượt quá trần 8% ROE. |

---

## 3. PHÂN TÍCH CHUỖI 6 LỆNH THUA NHANH CỦA RUN 2 (#17, 26, 27, 32, 34, 36)

Cả 6 lệnh thua này đều có đặc điểm chung:
- Đều dính `HARD_SL` rất nhanh sau 7–13 nến.
- MFE (Maximum Favorable Excursion) cực thấp ($\le 3.0\%$).
- **Bản chất vật lý:** Sau khi vào lệnh, giá không hề tiếp diễn xu hướng mà quay đầu đâm thủng ngay SL. Điều này xảy ra do 2 nguyên nhân:
  1. **Độ trễ của KAMA:** Khi thị trường đã chuyển chế độ từ Trend sang Reversal/Range, KAMA vẫn còn giữ độ dốc cũ trong vài nến tiếp theo, khiến `Trend_PB` vào lệnh ngược chiều sóng mới.
  2. **SL bị Kẹp cứng (Clamped SL):** Khi SL hình học đề xuất quá rộng bị Sniper Guard kẹp cứng về trần 8%–15% ROE, mức SL mới trở nên quá nông và nằm lơ lửng ngay trong vùng nhiễu 1m, dễ dàng bị quét sạch.

---

## 4. ĐỀ XUẤT CẢI TIẾN CỐT LÕI CHO TREND_PB

1. **Khử bỏ hoàn toàn KAMA Suction Trap:**
   - Thay `base_support = low.min(kama_mid)` bằng:
     ```rust
     // Neo SL thuần túy theo đáy nhịp hồi + đệm ATR
     let sl = low - buffer;
     ```
   - Điều này đưa khoảng cách SL về đúng tỷ lệ chuẩn của nhịp hồi (thường chỉ 5% - 8% ROE), triệt tiêu hoàn toàn hiện tượng proposal điểm 0.96 bị Sniper Guard VETO oan.
2. **Nâng cấp bộ lọc nến rút chân:**
   - Thay vì cấm cứng `close < open`, chuyển sang điều kiện logic linh hoạt hơn:
     ```rust
     // Cho phép nến rút chân mạnh (CLV >= 0.40) dù thân nến đỏ nhẹ
     if clv < 0.15 || (close < open && clv < 0.40) {
         return None;
     }
     ```
3. **Bổ sung xác nhận động lượng nhịp hồi (Trajectory Filter):**
   - Kiểm tra volume nhịp hồi: Không cho phép mở vị thế nếu nến trước đó là một cây nến xả cực lớn xuyên thủng KAMA (`vol_z > 2.5` và thân nến đối kháng $> 1.5 \text{ ATR}$).
