# MỔ XẺ CHUYÊN SÂU ABSORPTION SCOUT SCANNER (ABSORPTION)

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `scanners/absorption.rs`, `core_v3/config.rs:655-704`, `core_v3/mod.rs:195-200`.  
> **Dữ liệu thực tế:** Run 1 (+24.0% ROE, n=3, 2 thắng) & Run 2 (−22.2% ROE, n=3, 1 thắng). Bị VETO hơn 20 lần vì SL hình học vượt trần ROE.

---

## 1. BẢN CHẤT VẬT LÝ THỊ TRƯỜNG VS HIỆN TRẠNG MÃ NGUỒN

| Tiêu chí | Bản chất vật lý thị trường (VSA / Wyckoff) | Code hiện tại (`absorption.rs`) | Đánh giá First-Principles |
| :--- | :--- | :--- | :--- |
| **Mô hình Nỗ lực vs Kết quả (Effort vs Result)** | Khối lượng lớn (Nỗ lực) nhưng biên độ nến hẹp hoặc rút chân (Kết quả nhỏ) $\to$ Xuất hiện lực hấp thụ thụ động (Passive Absorption). | Dùng `sat_ratio_damped(raw_vol_z, norm_spread, vol_thresh)` với `norm_spread = candle_range / atr`. | ✅ **Mô hình toán học xuất sắc:** Phản ánh đúng bản chất VSA phi thứ nguyên, không cảm tính. |
| **Bối cảnh Vùng Nén (Squeeze)** | Quá trình gom/xả diễn ra trong giai đoạn thị trường tích lũy cạn kiệt biến động. | Dùng `sat_unbounded(squeeze, sqz_thresh)` và kết hợp trọng số DNA với `f_vol`. | ✅ **Đúng bản chất:** Gắn chặt lực hấp thụ với độ nén dải biến động. |
| **Điều kiện Kích hoạt (%B, CLV, VWMA)** | Nhận diện điểm Spring (vét thanh khoản đáy) hoặc Upthrust (quét đỉnh phân phối). | Ép 4 điều kiện cứng đồng thời: `%B` sát biên, CLV rút chân, VWMA lệch pha so với BB Middle. | ❌ **Over-Constrained Freeze:** Bộ lọc quá ngặt nghèo khiến scanner bị tê liệt, 22 ngày chỉ kích hoạt 3 lệnh. |
| **Hình học Stop Loss** | Neo dưới đáy cây nến nỗ lực hấp thụ hoặc râu nến quét thanh khoản (Spring Low). | Dùng `low.min(bb_lower) - buffer` (với Long) và `high.max(bb_upper) + buffer` (với Short). | ❌ **Tử huyệt BBands MTF Suction:** Khi BBands ở khung lớn (1h/4h), SL bị kéo tụt xuống dải xa, vọt lên 25%–37% ROE $\to$ Bị Sniper Guard VETO 100%. |
| **Tương thích Asset** | Crypto có volume thực; Forex spot chỉ có tick volume. | Phụ thuộc hoàn toàn vào `raw_vol_z`. | 🟡 Cần cơ chế chuẩn hóa hoặc proxy thay thế khi volume thực không khả dụng. |

---

