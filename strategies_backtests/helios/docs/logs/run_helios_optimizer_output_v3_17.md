    Finished `release` profile [optimized] target(s) in 5.86s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
🆕 Starting fresh optimization run for Helios V3.
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.
Config Loaded:
  Workers: Pop=80, Gen=10
  Scout Method: lhs
  Exploration Strategy: random_restart
  Max Threads: Some(7)
Adding Initial Job... (102 parameters detected automatically from Struct)
>>> STARTING HELIOS V3 EVOLUTION <<<
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔥 Precomputing MTF data for Timeframes: ["3m", "5m", "15m", "1h", "4h"]
  -> Resampling: 3m
  -> Creating Lookup Table: 3m
  -> Resampling: 5m
  -> Creating Lookup Table: 5m
  -> Resampling: 15m
  -> Creating Lookup Table: 15m
  -> Resampling: 1h
  -> Creating Lookup Table: 1h
  -> Resampling: 4h
  -> Creating Lookup Table: 4h
[Prospector] ✅ Precomputation finished in 19.90ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 880.08ms
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 930.92ms
[Prospector] ✋ Random Restart limit reached (2/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (3/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (4/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (5/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (6/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (7/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (8/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (9/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (10/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (11/2). No more exploration.
--- Prospector Finished in 174.57s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 249.97     | -19.38     | 19       | 57.89      | 21.92      | -2.57      | 8.02       | 0.0940     | 3983046.8433
------------------------------------------------------------------------------------------------------------------------
2     | 245.73     | -25.80     | 73       | 42.47      | 12.88      | -3.32      | 3.05       | 0.1345     | 4018611.5406
------------------------------------------------------------------------------------------------------------------------
3     | 206.70     | -19.43     | 23       | 52.17      | 22.46      | -2.57      | 5.19       | 0.1160     | 859046.4542
------------------------------------------------------------------------------------------------------------------------
4     | 196.08     | -25.80     | 75       | 38.67      | 12.88      | -3.32      | 2.60       | 0.1312     | 605958.9368
------------------------------------------------------------------------------------------------------------------------
5     | 185.62     | -18.16     | 15       | 33.33      | 18.78      | -1.08      | 7.83       | 0.2393     | 670054.0843
------------------------------------------------------------------------------------------------------------------------
6     | 178.46     | -18.28     | 16       | 43.75      | 18.87      | -1.08      | 7.65       | 0.2061     | 310336.7739
------------------------------------------------------------------------------------------------------------------------
7     | 113.64     | -17.41     | 15       | 40.00      | 18.23      | -2.50      | 6.43       | 0.2033     | 14902.0881
------------------------------------------------------------------------------------------------------------------------
8     | 111.39     | -38.57     | 96       | 27.08      | 16.59      | -3.26      | 1.87       | 0.0569     | 8037.2108
------------------------------------------------------------------------------------------------------------------------
9     | 109.98     | -26.94     | 27       | 40.74      | 22.14      | -2.57      | 2.77       | 0.0968     | 8718.4947
------------------------------------------------------------------------------------------------------------------------
10    | 103.13     | -19.63     | 15       | 40.00      | 18.23      | -3.50      | 4.93       | 0.1901     | 7212.0394
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 4106         | -0.81      | 242        | ANOMALY
1     | SHORT      | 4673         | 11.75      | 402        | ABSORPTION
2     | SHORT      | 5075         | -1.18      | 717        | MOMENTUM_IGN
3     | SHORT      | 6118         | -1.17      | 586        | REVERSAL
4     | LONG       | 7029         | -1.72      | 774        | ANOMALY
5     | SHORT      | 8270         | 9.95       | 2964       | ANOMALY
6     | LONG       | 11234        | 9.11       | 5856       | ABSORPTION
7     | SHORT      | 17090        | -0.47      | 888        | ABSORPTION
8     | SHORT      | 18304        | -1.87      | 255        | ANOMALY
9     | LONG       | 18884        | 6.34       | 4046       | ABSORPTION
10    | LONG       | 22930        | 1.31       | 1718       | ANOMALY
11    | LONG       | 24648        | 7.56       | 2555       | ANOMALY
12    | SHORT      | 27203        | 57.72      | 4736       | ABSORPTION
13    | LONG       | 31940        | 7.60       | 238        | ANOMALY
14    | LONG       | 32426        | -11.25     | 930        | ABSORPTION
15    | LONG       | 33686        | 35.38      | 3077       | ABSORPTION
16    | SHORT      | 36763        | -7.44      | 641        | ABSORPTION
17    | SHORT      | 37745        | 29.00      | 4865       | ABSORPTION
18    | LONG       | 42610        | 0.18       | 631        | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>