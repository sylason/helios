# Helios V3 Core Review: Arbiter & Decision Making

## Review: `arbiter.rs` (Bộ điều phối đa luồng)

### Quan sát & Phân tích
1. **Lỗi map Multiplier của MOMENTUM_IGN:**
   Trong `arbiter.rs` dòng 77, Scanner `MOMENTUM_IGN` được gán với `matrix.squeeze_multiplier[i]`.
   - **Vấn đề:** `squeeze_multiplier` (được tính từ `squeeze_intensity`) đạt giá trị cao (gần 1.0) khi thị trường ĐANG NÉN CHẶT (biến động cực thấp, BB chui vào trong KC). Chiến thuật Momentum Ignition (bắt cú nổ bứt phá) lại được chấm điểm cao nhất khi thị trường... đi ngang nén chặt và chưa có biến động? Điều này là sai về mặt logic. Khi có một cây nến Breakout xảy ra (đặc trưng của Momentum Ignition), dải BB sẽ bung rộng ra (Squeeze Release), lúc này `squeeze_intensity` sẽ rớt thẳng về 0. Do đó, điểm số của Momentum Ignition sẽ bị nhân với 0!
   - **Đề xuất:** Cần map `MOMENTUM_IGN` với một multiplier đo lường sự MỞ RỘNG biến động, ví dụ như `chaos` hoặc tạo một `momentum_multiplier` riêng.

2. **Cách Arbiter tính điểm (Hard Approval Threshold):**
   - Arbiter sử dụng công thức: `confidence = proposal.raw_score * multiplier;` sau đó so sánh `confidence > self.min_approval_confidence`.
   - Vì `min_approval_confidence` là một tham số cố định (trong Rank 1 là 0.4309), điều này gây ra sự bất công cho các chiến thuật có Multiplier yếu (như Reversal).
   - **Đề xuất:** Giữ nguyên cấu trúc này nhưng cải thiện cách tính toán các Multiplier trong `matrix.rs` để chúng không bị triệt tiêu lẫn nhau (ví dụ: dùng phép cộng trọng số thay vì phép nhân toàn bộ).

3. **Cơ chế Suggested SL bị bỏ ngỏ:**
   - Arbiter cho phép các scanner trả về `suggested_sl` thông qua `Proposal`. Nếu `final_sl` có giá trị, hệ thống sẽ bỏ qua `get_initial_sl` (vốn tính bằng `tsl_atr_normal`).
   - Ta cần kiểm tra xem các Scanner trong `scanners/` có thực sự gửi `suggested_sl` hay không, nếu không, toàn bộ chiến lược chỉ đang phụ thuộc vào một con số SL mù (Blind ATR SL) thay vì SL dựa trên cấu trúc nến/hỗ trợ kháng cự cục bộ.
