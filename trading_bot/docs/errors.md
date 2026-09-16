PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example walk_forward_smart_seed
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
   Compiling bot_engine v0.1.0 (E:\Projects\Automations\trading_backtest\trading_bot\bot_engine)
    Finished `release` profile [optimized] target(s) in 1m 37s
     Running `target\release\examples\walk_forward_smart_seed.exe`
2026-09-15T01:58:39.753394Z  INFO walk_forward_smart_seed: ================================================================================
2026-09-15T01:58:39.753728Z  INFO walk_forward_smart_seed: 🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%
2026-09-15T01:58:39.755595Z  INFO walk_forward_smart_seed:    Symbol: ETHUSDT, Strategy: walk_forward_smart_seed
2026-09-15T01:58:39.755794Z  INFO walk_forward_smart_seed:    Total Fetch: 18 days | Train Window: 14 days
2026-09-15T01:58:39.756006Z  INFO walk_forward_smart_seed:    📌 [N MINS] Tần suất Re-Optimize GA:       240 phút
2026-09-15T01:58:39.756260Z  INFO walk_forward_smart_seed:    📌 [M MINS] Tần suất Single Backtest Eval:  240 phút
2026-09-15T01:58:39.756422Z  INFO walk_forward_smart_seed:    💰 [ACCOUNT] Initial Cash: $60.00 | Leverage: 20x | Comm: 0.0004
2026-09-15T01:58:39.756538Z  INFO walk_forward_smart_seed:    🧠 [MEMORY] Max Live Candles: 43100 (Đảm bảo 50+ nến MTF 4h)
2026-09-15T01:58:39.756611Z  INFO walk_forward_smart_seed:    🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới
2026-09-15T01:58:39.756683Z  INFO walk_forward_smart_seed:    🎯 [PAPER TRADING]: BẬT (Active)
2026-09-15T01:58:39.756755Z  INFO walk_forward_smart_seed: ================================================================================
2026-09-15T01:58:39.805865Z  INFO walk_forward_smart_seed: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 18 ngày từ Binance...
2026-09-15T01:58:40.506254Z  INFO walk_forward_smart_seed: 📊 [Data Summary] Tổng số nến: 26512 | Từ 2026-08-27 15:43 UTC -> Đến 2026-09-15 01:58 UTC
2026-09-15T01:58:40.506474Z  INFO walk_forward_smart_seed: --------------------------------------------------------------------------------
2026-09-15T01:58:40.506628Z  INFO walk_forward_smart_seed: 🧬 [INITIAL OPTIMIZE] Chạy GA Optimizer huấn luyện ban đầu trên 30 ngày...
2026-09-15T01:58:40.507296Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho ETHUSDT (Khoảng MS: 1787845380000 -> 1789054980000)...
2026-09-15T01:58:40.524963Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đã lấy 20142 nến từ Database. Bắt đầu tối ưu hóa...
2026-09-15T01:58:40.527459Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [WFE Split] In-Sample Train: 17262 nến | Out-Of-Sample Validation: 2880 nến (48h)
2026-09-15T01:58:40.552683Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["5m", "15m", "1h", "4h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 6
[Prospector] 🎲 Random Restart triggered (1/3). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 3.47s
[Prospector] 🗑️ Scout Job #2 pruned: Sterile Population (Variance < min_variance) (fitness=-100.00, variance=0.00000)
[Prospector] 🎲 Random Restart triggered (2/3). Scout Job #3 queued.
[Prospector] Scout Job #3 done in 1.58s
[Prospector] 🗑️ Scout Job #3 pruned: Sterile Population (Variance < min_variance) (fitness=-100.00, variance=0.00000)
[Prospector] 🎲 Random Restart triggered (3/3). Scout Job #5 queued.
[Prospector] Scout Job #5 done in 2.64s
[Prospector] 🗑️ Scout Job #5 pruned: Sterile Population (Variance < min_variance) (fitness=-100.00, variance=0.00000)
[Prospector] ✋ Random Restart limit reached (3/3). No more exploration.
--- Prospector Finished in 19.46s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 4 champions to helios_champions_results.json
2026-09-15T01:59:02.589961Z  INFO bot_engine::optimizer::prospector_runner: ⏱️ [OPTIMIZER DURATION] 22.08s | In-Sample Candles: 17262 | Best Return: 34.12% | Max DD: -5.97% | Trades: 22 (Win: 72.7%)
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["5m", "15m", "1h", "4h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
2026-09-15T01:59:02.612241Z  WARN bot_engine::optimizer::prospector_runner: 🚨 [WFE GATE: REJECTED - OVERFIT DETECTED] Symbol: ETHUSDT | Candidate DNA Overfit! In-Sample Return: 34.12% BUT OOS Val Return: -0.61% (Max DD: -8.58%, Trades: 6, Win Rate: 50.0%) -> REJECTED! Preserving active Champion DNA in DB & Actor to protect capital!
2026-09-15T01:59:02.615040Z  INFO walk_forward_smart_seed: 🏆 [INITIAL CHAMPION] In-Sample Return: 125.04% | Max DD: 0.00%
2026-09-15T01:59:02.615199Z  INFO walk_forward_smart_seed: --------------------------------------------------------------------------------
2026-09-15T01:59:02.615414Z  INFO walk_forward_smart_seed: 🚀 [ENGINE SETUP] Khởi tạo AssetActor ở chế độ ExecutionMode::Paper...
2026-09-15T01:59:02.806038Z  INFO bot_engine::controller::asset_actor: 🎬 Actor [ETHUSDT] started with strategy 'walk_forward_smart_seed'.
2026-09-15T01:59:02.806453Z  INFO bot_engine::controller::asset_actor: 📥 [ETHUSDT] Syncing history until timestamp 1789054980000 into Rolling Buffer...
2026-09-15T01:59:02.821040Z  INFO bot_engine::controller::asset_actor: ✅ [ETHUSDT] Synchronized 20142 historical 1m candles into Rolling Buffer (Total buffer: 20142).
2026-09-15T01:59:02.822223Z  INFO bot_engine::controller::asset_actor: 🧬 [ETHUSDT] Searching for best DNA in DB...
2026-09-15T01:59:02.822708Z  INFO bot_engine::controller::asset_actor: ✨ [ETHUSDT] DNA loaded successfully! (Fitness: 125.0411)
2026-09-15T01:59:02.823073Z  WARN helios::strategy::core_v3: ⚠️ [DNA LENGTH MISMATCH] Yêu cầu 121 gen nhưng DNA chỉ có 119 gen -> Bỏ qua nạp DNA không tương thích để chống crash!

thread 'tokio-rt-worker' (5052) panicked at E:\Projects\Automations\trading_backtest\strategies_backtests\helios\src\strategy\core_v3\config.rs:591:46:
index out of bounds: the len is 25 but the index is 25
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\walk_forward_smart_seed.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine>