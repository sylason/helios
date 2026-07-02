# BẢN THIẾT KẾ CHIẾN LƯỢC HELIOS (BLUEPRINTS)

Tài liệu này hệ thống lại toàn bộ tư duy vận hành của Helios, từ việc quan sát thị trường đến việc ra quyết định tấn công. Chúng ta sẽ xây dựng tài liệu này theo từng phần nhỏ, đi từ tổng quan đến chi tiết.

---

## TỔNG QUAN: HỆ THỐNG 4 CẤP ĐỘ (The 4-Level Architecture)

Để dễ hình dung, hãy coi Helios là một **Tòa tháp chỉ huy**, nơi dữ liệu đi từ dưới lên trên qua 4 tầng lọc:

1.  **Tầng 1 - Giác quan (Sensors):** Nơi tiếp nhận dữ liệu thô (giá, khối lượng). Nó trả lời câu hỏi: *"Thị trường có đang an toàn để chơi hay không?"*. Nếu tầng này báo "Nguy hiểm" (Hỗn loạn), tòa tháp sẽ đóng cửa ngay lập tức.
2.  **Tầng 2 - Nhận thức (Matrix & Regime):** Nơi phân tích dữ liệu để hiểu "Thế trận". Nó trả lời câu hỏi: *"Thị trường đang ở trạng thái nào?"* (Đang chạy, đang nghỉ, hay đang nén lại).
3.  **Tầng 3 - Kế hoạch (Playbook):** Sau khi biết thế trận, đây là nơi chọn "Bài đánh". Nó trả lời câu hỏi: *"Với thế trận này, ta nên dùng đội đặc nhiệm nào?"*.
4.  **Tầng 4 - Thực thi (Tactical Sniper):** Nơi các tay súng trực tiếp bóp cò. Nó trả lời câu hỏi: *"Vào lệnh ở mức giá nào, dừng lỗ ở đâu?"*.

---

## PHẦN 1: CÁC TRẠNG THÁI THỊ TRƯỜNG (MARKET REGIMES)

Chúng ta chia thị trường thành 6 "Thế trận" chính. Mỗi thế trận đại diện cho một hành vi của đám đông:

### 1. Thế trận "Nén lò xo" (Squeeze)
*   **Mô tả:** Giống như một cái lò xo bị ép chặt. Giá di chuyển trong một phạm vi cực hẹp, biến động giảm xuống mức tối thiểu. Đây là giai đoạn thị trường đang tích lũy năng lượng để chuẩn bị cho một cú bùng nổ mạnh.
*   **Hành vi đám đông:** Người mua và người bán đang thăm dò nhau, không ai dám đẩy giá đi xa.

### 2. Thế trận "Bám sóng" (Trending)
*   **Mô tả:** Giống như một con sông đang chảy xuôi dòng. Giá di chuyển rõ rệt theo một hướng (Lên hoặc Xuống). Xu hướng này có độ dốc vừa phải và bền vững.
*   **Hành vi đám đông:** Đám đông đồng lòng đẩy giá đi theo một hướng nhất định.

### 3. Thế trận "Đi ngang trong biên độ" (Ranging)
*   **Mô tả:** Giống như quả bóng nảy qua nảy lại giữa sàn nhà và trần nhà. Giá không có hướng đi rõ rệt, chỉ chạy lên rồi lại chạy xuống trong một cái hộp.
*   **Hành vi đám đông:** Không có phe nào (Mua hay Bán) chiếm ưu thế hoàn toàn.

### 4. Thế trận "Gom hàng" (Absorption)
*   **Mô tả:** Một trạng thái đặc biệt của đi ngang. Giá không chạy nhiều nhưng khối lượng giao dịch tăng vọt. 
*   **Giải thích từ lóng - Absorption (Hấp thụ):** Giống như một cái khăn thấm nước. Phe bán đẩy hàng ra bao nhiêu thì có một "ông lớn" bí mật (cá mập) đứng dưới hứng và mua hết bấy nhiêu khiến giá không thể giảm thêm.
*   **Hành vi đám đông:** Các nhà đầu tư lớn đang âm thầm tích lũy vị thế trước khi đẩy giá đi xa.

