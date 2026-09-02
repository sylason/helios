[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
2026-09-02T16:17:50.009516Z  INFO walk_forward_smart_seed: 📊 [SINGLE BACKTEST EVAL #0012] Time: 2026-07-06 03:21 | Window: 30D (43158 candles) | Return: 315.93% | Max DD: -14.21% | Trades: 47 | Win Rate: 46.81% | Duration: 123.99ms | Paper Trade Active: true
2026-09-02T16:17:50.012128Z  INFO walk_forward_smart_seed: 🧬 [RE-OPTIMIZE TRIGGERED #0015] Cửa sổ tối ưu GA: [2026-06-06 03:21 -> 2026-07-06 03:21]
2026-09-02T16:17:50.012396Z  INFO walk_forward_smart_seed: 🌱 [SMART SEEDING 0.5%] Đã chọn Rank 1 Champion DNA (110 chiều) làm hạt giống tinh nhuệ duy nhất.
2026-09-02T16:17:50.012625Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho ETHUSDT (Khoảng MS: 1780716060000 -> 1783308060000)...
2026-09-02T16:17:50.052690Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đã lấy 43158 nến từ Database. Bắt đầu tối ưu hóa...
2026-09-02T16:17:50.219902Z  INFO bot_engine::optimizer::prospector_runner: 🌱 [Smart Seeding] Tiêm 1 hạt giống Rank 1 Champion từ chu kỳ trước vào Generation 0.
[Prospector] 🌱 Smart Seeding: Warm Starting with Top 1 champion from previous sessions.
2026-09-02T16:17:50.260783Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["5m", "15m", "1h", "4h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 3
[Prospector] 🎲 Random Restart triggered (1/5). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 6.73s
[Prospector] 🌟 Scout Job #2 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=0.02, variance=2560.52945)
[Prospector] 🎲 Random Restart triggered (2/5). Scout Job #4 queued.
[Prospector] 🎲 Random Restart triggered (3/5). Scout Job #5 queued.
[Prospector] Scout Job #5 done in 8.45s
[Prospector] 🌟 Scout Job #5 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=0.63, variance=3223.14706)
[Prospector] 🎲 Random Restart triggered (4/5). Scout Job #7 queued.
[Prospector] 🎲 Random Restart triggered (5/5). Scout Job #9 queued.
[Prospector] ✋ Random Restart limit reached (5/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (6/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (7/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (8/5). No more exploration.
[Prospector] Scout Job #7 done in 13.26s
[Prospector] 🌟 Scout Job #7 PROMOTED to Hotspot: Warmup Period (Scout history <= warmup_jobs) (fitness=3.62, variance=2792.48595)
[Prospector] Scout Job #9 done in 21.50s
[Prospector] 🗑️ Scout Job #9 pruned: Below Quantile Rank (Not in Top-K) (fitness=0.53, variance=5407.47974)
[Prospector] Scout Job #4 done in 6.66s
[Prospector] 🗑️ Scout Job #4 pruned: Below Quantile Rank (Not in Top-K) (fitness=0.81, variance=5415.96129)
[Prospector] ✋ Random Restart limit reached (9/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (10/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (11/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (12/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (13/5). No more exploration.
[Prospector] ✋ Random Restart limit reached (14/5). No more exploration.
--- Prospector Finished in 281.83s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
2026-09-02T16:22:41.390467Z  INFO bot_engine::optimizer::prospector_runner: ⏱️ [OPTIMIZER DURATION] 291.38s | Candles: 43158 | Best Return: 340.44% | Max DD: -14.21% | Trades: 50 (Win: 46.0%)