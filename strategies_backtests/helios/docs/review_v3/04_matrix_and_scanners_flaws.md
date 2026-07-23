# Helios V3 Core Review: Market Matrix & Scanners

## Review: `matrix.rs` & `scanners/*.rs` (Lỗ hổng chấm điểm tín hiệu)

### Quan sát & Phân tích
1. **Thiên vị Tích lũy (Multiplicative Bias) trong `matrix.rs`:**
   Các Regime Multipliers được thiết kế rất thông minh để định tuyến trạng thái thị trường, tuy nhiên công thức tính đang dùng phép NHÂN (Multiplication) cho quá nhiều biến số có giá trị < 1.0.
   Ví dụ `reversal_multiplier`:
   ```rust
   reversal_multiplier.push(((1.0 - t_mul) * momentum_extremity * z_abs).max(0.0).min(1.0));
   ```
   Nếu Trend đang đi ngang hoàn toàn (`t_mul` = 0.2 => `1.0 - t_mul` = 0.8), Momentum rất cao (`momentum_extremity` = 0.8), Giá giãn rất mạnh (`z_abs` = 0.8). Cả 3 điều kiện đều cực kỳ ủng hộ đảo chiều. 
   Nhưng điểm số thực tế: `0.8 * 0.8 * 0.8 = 0.512`. 
   Trong khi đó `squeeze_multiplier` chỉ cần một biến duy nhất: `squeeze_intensity` (có thể dễ dàng đạt 0.8 - 0.9).
   - **Hậu quả:** Các chiến lược càng tinh vi, càng đòi hỏi nhiều điều kiện hội tụ thì điểm số cuối cùng càng bị bóp nghẹt. Kết hợp với `min_approval_confidence` = 0.43, `REVERSAL` gần như không bao giờ có thể lọt qua vòng kiểm duyệt của Arbiter hoặc luôn thua `ABSORPTION`.
   - **Đề xuất:** Cần chuyển đổi sang hệ thống Trọng số Cộng (Weighted Sum). Ví dụ: 
     `reversal_score = (w1 * (1.0 - t_mul) + w2 * momentum_extremity + w3 * z_abs) / (w1 + w2 + w3)`

2. **Scanner `momentum_ignition.rs`:**
   Scanner này phụ thuộc vào `vol_raw > vol_thresh`.
   Tuy nhiên, `vol_thresh` lại lấy từ biến `config.tactical_sniper.squeeze.break_confirm_mult`. Việc vay mượn tham số của Squeeze cho Momentum_Ignition là chưa tối ưu, vì nó trói buộc 2 chiến lược vào cùng một tham số cấu hình.

3. **Scanner `range_osc.rs`:**
   Scanner này là minh chứng tốt cho việc sử dụng `suggested_tp`. Nó truyền `suggested_tp = bb_u` hoặc `bb_l` và hệ thống đã map thành công vào giá trị `tp_price` của `ferro_trade`. Nhờ đó Range_Osc ăn được những con sóng nhỏ trong vùng đi ngang.

4. **Sức mạnh của `ANOMALY`:**
   Mã nguồn `anomaly.rs` đo lường `upper_rejection` (Râu nến) so với thân nến. Cơ chế bắt bóng râu nến (Wick rejection) này kết hợp với Z-Score > Threshold đã hoạt động cực kỳ hoàn hảo. Nó chứng minh rằng thị trường thuật toán rất chuộng các điểm vào lệnh tại những vùng "quét thanh lý" (Liquidation Wicks).