### 5. Thế trận "Bất thường" (Anomaly / Flash Crash)
*   **Mô tả:** Giống như một vụ tai nạn bất ngờ. Giá đột ngột tăng vọt hoặc sụt giảm cực mạnh trong thời gian rất ngắn, thường kèm theo sự hoảng loạn.
*   **Hành vi đám đông:** Hoảng loạn tột độ, các lệnh bán tháo hoặc mua đuổi diễn ra mù quáng.

### 6. Thế trận "Tăng hưng phấn" (Parabolic)
*   **Mô tả:** Giá không còn tăng/giảm đều mà bắt đầu bay thẳng đứng (hoặc rơi tự do) như tên lửa. 
*   **Giải thích từ lóng - Parabolic (Đường cong Parabol):** Đây là giai đoạn cuối của một xu hướng. 
    *   **Chiều lên:** Mọi người đều sợ bỏ lỡ cơ hội (FOMO) và lao vào mua bằng mọi giá.
    *   **Chiều xuống:** Mọi người hoảng loạn cực độ, bán tống bán tháo (Panic Selling).
*   **Hành vi đám đông:** Hưng phấn hoặc hoảng sợ quá đà, thiếu lý trí. Đây là lúc giá chạy nhanh nhất nhưng cũng dễ "đứt bóng" nhất.

---

## MỘT SỐ LƯU Ý QUAN TRỌNG

Trước khi sang phần tiếp theo, chúng ta cần làm rõ 3 khái niệm mà bạn vừa thắc mắc:

### 1. Sự chồng lấn (Overlap) giữa các thế trận
Thị trường không phải lúc nào cũng chỉ có một màu. Các thế trận có thể nằm chồng lên nhau:
*   **Gom hàng (Absorption)** thường xảy ra **bên trong** một vùng **Đi ngang (Ranging)** hoặc **Nén lò xo (Squeeze)**. Nó giống như việc bạn nhìn thấy một cái hộp (Đi ngang), nhưng soi kỹ bên trong thì thấy nước đang thấm vào khăn (Gom hàng).
*   **Bất thường (Anomaly)** có thể xảy ra ngay giữa một xu hướng **Bám sóng (Trending)**.

### 2. "Breakout" (Phá vỡ) là gì?
Trong trading, bạn sẽ nghe từ này rất nhiều. 
*   **Breakout không phải là một "Thế trận", mà là một "Sự kiện"**. 
*   Nó là khoảnh khắc giá thoát ra khỏi một cái hộp (Đi ngang) hoặc một cái lò xo (Nén). 
*   **Ví dụ:** Thế trận "Nén lò xo" là giai đoạn chuẩn bị, còn "Breakout" là lúc lò xo bung ra để bắt đầu một "Thế trận Bám sóng" mới.

### 3. Có thế trận nào khác không?
Còn một trạng thái nữa là **"Cạn kiệt" (Exhaustion)**:
*   **Mô tả:** Giống như một vận động viên đang chạy maraton nhưng bắt đầu hết hơi. Giá vẫn nhích lên nhưng bước đi rất nặng nề, khối lượng giao dịch giảm dần. 
*   Đây là trạng thái chuyển giao từ **Bám sóng** sang **Đi ngang** hoặc **Đảo chiều**.

---

## PHẦN 2: CÁC ĐỐI SÁCH (TACTICAL PLAYBOOKS)

Khi Helios đã nhận diện được "Thế trận", nó sẽ cử các **Đội đặc nhiệm** phù hợp để thực thi lệnh. Ở giai đoạn này, chúng ta phân tách rõ hành vi của từng đội:

### 1. Đội Squeeze SWAT (Chuyên đánh Phá vỡ lò xo)
*   **Mục tiêu:** Ăn trọn cú bùng nổ ngay khi lò xo vừa bung ra.
*   **Cách hành xử:**
    *   **Kiểu Quyết liệt (Aggressive):** Chỉ cần giá vừa chớm thoát khỏi biên là "bắn" ngay. Ưu điểm là vào được giá tốt nhất, nhược điểm là dễ bị "lừa" nếu giá chỉ nhích ra rồi quay lại ngay.
    *   **Kiểu Xác nhận (Confirmed):** Đợi giá thoát hẳn ra ngoài, sau đó quay lại chạm nhẹ vào biên cũ (gọi là Retest) mà không bị thủng thì mới vào. Ưu điểm là cực kỳ an toàn, nhược điểm là đôi khi giá bay luôn không quay lại, khiến ta bị lỡ tàu.

