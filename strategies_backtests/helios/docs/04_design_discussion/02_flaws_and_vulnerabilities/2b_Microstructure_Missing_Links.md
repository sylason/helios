# 2b. Lỗ hổng Dữ liệu Vi mô (Microstructure Missing Links)

> [!WARNING]
> Helios hiện tại đánh giá trận địa dựa trên nến 1-phút (OHLCV). Khi đánh đòn bẩy lớn, sự khác biệt giữa sống và chết không nằm ở hình dáng nến, mà nằm ở những gì xảy ra bên trong cây nến đó.

## Sát thủ vô hình: Slippage (Trượt giá)

Khi hệ thống phát hiện Squeeze Breakout và bắn tín hiệu `Market Buy`, ở đòn bẩy 100x, khối lượng danh nghĩa của bạn rất lớn.
Nếu sổ lệnh (Order Book) đang mỏng (Liquidity void), lệnh Market của bạn sẽ "ăn" sạch các lệnh Ask phía trên.
Kết quả: Giá khớp lệnh thực tế của bạn sẽ cao hơn giá `current_close` rất nhiều.
Với đòn bẩy 100x, trượt giá 0.1% nghĩa là bạn đã âm 10% ngay khi lệnh vừa được mở!

## Vấn đề về Khối lượng (Volume)

Helios đo `volume_force` bằng tổng khối lượng giao dịch. Tuy nhiên, 1000 BTC giao dịch tại một nến có thể là:
- A. 1000 BTC do phe Mua chủ động đớp thẳng vào lệnh Bán (Aggressive Buying).
- B. 1000 BTC do phe Bán chủ động xả thẳng vào lệnh Mua (Aggressive Selling).

Tổng Volume không cho biết phe nào đang hung hãn.

## Giải pháp đề xuất
1. **Tích hợp Delta Volume / Footprint:** Cần thu thập dữ liệu về chênh lệch giữa lệnh Mua chủ động và lệnh Bán chủ động (Delta). Chỉ bóp cò khi Delta đồng thuận với hướng đột phá.
2. **Theo dõi Độ sâu Order Book (Market Depth):** Đo đạc chênh lệch giữa tường Ask và tường Bid. Nếu tường Bid dày đặc, đó là tấm khiên an toàn để vào lệnh Long.
