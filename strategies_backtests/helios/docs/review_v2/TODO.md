# 🛠️ HELIOS V2 — DANH SÁCH TÁC VỤ CHI TIẾT (MASTER TODO LIST)

> File này ánh xạ toàn bộ 9 báo cáo Review V2 và các quyết định thiết kế (Grill-Me) thành các task code cụ thể, phân nhóm theo từng file source code để đảm bảo không bỏ sót bất kỳ logic nào.

## 📁 1. `src/strategy/config.rs` (Cập nhật DNA & Tham số)
- [ ] **[FIX - P0]** Cập nhật `adx_moderate` DNA xuống dải `[0.3, 0.7]` (percentile) để khớp với đầu vào chuẩn hóa (Báo cáo 03).
- [ ] **[FIX - P0]** Cập nhật `chaos_z_threshold` DNA thành dải `[0.1, 0.9]` (percentile) (Báo cáo 02).
- [ ] **[FIX - P2]** Giảm dải max của `be_profit_target_rr` xuống `[0.5, 2.5]` để Breakeven khả thi (Quyết định Grill-Me 5A).
- [ ] **[ADD - P2]** Thêm `absorption_sl_factor` vào `RegimeAbsorptionParams` để bỏ magic number 0.7 (Báo cáo 05).
- [ ] **[ADD - P2]** Thêm tham số `time_decay_bars` (vd: `[100, 300]`) vào `TacticalRiskParams` phục vụ Time-based Exit mềm (Quyết định Grill-Me 1B).

## 📁 2. `src/strategy/matrix.rs` (Tầng Nhận thức)
- [ ] **[FIX - P0]** Đưa `trend_strength` từ `ADX/100` (tuyến tính) về `rolling_percentile` (Giải quyết Root Cause - Báo cáo 03, 07).
- [ ] **[FIX - P0]** Đưa `chaos` từ `NATR/100` về `rolling_percentile` (Báo cáo 02).
- [ ] **[CLEAN - P3]** Xóa `trend_angle` thô khỏi struct `MarketMatrix` vì không được sử dụng ở đâu (Báo cáo 02).

## 📁 3. `src/strategy/logic/regime.rs` (Tầng Phân rã Thế trận)
- [ ] **[FIX - P1]** Viết lại công thức `scores.ranging`. Bỏ logic `1.0 - max(all)` khiến Ranging luôn chết. Tính điểm độc lập dựa trên việc giá đi ngang (trend yếu) và biên độ dao động (Báo cáo 03, 07).
- [ ] **[FIX - P1]** Gỡ bỏ cổng lọc IF quá khắt khe `bb_width_norm < range_width_max` của Absorption, đưa nó vào công thức tính score (Báo cáo 03, 07).

## 📁 4. `src/strategy/logic/playbook.rs` (Tầng Trọng tài Chiến thuật)
- [ ] **[FIX - P1]** Nới lỏng điều kiện `RSI bias` cho `AbsorptionScout` để tránh trường hợp Regime thắng nhưng Playbook lại cấm (Báo cáo 04).
- [ ] **[VERIFY - P1]** Duy trì logic Parabolic Wait (không vào lệnh TrendPullback khi giá quá dốc) theo triết lý "Tránh dao rơi" (Quyết định Grill-Me 4C).
- [ ] **[VERIFY - P3]** Duy trì "Single-position lock" (không cho nhồi lệnh) để đảm bảo an toàn tuyệt đối (Quyết định Grill-Me 2A).

## 📁 5. `src/strategy/logic/tactical.rs` (Tầng Bóp cò)
- [ ] **[FIX - P1]** Squeeze Trigger: Thay thế `macd_hist > -0.01` (magic number khác biệt theo coin) bằng logic `macd_hist[i] > macd_hist[i-1]` để kiểm tra xu hướng động lượng (Báo cáo 05).
- [ ] **[FIX - P2]** Absorption Trigger: Thay `0.7` cứng bằng tham số cấu hình DNA `absorption_sl_factor` (Báo cáo 05).
- [ ] **[OPTIMIZE - P3]** RangeOscillator: Giảm bớt số lượng điều kiện AND (hiện có 5) xuống mức hợp lý để dễ kích hoạt hơn (Báo cáo 05).
- [ ] **[OPTIMIZE - P3]** AnomalyFade: Bỏ bớt điều kiện nến rút chân (râu nến) nhiễu, giảm số lượng AND để tăng cơ hội vào lệnh (Báo cáo 05).

## 📁 6. `src/strategy/mod.rs` (Tầng Điểu phối & Bộ nhớ)
- [ ] **[FIX - P2]** Làm mượt ATR: Thay `volatility_benchmark = current_atr` bằng Exponential Moving Average (EMA) để Trailing Stop không bị giật (Báo cáo 06).
- [ ] **[FIX - P2]** Dời `peak_price` tracking từ `High/Low` sang giá `Close` để tránh bị cắt lỗ oan bởi râu nến quét (Quyết định Grill-Me 3A).
- [ ] **[ADD - P2]** Lưu vết thời gian: Theo dõi số nến đã mở của lệnh hiện tại vào `TacticalMemory` (cho Time-based Exit).

## 📁 7. `src/strategy/logic/risk.rs` (Tầng Quản trị Rủi ro)
- [ ] **[FEAT - P2]** Thêm Time-based Exit Mềm (Time Decay SL). Nếu lệnh mở vượt quá `time_decay_bars`, tự động thu hẹp `tsl_atr_multiplier` (siết Trailing Stop liên tục) để ép giá cắn SL trong hòa vốn hoặc dời dần về Entry (Quyết định Grill-Me 1B).
