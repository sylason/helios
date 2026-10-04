# KIỂM TOÁN ĐỘ PHỦ HÀNH VI THỊ TRƯỜNG — 4 SCANNER ĐANG CHẠY

> **Phạm vi:** `trend_pb.rs`, `momentum_ignition.rs`, `reversal.rs`, `absorption.rs` + `arbiter.rs`, `gatekeeper.rs`, `matrix.rs`, `normalization.rs`.
> **Nguồn số liệu:** bảng "PAPER TRADER FINAL SUMMARY" cuối `trades.log` Run 1 (25 lệnh, +10.82%) và Run 2 (40 lệnh, −9.03%).
> **Giới hạn:** n rất nhỏ (3–18 lệnh/playbook), chỉ 1 cặp ETHUSDT, 1 giai đoạn ~10 ngày. Mọi con số bên dưới là **mô tả mẫu**, không phải bằng chứng thống kê có ý nghĩa.

---

## 0. ĐÍNH CHÍNH LƯỢT TRẢ LỜI TRƯỚC (bắt buộc đọc)

Khi đối chiếu lại với log và code, các phát biểu sau của tôi **không có căn cứ hoặc sai**:

| # | Tôi đã nói | Thực tế kiểm chứng |
|:-:|:---|:---|
| 1 | Bảng Price_Z/Vol_Z cho #0003, #0023, #0039 (Run 2), #0008, #0021 (Run 1) | **Tôi chưa đọc telemetry của các lệnh này.** Các con số đó không có nguồn. Chỉ 4 lệnh tôi thực sự đọc: Run 2 #6 (Z +4.22, Vol_Z +5.68, thua), Run 1 #22 (Z +3.92, Vol_Z +4.96, thua), Run 1 #23 (Z +2.29, thắng), Run 1 #24 (Z +2.52, **thua**). |
| 2 | "Z ≥ 3.9 thua, Z 2.0–2.8 thắng" như một ranh giới sạch | **Bị #24 bác bỏ** (Z +2.52, Vol_Z +0.50, vẫn thua −13.5%). Chỉ có 2 điểm Z>3.9, cả hai thua. Chưa đủ để khẳng định ngưỡng. |
| 3 | "Momentum đè bẹp Trend_PB (0.95–0.99 vs 0.65–0.80)" | **Sai.** Log Run 2 cho TREND_PB raw 0.955–0.969. Mọi lệnh được duyệt tôi đã đọc đều ghi `Sole approved proposal` — **chưa thấy lần đấu thầu nào có 2 scanner cùng thắng điều kiện**. |
| 4 | "Reversal và Trend_PB triệt tiêu nhau (deadlock Long vs Short)" | **Sai về mặt logic** (xem §4.1): cả hai đều bắt buộc dấu CLV khớp với chiều lệnh nên không thể ra 2 chiều ngược nhau trên cùng 1 nến. |
| 5 | "Absorption bị VETO 95%", các phần trăm "70% / 60% / 50% / 20%" | **Bịa, không đo.** 212/293 là số veto của *tất cả* scanner cộng lại. |
| 6 | Trích dẫn khoa học (Cont 2014 ↔ CLV; Jegadeesh-Titman ↔ momentum 1m; "quỹ lớn đều dùng…") | Cont et al. 2014 nói về **sổ lệnh L2**, không phải CLV nến. Jegadeesh-Titman là momentum **cross-section cổ phiếu 3–12 tháng**. Khẳng định về cách các quỹ vận hành **không kiểm chứng được**. Xem §6. |

Điều đúng ở lượt trước: `RANGE_OSC` chưa được cắm vào arbiter (`mod.rs:122-125` chỉ add 4 scanner); `momentum_ignition.rs` không có trần `Price_Z`.

---

## 1. HIỆU SUẤT THỰC TẾ THEO PLAYBOOK (ROE %, tính từ bảng final summary)

