# HELIOS V3.3 ARCHITECTURE REVIEW: PART 13 - CONFIG & DNA SPACE AUDIT

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #12: [`core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)** và cấu trúc dải tham số DNA V3.3.

---

## 🏛️ 1. MỤC ĐÍCH & KIẾN TRÚC CONFIG THEO NGUYÊN TẮC CLEAN CODE

### Chốt Lựa Chọn Cấu Trúc (Phương Án B):
- Toàn bộ các chỉ báo độc lập per-playbook (`rsi_trend`, `rsi_reversal`, `rsi_momentum`, `stoch_trend`, `stoch_range`...) được nhóm tập trung trong struct **`IndicatorsConfig`**.
- **Đánh giá Clean Code**:
  - **Single Responsibility Principle (SRP)**: `IndicatorsConfig` đóng vai trò là một **Config Registry / Schema Manifest** duy nhất. Khi người dùng hoặc nhà phát triển đọc file JSON cấu hình, họ có thể ngay lập tức nhìn thấy toàn bộ thước đo chỉ báo ở cùng một nơi mà không phải đi mò mẫm sâu vào từng struct playbook.
  - **Tách biệt mối quan tâm (Separation of Concerns)**: Struct `TacticalSniperConfig` chỉ tập trung chứa các ngưỡng logic kích hoạt lệnh (Trigger Thresholds), còn `IndicatorsConfig` tập trung chứa định nghĩa chu kỳ/khung thời gian (Periods & Timeframes).

---

## 🔬 2. TỔNG HỢP TOÀN BỘ CÁC BIẾN TINH GIẢM TRONG CONFIG V3.3

Nhằm giải quyết "Lời nguyền chiều dữ liệu" (Curse of Dimensionality), dải DNA của GA Optimizer được rút ngắn đáng kể:

| Struct Config | Các Biến Được Tinh Giảm / Loại Bỏ | Lý Do Tinh Giảm |
|---|---|---|
| `IndicatorsConfig` | Xóa `LinRegParams`, xóa `rsi`/`stochastic` dùng chung | Thay bằng `KAMA Slope` và các Cảm biến Độc lập |
| `ArbiterConfig` | Xóa `min_approval_confidence` đơn lẻ | Thay bằng 3 dải `min_confidence` động theo Regime |
| `MatrixWeightsConfig` | Xóa 100% struct `MatrixWeightsConfig` | Dùng công thức Tanh Scale cộng trực tiếp |
| `TacticalPullbackParams` | Xóa `stoch_vs_kama_weight` | Dùng điều kiện Stochastic Oversold $K < 30$ |
| `TacticalMomentumParams` | Xóa `rsi_climax`, `vol_max_multiplier`, `vol_vs_break_weight` | Thay bằng Breakout BB Upper & Vol Z-Score |
| `TacticalRangeParams` | Xóa `bb_vs_stoch_weight`, `range_bb_tolerance_pct` | Dùng điều kiện Stochastic Oversold $K < 25$ tại biên BB |
| `TacticalReversalParams` | Xóa `z_vs_rsi_weight` | Kiểm tra trực tiếp Z-Score và RSI cực đại |
| `TacticalAnomalyParams` | Xóa `pattern_vs_wick_weight`, `z_vs_rsi_weight` | Đo râu nến theo ATR trong Volatile Regime |
| `TacticalAbsorptionParams` | Xóa `macd_lookback`, `vol_vs_squeeze_weight`, `trend_filter_threshold` | Thay bằng `VWMA vs SMA` và Volume Z-Score |
| `RiskManagementConfig` | Xóa `cooldown_duration_bars` | Loại bỏ 100% đợt đóng băng 500 nến |
