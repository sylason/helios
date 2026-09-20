# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ: MOMENTUM IGNITION SCANNER & HỆ THỐNG LIÊN ĐỚI (TODO.MD)

Tài liệu này tổng hợp toàn bộ các đầu việc kỹ thuật cần triển khai sửa lỗi và nâng cấp cho **`momentum_ignition.rs`** cùng các thành phần liên đới trong hệ thống Helios V3.3.

---

## 🚨 NHÓM P0: CẤP THIẾT - SỬA LỖI HỆ THỐNG CHẾT NGƯỜI (SYSTEM-BREAKING BUGS)

Các lỗi này trực tiếp dẫn đến việc tài khoản bị cướp quyền kiểm soát 100% và thua lỗ liên hoàn trong Paper Trading.

- [ ] **TODO-P0-01: Sửa lỗi cướp thầu ngược (Adverse Bid-Sniping Bug) trong `arbiter.rs`**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:175-182`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L175-L182)
  - **Hiện trạng**: Dòng 178 `if dist_new < dist_curr` tự động chọn đề xuất có khoảng cách Stop Loss gần nhất thay vì điểm tự tin cao nhất. Do Momentum Ignition đặt SL siêu sát, nó luôn cướp sạch mọi lệnh của TrendPB và Absorption.
  - **Hành động**: Đảo ngược thứ tự ưu tiên:
    1. So sánh `conf - current_conf`: Nếu chênh lệch $> 0.05$, chọn bên có Confidence cao hơn.
    2. Chỉ khi Confidence tương đương (chênh lệch $\le 0.05$), mới dùng khoảng cách Stop Loss làm tiêu chí phụ (Tie-breaker).
  - **Xác minh**: Viết unit test mô phỏng trường hợp TrendPB ($conf = 0.85, dist = 2.0$) so găng với Momentum ($conf = 0.25, dist = 0.4$), đảm bảo TrendPB chiến thắng.

- [ ] **TODO-P0-02: Bổ sung kiểm tra màu nến & Định lượng hóa qua CLV (Close Location Value)**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:41-69`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L41-L69)
  - **Hiện trạng**: Scanner không đọc `tick.opens()[i]`, không kiểm tra `close > open`, không có bộ lọc râu nến. Mua đúng đỉnh nến Shooting Star rút chân đỏ loét hoặc Bán đúng đáy nến Hammer.
  - **Hành động (Chuẩn Định Lượng Quant)**:
    - Trích xuất `open = tick.opens()[i]`, `high = tick.highs()[i]`, `low = tick.lows()[i]`.
    - Bắt buộc màu nến: Long đòi hỏi `close > open` (nến xanh); Short đòi hỏi `close < open` (nến đỏ).
    - Đo lực nến bằng **CLV (Close Location Value)**:
      $$\text{CLV} = \frac{2 \cdot \text{Close} - \text{High} - \text{Low}}{\text{High} - \text{Low}} \in [-1.0, +1.0]$$
      - Long: Bắt buộc $\text{CLV} \ge \text{min\_clv}$ (đóng cửa sát đỉnh nến, loại bỏ 100% râu trên dài / Shooting Star).
      - Short: Bắt buộc $\text{CLV} \le -\text{min\_clv}$ (đóng cửa sát đáy nến, loại bỏ 100% râu dưới dài / Hammer).

- [ ] **TODO-P0-03: Sửa dải gen độc hại của `rsi_threshold` trong `config.rs`**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/config.rs:408-410`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L408-L410)
  - **Hiện trạng**: `#[dna(min = 1.0, max = 99.0, step = 3.0)]`. Khi gen lặn xuống $< 50.0$, logic Long và Short bị đảo lộn toán học, kích hoạt tín hiệu bừa bãi trong vùng nhiễu.
  - **Hành động**: Thu hẹp dải gen về đúng bản chất xung lực tăng tốc:
    `#[dna(min = 55.0, max = 80.0, step = 1.0)]`.

