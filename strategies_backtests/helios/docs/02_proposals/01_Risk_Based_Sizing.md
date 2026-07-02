# Đề xuất 01: Quản lý vốn theo Đô-la (Dollar-Risk Based Sizing)

Dựa trên câu trả lời của bạn ở Q1 (Sử dụng 10% của quỹ $60 và điều chỉnh trailing stop tùy theo lợi nhuận).

## Vấn đề của cách cũ
Nếu bạn trích 10% của $60 (tức là $6) để làm Margin cho một lệnh đòn bẩy 100x.
Vị thế danh nghĩa của lệnh sẽ là `$6 * 100 = $600`.
Chỉ cần giá đi ngược $1\%$ (tức là $6), bạn sẽ bị thanh lý toàn bộ $6 tiền Margin đó ngay lập tức, bất chấp Dừng lỗ (SL) của bạn đang đặt ở đâu. 

## Đề xuất Giải pháp: Cố định Rủi ro Đô-la

Thay vì nói "Tôi dùng 10% vốn để đánh", hãy nói: **"Tôi chấp nhận MẤT TỐI ĐA $6 cho lệnh này"**.

Khi hệ thống có điểm bóp cò, nó sẽ làm phép toán sau:
1. Tìm điểm Stop Loss dựa trên ATR (để né độ nhiễu). Ví dụ: Khoảng cách SL là $20 giá.
2. Tính số Hợp đồng (Size) sao cho: Nếu giá chạy ngược 20 giá chạm SL, bạn **vừa vặn mất đúng $6**.
   `Size = $6 / $20 = 0.3 Hợp đồng`
3. Tính Margin yêu cầu: `Margin = (0.3 * Giá hiện tại) / Đòn bẩy 100`.
4. Nếu `Margin > $6` (vượt quá ngân sách 10% cho phép) -> **Không vào lệnh** (Vì SL quá rộng, quá rủi ro).

Cách này đảm bảo: Dù bạn đánh đòn bẩy 10x hay 100x, rủi ro là NHƯ NHAU ($6), đòn bẩy chỉ dùng để giảm số tiền bị sàn giam giữ (Margin).

## Trailing Stop Động năng (Dynamic Trailing)
Bạn muốn SL hẹp ở vùng Volatility thấp và rộng khi có lãi lớn:
Điều này hoàn toàn khả thi và cần thêm vào code `risk.rs`:
- Khi lệnh mới mở, Trailing SL = Cực Hẹp (Tight) để bảo vệ $6.
- Khi lệnh lãi > 2R (Gấp đôi rủi ro), Trailing SL giãn ra (Loose) = Cực Rộng để thả nổi cho trend chạy hết cỡ, đồng thời khóa điểm Breakeven.
