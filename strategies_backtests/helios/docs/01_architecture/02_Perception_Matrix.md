# 02. Ma trận Nhận thức (Perception Matrix)

Tài liệu phản ánh file `src/strategy/matrix.rs`.

## Chức năng
Đây là module cốt lõi tạo nên "Sự thích nghi của Mahoraga". Nó không nhìn vào giá trị tĩnh (VD: RSI = 70), mà chuẩn hóa mọi thứ về thang điểm tương đối để biết hiện tại so với quá khứ đang như thế nào.

## Các Công cụ Chuẩn hóa (`utils/normalization.rs`)
1. **Rolling Percentile (Bách phân vị trượt):** 
   - So sánh giá trị hiện tại với một cửa sổ quá khứ (ví dụ 100 nến trước). 
   - Nếu Percentile = 1.0, nghĩa là cao nhất trong 100 nến qua. Nếu 0.0 là thấp nhất.
   - *Ưu điểm:* Giải quyết triệt để vấn đề dữ liệu Đa khung bị điền dồn (Forward Fill). Khi gặp 5 cây nến có cùng một giá trị H1, hàm này trả về 0.5 (Neutral) thay vì gây sai lệch.
2. **Tanh Scale:** Nén một dải số bất kỳ về khoảng `[-1, 1]` một cách mượt mà.
3. **Z-Score Trượt:** Đo độ bất thường. Nếu Z-Score > 3, biến cố đó chỉ xảy ra < 0.3% thời gian.

## 13 Khía cạnh của Ma trận (Matrix Dimensions)
1. `trend_strength` (ADX chia 100)
2. `trend_dir` (Độ dốc điều chỉnh Volatility -> Percentile)
3. `trend_angle` (Góc thật) và `trend_angle_adj` (Percentile của góc)
4. `squeeze_intensity` (Độ co hẹp BB/KC -> Percentile)
5. `bb_width_norm` (Tốc độ nở của BB)
6. `momentum` (RSI -> Percentile)
7. `liquidity` (MFI -> Percentile)
8. `chaos` (NATR) và `chaos_ratio` (Tỉ lệ bùng nổ Volatility -> Percentile)
9. `entropy` (Kaufman Efficiency Ratio -> Percentile)
10. `volume_force` (Volume/Vol_MA -> Percentile)
11. `price_z_score` (Z-Score của giá -> Tanh)

> [!TIP]
> Việc dùng Rolling Percentile tạo ra một bề mặt tham số (Gradient) cực kỳ mượt mà. Nhờ đó, thuật toán Tiến hóa (Genetic Algorithm) của Optimizer dễ dàng tìm ra thông số tối ưu hơn rất nhiều so với dùng ngưỡng Hardcode (If-else cứng).
