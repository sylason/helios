# Hệ thống Thực thi và Quản trị Rủi ro (V3)

## 1. Triết lý Quản trị Vốn V3 (Separation of Concerns)
Trong V3, ranh giới giữa **Helios (Strategy Engine)** và **Trading Bot (Execution Gateway)** được phân định rõ ràng:
- **Helios chỉ tính toán "Sự thật Toán học" (Mathematical Truth):** Tính ra khoảng cách Stoploss dựa trên độ nhiễu của thị trường (ATR), và tính ra phần trăm quy mô lệnh lý tưởng dựa trên rủi ro cố định. Nó không quan tâm bạn đang trade ở sàn nào, có bao nhiêu tiền, hay giới hạn Notional Value tối thiểu là bao nhiêu.
- **Trading Bot chịu trách nhiệm "Thực thi Thực tế" (Physical Execution):** Chuyển đổi phần trăm rủi ro thành USD/USDT, ép vào giới hạn tài khoản, kiểm tra `Min Notional` của Binance.

## 2. Dynamic Risk-Based Sizing (Định cỡ lệnh theo rủi ro)

Cách tiếp cận của V2: `Vào lệnh = Cố định 20% vốn`. (Rủi ro: Stoploss xa + Đòn bẩy cao = Cháy tài khoản).
Cách tiếp cận của V3: `Rủi ro cố định (VD: 5% vốn/lệnh)`.

**Quy trình tính toán trong Helios:**
1. Trọng tài duyệt tín hiệu MUA ở giá `$1000`.
2. Hệ thống tính `ATR = $20`.
3. Khoảng cách Stoploss (SL) = `1.5 * ATR = $30` (Giá SL là `$970`). Khoảng cách tương đối là `30 / 1000 = 3%`.
4. Mức độ rủi ro mong muốn của User: `5%` vốn.
5. **Quy mô lệnh lý tưởng (Ideal Position Size):** `5% / 3% = 1.66` (Tức là Helios đề xuất vào lệnh bằng 1.66 lần vốn sở hữu hiện tại - có sử dụng đòn bẩy).

**Quy trình trong Trading Bot (Ngoài lề để hiểu ngữ cảnh):**
1. Nhận đề xuất Size `1.66` từ Helios. Vốn tài khoản đang có: `$60`.
2. Quy mô lệnh (bằng USD): `$60 * 1.66 = $99.6`.
3. Kiểm tra trần rủi ro tĩnh của người dùng (Max $10/lệnh do vốn ít): Ép xuống còn `$10`.
4. Kiểm tra Min Notional của Binance (Tối thiểu $5/lệnh): `$10 > $5` -> Lệnh hợp lệ. (Nếu Size cuối cùng < $5, Trading Bot từ chối lệnh).

## 3. Quản lý Chốt Lời (Take Profit & Trailing Stop)

Do hạn chế về vốn (chỉ có thể vào lệnh ở kích thước tối thiểu của sàn), việc **Chốt lời từng phần (Partial TP)** là bất khả thi vì không thể cắt 50% khối lượng lệnh. Vì vậy, Helios V3 sẽ áp dụng cơ chế **Trailing Stop làm vũ khí duy nhất** để quản lý giao dịch.

### 3.1. Trailing Stop (Khóa Lãi Động & Tự Động Hòa Vốn)
Hệ thống sẽ liên tục dời Stoploss bám theo giá (dựa trên mức `Peak Price` - Giá cao nhất/thấp nhất đạt được từ lúc mở lệnh).
- Khoảng cách bám đuổi (Trailing Distance) được tính bằng ATR nhân với `Squeeze/Trend Factor` (thị trường càng nhiễu thì thả lỏng dây, thị trường êm thì kéo căng dây).
- **Tự động Hòa vốn (Breakeven):** Khi giá chạy thuận lợi một đoạn đủ xa, mức Trailing Stop sẽ tự động vượt qua giá vốn cộng với phí giao dịch (Maker/Taker + Slippage). Kể từ thời điểm đó, lệnh được đảm bảo ít nhất là hòa vốn hoặc có lãi.
- Lệnh sẽ tự động đóng toàn bộ khi giá quay đầu quét qua vạch Trailing Stop. Hệ thống không đoán đỉnh, chỉ đi theo sóng tới khi sóng vỡ.
