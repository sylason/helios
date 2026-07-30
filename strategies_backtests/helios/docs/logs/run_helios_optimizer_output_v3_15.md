    Finished `release` profile [optimized] target(s) in 2m 00s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
🆕 Starting fresh optimization run for Helios V3.
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.
Config Loaded:
  Workers: Pop=250, Gen=25
  Scout Method: lhs
  Exploration Strategy: random_restart
  Max Threads: Some(7)
Adding Initial Job... (101 parameters detected automatically from Struct)
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
[Prospector] ✅ Precomputation finished in 21.33ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/3). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 989.77ms
[Prospector] 🎲 Random Restart triggered (2/3). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 897.73ms
[Prospector] 🎲 Random Restart triggered (3/3). Scout Job #6 queued.
[Prospector] Scout Job #6 done in 1.43s
[Prospector] ✋ Random Restart limit reached (3/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (4/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (5/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (6/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (7/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (8/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (9/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (10/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (11/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (12/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (13/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (14/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (15/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (16/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (17/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (18/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (19/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (20/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (21/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (22/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (23/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (24/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (25/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (26/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (27/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (28/3). No more exploration.
[Prospector] ✋ Random Restart limit reached (29/3). No more exploration.
--- Prospector Finished in 1744.98s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 338.18     | -13.34     | 50       | 62.00      | 13.87      | -2.68      | 4.99       | 0.4440     | 129238478.3949
------------------------------------------------------------------------------------------------------------------------
2     | 314.92     | -21.25     | 50       | 62.00      | 13.87      | -3.55      | 4.14       | 0.3060     | 48516298.5601
------------------------------------------------------------------------------------------------------------------------
3     | 287.05     | -18.71     | 33       | 60.61      | 11.05      | -1.30      | 7.62       | 0.1823     | 50591021.9703
------------------------------------------------------------------------------------------------------------------------
4     | 260.03     | -18.61     | 34       | 55.88      | 11.07      | -1.30      | 5.90       | 0.1684     | 14376036.1480
------------------------------------------------------------------------------------------------------------------------
5     | 247.63     | -18.87     | 34       | 55.88      | 11.02      | -1.31      | 5.59       | 0.1708     | 9309943.3973
------------------------------------------------------------------------------------------------------------------------
6     | 242.76     | -21.24     | 52       | 55.77      | 13.74      | -3.55      | 3.27       | 0.2938     | 4544053.6542
------------------------------------------------------------------------------------------------------------------------
7     | 237.33     | -14.21     | 26       | 65.38      | 17.60      | -2.47      | 7.05       | 0.0975     | 4574209.2974
------------------------------------------------------------------------------------------------------------------------
8     | 234.71     | -14.39     | 165      | 41.82      | 7.36       | -1.15      | 2.42       | 0.3368     | 3385903.4549
------------------------------------------------------------------------------------------------------------------------
9     | 221.30     | -17.21     | 34       | 52.94      | 11.02      | -1.77      | 4.76       | 0.2582     | 2798852.3213
------------------------------------------------------------------------------------------------------------------------
10    | 205.57     | -13.05     | 165      | 40.00      | 7.36       | -1.02      | 2.29       | 0.3710     | 1225870.7044
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 4015         | 10.82      | 1058       | ANOMALY
1     | SHORT      | 5073         | 3.38       | 2          | MOMENTUM_IGN
2     | SHORT      | 5075         | 0.07       | 2          | MOMENTUM_IGN
3     | SHORT      | 5077         | -1.91      | 14         | MOMENTUM_IGN
4     | LONG       | 5304         | -0.23      | 434        | ANOMALY
5     | LONG       | 5897         | 0.03       | 436        | ANOMALY
6     | LONG       | 6333         | 0.62       | 877        | ANOMALY
7     | SHORT      | 7210         | -0.64      | 560        | ABSORPTION
7     | SHORT      | 7210         | -0.00      | 561        | ABSORPTION
8     | SHORT      | 7968         | 6.20       | 1624       | ANOMALY
9     | SHORT      | 9592         | 2.22       | 199        | ANOMALY
10    | SHORT      | 9791         | 4.58       | 379        | ABSORPTION
11    | LONG       | 10267        | -1.47      | 842        | ABSORPTION
12    | LONG       | 11268        | 1.96       | 39         | RANGE_OSC
13    | LONG       | 11307        | 0.50       | 1162       | ABSORPTION
14    | LONG       | 12469        | 0.97       | 604        | ANOMALY
15    | SHORT      | 13073        | -0.67      | 517        | ABSORPTION
16    | LONG       | 13750        | 10.19      | 2839       | ABSORPTION
17    | LONG       | 16589        | -0.99      | 438        | ANOMALY
18    | LONG       | 17187        | 0.59       | 439        | ANOMALY
19    | LONG       | 17626        | 14.12      | 2405       | ABSORPTION
20    | LONG       | 20031        | 8.72       | 879        | ANOMALY
21    | SHORT      | 20910        | -3.32      | 611        | ABSORPTION
22    | SHORT      | 21680        | -0.36      | 930        | ABSORPTION
23    | SHORT      | 22774        | -0.24      | 809        | ABSORPTION
24    | SHORT      | 23742        | 4.14       | 1488       | ANOMALY
25    | LONG       | 25231        | 12.67      | 1479       | ANOMALY
26    | LONG       | 26710        | -2.06      | 447        | ABSORPTION
27    | SHORT      | 27316        | -3.13      | 1131       | ABSORPTION
28    | LONG       | 28606        | -0.47      | 709        | ANOMALY
29    | SHORT      | 29478        | 6.64       | 1076       | ABSORPTION
30    | LONG       | 30554        | -3.17      | 567        | ABSORPTION
31    | SHORT      | 31303        | 50.26      | 630        | ABSORPTION
32    | LONG       | 31933        | -13.47     | 1          | ANOMALY
33    | LONG       | 31934        | 7.49       | 2          | ANOMALY
34    | LONG       | 31936        | -11.98     | 1          | ANOMALY
35    | LONG       | 31937        | 2.74       | 2          | ANOMALY
36    | SHORT      | 31939        | 0.42       | 2          | MOMENTUM_IGN
37    | LONG       | 31941        | -6.07      | 2          | ANOMALY
38    | LONG       | 32202        | 3.69       | 1049       | ANOMALY
39    | SHORT      | 33251        | -0.38      | 491        | ABSORPTION
40    | LONG       | 33903        | 31.25      | 1767       | ABSORPTION
41    | LONG       | 35670        | 9.58       | 937        | ANOMALY
42    | SHORT      | 36607        | 15.29      | 763        | ABSORPTION
43    | LONG       | 37370        | 4.20       | 1244       | ABSORPTION
44    | SHORT      | 38614        | 1.06       | 1306       | ABSORPTION
45    | SHORT      | 39920        | 36.94      | 1513       | ANOMALY
46    | SHORT      | 41433        | -9.75      | 535        | ANOMALY
47    | LONG       | 42127        | 7.91       | 485        | RANGE_OSC
48    | SHORT      | 42612        | 3.94       | 629        | ANOMALY
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>