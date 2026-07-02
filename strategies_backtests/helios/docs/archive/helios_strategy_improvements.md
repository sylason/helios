# Nhật ký cải thiện Logic chiến lược Helios

Tài liệu này ghi lại các ý tưởng nhằm tăng độ chính xác của hệ thống trong việc nhận diện trạng thái thị trường, dựa trên dữ liệu thuần túy và không dự đoán.

## 1. Kiểm tra sự đồng nhất của xu hướng
*   **Vấn đề:** Hiện tại, hệ thống coi là "có xu hướng rõ rệt" bằng cách cộng trung bình giữa "Tốc độ di chuyển" và "Góc di chuyển". Điều này dẫn đến việc nếu một chỉ số cực cao và chỉ số kia cực thấp, hệ thống vẫn bị đánh lừa.
*   **Mục tiêu:** Chỉ công nhận có xu hướng khi **CẢ HAI** chỉ số (Tốc độ và Góc) đều vượt qua một ngưỡng tối thiểu nhất định. Điều này loại bỏ các trường hợp giá biến động mạnh nhưng không có hướng rõ ràng.

## 2. Nhận diện trạng thái "Tích lũy năng lượng"
*   **Vấn đề:** Khi giá dao động trong phạm vi hẹp, chúng ta cần phân biệt giữa việc "thị trường ít hoạt động" và "thị trường đang có hoạt động giao dịch lớn nhưng giá chưa thoát ra".
*   **Mục tiêu:** Kết hợp thêm chỉ số "Độ thẳng của đường giá". Nếu hoạt động giao dịch (Volume) tăng cao, giá vẫn ở trong phạm vi hẹp, nhưng đường giá chạy dích dắc (độ thẳng thấp), đó là dấu hiệu của việc các lệnh mua/bán đang khớp nhau liên tục tại một vùng giá. Hệ thống cần nhận diện đây là một trạng thái đặc biệt thay vì chỉ là đi ngang thông thường.

## 3. So sánh biến động theo tỷ lệ thay vì con số tuyệt đối
*   **Vấn đề:** Các con số đo lường sự giật giá (biến động) thường thay đổi theo thời gian. Một con số 2.0 có thể là lớn ở hôm nay nhưng lại là bình thường ở tuần sau.
*   **Mục tiêu:** Thay vì dùng con số cố định, hệ thống sẽ tính toán: `Biến động hiện tại / Biến động trung bình của 100 nến trước đó`. Nếu tỷ lệ này > 2 (nghĩa là giá đang giật gấp đôi bình thường), hệ thống sẽ đánh dấu đây là trạng thái "Bất thường". Điều này giúp hệ thống tự thích nghi với mọi điều kiện thị trường mà không cần chúng ta phải chỉnh sửa thông số thủ công.

## 4. Thiết lập vùng đệm cho hướng xu hướng
*   **Vấn đề:** Hiện tại, chỉ cần chỉ số hướng (trend_dir) nhích lên hoặc xuống một mức siêu nhỏ (ví dụ 0.00001), hệ thống đã lập tức đổi trạng thái từ "Trung lập" sang "Tăng" hoặc "Giảm". Điều này gây ra hiện tượng tín hiệu bị nhiễu, đổi chiều liên tục khi giá thực tế đang đi ngang.
*   **Mục tiêu:** Thiết lập một vùng đệm tối thiểu. Chỉ khi chỉ số hướng vượt qua một ngưỡng xác định (ví dụ > 0.1 hoặc < -0.1) thì mới công nhận sự thay đổi hướng. Các giá trị nằm trong vùng đệm sẽ được coi là "Trung lập" để giữ sự ổn định cho chiến thuật.

## 5. Đồng bộ đơn vị đo lường biến động
*   **Vấn đề:** Trong mã nguồn, hệ thống đang so sánh một giá trị tuyệt đối (biến động thực tế của nến hiện tại) với một con số tỷ lệ (ngưỡng nhân bản biến động trong cấu hình). Việc so sánh hai đơn vị khác nhau này khiến logic nhận diện trạng thái "Bất thường" không bao giờ được kích hoạt.
*   **Mục tiêu:** Đảm bảo dữ liệu đưa vào và ngưỡng so sánh phải cùng một đơn vị (ví dụ: cả hai đều là tỷ lệ phần trăm hoặc cả hai đều là con số tuyệt đối). Điều này đảm bảo các bộ lọc bảo vệ của hệ thống hoạt động đúng như thiết kế.

