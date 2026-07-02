# 1c. Giao thức Hạt nhân (The Nuke Protocol)

## Ý tưởng Cốt lõi
Trong chiến tranh, bom hạt nhân là vũ khí hủy diệt hàng loạt chỉ được sử dụng ở trạng thái tuyệt đối chắc chắn, vì rủi ro của nó là quá lớn.

Trong Trading, **Đòn bẩy 100x chính là Bom Hạt Nhân.** Nó có thể biến số vốn nhỏ thành gia tài, nhưng cũng có thể quét sạch tài khoản chỉ trong 1 giây.

## Điều kiện Kích hoạt Giao thức Hạt nhân (Nuke Criteria)
Để Helios dám kích hoạt đòn bẩy 100x, hội đồng phải thỏa mãn bộ quy tắc sinh tử sau:
1. **Sự đồng thuận Đa khung (MTF Confluence):** H1 (Trend) + M15 (Zone) + M1 (Trigger) phải chỉ về cùng một hướng.
2. **Bảo kê Thanh khoản (Liquidity Wall):** Điểm vào lệnh (Entry) phải nằm ngay sát một bức tường Limit Order khổng lồ của phe cùng chiều, đóng vai trò như khiên chắn đạn.
3. **Volatility Shrink (Biến động co hẹp cực đại):** SL phải đủ nhỏ. Nếu ATR đang là 2% (nghĩa là nến giật 2%), việc đánh 100x là tự sát vì SL sẽ vượt quá giới hạn cháy tài khoản (1%).

## Cầu chì An toàn (The Dead Man's Switch)
Tuyệt đối không sử dụng "Nuke" trong trạng thái **Anomaly** hoặc **Parabolic** trừ khi bạn là người chuyên đánh săn thanh lý (Liquidation Hunting). Hạt nhân chỉ an toàn khi thả vào vùng **Squeeze Breakout** (khi năng lượng mới bung ra và rủi ro giật ngược là thấp nhất).
