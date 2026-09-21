# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ: MOMENTUM IGNITION SCANNER & HỆ THỐNG LIÊN ĐỚI (TODO.MD)

Tài liệu này tổng hợp toàn bộ các đầu việc kỹ thuật cần triển khai sửa lỗi và nâng cấp cho **`momentum_ignition.rs`** cùng các thành phần liên đới trong hệ thống Helios V3.3.

---

## 🚨 NHÓM P0: CẤP THIẾT - SỬA LỖI HỆ THỐNG CHẾT NGƯỜI (SYSTEM-BREAKING BUGS)

Các lỗi này trực tiếp dẫn đến việc tài khoản bị cướp quyền kiểm soát 100% và thua lỗ liên hoàn trong Paper Trading.

- [ ] **TODO-P0-01: Sửa lỗi cướp thầu ngược (Adverse Bid-Sniping Bug) trong `arbiter.rs` [CHUYỂN SANG 04_ARBITER]**
  - **Trạng thái**: 🔄 **CHUYỂN SANG MÔ-ĐUN `04_arbiter`** (Theo ranh giới kiến trúc: Scanner chỉ phát đề xuất, Trọng tài Arbiter độc quyền quyết định quy tắc đấu thầu và giải quyết xung đột tín hiệu).
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:175-182`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L175-L182)
  - **Hiện trạng**: Dòng 178 `if dist_new < dist_curr` tự động chọn đề xuất có khoảng cách Stop Loss gần nhất thay vì điểm tự tin cao nhất. Do Momentum Ignition đặt SL siêu sát, nó luôn cướp sạch mọi lệnh của TrendPB và Absorption.
  - **Hành động**: Đảo ngược thứ tự ưu tiên:
    1. So sánh `conf - current_conf`: Nếu chênh lệch $> 0.05$, chọn bên có Confidence cao hơn.
    2. Chỉ khi Confidence tương đương (chênh lệch $\le 0.05$), mới dùng khoảng cách Stop Loss làm tiêu chí phụ (Tie-breaker).
  - **Xác minh**: Viết unit test mô phỏng trường hợp TrendPB ($conf = 0.85, dist = 2.0$) so găng với Momentum ($conf = 0.25, dist = 0.4$), đảm bảo TrendPB chiến thắng.

- [x] **TODO-P0-02: Bổ sung kiểm tra màu nến & Định lượng hóa qua CLV (Close Location Value) [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Đã áp dụng Màu nến (`close > open` / `close < open`) và CLV Guard ($\text{CLV} \ge \text{min\_clv}$ / $\text{CLV} \le -\text{min\_clv}$).
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:60-72`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L60-L72)

- [x] **TODO-P0-03: Sửa dải gen độc hại của `rsi_threshold` trong `config.rs` [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Đã thu hẹp dải gen về $[50.0, 80.0]$, step $1.0$, default $60.0$.
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)