### 2. Đội Trend SWAT (Chuyên đánh Bám theo dòng chảy)
*   **Mục tiêu:** Không mua đuổi khi giá đã bay cao, mà đợi giá "hồi sức" rồi mới vào.
*   **Giải thích từ lóng - Pullback (Hồi giá):** Giống như một con sóng lớn tiến vào bờ rồi lại rút ra một chút để lấy đà cho con sóng tiếp theo.
*   **Cách hành xử:** Khi xu hướng đang mạnh, đội này sẽ đứng nhìn. Chỉ khi giá giảm nhẹ (hồi lại) về các vùng hỗ trợ an toàn và có dấu hiệu bật lên trở lại, đội này mới bóp cò.

### 3. Đội Momentum SWAT (Chuyên đánh Đuổi theo xung lực) - *Mới*
*   **Mục tiêu:** Tấn công khi thị trường có một lực đẩy cực mạnh (thường do tin tức tốt hoặc cá mập ra tay).
*   **Cách hành xử:** Không đợi lò xo nén, không đợi hồi giá. Nếu thấy dòng tiền đổ vào ồ ạt (Khối lượng vọt lên) và giá bắt đầu chạy nhanh, đội này sẽ lao vào "đuổi" theo giá ngay lập tức. Đây là đội đánh nhanh, thắng nhanh và rút nhanh.

### 4. Đội Range SWAT (Chuyên đánh Đảo chiều trong biên độ) - *Mới*
*   **Mục tiêu:** Kiếm tiền khi thị trường "ngủ đông" (đi ngang).
*   **Giải thích từ lóng - Mean Reversion (Trở về trung bình):** Giống như một cái dây thun, bạn kéo càng căng thì nó càng muốn bật về vị trí cũ.
*   **Cách hành xử:** Khi giá chạm vào "trần" hoặc "sàn" của cái hộp đi ngang, đội này sẽ đánh ngược hướng để đưa giá quay về đường trung tâm. Đây là đội chơi "bóng bàn", cứ giá sang biên này thì đánh sang biên kia.

### 5. Đội Reversal SWAT (Chuyên đánh Chặn đầu khi cạn kiệt) - *Mới*
*   **Mục tiêu:** Bắt đỉnh hoặc bắt đáy khi xu hướng đã chạy quá xa và bắt đầu "hết hơi".
*   **Giải thích từ lóng - Fading (Đánh ngược):** Hành động đặt cược chống lại xu hướng hiện tại.
*   **Cách hành xử:** Đội này cực kỳ kiên nhẫn. Họ chỉ xuất hiện khi giá đã bay thẳng đứng (Parabolic) và bắt đầu có dấu hiệu đuối sức (vận động viên hết hơi). Họ sẽ tung cú đấm "phản công" ngay khi đám đông đang hưng phấn nhất để ăn cú sập mạnh.

### 6. Đội Anomaly SWAT (Chuyên đánh Phản ứng nhanh khi có tai nạn)
*   **Mục tiêu:** Sống sót và kiếm lợi nhuận từ các cú sập hầm (Flash Crash) hoặc giá giật mạnh bất ngờ.
*   **Cách hành xử:** Tương tự như đội Reversal nhưng phản ứng nhanh hơn. Khi giá sập quá sâu trong thời gian quá ngắn, đội này sẽ nhảy vào mua để ăn cú hồi kỹ thuật ngay sau đó. Đây là đội "nhặt tiền rơi" trong lúc hỗn loạn.

---

## PHẦN 3: GIÁC QUAN (SENSORS) - NHÌN THẤY NHỮNG GÌ KHÔNG THỂ THẤY

Để nhận diện được 6 thế trận trên, Helios đeo 4 loại "Kính ngắm" đặc biệt. Mỗi kính giúp nó nhìn thấu một khía cạnh của thị trường:

