# 1b. Nghệ thuật Do thám StarCraft (The StarCraft Reconnaissance)

## Ý tưởng Cốt lõi
Trong StarCraft, bạn không thể chiến thắng nếu chỉ chăm chăm xây lính mà không dùng "Observer" hoặc "Scan" để do thám căn cứ địch (Fog of War). Trading cũng vậy, "Fog of War" chính là vùng mù của thị trường.

Helios thực hiện công tác do thám qua **Sensors** và **Matrix**.

## Các Đơn vị Do Thám của Helios
- **Biến động (ATR/NATR):** Do thám độ nguy hiểm của chiến trường.
- **Dải Băng (BB/Keltner):** Radar quét vùng nén lò xo.
- **Khối lượng (Volume Force):** Cảm biến tiếng ồn, phát hiện quân địch đang di chuyển ồ ạt hay đi lẻ tẻ.

## Thiếu sót Hiện tại (Fog of War chưa được mở)
Khả năng do thám của Helios vẫn còn ở cấp độ "Macro" (Nhìn xu hướng lớn). Đối với đòn bẩy 100x, bạn cần do thám ở cấp độ "Micro" (Từng con lính):
1. **Footprint/Order Flow:** Ai đang đánh chủ động (Market Order) vào các mức giá giới hạn (Limit Order)?
2. **Liquidity Heatmap:** Lệnh cắt lỗ của đám đông đang tập trung ở đâu? (Market Makers thường đẩy giá về đó để lấy thanh khoản).

**Kết luận:** Helios cần được trang bị thêm bộ Sensor "X-Quang" như Order Book Imbalance hoặc Delta Volume để nhìn xuyên sương mù ở khung thời gian nhỏ (Tick/1-Second).