| Playbook | Run 1: n / thắng / Σ ROE | Run 2: n / thắng / Σ ROE | Nhận xét khách quan |
|:---|:---:|:---:|:---|
| **TREND_PB** | 12 / 7 / **+72.8** | 14 / 6 / **+21.6** | Dương cả 2 run. **Mong manh:** Run 2 bỏ #19 và #37 thì còn −40.4; Run 1 bỏ #6 và #13 thì còn +6.2. Lợi nhuận đến từ ít cú thắng lớn (+41%, +40%). |
| **MOMENTUM_IGN** | 10 / 3 / **−39.3** | 18 / 7 / **−38.2** | **Âm cả 2 run** (28 lệnh, 10 thắng). Avg thắng ≈ 10–15, avg thua ≈ 10–12. Đây là tín hiệu mạnh nhất trong dữ liệu, mạnh hơn giả thuyết "mua đỉnh". |
| **REVERSAL** | 0 lệnh | 5 / 3 / −2.9 | Avg thắng +7.9 vs avg thua −13.3 (R:R ≈ 0.6). Khớp với phân tích §3.3. |
| **ABSORPTION** | 3 / 2 / +24.0 | 3 / 1 / −22.2 | n=3, không kết luận được. |

> [!IMPORTANT]
> Nếu phải rút gọn xuống 2–3 scanner dựa trên dữ liệu hiện có, **TREND_PB là ứng viên duy nhất có dấu hiệu dương nhất quán**; **MOMENTUM_IGN là ứng viên âm nhất quán**. Nhưng n=28 lệnh MOM, 1 asset: cần backtest dài hơn trước khi quyết định bỏ hay sửa.

---

## 2. DANH MỤC HÀNH VI THỊ TRƯỜNG (theo vật lý quan sát được, không theo mẫu hình retail)

| ID | Hành vi | Dấu hiệu đo được |
|:-:|:---|:---|
| B1 | Xu hướng hiệu quả + nhịp hồi có trật tự | ER cao, KAMA dốc, giá hồi về vùng giá trị, động lượng ngược chiều yếu |
| B2 | Xu hướng, nhịp hồi nông (không chạm giá trị) | ER cao, giá đi ngang ngắn rồi tiếp tục |
| B3 | Nén biến động → bung nén (nến bung đầu tiên) | Squeeze cao ở nến trước, nến hiện tại đóng ngoài dải |
| B4 | Mở rộng muộn / nối đuôi sau cú bung | Giá đã cách xa vùng giá trị (|Z| lớn), dải đã mở rộng |
| B5 | Cao trào + từ chối (climax / exhaustion) | Cú đẩy kéo dài, nến từ chối (CLV đảo), khối lượng bất thường |
| B6 | Đi ngang dao động (mean-reverting range) | ER thấp, biến động bình thường, giá hồi quy về giữa biên |
| B7 | Hấp thụ (nỗ lực lớn, kết quả nhỏ) tại biên | Volume cao, biên độ nến hẹp, giá không đi tiếp |
| B8 | Bão hỗn loạn (whipsaw) | Biến động cao, ER thấp |
| B9 | Biến động chết (< chi phí) | NATR < ngưỡng phí |
| B10 | Cú sốc thanh khoản / thanh lý dây chuyền / gap | Nến cực đại, trượt giá, đuôi béo |
| B11 | Bung giả (failed breakout / trap) | Giá bung rồi quay lại trong dải |
| B12 | Chuyển chế độ (trend→range, range→trend) | ER/độ dốc đổi dấu, KAMA phản ứng trễ |
| B13 | Mùa vụ thanh khoản (phiên Á/Âu/Mỹ, funding, tin tức) | Giờ trong ngày, lịch sự kiện |
| B14 | Ngữ nghĩa volume khác nhau theo asset | Crypto = volume thật; Forex spot = tick volume (proxy) |
| B15 | Chi phí ma sát so với kỳ vọng di chuyển | Phí khứ hồi vs khoảng SL / mục tiêu |

---

## 3. MA TRẬN ĐỘ PHỦ (✅ xử lý đúng bản chất · 🟡 một phần/có lỗ hổng · ❌ không xử lý)

