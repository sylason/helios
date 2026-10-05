PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example walk_forward_smart_seed
    Finished `release` profile [optimized] target(s) in 0.84s
     Running `target\release\examples\walk_forward_smart_seed.exe`
📁 [LOGGING SETUP] Thư mục log độc lập phiên này: examples/logs/run_walk_forward_smart_seed_20261005_001558
2026-10-04T17:15:58.901315Z  INFO walk_forward_smart_seed: ================================================================================
2026-10-04T17:15:58.901475Z  INFO walk_forward_smart_seed: 🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%
2026-10-04T17:15:58.901619Z  INFO walk_forward_smart_seed:    Symbol: ETHUSDT, Strategy: walk_forward_smart_seed
2026-10-04T17:15:58.901710Z  INFO walk_forward_smart_seed:    Total Fetch: 22 days | Train Window: 14 days | WFE OOS: 3 days (4320 bars)
2026-10-04T17:15:58.901814Z  INFO walk_forward_smart_seed:    📌 [N MINS] Tần suất Re-Optimize GA:       240 phút
2026-10-04T17:15:58.901964Z  INFO walk_forward_smart_seed:    📌 [M MINS] Tần suất Single Backtest Eval:  240 phút
2026-10-04T17:15:58.902099Z  INFO walk_forward_smart_seed:    💰 [ACCOUNT] Initial Cash: $60.00 | Leverage: 20x | Comm: 0.0004
2026-10-04T17:15:58.902203Z  INFO walk_forward_smart_seed:    🧠 [MEMORY] Max Live Candles: 43100 (Đảm bảo 50+ nến MTF 4h)
2026-10-04T17:15:58.902283Z  INFO walk_forward_smart_seed:    🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới
2026-10-04T17:15:58.902361Z  INFO walk_forward_smart_seed: ================================================================================
2026-10-04T17:15:59.048345Z  INFO walk_forward_smart_seed: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 22 ngày từ Binance...
2026-10-04T17:16:06.120520Z  INFO walk_forward_smart_seed: 📊 [Data Summary] Tổng số nến: 31651 | Từ 2026-09-12 17:16 UTC -> Đến 2026-10-04 17:16 UTC
2026-10-04T17:16:06.120867Z  INFO walk_forward_smart_seed: --------------------------------------------------------------------------------
2026-10-04T17:16:06.121027Z  INFO walk_forward_smart_seed: 🧬 [INITIAL OPTIMIZE] Chạy GA Optimizer huấn luyện ban đầu trên 30 ngày...

thread 'main' (12556) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 3m not found in cache. Did you forget to precompute?
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\walk_forward_smart_seed.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)

---

## 🛠️ NGUYÊN NHÂN & GIẢI PHÁP ĐÃ XỬ LÝ (2026-10-05)

### 1. Nguyên Nhân Gốc Rễ (Root Cause):
- Trong `helios::strategy::core_v3::config`, các cảm biến vi mô (`AtrParams`, `PriceZScoreParams`, `VolZScoreParams`) hỗ trợ `#[dna(subset = "1m, 3m, 5m")]`.
- File cấu hình `helios_optimizer_config.json` đã khai báo `"available_timeframes": ["1m", "3m", "5m", "15m", "1h", "4h"]`.
- GA Optimizer chạy In-Sample huấn luyện ra Champion DNA lựa chọn khung `3m` cho cảm biến vi mô.
- Tuy nhiên, tại `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:15-20`, hằng số `HELIOS_REQUIRED_TIMEFRAMES` bị hardcode cũ chỉ gồm 4 khung `[5m, 15m, 1h, 4h]` (thiếu `3m`).
- Khi `ProspectorRunner` gọi `HeliosBridge::diagnostic_backtest` để thẩm định OOS qua WFE Gate, `BacktestBuilder` chỉ resample 4 khung mà không có `3m` $\to$ `context.rs:136` panic crash.

### 2. Các File Đã Điều Chỉnh:
1. `trading_bot/bot_engine/src/optimizer/helios_bridge.rs`:
   - Mở rộng `HELIOS_REQUIRED_TIMEFRAMES: [Timeframe; 5]` bổ sung `Timeframe::T3m`.
2. `ferro_trade/src/optimizer/prospector.rs`:
   - Bổ sung `"3m".to_string()` vào `available_timeframes` trong `ProspectorConfig::default()`.