## 6. Tính toán vị trí tương đối của giá (Z-Score thực)
*   **Vấn đề:** Hiện tại chỉ số đo lường "độ lệch" của giá đang dùng một công thức giả lập cố định (luôn trả về giá trị quanh mức 2.0). Nó không phản ánh được việc giá đang thực sự nằm ở đâu so với mức trung bình.
*   **Mục tiêu:** Tính toán chính xác khoảng cách từ giá hiện tại đến đường trung bình, chia cho độ lệch chuẩn. Chỉ số này sẽ cho biết chính xác khi nào giá đang ở trạng thái "quá mua cực hạn" hoặc "quá bán cực hạn" một cách khách quan, giúp chiến thuật phản ứng chính xác với các cú sốc thị trường.

## 7. Chuẩn hóa độ dốc xu hướng theo mức độ biến động (Khắc phục lỗi hướng trend)
*   **Vấn đề:** Việc xác định hướng Tăng/Giảm đang dựa trên một con số cố định (100.0) cho mọi tài sản. Điều này cực kỳ nguy hiểm vì nó làm hệ thống mất phương hướng: có lúc giá tăng rõ rệt nhưng hệ thống vẫn báo "Trung lập", hoặc giá chỉ nhích nhẹ nhưng hệ thống đã báo "Tăng mạnh".
*   **Mục tiêu:** Thay đổi cách tính hướng thị trường. Độ dốc của đường giá phải được so sánh với mức độ giật giá tự nhiên (ATR) của chính nó. Một xu hướng chỉ được coi là Bullish (Tăng) nếu tốc độ đi lên của nó lớn hơn mức biến động ngẫu nhiên của thị trường. Điều này đảm bảo "la bàn" của Helios luôn chỉ đúng hướng bất kể tài sản đang giao dịch là gì.

## 8. Kích hoạt tính năng duy trì bộ nhớ (Persistence)
*   **Vấn đề:** Hệ thống hiện có cấu trúc bộ nhớ nhưng không thực hiện việc ghi lại trạng thái của nến trước đó. Điều này khiến hệ thống bị "mất trí nhớ" sau mỗi nến, không thể nhận ra các mô hình diễn ra trong một khoảng thời gian dài.
*   **Mục tiêu:** Cập nhật bộ nhớ sau mỗi lần xử lý dữ liệu. Hệ thống cần biết nó vừa bước ra từ trạng thái nào để có phản ứng phù hợp (ví dụ: nếu vừa thoát khỏi vùng hỗn loạn, cần một khoảng thời gian chờ đợi để đảm bảo thị trường đã thực sự ổn định trở lại).

## 9. Học từ kết quả giao dịch (Thích nghi kiểu Mahoraga)
*   **Vấn đề:** Hệ thống hiện chưa lưu lại việc lệnh vừa rồi thắng hay thua. Một hệ thống thông minh cần biết rút kinh nghiệm: nếu vừa thua một lệnh do thị trường giật mạnh, nó nên tự động nghỉ ngơi hoặc giảm mức độ rủi ro ở lệnh tiếp theo.
*   **Mục tiêu:** Bổ sung cơ chế ghi nhớ kết quả giao dịch và các đặc điểm thị trường tại thời điểm thua lỗ. Điều này giúp Helios không lặp lại cùng một sai lầm trong cùng một điều kiện thị trường.

## 10. Loại bỏ "Điểm mù" của hệ thống cảm biến
*   **Vấn đề:** Hệ thống hiện nhìn thấy các chỉ báo nhưng lại không lưu trữ chuỗi giá gốc và các giá trị trung bình làm điểm chuẩn. Điều này khiến các tầng tính toán phía sau (như Matrix) không có hệ quy chiếu để biết "mạnh/yếu" hay "cao/thấp" một cách thực sự thích nghi.
*   **Mục tiêu:** Bổ sung vào Sensors các chuỗi dữ liệu gốc (Closes) và các chỉ số đo mức trung bình (SMA của biến động). Điều này cung cấp "hệ quy chiếu" giúp mọi quyết định của Helios luôn dựa trên sự so sánh tương quan thay vì các con số tuyệt đối cứng nhắc.