| ID | TREND_PB | MOMENTUM_IGN | REVERSAL | ABSORPTION | Gatekeeper / khác |
|:-:|:-:|:-:|:-:|:-:|:-:|
| B1 | ✅ (có lỗ hổng, xem 3.1) | — | — | — | |
| B2 | ❌ (loại do `stoch ≤ thresh`) | — | — | — | |
| B3 | — | 🟡 **không kiểm tra nén trước đó** | — | — | |
| B4 | — | ❌ **không có trần Z** | — | — | |
| B5 | — | — | 🟡 chọn "volume cạn" thay vì "volume cao trào" | — | |
| B6 | — | — | 🟡 chỉ gián tiếp | — | **Không ai sở hữu** (`RANGE_OSC` chưa cắm) |
| B7 | — | — | — | 🟡 SL hình học sai | |
| B8 | | | | | ✅ Chaos gate (`gatekeeper.rs:123-137`) |
| B9 | | | | | ✅ NATR floor (`gatekeeper.rs:107-121`) |
| B10 | ❌ | ❌ (có thể lao vào) | ❌ | ❌ | ❌ gate phản ứng trễ (percentile) |
| B11 | ❌ | ❌ | ❌ | 🟡 spring/upthrust | |
| B12 | 🟡 vào lệnh khi KAMA chưa kịp đổi dốc | | | | |
| B13 | ❌ | ❌ | ❌ | ❌ | ❌ không có bộ lọc phiên |
| B14 | ✅ không dùng volume | ❌ phụ thuộc `vol_z` | ❌ `vol_ratio`, MFI | ❌ `vol_z`, VWMA | |
| B15 | ❌ | ❌ | ❌ | ❌ | NATR floor chỉ là sàn thô |

### 3.1. TREND_PB (`trend_pb.rs`)