- [x] **TODO-P0-04: Bổ sung Điều Kiện Tiên Quyết Cứng (Hard Breakout Pre-condition) [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Early Exit Guard trả về `None` lập tức nếu $Close \in [BB_l, BB_u]$ trước khi bóc tách chỉ báo; Đo độ vượt dải qua hàm $\tanh(\Delta / ATR)$.
  - **Vị trí**: [`momentum_ignition.rs:44-57`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L44-L57)

- [x] **TODO-P0-05: Bổ sung Chốt Chặn Xu Hướng Vĩ Mô (Macro Counter-Trend Veto) [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Thêm gen DNA `max_adverse_slope` $[0.05, 0.35]$, VETO cấm Long khi $trend\_dir < -max\_adverse$ và cấm Short khi $trend\_dir > max\_adverse$.
  - **Vị trí**: [`momentum_ignition.rs:72-88`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L72-L88)

- [x] **TODO-P0-06: Khử Bỏ Toàn Bộ Các Hằng Số Hardcoded Trong Code [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Khử bỏ toàn bộ Sigmoid, thay bằng $\tanh$. Bổ sung 2 gen DNA `w_breakout` và `w_volume`, chuẩn hóa lồi 3 thành phần. Loại bỏ hoàn toàn sàn SL hardcode `0.002` và `.max(0.05 * atr)`.
  - **Vị trí**: [`momentum_ignition.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs)

---

## ⚡ NHÓM P1: QUAN TRỌNG - VI CẤU TRÚC GIÁ & BẢO VỆ STOP LOSS (MICROSTRUCTURE & RISK GUARDS)

Ngăn chặn hiện tượng dính Hard Stop Loss trong 2-3 bars và bẫy nến kiệt sức (Climax/Liquidity Grab).

- [ ] **TODO-P1-01: Chặn Bẫy Nến Kiệt Sức (Buying/Selling Climax Filter) [TẠM HOÃN / MÂU THUẪN USE CASE]**
  - **Trạng thái**: ⏸️ **TẠM HOÃN XEM XÉT** (Bản chất Momentum bùng nổ đòi hỏi RSI cao; chặn trần RSI > 80 sẽ tự triệt tiêu bản chất đánh Breakout. Việc quản lý rủi ro khi kiệt sức thuộc về `risk.rs`).
  - **Vị trí**: [`momentum_ignition.rs:55, 60`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L55)
  - **Hiện trạng**: Xu hướng đã tăng quá nóng (RSI > 85), nến cuối cùng bung thanh khoản để xả hàng (Buying Climax) nhưng Scanner vẫn nhảy vào Long đu đỉnh.
  - **Hành động**: Đánh giá lại: Có nên chặn trần cứng hay để Risk Manager cắt lỗ nhanh qua Trailing Stop.

- [x] **TODO-P1-02: Tái cấu trúc cơ chế đặt Stop Loss chống Flash Stop-Out 2-3 bars [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Nới dải SL buffer lên $[0.20, 2.00] \times ATR$, loại bỏ sàn gán cứng, bàn giao cho `RiskManager::clamp_sl_price`.
  - **Vị trí**: [`momentum_ignition.rs:133-145`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L133-L145)

- [x] **TODO-P1-03: Sửa dải gen `vol_z_threshold` và `struct_sl_buffer_atr` trong `config.rs` [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. `vol_z_threshold` $[1.0, 3.5]$, `struct_sl_buffer_atr` $[0.20, 2.00]$.
  - **Vị trí**: [`config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)

---

## 🛠️ NHÓM P2: TRUNG HẠN - LỌC NHIỄU BIẾN ĐỘNG & AN TOÀN SỐ HỌC (ROBUSTNESS & FILTERS)

- [ ] **TODO-P2-01: Thêm bộ lọc bung mở dải Bollinger (Bandwidth Expansion Filter) [HỦY BỎ - OVER-ENGINEERED]**
  - **Trạng thái**: ❌ **HỦY BỎ DO OVER-ENGINEERED** (Theo nguyên lý toán học của Bollinger Bands: Khi nến bùng nổ đâm xuyên dải với thân dài và volume lớn, độ lệch chuẩn $\sigma$ bắt buộc phải nở rộng. Kiểm tra thêm là dư thừa 100%).
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:53`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L53)

- [ ] **TODO-P2-02: Khắc phục điểm nền MFI trong `MarketMatrixV3.flow_axis` [CHUYỂN SANG 10_MATRIX]**
  - **Trạng thái**: 🔄 **CHUYỂN SANG MÔ-ĐUN `10_matrix`**.
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:108-114`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L108-L114)

- [x] **TODO-P2-03: Bổ sung bảo vệ `NaN` cho `flow_force` [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Đã bổ sung `flow_force.is_nan()` và kiểm tra an toàn `get(i)` cho toàn bộ vector.
  - **Vị trí**: [`momentum_ignition.rs:44-50`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L44-L50)

---

## 🧪 NHÓM P3: KIỂM THỬ & XÁC MINH HỆ THỐNG (TESTING & BENCHMARKING)

- [x] **TODO-P3-01: Viết Unit Test cho Bộ Lọc Râu Nến & Màu Nến [ĐÃ XONG]**
  - **Trạng thái**: ✅ **HOÀN THÀNH**. Đã viết và pass 4 unit tests:
    - `test_momentum_shooting_star_rejected_by_clv`: Test nến rút chân đỉnh bị CLV guard từ chối.
    - `test_momentum_inside_band_early_exit`: Test giá trong dải bị loại bỏ ngay tại Early Exit Guard.
    - `test_momentum_macro_counter_trend_veto`: Test nến bùng nổ ngược trend KAMA bị Veto.
    - `test_momentum_valid_breakout_approved`: Test nến bùng nổ chuẩn được cấp Proposal Long với SL cấu trúc chuẩn.
- [ ] **TODO-P3-02: Viết Unit Test Đấu Thầu Arbiter Chống Cướp Thầu [CHUYỂN SANG 04_ARBITER]**
  - **Trạng thái**: 🔄 **CHUYỂN SANG MÔ-ĐUN `04_arbiter`**.
- [ ] **TODO-P3-03: Chạy lại Walk-Forward Optimization & Paper Simulation [GIAI ĐOẠN CUỐI]**
  - **Trạng thái**: ⏳ **CHỜ KIỂM TOÁN XONG CÁC MÔ-ĐUN CÒN LẠI**.