## 11. Tách biệt rõ ràng tham số Tốc độ và Gia tốc
*   **Vấn đề:** Playbook đang sử dụng tham số Gia tốc (Acceleration) làm ngưỡng cho Tốc độ (RSI). Do giá trị gia tốc thường rất nhỏ so với tốc độ, việc này khiến các bộ lọc bị vô hiệu hóa (luôn luôn đúng), làm hệ thống mất đi khả năng phân biệt giữa các tình huống cần vào lệnh nhanh hay chậm.
*   **Mục tiêu:** Sử dụng đúng loại tham số cho từng mục đích. Ngưỡng cho RSI phải là các tham số về Tốc độ/Động lượng để hệ thống có thể chọn lọc chính xác thời điểm vào lệnh.

## 12. Tham số hóa các màng lọc cảm tính
*   **Vấn đề:** Một số ngưỡng lọc hướng giá (Bias) trong trạng thái Gom hàng đang bị để cố định (0.55/0.45). Điều này vi phạm nguyên tắc thích nghi vì mỗi loại tài sản có mức độ nhạy cảm khác nhau.
*   **Mục tiêu:** Đưa toàn bộ các con số này vào cấu hình (Config). Việc này cho phép bộ máy tối ưu tìm ra các "điểm chạm" nhạy cảm nhất đối với từng tài sản cụ thể, thay vì áp dụng một công thức chung cho tất cả.

## 13. Tăng cường kiểm soát rủi ro khi thị trường bất thường (Anomaly)
*   **Vấn đề:** Hiện tại, khi gặp trạng thái "Bất thường" (Flash Crash), hệ thống lập tức kích hoạt chiến thuật đánh ngược (Fade). Đây là hành động cực kỳ nguy hiểm nếu đà rơi của giá chưa thực sự cạn kiệt.
*   **Mục tiêu:** Bổ sung các điều kiện kiểm tra sự cạn kiệt (Exhaustion) như RSI chạm mức cực hạn hoặc giá bắt đầu có dấu hiệu chững lại trước khi cho phép vào lệnh đánh ngược.

## 14. Đồng nhất tên gọi các thành phần hệ thống
*   **Vấn đề:** Một số tham số trong Playbook (như `matrix_rsi`) đang có tên gọi khác với định nghĩa trong Matrix (`momentum`). Sự thiếu đồng bộ này gây khó khăn cho việc bảo trì và nâng cấp.
*   **Mục tiêu:** Đồng bộ hóa tên gọi theo một chuẩn duy nhất trên toàn bộ hệ thống để đảm bảo tính nhất quán và dễ hiểu cho mọi lập trình viên.

## 15. Nhận diện trạng thái Parabolic bằng gia tốc góc
*   **Vấn đề:** Hiện tại chỉ sử dụng góc tuyệt đối để xác định trạng thái Parabolic. Một xu hướng mạnh có thể có góc cao nhưng vẫn bền vững. Trạng thái Parabolic thực sự xảy ra khi góc tăng liên tục (gia tốc góc dương).
*   **Mục tiêu:** So sánh góc hiện tại với trung bình các góc trước đó. Nếu góc hiện tại lớn hơn đáng kể so với trung bình, đó mới là dấu hiệu của sự hưng phấn quá đà cần siết chặt quản lý rủi ro.

## 16. Trailing Stop dựa trên giá đỉnh (Peak Price) thay vì giá hiện tại
*   **Vấn đề:** Hệ thống đang tính SL dựa trên giá đóng cửa của nến hiện tại. Nếu giá tăng mạnh rồi rút râu, SL sẽ bị đẩy lên quá cao theo cái râu đó, dẫn đến việc lệnh bị dừng lỗ quá sớm khi giá điều chỉnh nhẹ ở nến sau.
*   **Mục tiêu:** Sử dụng giá cao nhất (cho lệnh Long) hoặc thấp nhất (cho lệnh Short) kể từ khi mở lệnh để tính toán Trailing Stop. Điều này đảm bảo SL chỉ dời lên khi giá thực sự đạt được tầm cao mới bền vững.

