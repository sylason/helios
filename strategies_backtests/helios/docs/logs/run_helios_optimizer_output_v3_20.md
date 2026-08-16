    Finished `release` profile [optimized] target(s) in 8.70s
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
Adding Initial Job... (109 parameters detected automatically from Struct)
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
[Prospector] ✅ Precomputation finished in 50.47ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 4.38s
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 684.54ms
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
--- Prospector Finished in 601.23s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 138.74     | -27.87     | 32       | 50.00      | 12.89      | -2.06      | 3.01       | 0.1490     | 46636.7172
------------------------------------------------------------------------------------------------------------------------
2     | 123.16     | -22.17     | 23       | 43.48      | 9.08       | -2.49      | 4.65       | 0.4629     | 31837.6813
------------------------------------------------------------------------------------------------------------------------
3     | 106.09     | -31.07     | 35       | 42.86      | 12.71      | -2.06      | 2.59       | 0.1524     | 8325.6801
------------------------------------------------------------------------------------------------------------------------
4     | 105.84     | -22.17     | 21       | 42.86      | 9.08       | -2.49      | 4.29       | 0.4380     | 11103.8803
------------------------------------------------------------------------------------------------------------------------
5     | 85.44      | -15.62     | 35       | 22.86      | 18.27      | -1.98      | 3.42       | 0.4168     | 2945.8170
------------------------------------------------------------------------------------------------------------------------
6     | 79.58      | -18.82     | 21       | 38.10      | 9.08       | -1.68      | 2.99       | 0.4199     | 1940.7128
------------------------------------------------------------------------------------------------------------------------
7     | 77.79      | -24.99     | 35       | 42.86      | 12.71      | -2.06      | 2.59       | 0.3421     | 1885.7523
------------------------------------------------------------------------------------------------------------------------
8     | 73.01      | -14.49     | 33       | 21.21      | 18.27      | -1.98      | 3.30       | 0.4750     | 1402.1506
------------------------------------------------------------------------------------------------------------------------
9     | 54.30      | -16.08     | 38       | 18.42      | 17.73      | -1.98      | 2.48       | 0.4636     | 335.1474
------------------------------------------------------------------------------------------------------------------------
10    | 54.07      | -7.72      | 22       | 50.00      | 21.32      | -1.98      | 4.44       | 1.0475     | 702.4861
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 6878         | 1.18       | 1055       | ABSORPTION
1     | SHORT      | 7933         | 6.20       | 1959       | ABSORPTION
2     | SHORT      | 9966         | 1.61       | 617        | ABSORPTION
3     | LONG       | 10787        | -4.19      | 324        | ABSORPTION
4     | LONG       | 11353        | 2.78       | 1051       | ABSORPTION
5     | LONG       | 12869        | -0.75      | 990        | ABSORPTION
6     | LONG       | 13895        | 7.98       | 2694       | ABSORPTION
7     | SHORT      | 16723        | 0.09       | 768        | ABSORPTION
8     | LONG       | 17794        | 9.05       | 2237       | ABSORPTION
9     | SHORT      | 20067        | -5.11      | 174        | ABSORPTION
10    | LONG       | 20477        | -0.37      | 500        | ABSORPTION
11    | SHORT      | 21123        | -3.82      | 401        | ABSORPTION
12    | SHORT      | 21698        | -0.25      | 1096       | ABSORPTION
12    | SHORT      | 21698        | -0.00      | 1097       | ABSORPTION
13    | SHORT      | 23045        | -2.48      | 556        | ABSORPTION
14    | LONG       | 23705        | -0.13      | 349        | ABSORPTION
15    | SHORT      | 24166        | 0.01       | 741        | ABSORPTION
16    | LONG       | 24946        | 6.78       | 1888       | ABSORPTION
17    | SHORT      | 26948        | 8.68       | 1668       | ABSORPTION
18    | LONG       | 28645        | -1.85      | 698        | ABSORPTION
19    | SHORT      | 29486        | 4.17       | 1090       | ABSORPTION
20    | LONG       | 30667        | -2.01      | 669        | ABSORPTION
21    | SHORT      | 31338        | 33.25      | 595        | ABSORPTION
22    | SHORT      | 32079        | 8.43       | 1678       | ABSORPTION
23    | LONG       | 34081        | 25.82      | 1589       | ABSORPTION
24    | LONG       | 36338        | -9.42      | 249        | ABSORPTION
25    | SHORT      | 37004        | -5.46      | 366        | ABSORPTION
26    | LONG       | 37509        | -0.39      | 679        | ABSORPTION
27    | SHORT      | 38305        | -8.50      | 70         | ABSORPTION
28    | SHORT      | 39126        | -7.99      | 590        | ABSORPTION
29    | SHORT      | 40387        | 9.34       | 1039       | ABSORPTION
30    | LONG       | 41538        | 10.60      | 1703       | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>