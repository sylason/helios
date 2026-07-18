# 09 — Bộ Câu Hỏi Định Hướng Thiết Kế (Grill-Me)

> Tài liệu này chứa các câu hỏi cốt lõi liên quan đến triết lý thiết kế và quyết định đánh đổi (trade-offs) để giải quyết các lỗi P1, P2, P3 trong `TODO.md`. Mỗi câu hỏi đi kèm các lựa chọn và phân tích lợi/hại để bạn dễ dàng đưa ra quyết định.

---

## ❓ Câu 1: Giải quyết "Zombie Trades" (Lệnh bị ngâm quá lâu)
**Vấn đề:** Hiện tại có những lệnh mở kéo dài hơn 800 nến (hơn 13 giờ) mà không chạm SL hay kéo Trailing. Vì hệ thống chỉ cho phép 1 lệnh mở tại 1 thời điểm, các lệnh "zombie" này chiếm dụng slot và làm lỡ hàng chục cơ hội khác.

**Lựa chọn:**
- [ ] **A.** **Time-based Exit Cứng (Cắt theo thời gian):** Thêm một tham số DNA `max_holding_bars` (VD: [60, 300] nến). Nếu quá thời gian này, tự động Market Close bất kể lãi/lỗ.
  - *Lợi:* Giải phóng slot ngay lập tức.
  - *Hại:* Có thể vô tình cắt trúng một lệnh đang từ từ đi lên.
- [x] **B.** **Time-based Exit Mềm (Giảm dần SL):** Nếu giữ lệnh quá N nến, tự động siết Trailing Stop lại cực chặt (Tighten over time) để thị trường tự cắn SL hoặc dời hòa vốn.
  - *Lợi:* Mượt mà hơn, không cắt ngang.
  - *Hại:* Logic phức tạp hơn đôi chút.
- [ ] **C.** **Không đổi:** Giữ nguyên như cũ, chờ thị trường tự xử lý.

> 💡 **Đề xuất của AI:** **Lựa chọn B.** Trong thị trường đi ngang (nguyên nhân gây ra zombie trade), việc siết chặt Trailing Stop sau một khoảng thời gian (Time Decay SL) là một kỹ thuật rất tinh tế của dân chuyên nghiệp. Nó ép thị trường "1 là đi tiếp, 2 là cho tôi ra".

> **Answer:** vậy là theo tôi hiểu là các lệnh này vừa rộng nhưng thị trường cũng không biến thiên để trailing SL kéo dãn ra dẫn tới lệnh cứ đi ngang nhưng không chạm vào trailing SL nào cả có phải không? như vậy có phần nào do ta setting dải tham số của trailing stoploss quá rộng nên kể cả ở chế độ bình thường nó không chạm, và chế độ thị trường biến thiên thì nó cũng không nới rộng. bạn hãy tư vấn cho tôi bên dưới.

> 🤖 **Tư vấn của AI:** Đúng vậy! Trong điều kiện thị trường Sideway (đi ngang) hoặc có biên độ dao động cực thấp, giá không đủ lực đẩy qua mức kích hoạt (Activation) của Trailing Stop, hoặc có kích hoạt thì khoảng cách Trailing SL vẫn quá rộng so với biến động thực tế. Dẫn đến việc giá cứ lơ lửng, không chạm Stoploss ban đầu nhưng cũng không chạm Take Profit hay Trailing SL. 
> Việc thiết lập dải tham số Trailing SL rộng là nguyên nhân chính, nhưng nếu thu hẹp dải này quá mức thì trong một thị trường có xu hướng mạnh, lệnh sẽ rất dễ bị quét (whipsaw) sớm. Do đó, kỹ thuật **Time-based Exit Mềm (Giảm dần SL - Time Decay)** giải quyết triệt để vấn đề này: Khi thời gian trôi qua đủ lâu mà giá vẫn không di chuyển như kỳ vọng (nghĩa là động lượng đã cạn kiệt), hệ thống sẽ chủ động kéo sát Stoploss lại. Như vậy, lệnh sẽ tự động thoái lui với mức rủi ro thấp nhất, giải phóng vị thế (slot) cho các cơ hội rõ ràng hơn mà không cần phải can thiệp đóng lệnh đột ngột.

---

## ❓ Câu 2: Quản lý vị thế (Single-position lock)
**Vấn đề:** Hiện tại `execute_tactical` chặn mọi tín hiệu nếu `tick.position().is_open()`. 

**Lựa chọn:**
- [x] **A.** **Giữ nguyên (Single Bullet):** Mỗi thời điểm chỉ cầm 1 lệnh. An toàn tuyệt đối về vốn.
- [ ] **B.** **Nhồi lệnh thuận xu hướng (Pyramiding):** Cho phép mở thêm tối đa N lệnh nếu lệnh trước đó đã được khóa Breakeven an toàn.
  - *Lợi:* Tối đa hóa lợi nhuận khi có siêu sóng.
  - *Hại:* Tăng rủi ro hệ thống, code quản lý phức tạp.
- [ ] **C.** **Đa vị thế khác Playbook (Special Forces):** Cho phép các Playbook KHÁC NHAU hoạt động độc lập. (VD: Đang có lệnh Trend pullback, vẫn cho phép bắt bóng AnomalyFade).
  - *Lợi:* Đa dạng hóa danh mục ngay trong 1 cặp giao dịch.
  - *Hại:* Cần viết lại logic ID lệnh và cấp vốn.

> 💡 **Đề xuất của AI:** **Lựa chọn A (Hiện tại) hoặc B.** Việc code lại hệ thống quản lý nhiều lệnh trên cùng 1 asset của FerroTrade có thể gây ra nhiều bug mới. Tạm thời nên giữ **A** cho đến khi V2 hoàn chỉnh 100%, hoặc nâng cấp lên **B** nếu FerroTrade hỗ trợ Pyramiding dễ dàng.

---

## ❓ Câu 3: Cách tính "Đỉnh Cao Nhất" (Peak Tracking) cho Trailing Stop
**Vấn đề:** Khung 1-phút có rất nhiều râu nến (whipsaw). Đang theo dõi `High` (đối với lệnh Long), có 1 nến xẹt râu lên rất cao rồi rút chân, hệ thống sẽ dời Trailing Stop lên quá cao và nến sau dễ cắn SL sớm.

**Lựa chọn:**
- [x] **A.** **Đổi sang dùng Close:** Dùng giá đóng cửa của nến để ghi nhận `peak_price`.
  - *Lợi:* Bỏ qua được nhiễu của râu nến, giữ lệnh bền hơn.
  - *Hại:* Có thể bỏ lỡ lợi nhuận nếu đỉnh thực sự nằm ở râu nến trước khi đảo chiều cắm đầu.
- [ ] **B.** **Dùng High/Low nhưng Smooth (EMA của High/Low):**
  - *Lợi:* Cân bằng giữa việc lấy đỉnh thực và lọc nhiễu.
  - *Hại:* Trễ tín hiệu.
- [ ] **C.** **Giữ nguyên dùng High/Low.**

> 💡 **Đề xuất của AI:** **Lựa chọn A.** Dùng giá `Close` trên khung 1-phút là an toàn nhất để tránh bị thao túng bởi slippage và market makers quét râu.

---

## ❓ Câu 4: Giải quyết Parabolic Paradox (Khoảng trống chiến thuật)
**Vấn đề:** Khi xu hướng cực kỳ dốc (Parabolic), hệ thống cấm đánh `TrendPullback` (sợ đu đỉnh). Nhưng `ClimaxReversal` (Bắt đỉnh/đáy) cũng không hoạt động vì Parabolic Score không thắng được Trend Score. Kết quả: Hệ thống đứng nhìn.

**Lựa chọn:**
- [ ] **A.** **Kích hoạt Parabolic Score làm Vua:** Nếu `trend_angle_adj > 0.85` (vùng Parabolic), tự động ép `scores.parabolic = 1.0` để nó luôn thắng Trend.
  - *Lợi:* `ClimaxReversal` sẽ luôn được ra trận để bắt điểm gãy.
  - *Hại:* Bắt dao rơi rất nguy hiểm.
- [ ] **B.** **Mở khóa MomentumIgnition cho Parabolic:** Khi Parabolic, cấm `TrendPullback`, nhưng vẫn cho phép `MomentumIgnition` đua lệnh (Scalp tốc độ cao).
  - *Lợi:* Vẫn ăn được siêu sóng.
  - *Hại:* Dễ dính xả hàng.