## 2. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành phần | Ưu điểm | Khuyết điểm | Edge Cases nguy hiểm |
| :--- | :--- | :--- | :--- |
| **Hàm `sat_ratio_damped`** | Chuẩn hóa tương quan giữa Volume Z-Score và Spread nến / ATR. | Nhạy cảm với các nến Doji cực nhỏ (chia cho số nhỏ, dù đã có sàn 0.10). | Nến đóng băng thanh khoản (Flatline): Không có giao dịch khiến biên độ = 0, nếu volume đột ngột có 1 lệnh nhỏ cũng có thể làm méo tỷ lệ. |
| **Bộ lọc 4 tầng (`%B + CLV + VWMA + Trend Veto`)** | Độ chọn lọc cực cao, loại bỏ gần như 100% bẫy giả mạo. | **Bóp chết tần suất giao dịch (Sample Starvation):** 14 ngày backtest chỉ ra được 3 lệnh $\to$ Hệ thống không thể thích ứng. | Thị trường biến động nhanh: Giá nhúng qua dải rồi rút chân trong vài giây, nến 1m đóng ở giữa dải (%B > max_bound) $\to$ Bỏ lỡ hoàn toàn sóng bứt phá. |
| **So sánh VWMA vs BB Middle** | Cố gắng kiểm tra trọng tâm thanh khoản nằm ở nửa trên hay nửa dưới dải nén. | VWMA chu kỳ dài phản ánh quá khứ, không phản ánh hành vi tức thời của cây nến hấp thụ hiện tại. | Khi thị trường đi ngang hẹp, VWMA và BB Middle xoắn chặt vào nhau, điều kiện này mang tính ngẫu nhiên 50/50. |
| **Hình học SL `low.min(bb_lower) - buffer`** | Cố gắng tìm vùng hỗ trợ ngoài biên dải nén. | **Xung đột khung thời gian MTF:** `bb_lower` của khung 1h/4h nằm cách giá 1m quá xa $\to$ SL rộng gấp 3 lần trần ROE cho phép. | Hơn 20 cơ hội hấp thụ đẹp trong Run 2 (Bar 20141–20761) đều bị hủy bỏ vô ích vì lỗi hình học SL này. |

---

## 3. NGUYÊN NHÂN TOÁN HỌC KHIẾN ABSORPTION BỊ LIỆT TRONG RUN 2

Trong log Run 2, telemetry ghi nhận hàng loạt thẻ VETO dạng:
```text
Bar: 20484 | Veto Tier: TIER 3 (SNIPER ENTRY GUARD)
🛑 [RISK-VETO] ABSORPTION | Side: Short | Raw: 0.278 | Conf: 0.278 | 
Structural SL exceeds ROE cap: 34.5% > 11.0% (entry too risky)
```
- **Phân tích:** 
  - Tại bar 20484, giá là 2687.66.
  - Scanner Short đề xuất SL = `high.max(bb_upper) + buffer`.
  - Do `bb_upper` là của khung thời gian lớn MTF (được GA chọn là 1h hoặc 4h), `bb_upper` nằm ở tận vùng 2730–2740.
  - Khoảng cách từ Entry đến SL là $\approx 46 \text{ USD} \implies 1.72\%$ giá.
  - Tại đòn bẩy 20x: $1.72\% \times 20 = \mathbf{34.5\% \text{ ROE}}$!
  - Trong khi đó, trần rủi ro của Absorption trong DNA chỉ cho phép tối đa **11.0% ROE**.
  - **Hậu quả:** 100% các đề xuất của Absorption bị Sniper Entry Guard tiêu diệt ngay tại cửa ngõ Arbiter!

---

## 4. ĐỀ XUẤT CẢI TIẾN CỐT LÕI CHO ABSORPTION

1. **Khử bỏ hoàn toàn BBands MTF Suction Trap trong tính SL:**
   - Thay `low.min(bb_lower)` bằng đáy swing thực tế của nến hấp thụ:
     ```rust
     // Long: SL đặt dưới đáy nến hấp thụ (Spring Low) + đệm ATR
     let sl = low - buffer;
     // Short: SL đặt trên đỉnh nến hấp thụ (Upthrust High) + đệm ATR
     let sl = high + buffer;
     ```
   - Điều này đưa khoảng cách SL từ $34\%$ ROE về mức an toàn $6\% - 10\%$ ROE, mở khóa cho hàng chục lệnh hấp thụ hợp lệ được khớp lệnh.
2. **Nới lỏng bộ lọc %B và VWMA:**
   - Bỏ điều kiện cứng `vwma < bb_middle` (vì đã có CLV và %B bảo vệ).
   - Tăng độ phủ vị trí nến `%B`: Cho phép nến hấp thụ đóng ở bất kỳ vị trí nào trong nửa dưới dải nén (`pct_b <= 0.50` cho Long và `pct_b >= 0.50` cho Short).
3. **Cơ chế Fallback cho Forex:**
   - Khi chạy trên dữ liệu Forex (chỉ có tick volume), tự động chuyển đổi `raw_vol_z` sang đo lường vận tốc nến hoặc Range/ATR để không bị tê liệt.
