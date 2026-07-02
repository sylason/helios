# 04. Hội đồng Điều phối (Playbook Arbitrator)

Tài liệu phản ánh file `src/strategy/logic/playbook.rs` và `sensor.rs`.

## 1. Cầu chì Môi trường (Environment Fuse)
Trước khi Playbook được chọn, `sensor.rs` đánh giá mức độ ổn định của thị trường (`stability_index`).
- Nếu `Volume Force > 5.0` (Đột biến thanh khoản bất thường), `stability_index` = 0 ngay lập tức.
- Nếu không, chỉ số này được tính bằng Trung bình của (Độ nhiễu + Thanh khoản + Tuyến tính).
- Có một bộ đếm kiên nhẫn (Patience counter): Tăng 1 nếu thị trường ổn, trừ 3 nếu thị trường hỗn loạn -> Bảo thủ, chậm mở lệnh nhưng tắt lệnh rất nhanh.

## 2. Phân xử Xếp hạng (Arbitration)
1. Gộp 6 điểm số Regime vào một mảng và xếp hạng từ cao xuống thấp.
2. Lấy Kẻ đứng đầu (Thế trận có điểm cao nhất).
3. **Damping Score:** `Final_Score = Raw_Score * Stability_Index`. (Nếu thị trường quá nhiễu, điểm cuối cùng sẽ bị ép xuống thấp).
4. Nếu `Final_Score < min_score_threshold` -> Hệ thống chọn `Wait` (Đứng ngoài).

## 3. Ánh xạ Đội Đặc nhiệm
- Squeeze -> `SqueezeAggressive` hoặc `SqueezeConfirmed` (tùy thuộc Momentum có mạnh không).
- Trending -> Nếu Neutral hoặc Parabolic -> Wait. Ngược lại -> `TrendPullback`.
- Ranging -> `RangeOscillator`.
- Absorption -> Cần có độ lệch dòng tiền rõ ràng -> `AbsorptionScout`.
- Anomaly -> `AnomalyFade`.
- Parabolic -> `ClimaxReversal`.

> [!WARNING] DEAD CODE: CHIẾN THUẬT MOMENTUM BỊ BỎ RƠI
> Không có bất kỳ Regime nào trong khối lệnh ánh xạ (Match block) chuyển hướng sang `TacticalPlaybook::MomentumIgnition`. Toàn bộ 50 dòng code đánh Momentum trong file Tactical sẽ **không bao giờ được thực thi**.
