    Finished `release` profile [optimized] target(s) in 1m 10s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
🆕 Starting fresh optimization run for Helios V3.
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.
Config Loaded:
  Workers: Pop=400, Gen=15
  Scout Method: lhs
  Exploration Strategy: random_restart
  Max Threads: Some(7)
Adding Initial Job... (111 parameters detected automatically from Struct)
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
[Prospector] ✅ Precomputation finished in 20.34ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 926.77ms
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 975.25ms
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
--- Prospector Finished in 1136.23s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 299.67     | -20.23     | 24       | 41.67      | 19.48      | -0.55      | 15.36      | 0.0650     | 17224281.2422
------------------------------------------------------------------------------------------------------------------------
2     | 267.56     | -26.22     | 25       | 40.00      | 20.10      | -2.26      | 9.31       | 0.0704     | 7162448.8269
------------------------------------------------------------------------------------------------------------------------
3     | 233.10     | -24.96     | 27       | 29.63      | 19.39      | -1.71      | 6.70       | 0.0512     | 1665354.1175
------------------------------------------------------------------------------------------------------------------------
4     | 196.43     | -18.25     | 27       | 51.85      | 22.49      | -2.51      | 4.83       | 0.2392     | 549512.1932
------------------------------------------------------------------------------------------------------------------------
5     | 172.41     | -20.24     | 67       | 52.24      | 13.27      | -2.99      | 2.76       | 0.1927     | 221759.2153
------------------------------------------------------------------------------------------------------------------------
6     | 161.14     | -11.42     | 32       | 59.38      | 7.68       | -1.49      | 5.36       | 0.3431     | 543225.2798
------------------------------------------------------------------------------------------------------------------------
7     | 135.40     | -22.55     | 36       | 25.00      | 18.66      | -1.38      | 4.72       | 0.1978     | 36884.8525
------------------------------------------------------------------------------------------------------------------------
8     | 131.76     | -11.87     | 32       | 56.25      | 7.68       | -2.23      | 4.09       | 0.4535     | 133230.3979
------------------------------------------------------------------------------------------------------------------------
9     | 126.02     | -16.52     | 67       | 52.24      | 13.27      | -2.99      | 2.76       | 0.3647     | 30629.0937
------------------------------------------------------------------------------------------------------------------------
10    | 125.31     | -23.79     | 40       | 25.00      | 18.66      | -1.38      | 4.19       | 0.1930     | 21357.9573
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 4534         | 12.96      | 541        | ABSORPTION
1     | LONG       | 5085         | -0.29      | 653        | ANOMALY
2     | LONG       | 5981         | 0.60       | 50         | RANGE_OSC
3     | LONG       | 6031         | -0.47      | 352        | RANGE_OSC
4     | LONG       | 6626         | 1.00       | 78         | RANGE_OSC
5     | SHORT      | 6704         | 10.12      | 4818       | ABSORPTION
6     | LONG       | 11522        | -0.20      | 264        | ABSORPTION
7     | SHORT      | 12030        | -0.24      | 296        | ABSORPTION
8     | LONG       | 12569        | 0.68       | 117        | RANGE_OSC
9     | LONG       | 12686        | -1.32      | 211        | ABSORPTION
10    | LONG       | 13140        | -0.54      | 339        | ABSORPTION
11    | LONG       | 13722        | -0.90      | 214        | ABSORPTION
12    | LONG       | 14179        | 32.95      | 13061      | TREND_PB
13    | SHORT      | 27240        | -0.46      | 1586       | ABSORPTION
14    | SHORT      | 29069        | 63.10      | 2870       | ABSORPTION
15    | LONG       | 31940        | 9.33       | 238        | ANOMALY
16    | LONG       | 32178        | -0.59      | 331        | ANOMALY
17    | LONG       | 32752        | -1.81      | 471        | RANGE_OSC
17    | LONG       | 32752        | -0.00      | 472        | RANGE_OSC
18    | LONG       | 33467        | 33.44      | 3360       | TREND_PB
19    | SHORT      | 36827        | -1.78      | 453        | ABSORPTION
20    | SHORT      | 37523        | -3.17      | 295        | ABSORPTION
21    | SHORT      | 38061        | -2.98      | 240        | ABSORPTION
22    | SHORT      | 38544        | 30.37      | 4697       | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>