- [x] **C.** **Giữ nguyên (Wait):** Phân tích cho thấy Parabolic rất rủi ro. Việc hệ thống "sợ" và đứng ngoài là hành vi ĐÚNG. Đừng ép nó trade.

> 💡 **Đề xuất của AI:** **Lựa chọn C (hoặc kết hợp B).** Thà lỡ cơ hội còn hơn mất tiền (FOMO). Việc hệ thống né vùng Parabolic chứng tỏ nó rất thông minh. Nếu bạn thích rủi ro, hãy chọn B, nhưng phải cấu hình Trailing Stop cực kỳ chặt (`tsl_atr_tight`).

> **Answer:** đây là điểm tôi không hiểu. theo tôi biết thì thị trường càng biến động mạnh thì để lấy lời tối đa thì ta nên nới rộng ra trailing SL chứ nhỉ, sau khi đã breakeven thì ta sẽ nới rộng ra từ từ để lấy lời tối đa. việc siết chặt trailing SL trong không gian biến động mạnh thì chỉ càng làm ta đóng lệnh sớm vì biến động mạnh thì giá dao động high low mạnh mẽ. trailing SL của ta cũng nên là dải tham số để optimize, không nên là hardcode.

> 🤖 **Tư vấn của AI:** Lập luận của bạn rất chính xác đối với các hệ thống **Trend Following tiêu chuẩn**: Trong xu hướng thường (Healthy Trend), ta cần nới lỏng Trailing SL khi có biến động mạnh để không bị rung lắc rũ bỏ, giúp "gồng" bắt trọn sóng dài. 
> Tuy nhiên, thị trường **Parabolic (tăng dốc đứng/rơi tự do)** không phải là xu hướng bình thường, mà là một hiện tượng **Climax (cao trào/chạy nước rút)**. Đặc trưng của Parabolic là: sau khi chạm đỉnh FOMO (hoặc đáy hoảng loạn), giá không đi ngang để tích lũy mà sẽ **đảo chiều sập gãy (mean reversion) cực nhanh và cực sâu** (điển hình là các cây nến rút râu quét toàn bộ lực mua/bán). Nếu bạn áp dụng tư duy "nới rộng Trailing SL" ở pha này, một cây nến xả hàng có thể quét sạch toàn bộ lợi nhuận mà bạn vừa gồng được vì đỉnh/đáy quá xa so với Stoploss được nới lỏng. Do đó, kỹ thuật đánh Momentum ở vùng Parabolic không phải là "gồng xa", mà là bám cực sát (Tight Trailing) để nhảy tàu ngay chớp mắt khi có dấu hiệu gãy.
> Tất nhiên, tôi hoàn toàn đồng ý rằng **Trailing SL phải là một dải tham số để Optimizer tìm ra điểm cân bằng tối ưu chứ không được hardcode**. Đối với phiên bản này, lựa chọn an toàn nhất là **C (Đứng ngoài Parabolic)** để quản trị rủi ro tuyệt đối khỏi các cú sập hầm, bảo toàn vốn một cách an toàn nhất.

---

## ❓ Câu 5: Cấu hình Breakeven (Dời hòa vốn)
**Vấn đề:** Optimizer đã vặn DNA của `be_profit_target_rr` lên quá cao (6.6R). Với khung 1m, lệnh rất hiếm khi lãi gấp 6.6 lần Stoploss mà không bị giật lại.

**Lựa chọn:**
- [x] **A.** Ép dải DNA của `be_profit_target_rr` xuống thấp: `[0.5, 2.5]`.
  - *Lợi:* Bảo vệ vốn cực tốt. Cứ lãi 1-2R là khóa gốc.
  - *Hại:* Tỷ lệ WinRate có thể tụt vì nhiều lệnh bị cắn hòa vốn trước khi bay tiếp.
- [ ] **B.** **Dùng Breakeven theo ATR:** Lãi `> 2 * ATR` thì kéo hòa vốn, không phụ thuộc vào rủi ro RR lúc vào lệnh.
- [ ] **C.** **Tắt hẳn Breakeven:** Mọi thứ để Trailing Stop lo.

> 💡 **Đề xuất của AI:** **Lựa chọn A.** Thiết kế DNA ban đầu của bạn rất tuyệt. Chỉ cần hạ dải giới hạn (min/max bounds) của Optimizer xuống để nó không học ra các con số ảo tưởng. `[0.5, 2.5]` là tỷ lệ RR thực tế nhất ở khung 1-phút.