### 1. Kính đo "Nhịp tim" (Biến động - ATR/NATR)
*   **Nó nhìn thấy gì:** Độ "giật" của giá. 
*   **Hình ảnh ẩn dụ:** Hãy tưởng tượng nhịp tim của thị trường. Khi thị trường bình yên, nhịp tim đều và nhẹ. Khi có tin sốc, nhịp tim đập loạn xạ và cực mạnh. 
*   **Dùng để làm gì:** Nếu nhịp tim quá cao (Hỗn loạn), Helios sẽ ngắt cầu chì. Nếu nhịp tim quá thấp, nó biết thị trường đang "ngủ" hoặc "nén lò xo".

### 2. Kính đo "Tốc độ" (Động lượng - RSI)
*   **Nó nhìn thấy gì:** Sức mạnh của phe mua hoặc phe bán hiện tại.
*   **Hình ảnh ẩn dụ:** Giống như kim đồng hồ tốc độ trên xe máy. Nếu kim chỉ ở mức cao (80-90), nghĩa là xe đang phóng rất nhanh (quá mua). Nếu kim ở mức thấp (10-20), xe đang phanh gấp hoặc lao dốc mạnh (quá bán).
*   **Dùng để làm gì:** Giúp các đội SWAT biết khi nào giá đã chạy quá nhanh (cần dừng lại) hoặc khi nào giá đang lấy đà.

### 3. Kính đo "Độ chật hẹp" (Vùng nén - Bollinger Bands & Keltner Channels)
*   **Nó nhìn thấy gì:** Biên độ mà giá đang dao động.
*   **Hình ảnh ẩn dụ:** Giống như những bức tường trong một hành lang. 
    *   Nếu hai bức tường bóp sát lại nhau, giá đang ở trong một hành lang cực hẹp (**Nén lò xo**). 
    *   Nếu giá phá vỡ bức tường này, nó báo hiệu một cú bùng nổ.
*   **Dùng để làm gì:** Đây là "mắt thần" để nhận diện thế trận Squeeze và Ranging.

### 4. Kính đo "Nhiệt lượng" (Dòng tiền - Volume Force & MFI)
*   **Nó nhìn thấy gì:** Sự hưng phấn thực sự của đám đông thông qua tiền mặt đổ vào.
*   **Hình ảnh ẩn dụ:** Giống như tiếng reo hò của khán giả trong sân vận động. 
    *   **Volume Force:** Đo tiếng reo hò hiện tại to hay nhỏ so với bình thường.
    *   **MFI (Chỉ số dòng tiền):** Một loại "Siêu kính ngắm" kết hợp cả Tốc độ và Tiền mặt. Nó cho biết phe nào đang thực sự bỏ tiền túi ra để đẩy giá.

### 5. Kính đo "Độ thẳng của đường đi" (Efficiency Ratio - ER)
*   **Nó nhìn thấy gì:** Thị trường đang trật tự hay đang hỗn loạn (nhiễu).
*   **Hình ảnh ẩn dụ:** Nếu bạn đi đường thẳng tắp, đó là xu hướng chất lượng cao. Nếu bạn đi vòng vèo, dích dắc, đó là thị trường đang "nhiễu". Đây là cầu chì để Helios biết khi nào nên dừng lại.

### 6. Kính đo "Độ dốc & Hướng" (LinReg Slope & Angle)
*   **Nó nhìn thấy gì:** Chính xác góc nghiêng của đường giá.
*   **Hình ảnh ẩn dụ:** Giống như một cái thước đo độ. Khi góc dốc lên 80 độ, Helios biết giá đang bay kiểu **Parabolic**.

### 7. Kính đo "Nền tảng xu hướng" (KAMA & ADX)
*   **Nó nhìn thấy gì:** Bộ khung vững chắc của xu hướng.
*   **Hình ảnh ẩn dụ:** KAMA giống như một con đường trung tâm ổn định, còn ADX đo xem con đường đó có đang được gia cố chắc chắn không.

### 8. Kính đo "Động lượng sớm & Điểm cắt" (MACD & Stochastic)
*   **Nó nhìn thấy gì:** Các tín hiệu đảo chiều hoặc xác nhận sớm.
*   **Hình ảnh ẩn dụ:** Giống như đèn tín hiệu giao thông. Nó báo hiệu khi nào dòng chảy bắt đầu đổi chiều để các đội SWAT chuẩn bị bóp cò.

---

## ĐỀ XUẤT CẢI THIỆN: CHÚNG TA CÓ NÊN THÊM GÌ KHÔNG?

