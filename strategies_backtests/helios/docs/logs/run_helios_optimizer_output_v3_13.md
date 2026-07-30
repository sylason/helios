    Finished `release` profile [optimized] target(s) in 1m 49s
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
[Prospector] ✅ Precomputation finished in 21.78ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/3). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 1.38s
[Prospector] 🎲 Random Restart triggered (2/3). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 871.01ms
[Prospector] 🎲 Random Restart triggered (3/3). Scout Job #6 queued.
[Prospector] Scout Job #6 done in 846.91ms
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
--- Prospector Finished in 2593.03s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 339.95     | -20.18     | 33       | 60.61      | 17.93      | -2.97      | 5.25       | 0.1285     | 61023364.2481
------------------------------------------------------------------------------------------------------------------------
2     | 306.28     | -20.00     | 35       | 57.14      | 17.82      | -2.94      | 4.66       | 0.1201     | 23432183.3770
------------------------------------------------------------------------------------------------------------------------
3     | 300.08     | -20.00     | 35       | 57.14      | 17.82      | -2.94      | 4.49       | 0.1191     | 19550467.0132
------------------------------------------------------------------------------------------------------------------------
4     | 269.37     | -20.00     | 35       | 57.14      | 17.82      | -2.94      | 4.02       | 0.1111     | 7232995.2861
------------------------------------------------------------------------------------------------------------------------
5     | 257.56     | -14.84     | 24       | 45.83      | 22.08      | -2.17      | 7.69       | 0.3009     | 8216984.8803
------------------------------------------------------------------------------------------------------------------------
6     | 254.38     | -15.02     | 25       | 44.00      | 21.79      | -2.25      | 7.02       | 0.2999     | 7355751.8247
------------------------------------------------------------------------------------------------------------------------
7     | 248.83     | -16.82     | 29       | 55.17      | 16.93      | -1.59      | 7.82       | 0.1072     | 4925004.0604
------------------------------------------------------------------------------------------------------------------------
8     | 247.60     | -14.78     | 24       | 45.83      | 21.79      | -2.25      | 7.55       | 0.3122     | 5940896.5663
------------------------------------------------------------------------------------------------------------------------
9     | 241.01     | -16.82     | 30       | 50.00      | 17.13      | -1.48      | 7.05       | 0.0880     | 3556038.9000
------------------------------------------------------------------------------------------------------------------------
10    | 221.81     | -20.00     | 38       | 55.26      | 17.82      | -4.15      | 3.09       | 0.1192     | 1222894.4839
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | LONG       | 2144         | -0.51      | 1267       | ABSORPTION
1     | SHORT      | 3710         | -0.23      | 446        | ABSORPTION
2     | SHORT      | 4456         | 12.08      | 617        | ABSORPTION
3     | SHORT      | 5077         | -1.06      | 2184       | MOMENTUM_IGN
4     | SHORT      | 7560         | -0.14      | 375        | ABSORPTION
5     | SHORT      | 8238         | 10.38      | 2363       | ABSORPTION
6     | LONG       | 10601        | 1.69       | 1803       | ABSORPTION
7     | SHORT      | 12404        | -2.42      | 860        | ABSORPTION
8     | LONG       | 13721        | -2.74      | 404        | ABSORPTION
9     | LONG       | 14696        | 2.40       | 2143       | MOMENTUM_IGN
10    | SHORT      | 16839        | -2.08      | 651        | ABSORPTION
11    | LONG       | 17788        | 16.57      | 3126       | ABSORPTION
12    | LONG       | 21055        | 1.67       | 533        | ABSORPTION
13    | SHORT      | 21588        | -1.11      | 983        | ABSORPTION
14    | LONG       | 22869        | 1.60       | 1186       | ABSORPTION
15    | LONG       | 24457        | 8.15       | 2364       | ABSORPTION
16    | LONG       | 26821        | 0.51       | 338        | ABSORPTION
17    | SHORT      | 27159        | 10.65      | 1457       | ABSORPTION
18    | SHORT      | 28616        | -2.63      | 201        | ABSORPTION
19    | SHORT      | 29166        | 10.24      | 1508       | ABSORPTION
20    | SHORT      | 30674        | 65.86      | 1265       | ABSORPTION
21    | SHORT      | 31939        | -8.66      | 13         | MOMENTUM_IGN
22    | SHORT      | 32724        | 4.74       | 1033       | ABSORPTION
23    | LONG       | 33757        | 38.27      | 1913       | ABSORPTION
24    | SHORT      | 35670        | -16.80     | 123        | ABSORPTION
25    | LONG       | 36090        | 5.60       | 77         | RANGE_OSC
26    | LONG       | 36167        | 2.09       | 75         | RANGE_OSC
27    | SHORT      | 36242        | 35.62      | 1128       | ABSORPTION
28    | LONG       | 37370        | 6.11       | 1244       | ABSORPTION
29    | LONG       | 38614        | -22.67     | 202        | ABSORPTION
30    | SHORT      | 39545        | -8.71      | 374        | ABSORPTION
31    | SHORT      | 40218        | 31.65      | 1493       | ABSORPTION
32    | LONG       | 41711        | 7.82       | 1530       | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>