- [ ] **TODO-P0-04: Bổ sung Điều Kiện Tiên Quyết Cứng (Hard Breakout Pre-condition)**
  - **Vị trí**: [`momentum_ignition.rs:54-61`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L54-L61)
  - **Hiện trạng**: Dùng hàm Sigmoid mềm cấp điểm ảo $0.27 - 0.50$ ngay cả khi giá còn nằm sâu dưới dải BB. Kích hoạt breakout khi giá chưa hề breakout!
  - **Hành động**:
    - Long: Bắt buộc $\text{Close} > BB_{\text{upper}}$.
    - Short: Bắt buộc $\text{Close} < BB_{\text{lower}}$.
    - Nếu không thỏa mãn điều kiện tiên quyết, lập tức trả về `None`.

- [ ] **TODO-P0-05: Bổ sung Chốt Chặn Xu Hướng Vĩ Mô (Macro Counter-Trend Veto)**
  - **Vị trí**: [`momentum_ignition.rs:64`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L64)
  - **Hiện trạng**: Momentum Ignition hoàn toàn không có bộ lọc vĩ mô, sẵn sàng mở lệnh Long khi KAMA slope đang cắm đầu rơi tự do.
  - **Hành động**: Kế thừa `max_adverse_slope`:
    - Long: Nếu `trend_dir < -max_adverse_slope` $\to$ VETO cấm Long.
    - Short: Nếu `trend_dir > max_adverse_slope` $\to$ VETO cấm Short.

- [ ] **TODO-P0-06: Khử Bỏ Toàn Bộ Các Hằng Số Hardcoded Trong Code**
  - **Vị trí**: [`momentum_ignition.rs:54-86`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L54-L86)
  - **Hiện trạng**:
    - Sàn đệm SL `.max(0.05)` bị gán cứng.
    - Sàn bảo vệ tối thiểu `0.002` (0.2% giá) bị gán cứng.
    - Tỷ trọng `0.40 * f_break + 0.35 * f_rsi + 0.25 * f_vol` bị đóng đinh trong code.
    - Hệ số dốc `-2.0` và `-0.15` bị gán cứng.
  - **Hành động**: Chuyển các ngưỡng nhạy cảm thành tham số DNA hoặc chuẩn hóa theo ATR động, loại bỏ hoàn toàn các hằng số cảm tính.

---

## ⚡ NHÓM P1: QUAN TRỌNG - VI CẤU TRÚC GIÁ & BẢO VỆ STOP LOSS (MICROSTRUCTURE & RISK GUARDS)

Ngăn chặn hiện tượng dính Hard Stop Loss trong 2-3 bars và bẫy nến kiệt sức (Climax/Liquidity Grab).

- [ ] **TODO-P1-01: Chặn Bẫy Nến Kiệt Sức (Buying/Selling Climax Filter)**
  - **Vị trí**: [`momentum_ignition.rs:55, 60`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L55)
  - **Hiện trạng**: Xu hướng đã tăng quá nóng (RSI > 85), nến cuối cùng bung thanh khoản để xả hàng (Buying Climax) nhưng Scanner vẫn nhảy vào Long đu đỉnh.
  - **Hành động**: Thiết lập trần kẹp quá độ:
    - Long: Nếu $\text{RSI} > 80.0$, cấm vào lệnh bứt phá (đã quá trễ sóng, dễ dính nhịp điều chỉnh sâu).
    - Short: Nếu $\text{RSI} < 20.0$, cấm vào lệnh bứt phá.

- [ ] **TODO-P1-02: Tái cấu trúc cơ chế đặt Stop Loss chống Flash Stop-Out 2-3 bars**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:65-74`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L65-L74)
  - **Hiện trạng**: SL đặt tại `Low[i] - buffer` với `buffer = 0.10*ATR`, tổng khoảng cách chỉ $0.3 \sim 0.5 \cdot ATR$, bị quét sạch ngay nhịp retest tự nhiên đầu tiên.
  - **Hành động**:
    - Nâng sàn khoảng đệm SL `struct_sl_buffer_atr` lên tối thiểu `0.30 - 0.50 ATR`.
    - Neo Stop Loss dưới đáy của cụm 2 nến:
      - Long: `sl = low[i].min(low[i-1]) - buffer`.
      - Short: `sl = high[i].max(high[i-1]) + buffer`.
    - Thiết lập khoảng cách an toàn tối thiểu (SL Min Distance Guard): Đảm bảo $|Close - SL| \ge 0.80 \times ATR$ để vượt qua vùng nhiễu Brown.

- [ ] **TODO-P1-03: Sửa dải gen `vol_z_threshold` và `struct_sl_buffer_atr` trong `config.rs`**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/config.rs:412-423`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L412-L423)
  - **Hiện trạng**:
    - `vol_z_threshold`: Đang là `0.10 -> 1.00` (quá thấp, bị bão hòa bởi điểm nền MFI).
    - `struct_sl_buffer_atr`: Đang là `0.10 -> 3.00` (cận dưới quá sát).
  - **Hành động**:
    - Điều chỉnh `vol_z_threshold`: `#[dna(min = 1.20, max = 3.50, step = 0.10)]`.
    - Điều chỉnh `struct_sl_buffer_atr`: `#[dna(min = 0.50, max = 3.00, step = 0.15)]`.

