PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios> cargo run --release --example v3_run_helios_optimize
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
warning: constant `RISK_SIZE_RATIO` is never used
  --> examples\common.rs:14:11
   |
14 | pub const RISK_SIZE_RATIO: f64 = 0.2;  // Vào lệnh 1/5 tổng tài sản
   |           ^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `MIN_TRADE_AMOUNT` is never used
  --> examples\common.rs:15:11
   |
15 | pub const MIN_TRADE_AMOUNT: f64 = 10.0; // Tiền vào lệnh tối thiểu $10
   |           ^^^^^^^^^^^^^^^^

warning: `helios` (example "v3_run_helios_optimize") generated 2 warnings
    Finished `release` profile [optimized] target(s) in 46.20s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
🆕 Starting fresh optimization run for Helios V3.
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.
Config Loaded:
  Workers: Pop=100, Gen=7
  Scout Method: lhs
  Exploration Strategy: random_restart
  Max Threads: Some(3)
Adding Initial Job... (109 parameters detected automatically from Struct)
>>> STARTING HELIOS V3 EVOLUTION <<<
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["3m", "5m", "15m", "1h", "4h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 3
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 631.81ms
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] ✋ Random Restart limit reached (2/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (3/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (4/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (5/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (6/2). No more exploration.
[Prospector] Scout Job #4 done in 30.56s
[Prospector] ✋ Random Restart limit reached (7/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (8/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (9/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (10/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (11/2). No more exploration.
--- Prospector Finished in 135.37s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 211.28     | -20.49     | 36       | 33.33      | 15.82      | -2.16      | 4.52       | 0.0690     | 1508893.6728
------------------------------------------------------------------------------------------------------------------------
2     | 198.34     | -20.49     | 38       | 31.58      | 16.12      | -2.16      | 4.03       | 0.0629     | 885062.4657
------------------------------------------------------------------------------------------------------------------------
3     | 174.52     | -18.48     | 28       | 32.14      | 14.90      | -2.36      | 4.95       | 0.2599     | 362065.2083
------------------------------------------------------------------------------------------------------------------------
4     | 172.70     | -18.48     | 29       | 31.03      | 14.90      | -2.61      | 4.60       | 0.2580     | 329957.9494
------------------------------------------------------------------------------------------------------------------------
5     | 169.08     | -20.50     | 32       | 37.50      | 16.12      | -1.95      | 5.75       | 0.0722     | 291000.4024
------------------------------------------------------------------------------------------------------------------------
6     | 164.93     | -22.25     | 19       | 52.63      | 20.77      | -1.82      | 4.87       | 0.1679     | 149281.5848
------------------------------------------------------------------------------------------------------------------------
7     | 161.03     | -22.62     | 19       | 52.63      | 20.77      | -1.99      | 4.66       | 0.1737     | 122983.2099
------------------------------------------------------------------------------------------------------------------------
8     | 141.69     | -22.63     | 17       | 58.82      | 20.77      | -1.99      | 4.48       | 0.2610     | 50059.0032
------------------------------------------------------------------------------------------------------------------------
9     | 126.71     | -19.21     | 38       | 23.68      | 14.90      | -3.55      | 2.82       | 0.2665     | 30054.9870
------------------------------------------------------------------------------------------------------------------------
10    | 125.10     | -18.06     | 34       | 35.29      | 15.76      | -2.93      | 4.13       | 0.1374     | 32867.8278
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 4532         | 12.26      | 543        | TREND_PB
1     | SHORT      | 5075         | -4.36      | 91         | MOMENTUM_IGN
2     | LONG       | 5237         | -1.04      | 194        | TREND_PB
3     | LONG       | 5432         | -0.96      | 334        | TREND_PB
4     | LONG       | 5832         | -0.23      | 583        | TREND_PB
5     | LONG       | 6416         | -0.20      | 613        | TREND_PB
6     | SHORT      | 7239         | -1.01      | 340        | TREND_PB
7     | LONG       | 7659         | -1.81      | 225        | TREND_PB
8     | LONG       | 7924         | -0.16      | 856        | TREND_PB
9     | SHORT      | 8781         | 8.94       | 2093       | TREND_PB
10    | SHORT      | 10875        | -0.30      | 482        | TREND_PB
11    | LONG       | 11358        | 3.60       | 977        | TREND_PB
12    | SHORT      | 12399        | -0.24      | 279        | TREND_PB
13    | LONG       | 12704        | -0.94      | 229        | TREND_PB
14    | LONG       | 13115        | -0.40      | 344        | TREND_PB
15    | LONG       | 13690        | -0.59      | 191        | TREND_PB
16    | LONG       | 14196        | 4.32       | 1365       | TREND_PB
17    | LONG       | 15630        | -0.33      | 253        | ABSORPTION
18    | LONG       | 17626        | 7.87       | 583        | RANGE_OSC
19    | LONG       | 18209        | 12.74      | 3305       | ABSORPTION
20    | LONG       | 21593        | -0.65      | 525        | TREND_PB
21    | LONG       | 22494        | -2.00      | 561        | RANGE_OSC
22    | LONG       | 23230        | -0.25      | 266        | RANGE_OSC
23    | LONG       | 23497        | 4.17       | 133        | RANGE_OSC
24    | LONG       | 23630        | -1.10      | 491        | ABSORPTION
25    | LONG       | 24931        | 9.28       | 1931       | ABSORPTION
26    | SHORT      | 28621        | -1.49      | 264        | ABSORPTION
27    | SHORT      | 31503        | 46.50      | 436        | ABSORPTION
28    | LONG       | 31940        | 9.02       | 237        | ANOMALY
29    | SHORT      | 32219        | -8.44      | 192        | TREND_PB
30    | LONG       | 32412        | -1.42      | 507        | TREND_PB
31    | LONG       | 33440        | 43.31      | 2423       | TREND_PB
32    | SHORT      | 37409        | -1.19      | 338        | ABSORPTION
33    | LONG       | 38375        | -10.52     | 193        | ABSORPTION
34    | SHORT      | 39896        | -2.29      | 195        | ABSORPTION
35    | LONG       | 41718        | 6.68       | 1523       | TREND_PB
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>