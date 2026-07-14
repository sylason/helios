# Grill Me - System Design Questions

Để sử dụng file này, bạn có thể gọi command `/grill-me` hoặc tự xem và suy ngẫm, sau đó chúng ta sẽ thảo luận để làm rõ các điểm mù (blind spots) trong kiến trúc trước khi bước vào code.

> **Cách dùng:** Đánh dấu [x] vào lựa chọn của bạn hoặc viết thêm comment vào phần "Ghi chú thêm".

---

### Q1. Về cơ chế xử lý mất kết nối (WebSocket Disconnect) trong bản Demo
Khi Binance WebSocket ngắt kết nối trong lúc bot đang chạy, bạn muốn hành vi của Bot sẽ như thế nào?

- [ ] **A.** Đóng ngay lập tức mọi vị thế (Panic Close) để an toàn, sau đó thử kết nối lại.
- [x] **B.** Giữ nguyên vị thế, cố gắng kết nối lại (Exponential Backoff). Nếu sau 3 phút không được thì mới Panic Close.
- [ ] **C.** Chỉ in ra Log báo lỗi, không làm gì cả, phó mặc lệnh SL/TP cứng đã đặt trên sàn (Nếu có).

*Ghi chú thêm của bạn:*
thay vì panic close thì ta nên để trailing stoploss theo dõi trước. Nếu không kết nối được markprice API để trailing stoploss thì mới panic close sau 5 phút

---

### Q2. Về độ trễ tổng hợp K-line (Data Aggregation Latency)
Data Engine tổng hợp nến 1 phút từ các tick `aggTrade`. Do đó nến của bot sẽ luôn đóng chậm hơn sàn khoảng vài mili-giây. Bạn chấp nhận điều này như thế nào?

- [ ] **A.** Không sao, chậm 1-10ms để đổi lấy tính Data Integrity (tự tính) là hoàn toàn xứng đáng.
- [ ] **B.** Không được, tôi muốn tốc độ phải tuyệt đối. Hãy bỏ việc tự tính, subscribe thẳng kênh K-line 1m của Binance (dù có rủi ro bị sót nến).

*Ghi chú thêm của bạn:*
bỏ qua aggTrade và gọi kline 1m websocket và historical kline 1m để nhanh chóng kiểm tra chức năng demo.

---

### Q3. Về việc lưu trữ dữ liệu K-Line trong bản Local-First
Bot sử dụng DuckDB. Qua thời gian, dữ liệu nến 1 phút có thể lên tới hàng GB. Chính sách lưu trữ dữ liệu trên máy tính của bạn là gì?

- [ ] **A.** Giữ vô thời hạn (Lưu mọi thứ). Ổ cứng tôi rất nhiều chỗ trống. Rất tốt cho Backtest sau này.
- [x] **B.** Chỉ giữ Rolling Window (ví dụ: Giữ nến của 6 tháng gần nhất). Hàng tuần tự động xóa data cũ.
- [ ] **C.** Ở bản Demo không quan tâm. Production tính sau.

*Ghi chú thêm của bạn:*
giữ nến 1 tháng 3 ngày gần nhất. chỉ backtest và optimize trên dữ liệu 1 tháng gần nhất. tôi tắt mở app liên tục để bảo vệ máy tính cá nhân. do đó mỗi khi mở app, mở bot cặp asset phải kiểm tra khoảng trống gap nến 1m và bù lại. tự động xóa data cũ hàng ngày khi app mở.

---

### Q4. Về Hardcoded Strategy ở bản Demo
Bạn muốn chiến lược Demo được truyền vào Bot theo cách nào để test nhanh nhất?

- [ ] **A.** Hardcode hoàn toàn trong Rust. Muốn đổi tham số (ví dụ RSI 14 thành 21) phải sửa code và recompile.
- [ ] **B.** Viết một file JSON/YAML cấu hình bên ngoài. Bot đọc lúc khởi động.
- [ ] **C.** Có một ô Input trên Giao diện (React UI) để nhập số vào. (Tốn thêm xíu công UI)

*Ghi chú thêm của bạn:*
bot của ta và ferro_trade có chức năng optimizer sử dụng thuật toán genetic algorithm có thay đổi cho phù hợp mục đích của riêng tôi. app phải chạy optimizer thường xuyên liên tục để optimizer tìm ra các params tham số và config tối ưu để strategy sinh lời tối đa trong tình hình hiện tại. các params và config sẽ được lưu trong database ở format json, vì trong tương lai ta có thể viết thêm strategy, và mỗi strategy thì có thiết kế params khác nhau. tôi vẫn đang phân vân nên tạo bảng riêng cho từng strategy không hay lưu trong 1 attribute của bảng ở dạng json? tôi cần được tư vấn. vì strategy cần nhiều thời gian để nghiên cứu.