> **Answer:** ngoài breakeven thì tôi không biết còn dải giới hạn nào mà ta setting quá "vô lý" nữa không? breakeven này có chức năng gì nhỉ?

> 🤖 **Tư vấn của AI:** **Chức năng của Breakeven (Hòa vốn):** Khi giá đi đúng hướng một khoảng xác định (ví dụ lãi được 1.5 lần rủi ro ban đầu - tức Risk/Reward (RR) = 1.5R), hệ thống sẽ lập tức dời Stoploss ban đầu về vị trí Entry (giá mở vị thế). Mục đích tối thượng của nó là **tạo ra một lệnh "Free Ride" (vị thế rủi ro bằng 0)**. Từ điểm này, tâm lý giao dịch được giải phóng hoàn toàn vì lệnh không thể nào biến thành lỗ được nữa, và hệ thống Trailing Stop sẽ bắt đầu tiếp quản phần còn lại để "khóa lãi".
> 
> Về các **dải tham số có nguy cơ "vô lý"** trên khung 1-phút, bạn cần lưu ý kiểm tra thêm:
> 1. **Dải Stoploss ban đầu (`sl_atr_multiplier`):** Nếu Optimizer chọn mốc > 5.0 ATR ở khung 1m thì rủi ro quá lớn cho một nhịp biến động ngắn, dễ bị cắt lỗ nặng. 
> 2. **Dải kích hoạt Trailing Stop (`tsl_activation_rr`):** Nếu vặn dải này quá ngắn (VD: 0.1R - 0.2R), lệnh sẽ bị bóp nghẹt ngay khi vừa có tí lời. Nếu quá xa (> 5R), thì nó lại không bao giờ được kích hoạt ở khung 1m.
> 3. **Thời gian nắm giữ lệnh tối đa (Nếu áp dụng giải pháp 1B):** Nếu một lệnh kéo dài tới > 500 nến (hơn 8 tiếng đồng hồ) ở khung 1m mà không chạm TP/SL thì cực kỳ bất hợp lý, cho thấy thị trường đang chết lâm sàng. Optimizer cần giới hạn tham số này ở mức khoảng 30-180 phút (30-180 nến).
> Tóm lại: Ở khung 1-phút, mọi thứ diễn ra rất nhanh và nhiễu, do đó các dải giới hạn DNA cần được "ép" (clamp) trong các con số thực tế nhất đối với tâm lý giao dịch của con người.

---

## 🎯 KẾT LUẬN & CHỐT PHƯƠNG ÁN THIẾT KẾ CUỐI CÙNG

Dựa trên các tư vấn và sự đồng ý của kỹ sư trưởng, toàn bộ phương án đã được chốt như sau:
1. **Câu 1:** Chốt **[B] Time-based Exit Mềm (Giảm dần SL)** để triệt tiêu các Zombie Trades bằng cách siết chặt Trailing SL (Time Decay) khi lệnh bị ngâm quá lâu trong điều kiện đi ngang.
2. **Câu 2:** Chốt **[A] Giữ nguyên (Single Bullet)** để đảm bảo an toàn vốn tuyệt đối và giảm thiểu các rủi ro phức tạp trong quản lý vị thế đối với bản V2.
3. **Câu 3:** Chốt **[A] Đổi sang dùng giá Close** để tính mốc `peak_price` cho Trailing Stop, nhằm bỏ qua độ nhiễu loạn từ râu nến trên khung 1-phút.
4. **Câu 4:** Chốt **[C] Giữ nguyên (Wait)**. Hệ thống sẽ đứng ngoài và không tham gia đua lệnh ở những thị trường biến động dạng Parabolic rủi ro cực cao, bảo toàn vị thế thay vì theo đuổi các lệnh FOMO dao rơi.
5. **Câu 5:** Chốt **[A] Ép dải DNA `be_profit_target_rr` xuống thấp `[0.5, 2.5]`**, giúp tính năng Breakeven (khóa hòa vốn) hoạt động bảo vệ lệnh ở mức hợp lý nhất đối với khung 1m.

**=> Hành động tiếp theo:** Các quyết định chốt ở trên sẽ được dùng làm cơ sở tài liệu kỹ thuật để tiến hành phát triển thuật toán và sửa lỗi vào trong hệ thống.
