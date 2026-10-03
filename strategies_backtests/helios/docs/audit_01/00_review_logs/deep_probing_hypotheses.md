# CÁC GIẢ THUYẾT CHẤT VẤN SÂU (DEEP PROBING HYPOTHESES) & KIỂM CHỨNG TOÀN DIỆN
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/00_review_logs/`  
> **Ngày lập:** 03/10/2026  
> **Nguyên tắc:** First-Principles, Hoài nghi tiền đề, Không giả định ngầm.

---

## 🎯 DANH SÁCH 3 GIẢ THUYẾT CỐT TỬ CẦN KIỂM CHỨNG TRONG CODE

### 🧪 Giả Thuyết 1: Nguy Cơ Lookahead Bias Trong MTF Resampling & Indicators
- **Vấn đề đặt ra:** Trong backtest In-Sample 30 ngày, hệ thống tiền tính toán các nến đa khung thời gian (5m, 15m, 1h, 4h) qua `ferro_trade/src/utils/resample.rs` và `IndicatorManager`.
- **Câu hỏi First-Principles:** 
  1. Khi một cây nến 1m tại thời điểm $t$ đang được xử lý, cây nến 1h hoặc 4h tương ứng được trích xuất như thế nào?
  2. Có hiện tượng cây nến 1h lấy giá `Close`, `High`, `Low` của toàn bộ cây nến 1h khi nến 1h đó mới chỉ chạy được 5 hoặc 15 phút không?
  3. Nếu có Lookahead Bias, chiến lược trong backtest sẽ "biết trước tương lai", dẫn tới GA Optimizer tìm ra các cá thể có lợi nhuận ảo khổng lồ (>100%), nhưng khi chạy giả lập từng nến (OOS) hoặc Live thì sụp đổ hoàn toàn.
- **Phạm vi kiểm tra:**
  - `ferro_trade/src/utils/resample.rs`
  - `ferro_trade/src/backtest/engine.rs` và `data.rs`
  - `strategies_backtests/helios/src/strategy/core_v3/sensors.rs` và `matrix.rs`

---

### 🧪 Giả Thuyết 2: Sự Méo Mó Của Hàm Mục Tiêu (Fitness Gaming), Lỗ Hổng WFE Gate & Ma Sát Phí/Trượt Giá
- **Vấn đề đặt ra:** Tại sao Smart Seed chỉ chiếm 0.5% (1 cá thể trên 200) mà các cá thể cào lệnh vẫn liên tục sống sót và chiến thắng qua 47 chu kỳ Re-Optimize?
- **Câu hỏi First-Principles:**
  1. **Hàm Fitness:** `calculate_enhanced_fitness` đang thưởng quá đà cho Return tuyệt đối (`ret / (dd + 3.0)`). Hình phạt tần suất lệnh (`freq_weight`) quá nhẹ (tối đa chỉ phạt 30%). Hàm fitness hoàn toàn KHÔNG phạt tỷ lệ R:R suy thoái (Avg Win < Avg Loss) và không phạt các lệnh dính Hard SL 15% ROE.
  2. **Cổng WFE Gate:** Tại dòng 406 của `prospector_runner.rs`, cổng "Tolerant Fee Buffer Pass" cho phép OOS Return bị âm (tới -1.2%) nếu In-Sample Return $> 80\%$. Đây có phải là một cửa sau đưa các cá thể Overfit cào lệnh vào thực tế?
  3. **Độ Lệch Ma Sát (Frictional Cost Discrepancy):** 
     - Phí sàn trong Backtest GA: `commission_rate = 0.0004` (0.04%). Phí Taker Binance VIP0 thực tế: `0.0005` (0.05%).
     - Backtest GA có tính trượt giá (Slippage) khi dính Stop Loss không? Paper Engine và Live Engine xử lý trượt giá ra sao?
- **Phạm vi kiểm tra:**
  - `trading_bot/bot_engine/src/optimizer/prospector_runner.rs`
  - `ferro_trade/src/optimizer/prospector.rs` và `ga_worker.rs`
  - `trading_bot/bot_engine/src/execution/paper_trader.rs`

---

### 🧪 Giả Thuyết 3: Bệnh Lý Toàn Diện 6 Scanners & Lũng Đoạn Trọng Số Arbiter (Weight Hijacking)
- **Vấn đề đặt ra:** Ngoài `MOMENTUM_IGN` thiếu trần `Price_Z`, các Scanner khác như `TREND_PB`, `REVERSAL`, `ABSORPTION`, `RANGE_OSC`, `ANOMALY` đang chứa đựng những điểm mù và cạm bẫy nào?
- **Câu hỏi First-Principles:**
  1. **Scanner `TREND_PB`:** Lỗi KAMA Suction Trap (`low.min(kama_mid)`) kéo SL quá xa; ngưỡng độ dốc `kama_slope_threshold` có thể bị GA kéo xuống quá nhỏ làm mất tính chất Trend; bộ lọc nến `close < open` quá yếu ớt.
  2. **Scanner `REVERSAL`:** Bắt dao rơi khi `chaos_index` đang ở đỉnh bão.
  3. **Scanner `RANGE_OSC`:** Khi dải Bollinger Bands co thắt cực đại (Squeeze), TP bị thổi phồng ảo.
  4. **Cơ Chế Đấu Thầu Arbiter:** `final_confidence = raw_score * playbook_weight`. Trọng số `playbook_weight` đều nằm trong DNA gen. Liệu GA có đẩy trọng số của một scanner lên quá cao để cướp quyền áp đảo toàn bộ các scanner khác không?
- **Phạm vi kiểm tra:**
  - Toàn bộ thư mục `strategies_backtests/helios/src/strategy/core_v3/scanners/`
  - `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`
  - `strategies_backtests/helios/src/strategy/core_v3/risk.rs`
