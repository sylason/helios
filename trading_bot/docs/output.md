. Kế Hoạch Triển Khai Thực Thi (Baby Steps Roadmap)
 Bước 1 (Clean Engine):
Tinh giản struct EngineCooldownGuard: Bỏ last_losing_price, last_losing_side, locked_side. Chỉ giữ: locked_until_bar, consecutive_losses.
Viết lại hàm should_veto(current_bar_idx) -> Option<usize>: Chỉ kiểm tra current_bar_idx < locked_until_bar. Nếu còn bị khóa, trả về số nến còn lại.
Xóa bỏ hoàn toàn hàm calculate_rolling_atr 1m thừa thãi.
 Bước 2 (Clean Action Execution):
Cập nhật dòng 663 tại asset_actor.rs: Nếu should_veto trả về Some(remaining) 
→
→ Log rõ ràng: 🛡️ [CIRCUIT BREAKER ACTIVE] Cooldown còn X nến. Bỏ qua Action.
Giữ nguyên chốt chặn closed_intra_candle (No Same-Bar Re-entry).
 Bước 3 (Thống nhất Đóng Lệnh):
Đảm bảo cả 2 nhánh đóng lệnh (Intra-candle SL/TP và ClosePosition) đều gọi chung 1 logic kích hoạt Cầu Dao khi PnL 
≤
0
≤0.
 Bước 4 (Clean Strategy):
Vô hiệu hóa hoặc dọn dẹp logic đoán PnL mò trong HeliosStrategyV3 để Strategy chuyên tâm 100% vào việc sinh tín hiệu Alpha.
 Bước 5 (Unit Tests & Verification):
Chạy toàn bộ test cases của bot_engine và kiểm chứng trên dữ liệu Walk-Forward để xác nhận không còn bất kỳ hiện tượng Deadlock nào.