## 17. Nguy cơ khi nới rộng SL trong trạng thái Bất thường (Anomaly)
*   **Vấn đề:** Code hiện tại nới rộng (Loose) khoảng cách SL khi gặp Anomaly. Nếu chiến thuật đang là đánh ngược (Fade) một cú sập mạnh, việc nới SL chỉ làm tăng mức độ thua lỗ nếu đà sập tiếp diễn.
*   **Mục tiêu:** Phối hợp giữa Playbook và Risk. Nếu đang đánh ngược xu hướng (Counter-trend), SL phải được siết chặt hoặc giữ nguyên thay vì nới rộng, để bảo vệ tài khoản trước các cú sập không đáy.

## 18. Cơ chế tự động đưa về điểm hòa vốn (Breakeven)
*   **Vấn đề:** Hiện tại hệ thống chỉ dời SL theo ATR mà không có mốc chặn an toàn tại điểm vào lệnh. Điều này khiến các lệnh đã có lãi lớn vẫn có thể quay đầu thành lỗ.
*   **Mục tiêu:** Bổ sung logic Breakeven: Khi lợi nhuận đạt một ngưỡng nhất định (ví dụ 1.5R), hệ thống tự động dời SL về điểm vào lệnh cộng với một khoảng bù phí.

## 19. Sử dụng ATR trung bình để ổn định mức dừng lỗ
*   **Vấn đề:** Việc dùng giá trị ATR tức thời khiến khoảng cách SL bị co giãn quá mức khi có một nến biến động bất thường, gây ra hiện tượng dừng lỗ không ổn định.
*   **Mục tiêu:** [x] Sử dụng giá trị trung bình mượt của ATR (ví dụ SMA 20 của ATR) để tính toán khoảng cách SL. Điều này giúp mức dừng lỗ có sự ổn định và phản ánh đúng mức biến động chung của thị trường thay vì bị nhiễu bởi một vài nến cá biệt.

## 20. Duy trì trạng thái Thế trận (Regime Persistence)
*   **Vấn đề:** Hiện tại, thế trận thị trường được quyết định chỉ dựa trên nến hiện tại mà không có bộ nhớ. Điều này dẫn đến việc thế trận bị nhảy loạn (flickering) giữa các trạng thái (ví dụ Ranging và Squeeze) nếu các chỉ số dao động quanh ngưỡng.
*   **Mục tiêu:** [x] Áp dụng cơ chế "Độ trễ" (Hysteresis). Khi đã ở trong một thế trận (như Trending), cần một tín hiệu yếu đi đáng kể (thấp hơn ngưỡng kích hoạt một khoảng bù) thì mới chuyển về trạng thái Ranging. Điều này giúp chiến thuật ổn định hơn.

## 21. Tinh chỉnh thứ tự ưu tiên trong Thế trận
*   **Vấn đề:** Hiện tại Squeeze đang được ưu tiên hơn Absorption. Tuy nhiên, Absorption (Gom hàng) là một tín hiệu hành động giá (Price Action) cụ thể và mạnh mẽ hơn so với trạng thái nén Squeeze vốn có thể kéo dài rất lâu mà không có hành động gì.
*   **Mục tiêu:** [x] Xem xét đưa Absorption lên ưu tiên cao hơn hoặc phối hợp cả hai để không bỏ lỡ các điểm "kích nổ" khi Volume tăng vọt trong vùng Squeeze.

## 22. Chuẩn hóa tính toán Độ tin cậy (Confidence)
*   **Vấn đề:** Việc sử dụng các con số giới hạn cứng như `min(1.1)` trong tính toán độ tin cậy của xu hướng làm mất đi thông tin của những xu hướng cực mạnh và gây sai lệch trọng số toán học.
*   **Mục tiêu:** Loại bỏ các giới hạn ảo, sử dụng các hàm chuẩn hóa (như Sigmoid hoặc Tanh) để đảm bảo dữ liệu luôn nằm trong khoảng [0, 1] nhưng vẫn phản ánh đúng mức độ cực đoan của thị trường.

