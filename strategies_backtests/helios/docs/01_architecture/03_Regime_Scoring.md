# 03. Chấm điểm Thế trận (Regime Scoring)

Tài liệu phản ánh file `src/strategy/logic/regime.rs`.

## Chức năng
Tính toán 6 điểm số cho 6 loại thế trận. Không có thế trận nào đúng tuyệt đối, tất cả là xác suất.

## Công thức Chấm điểm (Tất cả được nén bởi Tanh để ra điểm [0, 1])
1. **Anomaly Score (Bất thường):** `tanh(Z_Score * 0.8 + Chaos * 0.2)`
2. **Trend Score (Xu hướng):** Trọng số kết hợp giữa Góc dốc (Angle) và Sức mạnh (ADX). Có bộ nhớ đệm (Hysteresis): Nếu nến trước đang là Trend, nến này sẽ dễ đạt điểm Trend hơn.
3. **Squeeze Score (Nén lò xo):** `tanh(Cường độ Squeeze / Ngưỡng)`
4. **Absorption Score (Gom hàng):** Chỉ chấm điểm nếu Dải BB đang hẹp. Bằng `tanh(Volume_Force / Ngưỡng)`.
5. **Parabolic Score (Hưng phấn):** `tanh(|Trend_Angle| / Ngưỡng)`.
6. **Ranging Score (Đi ngang):** Điểm mặc định nếu 5 thế trận trên đều yếu. Bằng `1.0 - MAX(5 điểm trên)`.

> [!CAUTION] BUG LOGIC PHÁ HỎNG NHẬN DIỆN PARABOLIC
> Trong `regime.rs` (dòng 83), điểm Parabolic dùng `trend_angle.abs() / angle_threshold`. 
> Tuy nhiên, biến `trend_angle` được truyền vào từ hàm cha đang là giá trị Percentile `[0, 1]`, trong khi `angle_threshold` ở Config lại là độ (Degrees: 60-85). 
> **Kết quả:** Phép chia `(0..1) / 85` ra số cực nhỏ, `tanh()` luôn trả về 0. Thế trận Parabolic hiện tại trong Code **BỊ MÙ**, điểm số luôn bằng 0.

## Xác định Thế trận Chủ đạo (Primary Regime)
Code sử dụng khối If liên hoàn với ngưỡng khởi điểm là 0.1 để chọn ra 1 nhãn duy nhất lưu vào `TacticalMemory`. Điều này giúp hệ thống Trailing Stoploss biết nó đang bảo vệ lệnh trong bối cảnh nào.