mỗi cặp asset có thể chọn strategy khác nhau, và ta chỉ bước vào trade khi khởi động bot asset và optimizer đã tìm ra params tối ưu, trong lúc bot đã mở, khi nào optimizer chạy xong và ta có bộ params mới thì bot asset phải lấy bộ params mới sử dụng. 

optimizer sẽ giúp tạo ra bộ params thích hợp nhất cho tình hình hiện tại của từng asset. nó sẽ giúp ta phân tích market, long/short ra sao, trailing stoploss như thế nào. 

---

### Q5. Paper Trading (Đánh Ảo) vs Testnet
Ở bản Demo, khi phát tín hiệu, lệnh sẽ được xử lý ra sao?

- [ ] **A.** Testnet: Gửi lệnh lên Binance Testnet (phải có Testnet API Key). Trải nghiệm trượt giá như thật.
- [ ] **B.** Paper Trading Local: Không gửi lên mạng. Tự tính lãi/lỗ ảo nội bộ trong Rust dựa vào giá `markPrice` hiện tại. Nhẹ đầu, dễ debug.

*Ghi chú thêm của bạn:*
...
Demo ta chia ra làm nhiều giai đoạn khác nhau:
- ta kiếm tra được hệ thống tắt mở bot asset và lấy dữ liệu từ binance.
- ta kiểm tra được lấy dữ liệu, không có khoảng trống.
- kiểm tra được hệ thống optimizer, lấy bộ params mới nhất println console và các stats được tối ưu.
- kiểm tra được papar trading bằng cách lệnh gọi thì ta println ra console, không gửi lên mạng. Paper Trading Local: Không gửi lên mạng. Tự tính lãi/lỗ ảo nội bộ trong Rust dựa vào giá `markPrice` hiện tại. Nhẹ đầu, dễ debug.
- kiểm tra được testnet: Gửi lệnh lên Binance Testnet (phải có Testnet API Key). Trải nghiệm trượt giá như thật.
- kiểm tra được trading thật khi mọi điều kiện hoàn hảo để thử nghiệm thực tế.
---

### BONUS: 
Có 1 số bổ sung và một số thông tin bạn hiểu nhầm:
- Ferro_trade là project backtest engine được dịch từ backtesting.py sang Rust nhưng tôi đã thêm chức năng và thay đổi cho mục đích thiết kế của tôi. nó chưa hoàn thiện nhưng đủ để ta áp dụng vào trading_bot này. vì thế ta không cần phải đưa nó vào bản mô tả trong trading_bot dù ta sẽ bổ sung một số chức năng cho nó phù hợp với live trading.
- ta nên mở websocket trước rồi mới lấy  dữ liệu historical kline để không bị khoảng trống gap nến khi chưa kịp gọi historical kline.
- ta nên gọi websocket và historical kline 1m trực tiếp. bỏ qua aggTrade. để tiết kiệm khả năng xử lý do aggregate data cũng tốn thời gian xử lý, CPU và RAM khi lưu trữ thêm. ta còn tốn thêm thời gian viết thêm business logic aggTrade và aggregate trong khi ta cần kiểm tra các chức năng quan trọng hơn như hệ thống thích nghi optimizer genetic algorithm liên tục, kiểm tra strategy helios trong thời gian thực live trading,...  
- hệ thống thích nghi phải đưa vào demo luôn nhưng sau khi ta xác nhận được rằng hệ thống đã lấy được kline và không có gap khoảng trống nào trong các nến để khi backtest sẽ không bị thiếu dữ liệu.
- giao diện ta không cần terminal, hay trades_logs, bảng trade logs, vì trade logs đã có trên binance nên ta chỉ cần gọi API binance là được. ta cần đơn giản hóa vấn đề. ta không cần logs, ta có thể in ra console bằng println!, như đã nói đây là demo. 
- giao diện ta nên có thông tin position hiện tại của cặp asset. 
- ta cần phải kiểm tra được các điều sau trong demo: 
    - hệ thống lấy được dữ liệu và không có gap.
    - hệ thống chạy optimizer liên tục và cập nhật bộ params tối ưu của strategy vào database với format là json. 
    - hệ thống optimizer phải in ra kết quả tối ưu nhất để đánh giá strategy trong môi trường thực tế và thời gian optimizer tính toán xử lý
    - in println buy hay sell market order trước khi đưa vào testnet.
    - kiểm tra testnet các chức năng ổn thì ta sẽ demo trade trên môi trường thật trong bản demo luôn. 

- tôi cần được tư vấn hệ thống lại folder skeleton structure, technical architecture trong folder project trading_bot/bot_engine nếu có thể. hãy viết ra file markdown bản mô tả technical như ta đang làm để làm việc tường minh khoa học chuyên nghiệp như software developer đang phát triển bằng phương pháp spec driven development. 