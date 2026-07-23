# Helios V3 Refactoring Master Plan

Đây là lộ trình nâng cấp toàn diện hệ thống Helios V3 dựa trên các lỗi logic và điểm nghẽn đã được phân tích.

## Phase 1: Thanh lọc Risk Management & Dọn dẹp Code
- `[x]` **Xóa/Comment Dead Code trong DNA:** Loại bỏ/Comment 7 thông số SL Factor vô dụng (`squeeze_sl_factor`, `trend_sl_factor`...) và các thông số Break-Even (`use_breakeven`, `be_profit_target_rr`) khỏi `config.rs`.
- `[x]` **Giải thích cấu hình:** Thêm comment chi tiết giải thích ý nghĩa cho tất cả các config phức tạp trong `config.rs` (ngoại trừ các biến period/timeframe cơ bản).
- `[x]` **Gắn SL linh động từ Scanner:** Sửa `mod.rs` để ưu tiên sử dụng `suggested_sl` từ Proposal, nếu không có mới dùng `tsl_atr_normal`. (Đã kiểm tra logic mod.rs: Nó đã dùng `decision.final_sl.unwrap_or_else`).

## Phase 2: Cải tổ Market Matrix (Giải phóng Reversal)
- `[x]` **Đổi Multiplier thành Trọng số Cộng (Weighted Sum):** Sửa lại logic trong `matrix.rs`. Thay vì lấy các biến nhỏ hơn 1 nhân với nhau (làm bóp nghẹt điểm số), hãy sử dụng công thức cộng trọng số, ví dụ: `reversal_multiplier = (w1*(1-t_mul) + w2*momentum + w3*z_abs) / (w1+w2+w3)`.
- `[x]` **Tách biệt Momentum Ignition:** Không sử dụng `squeeze_intensity` làm multiplier cho `MOMENTUM_IGN` nữa, vì nó ngược logic. Thay vào đó, map nó với `chaos` hoặc tạo một `momentum_multiplier` riêng đại diện cho sự Bung nén (Volatility Expansion).

## Phase 3: Hệ thống Dynamic Leverage (Chuẩn bị cho NUKE Playbook)
- `[ ]` **Dynamic Leverage trong Proposal:** Bổ sung `requested_leverage: Option<f64>` vào struct `Proposal` trong `arbiter.rs`.
- `[ ]` **Điều khiển Đòn bẩy từ Scanner:** Các Scanner bình thường sẽ truyền `requested_leverage = Some(20.0)`. Riêng NUKE Scanner trong tương lai sẽ truyền `Some(100.0)` hoặc `Some(125.0)`.
- `[ ]` **Áp dụng Đòn bẩy vào Position:** Sửa hàm `calculate_size` trong `risk.rs` và `mod.rs` để đọc `requested_leverage` từ Arbiter Decision thay vì dùng số cứng `20.0`.

## Phase 4: Kích hoạt & Tối ưu hóa Genetic Algorithm
- `[ ]` Xóa bỏ CSDL `helios_prospector_v3.db` cũ.
- `[ ]` Chạy lại `cargo run --release --example v3_run_helios_optimize`.
- `[ ]` Phân tích kết quả Rank 1 mới để xem sự phân bổ lợi nhuận, đặc biệt kiểm tra xem REVERSAL đã hồi sinh thành công hay chưa.

## Phase 5: Khắc phục Game Balancing & Đại tu Toán học (Additive + Hard Veto + Progressive Decay)
- `[x]` **Đại tu Nhận diện Thị trường (`matrix.rs`):** Chuyển toàn bộ các Regime Multipliers từ Phép Nhân sang Additive Logic + Veto Cứng nội bộ. Đảm bảo Momentum không bị chết oan khi Volume nổ mà Chaos chưa theo kịp.
- `[x]` **Triển khai Hard Veto & Additive Scoring trong `arbiter.rs`:** Sửa lại công thức `confidence = raw_score * multiplier`. Thay vào đó, áp dụng luật: Nếu `multiplier < 0.2` (Hard Veto), loại bỏ lệnh (`confidence = 0.0`). Nếu qua được Veto, dùng phép cộng trọng số `confidence = (raw_score * 0.6) + (multiplier * 0.4)` để các Scanner không bị triệt tiêu điểm lẫn nhau.
- `[x]` **Thêm Gene Config cho Progressive Time-Decay:** Bổ sung các biến `time_countdown_bars` (100-5000), `tighten_step_multiplier` vào file `config.rs` để Optimizer tự học thông số gồng lệnh / cắt máu tối ưu.
- `[x]` **Triển khai Progressive High-Watermark Decay trong `risk.rs`:**
  - Theo dõi số nến kể từ lần cuối cùng PnL tạo mức đỉnh mới cao nhất.
  - Tích hợp Squeeze Pause: Nếu thị trường đang nén (`is_squeeze == true`), **Dừng đếm ngược** (Tránh Whipsaw).
  - Cứ mỗi lần chạm mốc `time_countdown_bars` mà không phá đỉnh, kéo Trailing SL chặt dần (nhân SL ATR với `tighten_step_multiplier`), lặp lại cho đến khi lệnh chốt lời bằng Market Close.