Dù hiện tại "Vũ khí" của Helios đã rất đầy đủ, nhưng để thực sự đạt đến cấp độ "Elite", tôi đề xuất bổ sung thêm 2 loại kính ngắm này:

### 1. Kính "X-Quang" Khối lượng (Volume Profile)
*   **Nó nhìn thấy gì:** Tiền đang tập trung ở **Mức giá nào**, chứ không chỉ ở **Thời điểm nào**.
*   **Tại sao cần:** Giúp Helios biết đâu là "Vùng giá thật" mà các ông lớn đang giao dịch nhiều nhất, giúp đặt lệnh dừng lỗ (Stop Loss) cực kỳ an toàn.

### 2. Kính "Chuẩn hóa" Biến động (Z-Score)
*   **Nó nhìn thấy gì:** Sự bất thường được tính toán theo toán học xác suất.
*   **Tại sao cần:** Thay vì nhìn con số tuyệt đối, kính này báo cho Helios biết: *"Biến cố này chỉ xảy ra 1 lần trong 1000 ngày"*. Đây là cách tốt nhất để bắt các cú **Flash Crash** mà không bị nhầm với các dao động thông thường.

---

*(Bây giờ chúng ta đã có đủ 8 loại Giác quan hiện có. Phần tiếp theo sẽ là "Tầng 2: Nhận thức" - Nơi Helios kết hợp chúng lại).*

---

## PHẦN 4: NHẬN THỨC (PERCEPTION) - KHI CÁC GIÁC QUAN HỢP NHẤT

Đây là "Bộ não" của Tòa tháp chỉ huy. Ở tầng này, Helios không nhìn từng kính ngắm riêng lẻ nữa, mà nó tổng hợp dữ liệu để đưa ra một bản chẩn đoán về "Thế trận".

### 1. Cầu chì an toàn (The Safety Fuse)
Trước khi làm bất cứ việc gì, Helios kiểm tra xem "môi trường" có sống sót được không.
*   **Công thức:** Nếu (Nhịp tim quá nhanh) HOẶC (Độ thẳng đường đi quá thấp) HOẶC (Tiếng reo hò quá yếu) => **Hỗn loạn (Chaotic)**.
*   **Hành động:** Tắt điện, đóng cửa tòa tháp, không cho bất kỳ đội SWAT nào xuất quân.

### 2. Sổ tay chẩn đoán Thế trận (The Regime Diagnosis)
Khi môi trường đã an toàn, Helios bắt đầu soi 6 thế trận:

#### A. Chẩn đoán: "Nén lò xo" (Squeeze)
*   **Dấu hiệu:** Hai bức tường hành lang bóp cực sát nhau + Nhịp tim thị trường đang rất thấp.
*   **Kết luận:** Năng lượng đang tích tụ, chuẩn bị có biến lớn.

#### B. Chẩn đoán: "Bám sóng" (Trending)
*   **Dấu hiệu:** Góc lệch của thước đo độ đang nghiêng rõ rệt + Nền tảng xu hướng chắc chắn (ADX cao) + Đường đi thẳng tắp (ER cao).
*   **Kết luận:** Dòng chảy đang mạnh, ưu tiên đánh thuận chiều.

#### C. Chẩn đoán: "Đi ngang" (Ranging)
*   **Dấu hiệu:** Hai bức tường hành lang song song và rộng vừa phải + Góc lệch gần như bằng 0 + Tiếng reo hò ở mức trung bình.
*   **Kết luận:** Thị trường đang chơi bóng bàn, không có phe nào thắng thế.

#### D. Chẩn đoán: "Gom hàng" (Absorption)
*   **Dấu hiệu:** Giá đi ngang (giống Ranging) NHƯNG Tiếng reo hò (Volume) lại vang dội bất thường.
*   **Kết luận:** Có cá mập đang âm thầm hứng lệnh. Đây là cơ hội vào sớm trước khi có bùng nổ.

#### E. Chẩn đoán: "Bất thường" (Anomaly)
*   **Dấu hiệu:** Nhịp tim đập nhanh đột ngột + Tốc độ (RSI) nhảy vọt khỏi giới hạn thông thường + Giá lệch quá xa đường trung tâm.
*   **Kết luận:** Có tai nạn hoặc biến cố cực lớn. Chuẩn bị cho đội phản ứng nhanh vào "nhặt tiền".

