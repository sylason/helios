# 🛠️ HELIOS V2 — KẾ HOẠCH SỬA LỖI & NÂNG CẤP (REVIEW V2 TODO)

> Tệp này dùng để theo dõi tiến độ sửa các lỗi logic, nghẽn cổ chai và xung đột liên tầng được phát hiện trong đợt Review V2. Các nhiệm vụ được sắp xếp theo mức độ nghiêm trọng.

## 🔴 P0: Các lỗi gốc rễ (Root Causes) — Ưu tiên cao nhất
Gây hiệu ứng "phễu cạn", bão hòa điểm số, làm tê liệt 5/7 Playbook.

- [ ] **Matrix:** Chuyển `trend_strength` (ADX) từ tuyến tính (`/100`) sang `rolling_percentile` để cân bằng hệ quy chiếu với các chỉ số khác.
- [ ] **Matrix:** Chuyển `chaos` (NATR) từ tuyến tính (`/100`) sang `rolling_percentile`.
- [ ] **Config:** Cập nhật dải DNA `adx_moderate` và `chaos_z_threshold` để tương thích với giá trị Percentile thay vì tuyến tính.

## 🟠 P1: Xung Đột & Tắc Nghẽn Chiến Thuật (High Impact)
Gây triệt tiêu tín hiệu giữa các tầng, khiến lệnh không thể kích hoạt.

- [ ] **Regime:** Sửa cơ chế tính điểm Ranging `1.0 - max(...)`. Hiện tại Trend luôn cao khiến Ranging luôn ~0. Cần công bằng hơn.
- [ ] **Playbook/Regime:** Giải quyết "Parabolic Paradox". Hiện tại Parabolic cấm đánh TrendPullback, nhưng bản thân Parabolic không bao giờ thắng được Trend để ra lệnh ClimaxReversal. Cần mở khóa cho Parabolic hoạt động khi độ dốc quá cao.
- [ ] **Tactical (Squeeze):** Xóa điều kiện raw hardcoded `macd_hist > -0.01` và thay bằng kiểm tra cấu trúc động lượng hợp lý (vì raw MACD khác nhau tùy giá coin).

## 🟡 P2: Quản Trị Rủi Ro & Lệnh (Risk Management)
Gây nhiễu Trailing Stop, hoặc kẹt vốn quá lâu.

- [ ] **Mod/Risk:** Sửa `smooth_atr` (hiện đang lấy ATR tức thời nến hiện tại). Cần áp dụng EMA/SMA nhẹ để làm mượt, tránh Trailing Stop bị giật theo nến 1-phút.
- [ ] **Mod/Risk:** Đổi logic theo dõi `peak_price` từ High/Low sang Close để bảo vệ Trailing Stop khỏi hiện tượng quét râu nến (whipsaw wicks).
- [ ] **Config/Risk:** Giảm ngưỡng `be_profit_target_rr` (vốn đang học ra mức 6.6R) xuống mức khả thi hơn để Breakeven thực sự hoạt động.
- [ ] **Tactical (Absorption):** Xóa hệ số Stoploss hardcoded `0.7` và thay bằng biến lấy từ cấu hình DNA (`range_sl_factor` hoặc tạo biến mới).
- [ ] **Risk:** Bổ sung cơ chế Timeout (Time-based Exit). Các lệnh ngâm quá lâu (>100-200 nến) mà không lãi/lỗ sẽ bị đóng để giải phóng slot (tránh Zombie Trade).

## 🟢 P3: Tối Ưu Hóa & Dọn Dẹp (Cleanup & Tweak)
- [ ] **Matrix:** Xóa `trend_angle` (đơn vị độ thô) khỏi `MarketMatrix` vì không còn được sử dụng ở bất kỳ đâu (tiết kiệm bộ nhớ).
- [ ] **Tactical (Anomaly/Range):** Đơn giản hóa các điều kiện bóp cò (bớt số lượng AND lại). Anomaly cần tới 4 điều kiện khắt khe khiến nó gần như tuyệt chủng.
- [ ] **Tactical:** Suy nghĩ về cơ chế bỏ "Single Position Lock" (`!tick.position().is_open()`). Nếu xu hướng thuận lợi, có nên cho phép nhồi lệnh khác Tactical thay vì bị khóa cứng chỉ 1 lệnh hay không?
