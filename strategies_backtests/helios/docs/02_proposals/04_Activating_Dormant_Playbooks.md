# Đề xuất 04: Đánh thức các Đội SWAT ngủ đông

Dựa trên thắc mắc của bạn ở Q2: Tại sao Optimizer (chạy file `run_helios_optimize.rs`) chỉ báo về các lệnh Squeeze, Absorption, Trend Pullback mà lại KHÔNG HỀ thấy Parabolic, Momentum, Anomaly? 
Có phải do Optimizer yếu?

**KHÔNG, ĐÓ LÀ DO BUG TRONG MÃ NGUỒN.**

Quá trình tôi đọc mã nguồn đã chỉ ra 2 lỗi nghiêm trọng khiến các đội SWAT này vĩnh viễn bị giam cầm:

## 1. Mù Parabolic (Lỗi sai đơn vị)
Tại file `regime.rs`, điểm số của Parabolic được tính bằng phép chia:
`trend_angle.abs() / angle_threshold`

Tuy nhiên, `trend_angle` hiện tại đang là một con số **Percentile từ 0.0 đến 1.0**.
Trong khi đó, `angle_threshold` từ Optimizer đưa vào là độ thật (từ 60 đến 85 độ).
Khi lấy số cực nhỏ chia cho 85, điểm số luôn luôn tiến về 0.
-> **Thế trận Parabolic không bao giờ được nhận diện.**

## 2. Bỏ quên Momentum (Lỗi Dead Code)
Tại file `playbook.rs`, khối lệnh điều hướng (match block) chỉ quy định:
- Nếu Squeeze -> gọi `SqueezeAggressive`
- Nếu Trend -> gọi `TrendPullback`
- ...

Nhưng **không có bất kỳ nhánh nào** gọi lệnh `TacticalPlaybook::MomentumIgnition`. Toàn bộ 50 dòng code để đánh Momentum trong file `tactical.rs` bị bỏ xó hoàn toàn, bot không biết cách nào để dùng nó.

## Đề xuất sửa đổi
1. **Sửa `regime.rs`:** Cần truyền vào tham số `trend_angle_raw` thay vì `trend_angle_adj` để tính điểm Parabolic.
2. **Sửa `playbook.rs`:** Sửa lại logic ánh xạ để chia phần cho Momentum Ignition. Ví dụ: Nếu Trend rất mạnh và gia tốc cao, chuyển từ Trend Pullback sang Momentum Ignition.
3. Khi bạn sửa 2 lỗi này, bạn sẽ thấy Optimizer nhả ra các lệnh đa dạng hơn, bắt mọi trạng thái của thị trường như bạn mong muốn.
