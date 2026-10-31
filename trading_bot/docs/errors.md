PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example walk_forward_smart_seed
   Compiling ferro_trade_derive v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade\ferro_trade_derive)
   Compiling ferro_trade v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade)
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
   Compiling bot_engine v0.1.0 (E:\Projects\Automations\trading_backtest\trading_bot\bot_engine)
warning: unused import: `debug`
  --> examples\walk_forward_smart_seed.rs:15:34
   |
15 | use tracing::{info, warn, error, debug};
   |                                  ^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: `bot_engine` (example "walk_forward_smart_seed") generated 1 warning (run `cargo fix --example "walk_forward_smart_seed"` to apply 1 suggestion)
    Finished `release` profile [optimized] target(s) in 2m 28s
     Running `target\release\examples\walk_forward_smart_seed.exe`
📁 [LOGGING SETUP] Thư mục log độc lập phiên này: examples/logs/run_walk_forward_smart_seed_20261028_094751
2026-10-28T02:47:51.412389Z  INFO walk_forward_smart_seed: ================================================================================
2026-10-28T02:47:51.412673Z  INFO walk_forward_smart_seed: 🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%
2026-10-28T02:47:51.412781Z  INFO walk_forward_smart_seed:    Symbol: ETHUSDT, Strategy: walk_forward_smart_seed
2026-10-28T02:47:51.412923Z  INFO walk_forward_smart_seed:    Total Fetch: 45 days | Train Window: 40 days | WFE OOS: 3 days (4320 bars)
2026-10-28T02:47:51.413078Z  INFO walk_forward_smart_seed:    📌 [N MINS] Tần suất Re-Optimize GA:       240 phút
2026-10-28T02:47:51.413213Z  INFO walk_forward_smart_seed:    📌 [M MINS] Tần suất Single Backtest Eval:  240 phút
2026-10-28T02:47:51.413340Z  INFO walk_forward_smart_seed:    💰 [ACCOUNT] Initial Cash: $60.00 | Leverage: 20x | Comm: 0.0004
2026-10-28T02:47:51.413530Z  INFO walk_forward_smart_seed:    🧠 [MEMORY] Max Live Candles: 57600 (Đảm bảo 50+ nến MTF 4h)
2026-10-28T02:47:51.413660Z  INFO walk_forward_smart_seed:    🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới
2026-10-28T02:47:51.413805Z  INFO walk_forward_smart_seed: ================================================================================
2026-10-28T02:47:51.622191Z  INFO walk_forward_smart_seed: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 45 ngày từ Binance...
2026-10-28T02:48:06.206589Z  INFO walk_forward_smart_seed: 📊 [Data Summary] Tổng số nến: 64738 | Từ 2026-09-13 02:48 UTC -> Đến 2026-10-28 02:48 UTC
2026-10-28T02:48:06.206802Z  INFO walk_forward_smart_seed: --------------------------------------------------------------------------------
2026-10-28T02:48:06.206993Z  INFO walk_forward_smart_seed: 🧬 [INITIAL OPTIMIZE] Chạy GA Optimizer huấn luyện ban đầu trên 30 ngày...

thread '<unnamed>' (19124) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 30m not found in cache. Did you forget to precompute?
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\walk_forward_smart_seed.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)