| Khối logic | Đánh giá | Bằng chứng |
|:---|:---|:---|
| Né vùng nén (`squeeze > max_squeeze`, dòng 60) | ✅ hợp lý, nhưng comment dòng 59 viết "đất diễn của range_osc" → scanner đó **không tồn tại trong arbiter**. Vùng nén không có chủ. | `mod.rs:122-125` |
| Xác định xu hướng chỉ bằng độ dốc KAMA | 🟡 một biến duy nhất, trễ pha. `kama_slope_atr` bị kẹp [−1, 1] (`matrix.rs:89`) và log cho thấy nó **bão hòa ±1.00 liên tục** → `f_slope` hằng số, mất khả năng phân biệt xu hướng mạnh/rất mạnh. `trend_strength` (ADX) và Efficiency Ratio có trong matrix nhưng **không được dùng**. | log: `KAMA_Slope_ATR: +1.00` lặp hàng chục nến |
| "Pullback" = ảnh chụp tại 1 nến (gần KAMA + stoch thấp) | 🟡 không đo **quỹ đạo hồi**: độ sâu, tốc độ, volume nhịp hồi. Một nhịp bán mạnh khối lượng lớn (đảo chiều) và một nhịp hồi nhẹ cho cùng tín hiệu. | `trend_pb.rs:100-114` |
| `close < open` cộng `clv < 0.15` | 🟡 `close < open` thừa với CLV và **loại các nến rút chân thân đỏ** (CLV cao nhưng close<open). Hằng số 0.15 không phải DNA. | dòng 112, 159 |
| SL = `low.min(kama_mid) − buffer` | 🟡 khoảng SL tỉ lệ với việc giá cách KAMA bao xa; trong khi điểm `f_dist` thưởng giá **gần** KAMA. Cap `max_initial_sl_pct` (8% ROE) và hình học SL **không được ràng buộc với nhau** → log Run 2 có hàng chục lần `TREND_PB raw 0.96 bị VETO SL 10.4–14% > 8%`. | log Run 2 dòng 1621–2330 |
| Chuỗi thua nhanh | 🟡 6 lệnh Run 2 (#17, 26, 27, 32, 34, 36) thua HARD_SL sau 7–13 nến với MFE ≤ 3%. Không phân biệt được "SL nằm trong nhiễu" và "vào sai chế độ" nếu không phân tích đường giá sau khi bị quét. | final summary Run 2 |

### 3.2. MOMENTUM_IGN (`momentum_ignition.rs`)

| Khối logic | Đánh giá | Bằng chứng |
|:---|:---|:---|
| Mô tả header: "thoát khỏi vùng tích lũy" | ❌ **Code không đọc `squeeze`, `chaos`, `z_score`** (dòng 39–166). Điều kiện duy nhất về vị trí: `close > bb_upper` (hoặc `< bb_lower`). Nó bắt cả cú bung đầu tiên lẫn cú đuổi muộn lẫn cao trào. | đọc toàn file |
| Chấm điểm | ❌ `sat_unbounded(vol_z)` và `sat_penetration` đơn điệu tăng → nến càng cực đoan điểm càng cao (0.954 ở Z +4.22 / Vol_Z +5.68). Hàm điểm **không phân biệt "bắt đầu" và "kiệt sức"**. | `normalization.rs:17-48`; log Run 2 #6 |
| Màu nến + CLV (dòng 77-83) | ✅ lọc râu từ chối hợp lý | |
| Veto dốc KAMA ngược chiều | ✅ chặn bung ngược trend | dòng 90-101 |
| Hằng số `target_breakout = 0.25 ATR`, `15.0` cho RSI | 🟡 không phải DNA | dòng 109, 116 |
| SL = đáy/đỉnh nến bung − buffer | 🟡 nến bung lớn → SL xa → bị VETO (log Run 2 cuối: MOM Short raw 0.846, SL 15.4% > 12%). Scanner bị phạt đúng ở nến mạnh nhất. | log dòng 6807 |
| Hiệu suất | **Âm cả 2 run** (§1) | |

### 3.3. REVERSAL (`reversal.rs`)

| Khối logic | Đánh giá | Bằng chứng |
|:---|:---|:---|
| Tham số `_matrix` bỏ trống | ❌ Scanner **không đọc dốc KAMA, squeeze, chaos, trend_strength**. Không có veto "không bắt đáy khi xu hướng đang rất mạnh" (ABS và MOM có `max_adverse_slope`, REV không). | `reversal.rs:44` |
| Tên "Climax" nhưng `f_vol = sat_proximity(vol_ratio, 0.5)` | 🟡 điểm cao khi volume **thấp hơn 0.5× SMA** (volume cạn), điểm 0 khi volume ≥ 0.5×. Đây là mô hình "cạn kiệt", không phải "cao trào khối lượng". Hai trường phái khác nhau; code chọn một mà tên gọi nói điều khác. | dòng 104-105 |
| TP = `kama_mid`, SL = `low − wick_buffer` | ❌ **Không có cổng R:R.** Khoảng TP tối thiểu = 0.5 ATR (min displacement), khoảng SL = (close−low) + buffer (mặc định 0.5 ATR) → R:R < 1 ngay từ thiết kế ở vùng biên của điều kiện. Khớp với R:R thực tế ≈ 0.6 (Run 2). TP là một đường trung bình trễ, đứng yên. | dòng 63-70, 120-122 |
| Gate chaos | ✅ **Đã được Gatekeeper toàn cục chặn trước khi scanner chạy** (`arbiter.rs:168`). Mục `TODO-R2-03` (thêm chaos>0.65 vào reversal) **gần như trùng lặp** với gate (ngưỡng vào ≈0.70 theo log). Nên hạ ưu tiên/xóa. | log: `Chaos: 0.722 … Entering Shelter` |

### 3.4. ABSORPTION (`absorption.rs`)

| Khối logic | Đánh giá | Bằng chứng |
|:---|:---|:---|
| Effort vs Result (`sat_ratio_damped`, dòng 67-70) | ✅ đúng ý niệm VSA: volume cao / biên độ hẹp | |
| `%B`, CLV spring/upthrust, VWMA vs BB middle (dòng 95-101) | 🟡 hợp lý về ý niệm, nhưng 4 điều kiện cứng đồng thời → rất hiếm | |
| SL = `low.min(bb_lower) − buffer` | ❌ Trong ≥20 mẫu veto tôi đọc ở Run 2 (Bar 20141–20761), SL luôn **21–37% ROE** so với trần 11%. Các vị trí này nằm ở `Price_Z ≈ +0.7…+1.1`, tức giữa dải, không phải biên. | log Run 2 dòng 366–990 |
| **Lineage cảnh báo** | ❌ Log ghi `NATR 0.89%` nhưng `ATR 11.09 / Price 2687.66 = 0.41%`. Hai đại lượng cùng bản chất lệch ~2.2 lần → `atr_sl` và `natr` **khác chu kỳ/khung**. Chưa xác minh được trong factory khung nào cho BB/z_score/ATR. Nếu BB ở khung cao còn ATR/giá ở 1m thì SL bám BB sẽ luôn "quá xa". | log dòng 369 |
| Giả định volume | ❌ Phụ thuộc volume thật. Forex spot chỉ có tick volume. | |

---

## 4. CÁC SCANNER CÓ "NGÁNG CHÂN" NHAU KHÔNG?

### 4.1. Xung đột Long vs Short: **không thể xảy ra** với tập scanner hiện tại
Cả 4 scanner đều dùng cùng công thức CLV = (2C − H − L)/(H − L) và **bắt buộc dấu CLV khớp chiều lệnh**: TREND_PB Long `clv ≥ 0.15`, Short `clv ≤ −0.15`; MOM `±min_clv ≥ 0.05`; REV `±min_clv ≥ 0.05`; ABS `±min_clv ≥ 0`. Trên cùng 1 nến, CLV chỉ có 1 giá trị → mọi đề xuất Long có CLV > 0, mọi đề xuất Short có CLV < 0 → **không bao giờ có Long và Short cùng nến** (trừ CLV = 0 chính xác khi `abs.min_clv = 0`).
**Hệ quả:** nhánh deadlock (`arbiter.rs:470-478`) và tham số `conflict_confidence_margin` gần như là mã chết hiện tại.

### 4.2. Các hình thức tương tác thực sự (đã xác minh bằng code/log)

| # | Cơ chế | Bằng chứng | Mức độ |
|:-:|:---|:---|:---|
| I1 | **Điểm tự tin không có cùng thước đo.** Mỗi scanner dùng `tanh` neo theo ngưỡng riêng (0.76 tại ngưỡng của chính nó). "Cao nhất thắng" so sánh các đại lượng không hiệu chuẩn. Quan sát 4 lệnh MOM: conf 0.954 thua, 0.799 thua, 0.415 thắng, 0.405 thua → **chưa thấy quan hệ conf ↔ kết quả** (n=4, cần kiểm chứng bằng biểu đồ độ tin cậy trên toàn log). | `arbiter.rs:149-155`; log | Cao (chưa đo) |
| I2 | **Cooldown chung, không phân biệt playbook.** Log Run 1 dòng 5551: lệnh MOM Long thua → ABSORPTION **Short** bị `ENGINE COOLDOWN VETO`. Một playbook nhiễu (MOM, âm) có thể khóa playbook tốt (TPB, dương). | log Run 1 dòng 5550–5576 | Trung bình |
| I3 | **Chỉ 1 vị thế tại một thời điểm.** Scanner nào bắn trước chiếm chỗ; cơ hội tốt hơn sau đó bị bỏ qua. | thiết kế | Trung bình |
| I4 | **Veto SL theo từng scanner chạy trước đấu thầu** (`arbiter.rs:219-233`) và trần `max_initial_sl_pct` khác nhau mỗi playbook (DNA). Scanner đề xuất SL theo hình học riêng, risk cap riêng → TPB 0.96 bị veto, ABS 0.28 bị veto. Hai bên không "biết" nhau. | log Run 2 | Cao |
| I5 | **Mỗi playbook thắng mang theo cách thoát khác nhau** (REV có TP = KAMA; TPB/MOM/ABS TP=None). Cùng chiều, 2 scanner → winner quyết định chế độ thoát. | code | Thấp–TB |

> [!NOTE]
> Giả thuyết "MOM cướp quyền TPB" và "REV triệt tiêu TPB" **không có chứng cứ**. Thực tế: I1, I2, I4 mới là điểm cần xử lý.

---

## 5. ĐÁNH GIÁ TỔNG HỢP

| Câu hỏi | Trả lời dựa trên code + log |
|:---|:---|
| 4 scanner đã phủ hết hành vi chưa? | **Chưa.** Không ai xử lý: B2, B6 (range), B10 (sốc thanh khoản), B13 (phiên), B15 (ma sát). B3/B4 (MOM) và B5 (REV) chỉ xử lý một phần so với tên gọi/mô tả. |
| Scanner nào đúng bản chất nhất? | TREND_PB (ý tưởng và dữ liệu nhất quán dương), nhưng thiếu đo quỹ đạo hồi và dùng 1 chỉ báo trễ. |
| Scanner nào lệch nhất giữa mô tả và code? | MOM (mô tả "thoát tích lũy", code không đọc squeeze), REV (tên "climax", code thưởng volume cạn). |
| Đã dynamic chưa? | Ngưỡng chính là DNA, nhưng còn hằng số cứng: TPB `0.15`, `1e-4`, `0.002`; MOM `0.25 ATR`, `15.0`; REV TP = KAMA; ABS `0.10`. Quan trọng hơn: **SL geometry và risk cap không đồng bộ** (I4). |
| Dùng được cho Forex/mọi asset chưa? | TPB gần như có; MOM/REV/ABS phụ thuộc volume (B14). Chưa có bộ lọc phiên (B13). |

---

## 6. VỀ BẰNG CHỨNG KHOA HỌC (đã tra cứu lại)

| Chủ đề | Điều đã xác minh | Hạn chế khi áp vào hệ thống này |
|:---|:---|:---|
| Time-series momentum (Moskowitz–Ooi–Pedersen 2012) | 58 hợp đồng (chỉ số, FX, hàng hóa, trái phiếu); lợi suất 12 tháng dự báo 1–12 tháng tiếp theo | Khung **tháng**, không phải nến 1 phút; không chứng minh cho day-trading đòn bẩy cao |
| Intraday momentum (Gao–Han–Li–Zhou 2018) | Lợi suất 30 phút đầu dự báo 30 phút cuối, trên S&P 500 ETF | Cơ chế đặc thù phiên cổ phiếu; chưa có bằng chứng tương đương cho crypto/FX 24/7 |
| Quy tắc kỹ thuật intraday (tổng quan tìm kiếm) | Sau khi điều chỉnh data-snooping và chi phí, lợi nhuận **thường biến mất**; FX intraday khó có lợi nhuận vượt phí | Đây là rủi ro trực tiếp cho hệ thống tối ưu hóa GA 102 gen trên cửa sổ 30 ngày |
| KAMA, Bollinger squeeze, Wyckoff VSA | Là công cụ thực hành. **Tôi chưa tìm thấy bằng chứng bình duyệt** chứng minh EV dương cho cách dùng cụ thể trong hệ thống này | Cần tự chứng minh bằng OOS dài |

**Kết luận trung thực:** Không có phương pháp nào "đã được chứng minh" cho đúng bài toán day-trading đòn bẩy cao trên mọi asset. Bằng chứng mạnh nhất nằm ở trend-following khung dài; bằng chứng cho khung ngắn **yếu và có tranh cãi**. Điều này không phủ nhận hệ thống, nhưng nghĩa là **tiêu chuẩn thành công phải dựa trên OOS dài và đa asset (đúng §6.6 GEMINI.md), không dựa vào lý thuyết**.

---

## 7. CÁC HƯỚNG XỬ LÝ (để thảo luận, chưa triển khai)

| Phương án | Nội dung | Ưu | Khuyết | Edge cases |
|:---|:---|:---|:---|:---|
| **A. Thu hẹp còn TPB + (MOM đã sửa)** | Tắt REV và ABS mặc định; sửa MOM: yêu cầu nén ở nến trước + trần/hàm phạt theo Z | Ít tương tác nhất; khớp ý định 2–3 scanner; TPB là scanner dương duy nhất | Mất B5/B7; giảm số lệnh (n nhỏ hơn → khó đánh giá) | Nếu TPB chỉ sống nhờ 2 cú thắng lớn, bỏ các scanner khác không cứu được EV |
| **B. Giữ 4, nhưng ràng buộc hình học SL ↔ risk cap** | Scanner nhận trần ROE của chính playbook và tự kẹp/loại sớm | Giảm veto oan (I4) | Đụng nhiều file; che lấp vấn đề hình học BB/ATR nếu nó là lỗi khung thời gian | SL kẹp sát hơn → dễ bị nhiễu quét |
| **C. Hiệu chuẩn điểm tự tin (I1)** | Đo conf vs EV thực trên log dài, thay "cao nhất thắng" bằng điểm hiệu chuẩn | Giải quyết gốc của đấu thầu | Cần nhiều lệnh hơn hiện có | n nhỏ → hiệu chuẩn nhiễu |
| **D. Sửa kiểm tra lineage trước** | Xác minh khung thời gian của BB, z_score, ATR, NATR trong factory | Rẻ, có thể giải thích lỗi ABS (SL 21–37%) và độ lệch ATR/NATR | Có thể không thay đổi gì | — |

**Đề xuất thứ tự:** D (xác minh, chỉ đọc code) → quyết định A hay B → C sau khi có nhiều dữ liệu OOS.

---

## 8. VIỆC CẦN XÁC MINH TIẾP (chưa làm)

- [ ] Khung thời gian của `bb_*`, `z_score`, `atr_sl`, `natr`, `kama_*` trong sensor factory (nguyên nhân ATR/NATR lệch ×2.2).
- [ ] Đếm số nến có ≥2 đề xuất được duyệt cùng lúc trong cả 2 log (xác định I1 có thực sự xảy ra không).
- [ ] Biểu đồ độ tin cậy: conf vs ROE cho toàn bộ lệnh MOM/TPB.
- [ ] Phân tích đường giá sau khi HARD_SL của chuỗi thua nhanh TPB (nhiễu hay sai chế độ).
- [ ] Backtest dài hơn, đa asset trước khi kết luận bỏ/giữ MOM.
