# Kế hoạch Giải quyết Khủng hoảng (Stability & Matrix Refactor)

Vấn đề sụt giảm lợi nhuận từ 360% xuống 250% sau Phase 2 là một hệ quả phụ (side-effect) rất đáng giá để ta học được một bài học sâu sắc về logic lượng tử trong Trading. 

## Nguyên nhân Cốt lõi của sự sụt giảm
Sau khi phân tích, tôi đã phát hiện ra lỗ hổng chí mạng khi ta đổi từ **Phép Nhân (Multiplicative)** sang **Cộng Trọng Số (Weighted Sum)** trong `matrix.rs`.

**Ví dụ với REVERSAL:**
- Công thức cũ (Phép Nhân): `(1.0 - Trend) * Momentum * Z_Score` -> Hoạt động như cổng logic **AND**. Chỉ cần Trend mạnh (nghĩa là `1.0 - Trend = 0`), toàn bộ điểm số lập tức bằng `0.0`. Nó **phủ quyết (Veto)** lệnh giao dịch ngược trend ngay lập tức.
- Công thức mới (Cộng Trọng Số): `(1.0 - Trend)*0.3 + Momentum*0.35 + Z_Score*0.35`. Nếu Trend rất mạnh (`1.0 - Trend = 0`), nhưng Momentum và Z-Score lại rất cao (0.9), điểm số vẫn có thể đạt mức `0 + 0.315 + 0.315 = 0.63`. Lệnh bỗng nhiên **được duyệt** dù vi phạm điều kiện kiên quyết là "Không được ngược Trend".

> [!WARNING]
> Việc dùng Cộng Trọng Số vô tình mở toang cánh cửa cho hàng loạt tín hiệu rác đi lọt, dẫn đến số lượng lệnh lỗ tăng lên và Win Rate giảm xuống.

## Đề xuất Khắc phục (Hybrid Logic & Risk Manager)

### 1. Hybrid Multiplier Logic (Trong `matrix.rs`)
Ta sẽ kết hợp cả sức mạnh của Phép Nhân (để Phủ quyết) và Phép Cộng (để chấm điểm thành phần).
- **Quy tắc:** Bất cứ điều kiện nào mang tính **Bắt buộc (Veto)** thì phải dùng Phép Nhân. Các điều kiện mang tính **Bổ trợ (Contributing)** thì dùng Phép Cộng.
- **Ví dụ Reversal Mới:** `reversal_multiplier = (1.0 - t_mul) * ((momentum_extremity * 0.5) + (z_abs * 0.5))`
Như vậy, nếu có Trend, hệ số tự động về 0. Nếu không có Trend, hai điều kiện kia sẽ không tự bóp nghẹt nhau quá đáng.

### 2. Time-Based Hard Exit (Trong `risk.rs`)
Hiện tại lệnh ngâm lỗ (Stale) chỉ bị bóp SL (Tighten), dẫn đến việc gồng lỗ vẫn kéo dài thê thảm nếu giá cứ đi ngang cưa chân bàn.
- **Giải pháp:** Nếu lệnh vượt quá `time_decay_bars` mà PnL vẫn `< 0`, ta không cần chờ cắn SL nữa, trực tiếp ra lệnh **Đóng Vị Thế Ngay Lập Tức (Market Close)** để giải phóng biên độ.

## Cần User Phê duyệt
1. Bạn có đồng ý với logic "Hybrid Multiplier" để siết chặt lại bộ lọc tín hiệu không?
2. Việc Force Close lệnh sau khi hết thời gian `time_decay_bars` thay vì dời SL có phù hợp với triết lý của bạn không?
