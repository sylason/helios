# HELIOS V3.3 ARCHITECTURE REVIEW: PART 2 - MODULES AUDIT (FILES 1, 2, 3) & REDUNDANT PARAMETERS LIST

Tài liệu này ghi nhận kết quả rà soát chi tiết 3 File mã nguồn đầu tiên của **Helios Core V3**: [`mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs), [`sensors.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs), [`matrix.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs), và danh sách toàn bộ các biến dư thừa được tinh giảm.

---

## 🔍 1. RÀ SOÁT FILE #1: [`core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
*(Thành phần: Strategy Orchestrator & Vòng Lặp Điều Phối Lệnh)*

### A. Phát Hiện Nguyên Nhân Trade Gaps:
- Trong mã nguồn cũ, khi một lệnh bị thua (dù chỉ lỗ $-0.20\%$), `cooldown_bars_remaining` thiết lập khoảng nghỉ từ $10 \rightarrow 500$ nến.
- Trong 500 nến này, `next()` gặp `return` và **TỪ CHỐI DÒ TÍN HIỆU CỦA TOÀN BỘ 6 PLAYBOOKS**, gây ra các khoảng trống thời gian không trade 600 - 4.800 nến trong backtest!

### B. Quyết Định Thay Đổi:
- **Loại bỏ 100% `cooldown_duration_bars` trong `mod.rs`**: Tuyệt đối không đóng băng vô cớ luồng quét tín hiệu sau lệnh thua.
- **Giữ nguyên Strict Exclusive Order**: Chỉ quét tín hiệu mở lệnh mới khi vị thế hiện tại đã đóng (`is_pos_open == false`). Khi đang có vị thế mở, nhường 100% quyền cho `RiskManager` và `Take Profit`.

### ✂️ C. Danh Sách Biến Dư Thừa Được Tinh Giảm Cho File #1 (`mod.rs`):
1. **`cooldown_bars_remaining`**: Khóa đóng băng 500 nến dư thừa.
2. **`cooldown_duration_bars` (trong `StopLossConfig`)**: Tham số dải $0 \rightarrow 500$ nến gây overfit trong GA Optimizer.
3. **`equity_at_entry` & `was_position_open`**: Các biến cờ theo dõi trạng thái cooldown cũ không còn cần thiết.

---

## 🔍 2. RÀ SOÁT FILE #2: [`strategy/sensors.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs)
*(Thành phần: Indicators Cache & Data Binding - "Đôi Mắt" Của Helios)*

### A. Đánh Giá Đa Ngành:
1. **Xác suất Toán học**: Bộ 17 trường chỉ báo phủ kín Không gian Trạng thái 4D (Động lượng, Biến động, Xu hướng/Độ phẳng, Khối lượng/Dòng tiền).
2. **Hiệu năng Rust**: Sử dụng `Arc<Vec<f64>>` cho phép copy con trỏ zero-copy và truy cập mảng $O(1)$, tối ưu tuyệt đối cho Rayon Optimizer.
3. **Phân tích Thực chiến**: Bộ đôi Bollinger Bands + Keltner Channels (BB vs KC) là vũ khí tối thượng phát hiện Nén Lò Xo (Squeeze Compression).

### ✂️ B. Danh Sách Biến Dư Thừa Được Tinh Giảm Cho File #2 (`sensors.rs`):
1. **`vol_ma_series` (Volume SMA nhân `vol_ratio` thô)**: Thay thế bằng `vol_z_score` và `sensors.mfi` chuẩn hóa.
2. **`linreg_slope` & `linreg_angle`**: Thay thế bằng `kama_slope` và `kama_slope_adj`.
3. **Các biến chỉ báo dùng chung (Global Shared Sensor Periods)**: Tách các biến `rsi.period` và `stochastic.period` dùng chung thành các bộ cảm biến độc lập riêng cho từng Playbook (`rsi_trend`, `rsi_reversal`, `rsi_momentum`, `stoch_trend`, `stoch_range`).

---

## 🔍 3. RÀ SOÁT FILE #3: [`core_v3/matrix.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs)
*(Thành phần: Market Perception Matrix & Regime Multipliers)*

### A. Phát Hiện Bẫy Toán Học & Cải Tiến:

#### 1. Khắc Phục Bẫy Rolling Percentile ADX
- **Bẫy thống kê cũ**: `trend_multiplier = rolling_percentile(ADX)`. Khi thị trường sideway phẳng lặng (ADX = 12), Rolling Percentile vẫn xếp ADX = 12 ở mức $0.50$, khiến `range_multiplier = 1.0 - 0.50 = 0.50` (không vọt lên được $0.95$ để hỗ trợ `RANGE_OSC`).
- **Giải pháp nén Tanh Scale**: Chuyển sang chuẩn hóa Tanh Scale tuyệt đối:
  $$\text{trend\_multiplier} = \text{tanh\_scale}(\text{ADX} - 20.0, \text{sensitivity})$$
  - Khi ADX = 10 (Sideway phẳng) $\implies \text{trend\_multiplier} \approx 0.05 \implies \text{range\_multiplier} \approx 0.95$!

#### 2. Xóa Bỏ `vol_raw` & `vol_ratio` Gượng Ép
- **Vấn đề cũ**: Nhân volume nến 1m với `vol_ratio` (x60, x240) tạo nhiễu nổ volume giả.
- **Nâng cấp Hệ Thống Volume Chuẩn Quant**:
  $$\text{volume\_force} = 0.5 \times \text{tanh\_scale}(\text{vol\_z\_score}, 0.5) + 0.5 \times \left(\frac{\text{MFI}}{100}\right)$$
  Loại bỏ vĩnh viễn `vol_ratio` rác khỏi mã nguồn dự án!

### ✂️ B. Danh Sách Biến Dư Thừa Được Tinh Giảm Cho File #3 (`matrix.rs`):
1. **`vol_raw` & `vol_ratio`**: Phép nhân volume nến 1m gượng ép bị xóa bỏ.
2. **`trend_angle` & `trend_angle_adj` (tính từ OLS LinReg Slope)**: Thay thế bằng KAMA Slope (`kama_slope_adj`).
3. **Các trọng số nhân ma trận thừa trong `matrix_weights`**: Các trọng số nhân phân số (`ano_z_w`, `rev_m_w`, `abs_sqz_w`, `vol_chaos_w`) được tinh giảm nhờ công thức cộng Tanh Scale chuẩn hóa.