## 23. Đồng nhất đơn vị đo lường cho màng lọc Chaos
*   **Vấn đề:** Chỉ số Chaos (NATR) đang được so sánh trực tiếp với một ngưỡng cấu hình mà không qua chuẩn hóa. NATR là một giá trị biến đổi theo biên độ giá, trong khi ngưỡng cấu hình thường mang tính cố định hoặc tỷ lệ.
*   **Mục tiêu:** [x] Chuẩn hóa NATR về cùng một hệ quy chiếu (ví dụ: Z-Score của NATR hoặc tỷ lệ NATR hiện tại so với trung bình) để màng lọc bảo vệ hoạt động chính xác trên mọi khung thời gian và tài sản.

## 24. Chống nhiễu cho bộ đếm trạng thái Trật tự (Orderly Bars)
*   **Vấn đề:** Hệ thống hiện tại reset bộ đếm về 0 ngay lập tức khi có chỉ một nến vi phạm nhẹ. Điều này khiến hệ thống quá nhạy cảm và dễ bị mất cơ hội giao dịch khi thị trường chỉ chớm có biến động nhỏ rồi ổn định lại ngay.
*   **Mục tiêu:** [x] Áp dụng cơ chế "Phạt điểm" thay vì "Reset trắng". Ví dụ: Một nến xấu sẽ trừ đi 3 điểm tích lũy, thay vì đưa về 0. Điều này giúp hệ thống giữ được sự kiên nhẫn cần thiết với thị trường.

## 25. Bổ sung màng lọc chống hoảng loạn (Volume Spike)
*   **Vấn đề:** Cầu chì hiện tại chỉ kiểm tra thanh khoản thấp, nhưng lại bỏ qua trường hợp thanh khoản tăng vọt bất thường (Volume Spike) - dấu hiệu của tin tức chấn động hoặc hoảng loạn cực độ.
*   **Mục tiêu:** [x] Thiết lập ngưỡng trần cho Volume Force. Nếu Volume tăng đột ngột gấp many lần trung bình, hệ thống sẽ tự động kích hoạt trạng thái Chaotic để đứng ngoài quan sát, bảo vệ tài khoản khỏi các biến động không thể kiểm soát.

## 26. Phân tách rõ rệt logic giữa các loại Playbook
*   **Vấn đề:** Hiện tại logic thực thi (Tactical) của các cặp Playbook (Aggressive vs Confirmed) đang bị trùng lặp 100%. Điều này làm mất đi ý nghĩa của việc phân loại thế trận ở các tầng trên.
*   **Mục tiêu:** Thiết kế lại logic vào lệnh. Playbook "Confirmed" (Xác nhận) phải yêu cầu các điều kiện khắt khe hơn (ví dụ: mô hình nến đảo chiều, Stochastic cắt rõ rệt), trong khi "Aggressive" (Quyết liệt) có thể vào lệnh ngay khi giá chạm biên với các điều kiện nới lỏng hơn.

## 27. Loại bỏ tham số "viết chết" trong thực thi chiến thuật
*   **Vấn đề:** Các chiến thuật Absorption và Anomaly đang sử dụng các hệ số SL/TP và ngưỡng RSI cố định trong mã nguồn. Điều này khiến chúng không thể tối ưu hóa và không thích nghi được with các tài sản có đặc tính khác nhau.
*   **Mục tiêu:** [x] Đưa toàn bộ các hệ số nhân rủi ro, ngưỡng động lượng vào cấu hình (Config) để Sniper có thể được tinh chỉnh chính xác cho từng loại thị trường thông qua bộ máy tối ưu hóa.

## 28. Bổ sung điều kiện "Cạn kiệt" (Exhaustion) cho chiến thuật đánh ngược
*   **Vấn đề:** Việc bắt đáy/đỉnh (Fade) chỉ dựa trên Z-Score là hành động mạo hiểm. Giá có thể lệch rất xa but vẫn tiếp tục lao đi nếu lực quán tính còn mạnh.
*   **Mục tiêu:** [x] Bổ sung các màng lọc xác nhận sự cạn kiệt đà chạy của giá như: RSI đạt mức cực hạn (quá mua/bán sâu), Volume bắt đầu giảm dần sau một cú Spike, hoặc có nến rút chân xác nhận lực từ chối. Điều này giúp tăng tỷ lệ thắng cho các lệnh Fade Anomaly.




