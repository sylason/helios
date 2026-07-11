# 09 — Bộ Câu Hỏi Định Hướng Thiết Kế (Grill-Me)

> Tài liệu này chứa các câu hỏi cốt lõi liên quan đến triết lý thiết kế và quyết định đánh đổi (trade-offs) để giải quyết các lỗi P1, P2, P3 trong `TODO.md`. Mỗi câu hỏi đi kèm các lựa chọn và phân tích lợi/hại để bạn dễ dàng đưa ra quyết định.

---

## ❓ Câu 1: Giải quyết "Zombie Trades" (Lệnh bị ngâm quá lâu)
**Vấn đề:** Hiện tại có những lệnh mở kéo dài hơn 800 nến (hơn 13 giờ) mà không chạm SL hay kéo Trailing. Vì hệ thống chỉ cho phép 1 lệnh mở tại 1 thời điểm, các lệnh "zombie" này chiếm dụng slot và làm lỡ hàng chục cơ hội khác.

**Lựa chọn:**
- [ ] **A.** **Time-based Exit Cứng (Cắt theo thời gian):** Thêm một tham số DNA `max_holding_bars` (VD: [60, 300] nến). Nếu quá thời gian này, tự động Market Close bất kể lãi/lỗ.
  - *Lợi:* Giải phóng slot ngay lập tức.
  - *Hại:* Có thể vô tình cắt trúng một lệnh đang từ từ đi lên.
- [ ] **B.** **Time-based Exit Mềm (Giảm dần SL):** Nếu giữ lệnh quá N nến, tự động siết Trailing Stop lại cực chặt (Tighten over time) để thị trường tự cắn SL hoặc dời hòa vốn.
  - *Lợi:* Mượt mà hơn, không cắt ngang.
  - *Hại:* Logic phức tạp hơn đôi chút.
- [ ] **C.** **Không đổi:** Giữ nguyên như cũ, chờ thị trường tự xử lý.

> 💡 **Đề xuất của AI:** **Lựa chọn B.** Trong thị trường đi ngang (nguyên nhân gây ra zombie trade), việc siết chặt Trailing Stop sau một khoảng thời gian (Time Decay SL) là một kỹ thuật rất tinh tế của dân chuyên nghiệp. Nó ép thị trường "1 là đi tiếp, 2 là cho tôi ra".

---

## ❓ Câu 2: Quản lý vị thế (Single-position lock)
**Vấn đề:** Hiện tại `execute_tactical` chặn mọi tín hiệu nếu `tick.position().is_open()`. 

**Lựa chọn:**
- [X] **A.** **Giữ nguyên (Single Bullet):** Mỗi thời điểm chỉ cầm 1 lệnh. An toàn tuyệt đối về vốn.
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
- [X] **A.** **Đổi sang dùng Close:** Dùng giá đóng cửa của nến để ghi nhận `peak_price`.
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
- [ ] **C.** **Giữ nguyên (Wait):** Phân tích cho thấy Parabolic rất rủi ro. Việc hệ thống "sợ" và đứng ngoài là hành vi ĐÚNG. Đừng ép nó trade.

> 💡 **Đề xuất của AI:** **Lựa chọn C (hoặc kết hợp B).** Thà lỡ cơ hội còn hơn mất tiền (FOMO). Việc hệ thống né vùng Parabolic chứng tỏ nó rất thông minh. Nếu bạn thích rủi ro, hãy chọn B, nhưng phải cấu hình Trailing Stop cực kỳ chặt (`tsl_atr_tight`).

> Answer: đây là điểm tôi không hiểu, 

---

## ❓ Câu 5: Cấu hình Breakeven (Dời hòa vốn)
**Vấn đề:** Optimizer đã vặn DNA của `be_profit_target_rr` lên quá cao (6.6R). Với khung 1m, lệnh rất hiếm khi lãi gấp 6.6 lần Stoploss mà không bị giật lại.

**Lựa chọn:**
- [ ] **A.** Ép dải DNA của `be_profit_target_rr` xuống thấp: `[0.5, 2.5]`.
  - *Lợi:* Bảo vệ vốn cực tốt. Cứ lãi 1-2R là khóa gốc.
  - *Hại:* Tỷ lệ WinRate có thể tụt vì nhiều lệnh bị cắn hòa vốn trước khi bay tiếp.
- [ ] **B.** **Dùng Breakeven theo ATR:** Lãi `> 2 * ATR` thì kéo hòa vốn, không phụ thuộc vào rủi ro RR lúc vào lệnh.
- [ ] **C.** **Tắt hẳn Breakeven:** Mọi thứ để Trailing Stop lo.

> 💡 **Đề xuất của AI:** **Lựa chọn A.** Thiết kế DNA ban đầu của bạn rất tuyệt. Chỉ cần hạ dải giới hạn (min/max bounds) của Optimizer xuống để nó không học ra các con số ảo tưởng. `[0.5, 2.5]` là tỷ lệ RR thực tế nhất ở khung 1-phút.

> Answer: ngoài breakeven thì tôi không biết còn dải giới hạn nào "vô lý" nữa không? breakeven này có chức năng gì nhỉ? 
