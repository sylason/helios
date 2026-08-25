PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example paper_forward_test
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
  --> src\controller\asset_actor.rs:36:5
   |
29 | pub struct AssetActor {
   |            ---------- fields in this struct
...
36 |     monitor: Arc<WeightMonitor>,
   |     ^^^^^^^
...
46 |     market_filter: crate::models::market_info::MarketFilter,
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
    Finished `release` profile [optimized] target(s) in 8.72s
     Running `target\release\examples\paper_forward_test.exe`
2026-08-24T04:43:51.048432Z  INFO paper_forward_test: ================================================================================
2026-08-24T04:43:51.049192Z  INFO paper_forward_test: 🧪 BẮT ĐẦU THỬ NGHIỆM FULL ENGINE PAPER FORWARD TEST (15-MIN RE-OPT)
2026-08-24T04:43:51.049357Z  INFO paper_forward_test:    Symbol: ETHUSDT, Strategy: paper_forward_v3
2026-08-24T04:43:51.049533Z  INFO paper_forward_test:    Total Fetch: 77 days | Train Window: 62 days | Re-Opt Frequency: 240 mins
2026-08-24T04:43:51.049704Z  INFO paper_forward_test:    Execution Mode: ExecutionMode::Paper (Sliding Window Memory: 86400 candles)
2026-08-24T04:43:51.049861Z  INFO paper_forward_test: ================================================================================
2026-08-24T04:43:51.626396Z  INFO bot_engine::data_engine::sqlite_impl: 📂 SQLite initialized at: examples/storage/paper_forward_v3.db
2026-08-24T04:43:51.626575Z  INFO paper_forward_test: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 77 ngày từ Binance...
2026-08-24T04:43:51.669604Z  INFO bot_engine::data_engine::binance_rest: 📥 [API] Dữ liệu lịch sử chưa đủ (0 / 110880). Bắt đầu đồng bộ lùi về quá khứ...
2026-08-24T04:44:30.622567Z  INFO bot_engine::data_engine::binance_rest: ✅ [API] Hoàn tất Backward Sync. Đã kéo thêm: 110771 nến.
2026-08-24T04:44:30.631120Z  INFO bot_engine::data_engine::binance_rest: 🔄 Tìm thấy nến cuối cùng. Đang tải dữ liệu lấp khoảng trống (Gap Filler) từ 1787546640000...
2026-08-24T04:44:30.840309Z  INFO bot_engine::data_engine::binance_rest: ✅ Hoàn tất đồng bộ lịch sử về hiện tại. Tổng số nến: 1
2026-08-24T04:44:30.955567Z  INFO paper_forward_test: 📊 [Data Summary] Tổng số nến: 110772 | Từ: 2026-06-08 04:44:00 UTC -> Đến: 2026-08-24 04:44:00 UTC
2026-08-24T04:44:30.955898Z  INFO paper_forward_test: --------------------------------------------------------------------------------
2026-08-24T04:44:30.956141Z  INFO paper_forward_test: 🧬 [PHASE 1 - INITIAL TRAIN] Khởi chạy Prospector GA Optimizer ban đầu...
2026-08-24T04:44:30.956353Z  INFO paper_forward_test:    📌 Dữ liệu Huấn Luyện (In-Sample): [2026-06-08 04:44:00 UTC] -> [2026-08-09 04:44:00 UTC]
2026-08-24T04:44:30.963925Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho ETHUSDT (Khoảng MS: 1780893840000 -> 1786250640000)...
2026-08-24T04:44:31.035132Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đã lấy 89193 nến từ Database.
2026-08-24T04:44:31.507455Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["5m", "15m", "1h", "6h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 3
[Prospector] 🎲 Random Restart triggered (1/3). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 4.50s
[Prospector] 🌟 Scout Job #2 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=8.23, variance=586634.96868)
[Prospector] 🎲 Random Restart triggered (2/3). Scout Job #4 queued.
[Prospector] 🎲 Random Restart triggered (3/3). Scout Job #6 queued.
[Prospector] ✋ Random Restart limit reached (3/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (4/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (5/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (6/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (7/3). No more exploration.
[Prospector] Scout Job #6 done in 2.03s
[Prospector] 🌟 Scout Job #6 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=4.79, variance=464084.66802)
[Prospector] ✋ Random Restart limit reached (8/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (9/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (10/3). No more exploration.
[Prospector] Scout Job #4 done in 3.68s
[Prospector] 🌟 Scout Job #4 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=-50.00, variance=739254.32099)
[Prospector] ✋ Random Restart limit reached (11/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (12/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (13/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (14/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (15/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (16/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (17/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (18/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (19/3). No more exploration.
--- Prospector Finished in 127.20s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
2026-08-24T04:46:40.817401Z  INFO bot_engine::optimizer::prospector_runner: 🏆 [Optimizer] TOP 1 FOUND! Return: 68.82%, Max DD: -26.86%, Trades: 114
2026-08-24T04:46:41.000148Z  INFO bot_engine::data_engine::sqlite_impl: 🧬 [ETHUSDT] DNA for strategy 'paper_forward_v3' saved to DB.
2026-08-24T04:46:41.000487Z  INFO bot_engine::optimizer::prospector_runner: 💾 [Optimizer] Đã lưu DNA mới nhất vào Database.
2026-08-24T04:46:41.000711Z  INFO bot_engine::optimizer::prospector_runner: 📢 [Optimizer] Đã phát tín hiệu (Event-Driven) mang DNA mới tới các Actor.
2026-08-24T04:46:41.002288Z  INFO paper_forward_test: --------------------------------------------------------------------------------
2026-08-24T04:46:41.002549Z  INFO paper_forward_test: 🚀 [PHASE 2 - ENGINE SETUP] Khởi tạo EventBus & AssetActor (ExecutionMode::Paper)...
2026-08-24T04:46:41.470887Z  INFO bot_engine::controller::asset_actor: 📥 [ETHUSDT] Syncing history until timestamp 1786250640000 into Rolling Buffer...
2026-08-24T04:46:41.598155Z  INFO bot_engine::controller::asset_actor: ✅ [ETHUSDT] Synchronized 47473 historical 1m candles into Rolling Buffer (Total buffer: 43200).
2026-08-24T04:46:41.600027Z  INFO paper_forward_test: --------------------------------------------------------------------------------
2026-08-24T04:46:41.600050Z  INFO paper_forward_test: 📢 [UI_STATUS] [ETHUSDT] Trading -> Hệ thống Helios đã sẵn sàng!
2026-08-24T04:46:41.600220Z  INFO paper_forward_test: 🔄 [PHASE 3 - FORWARD EXECUTION] Khởi chạy luồng mô phỏng đồng bộ & Re-Optimize ngầm...
2026-08-24T04:46:41.600411Z  INFO paper_forward_test: --------------------------------------------------------------------------------
2026-08-24T04:46:41.827404Z  INFO paper_forward_test: 📊 Số nến Out-Of-Sample sẽ bơm: 21579 nến (Từ 2026-08-09 04:45 đến 2026-08-24 04:44:00 UTC)
2026-08-24T04:46:41.827737Z  INFO bot_engine::controller::asset_actor: ✨ [ETHUSDT] Đã cập nhật bộ não Helios với DNA mới từ Re-Optimizer! (Fitness: 68.8214)

thread 'main' (12448) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 6h not found in cache. Did you forget to precompute?
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\paper_forward_test.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine>