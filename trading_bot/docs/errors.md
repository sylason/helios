PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example paper_forward_test
   Compiling ahash v0.8.12
   Compiling hashbrown v0.14.5
   Compiling dashmap v6.2.1
   Compiling hashlink v0.9.1
   Compiling rusqlite v0.31.0
   Compiling ferro_trade v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade)
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
   Compiling bot_engine v0.1.0 (E:\Projects\Automations\trading_backtest\trading_bot\bot_engine)
warning: unused imports: `mpsc` and `watch`
 --> src\execution\ws_manager.rs:2:19
  |
2 | use tokio::sync::{mpsc, watch, broadcast};
  |                   ^^^^  ^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused imports: `NewOrderParams` and `NewOrderSideEnum`
  --> src\execution\ws_manager.rs:12:21
   |
12 |     websocket_api::{NewOrderParams, NewOrderSideEnum},
   |                     ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> src\optimizer\prospector_runner.rs:5:27
  |
5 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused variable: `connection`
   --> src\execution\ws_manager.rs:266:24
    |
266 |                     Ok(connection) => {
    |                        ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_connection`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused import: `TradeApi`
 --> src\execution\account_manager.rs:4:30
  |
4 |         RestApi, AccountApi, TradeApi,
  |                              ^^^^^^^^

warning: unused import: `AccountApi`
 --> src\execution\account_manager.rs:4:18
  |
4 |         RestApi, AccountApi, TradeApi,
  |                  ^^^^^^^^^^

warning: unused import: `TradeApi`
 --> src\execution\order_executor.rs:5:5
  |
5 |     TradeApi, NewOrderParams, NewOrderSideEnum, NewOrderTypeEnum,
  |     ^^^^^^^^

warning: unused variable: `weight_monitor`
   --> src\execution\ws_manager.rs:160:13
    |
160 |         let weight_monitor = self.weight_monitor.clone();
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_weight_monitor`

warning: fields `monitor` and `market_filter` are never read
  --> src\controller\asset_actor.rs:34:5
   |
27 | pub struct AssetActor {
   |            ---------- fields in this struct
...
34 |     monitor: Arc<WeightMonitor>,
   |     ^^^^^^^
...
44 |     market_filter: crate::models::market_info::MarketFilter,
   |     ^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: fields `symbol` and `last_candle_ts` are never read
   --> src\optimizer\helios_bridge.rs:163:13
    |
162 |         struct GlobalMtfCache {
    |                -------------- fields in this struct
163 |             symbol: String,
    |             ^^^^^^
164 |             last_candle_count: usize,
165 |             last_candle_ts: i64,
    |             ^^^^^^^^^^^^^^

warning: `bot_engine` (lib) generated 10 warnings (run `cargo fix --lib -p bot_engine` to apply 3 suggestions)
    Finished `release` profile [optimized] target(s) in 2m 08s
     Running `target\release\examples\paper_forward_test.exe`
2026-08-22T02:14:23.776462Z  INFO paper_forward_test: ================================================================================
2026-08-22T02:14:23.776720Z  INFO paper_forward_test: 🧪 BẮT ĐẦU THỬ NGHIỆM FULL ENGINE PAPER FORWARD TEST (15-MIN RE-OPT)
2026-08-22T02:14:23.776905Z  INFO paper_forward_test:    Symbol: ETHUSDT, Strategy: paper_forward_v3
2026-08-22T02:14:23.777110Z  INFO paper_forward_test:    Total Fetch: 32 days | Train Window: 30 days | Re-Opt Frequency: 240 mins
2026-08-22T02:14:23.777222Z  INFO paper_forward_test:    Execution Mode: ExecutionMode::Paper (Sliding Window Memory: 4000 candles)
2026-08-22T02:14:23.777369Z  INFO paper_forward_test: ================================================================================
2026-08-22T02:14:24.158937Z  INFO bot_engine::data_engine::sqlite_impl: 📂 SQLite initialized at: examples/storage/paper_forward_v3.db
2026-08-22T02:14:24.159422Z  INFO paper_forward_test: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 32 ngày từ Binance...
2026-08-22T02:14:24.161434Z  INFO bot_engine::data_engine::binance_rest: 📥 [API] Dữ liệu lịch sử chưa đủ (0 / 46080). Bắt đầu đồng bộ lùi về quá khứ...
2026-08-22T02:14:40.454624Z  INFO bot_engine::data_engine::binance_rest: ✅ [API] Hoàn tất Backward Sync. Đã kéo thêm: 46035 nến.
2026-08-22T02:14:40.455376Z  INFO bot_engine::data_engine::binance_rest: 🔄 Tìm thấy nến cuối cùng. Đang tải dữ liệu lấp khoảng trống (Gap Filler) từ 1787364900000...
2026-08-22T02:14:40.567247Z  INFO bot_engine::data_engine::binance_rest: ✅ Hoàn tất đồng bộ lịch sử về hiện tại. Tổng số nến: 0
2026-08-22T02:14:40.612937Z  INFO paper_forward_test: 📊 [Data Summary] Tổng số nến: 46035 | Từ: 2026-07-21 02:15:00 UTC -> Đến: 2026-08-22 02:14:00 UTC
2026-08-22T02:14:40.613146Z  INFO paper_forward_test: --------------------------------------------------------------------------------
2026-08-22T02:14:40.613407Z  INFO paper_forward_test: 🧬 [PHASE 1 - INITIAL TRAIN] Khởi chạy Prospector GA Optimizer ban đầu...
2026-08-22T02:14:40.613719Z  INFO paper_forward_test:    📌 Dữ liệu Huấn Luyện (In-Sample): [2026-07-21 02:15:00 UTC] -> [2026-08-20 02:15:00 UTC]
2026-08-22T02:14:40.614001Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho ETHUSDT (Khoảng MS: 1784600100000 -> 1787192100000)...
2026-08-22T02:14:40.649016Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đã lấy 43158 nến từ Database.
2026-08-22T02:14:40.722849Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["3m", "5m", "15m", "1h", "4h"]
  [WARNING] MTF data for '3m' was not precomputed in BacktestBuilder! Please configure .timeframes(&[...]) when building Backtest.
  [WARNING] MTF data for '5m' was not precomputed in BacktestBuilder! Please configure .timeframes(&[...]) when building Backtest.
  [WARNING] MTF data for '15m' was not precomputed in BacktestBuilder! Please configure .timeframes(&[...]) when building Backtest.
  [WARNING] MTF data for '1h' was not precomputed in BacktestBuilder! Please configure .timeframes(&[...]) when building Backtest.
  [WARNING] MTF data for '4h' was not precomputed in BacktestBuilder! Please configure .timeframes(&[...]) when building Backtest.
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 3

thread '<unnamed>' (5528) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 5m not found in cache. Did you forget to precompute?
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\paper_forward_test.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine>