---

## 🛠️ NHÓM P2: TRUNG HẠN - LỌC NHIỄU BIẾN ĐỘNG & AN TOÀN SỐ HỌC (ROBUSTNESS & FILTERS)

- [ ] **TODO-P2-01: Thêm bộ lọc bung mở dải Bollinger (Bandwidth Expansion Filter)**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:53`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L53)
  - **Hiện trạng**: Không phân biệt được nến bùng nổ giải phóng năng lượng (Squeeze Breakout) với nến bùng nổ giả trong dải hẹp (Whipsaw Fakeout).
  - **Hành động**: Kiểm tra độ dốc của dải Bollinger:
    `bb_width[i] = (bb_upper[i] - bb_lower[i]) / bb_middle[i]`.
    Chỉ kích hoạt bùng nổ khi `bb_width[i] >= bb_width[i-1] * 1.05` (dải BB đang nở rộng ít nhất 5%).

- [ ] **TODO-P2-02: Khắc phục điểm nền MFI trong `MarketMatrixV3.flow_axis`**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:108-114`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L108-L114)
  - **Hiện trạng**: `vf = 0.5 * |z_val| + 0.5 * mfi_norm`. Khi khối lượng bằng 0, điểm vẫn là 0.25 do MFI = 50.
  - **Hành động**: Tách riêng trục khối lượng thuần túy (Pure Volume Z-Score) hoặc sử dụng phép nhân trọng số:
    `vf = z_val.max(0.0) * (mfi_norm / 0.50)`.

- [ ] **TODO-P2-03: Bổ sung bảo vệ `NaN` cho `flow_force`**
  - **Vị trí**: [`strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:43-47`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L43-L47)
  - **Hiện trạng**: `flow_force` không được kiểm tra `is_nan()`.
  - **Hành động**: Bổ sung `|| flow_force.is_nan()` vào khối `if` bảo vệ ở dòng 45.

---

## 🧪 NHÓM P3: KIỂM THỬ & XÁC MINH HỆ THỐNG (TESTING & BENCHMARKING)

- [ ] **TODO-P3-01: Viết Unit Test cho Bộ Lọc Râu Nến & Màu Nến**
  - Test case nến Shooting Star: Mở 2400, Đỉnh 2450, Đóng 2410 (Râu trên 40 giá). Xác nhận Scanner phải trả về `None`.
  - Test case nến Marubozu tăng: Mở 2400, Đáy 2398, Đỉnh 2450, Đóng 2448. Xác nhận Scanner chấm điểm cao.
- [ ] **TODO-P3-02: Viết Unit Test Đấu Thầu Arbiter Chống Cướp Thầu**
  - Tạo 2 Proposal: TrendPB ($conf = 0.85, dist = 2.0$) và Momentum ($conf = 0.25, dist = 0.4$).
  - Xác nhận `ArbiterDecision.winning_scanner == Some(ScannerId::TrendPullback)`.
- [ ] **TODO-P3-03: Chạy lại Walk-Forward Optimization & Paper Simulation**
  - Chạy lại kịch bản tương đương `WFSS_37.log` trên cặp ETHUSDT.
  - Đo lường xem tỷ lệ thắng của Momentum Ignition có được cải thiện và thời gian giữ lệnh có thoát khỏi bẫy $2-3$ bars hay không.