#### F. Chẩn đoán: "Tăng hưng phấn" (Parabolic)
*   **Dấu hiệu:** Góc lệch bẻ cong gần như thẳng đứng + Tốc độ kim đồng hồ kịch trần.
*   **Kết luận:** Đám đông đang phát điên. Xu hướng sắp "gãy", chuẩn bị siết chặt túi tiền hoặc đánh ngược lại.

---

*(Giờ Tòa tháp đã biết mình đang ở đâu. Phần cuối cùng của bản thiết kế sẽ là "Sơ đồ tác chiến" - Kết nối Thế trận với Đội SWAT tương ứng và các quy tắc "Bóp cò" cụ thể).*

---

## PHẦN 5: SƠ ĐỒ TÁC CHIẾN (ACTION MAP)

Đây là bản tổng kết cuối cùng, chỉ đạo mọi hành động của Helios. Khi một Thế trận được xác nhận, Đội SWAT tương ứng sẽ vào vị trí và đợi lệnh "Bóp cò".

| Thế trận (Regime) | Đội SWAT (Playbook) | Điều kiện "Bóp cò" (Triggers) |
| :--- | :--- | :--- |
| **Nén lò xo** | **Squeeze SWAT** | Giá phá vỡ biên hành lang + Có tiếng reo hò xác nhận (MACD/Vol). |
| **Bám sóng** | **Trend SWAT** | Đợi giá "hồi sức" (giảm nhẹ) về đường KAMA + Có tín hiệu bật lên. |
| **Đang chạy mạnh** | **Momentum SWAT** | Giá chạy nhanh + Tiếng reo hò bùng nổ (Volume Spike). |
| **Đi ngang** | **Range SWAT** | Giá chạm trần/sàn hành lang + Có tín hiệu quay đầu (Stochastic). |
| **Gom hàng** | **Absorption SWAT** | Giá đi ngang + Tiếng reo hò cực lớn + Phe mua/bán chiếm ưu thế rõ rệt. |
| **Bất thường** | **Anomaly SWAT** | Giá lệch quá xa đường trung tâm + Xuất hiện lực cản (RSI cực hạn). |
| **Hưng phấn** | **Reversal SWAT** | Góc lệch thẳng đứng + Vận động viên "hết hơi" (RSI quay đầu). |

---

## PHẦN 6: QUẢN TRỊ RỦI RO (RISK MANAGEMENT) - CHIẾC ÁO GIÁP CO GIÃN

Helios không dùng một mức dừng lỗ cố định mà sử dụng cơ chế **"Dừng lỗ co giãn" (Elastic Trailing)**:

1.  **Khi tấn công (Squeeze/Momentum):** Áo giáp sẽ nới rộng một chút (Loose) để giá có không gian "thở" và bùng nổ, tránh bị văng ra bởi các rung lắc nhỏ.
2.  **Khi phòng thủ (Trending/Ranging):** Áo giáp ở mức bình thường (Normal), bám sát theo giá để bảo vệ lợi nhuận.
3.  **Khi nguy hiểm (Parabolic):** Áo giáp sẽ siết cực chặt (Tight). Vì lúc này giá có thể quay đầu bất cứ lúc nào, ta cần "khóa" lợi nhuận ngay lập tức.
4.  **Chặn lãi (Breakeven):** Khi lệnh đã có lãi đủ để trả tiền phí và một phần lợi nhuận, Helios sẽ tự động đưa điểm dừng lỗ về bằng đúng điểm vào lệnh. Lúc này, lệnh của bạn trở thành "Lệnh miễn phí" (Free Trade) - tệ nhất là hòa vốn, không thể thua.

---

### TỔNG KẾT
Bản thiết kế này giờ đây đã là một hệ thống hoàn chỉnh:
*   Biết nhìn thị trường (**Sensors**).
*   Biết phân tích thế trận (**Regime**).
*   Biết chọn bài đánh (**Playbook**).
*   Biết cách bóp cò và bảo vệ mình (**Tactical & Risk**).

---
*Tài liệu này sẽ là kim chỉ nam để chúng ta tiến hành sửa đổi mã nguồn (code) của Helios trong các bước tiếp theo.*
