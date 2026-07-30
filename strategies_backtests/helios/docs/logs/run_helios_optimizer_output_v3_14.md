    Finished `release` profile [optimized] target(s) in 1m 18s
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
[Prospector] ✅ Precomputation finished in 21.56ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/3). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 1.43s
[Prospector] 🎲 Random Restart triggered (2/3). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 786.36ms
[Prospector] 🎲 Random Restart triggered (3/3). Scout Job #6 queued.
[Prospector] Scout Job #6 done in 859.96ms
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
--- Prospector Finished in 2327.09s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 280.48     | -19.87     | 17       | 41.18      | 18.19      | -0.25      | 30.90      | 0.0904     | 13264883.6828
------------------------------------------------------------------------------------------------------------------------
2     | 278.96     | -20.20     | 16       | 43.75      | 18.12      | -0.25      | 35.21      | 0.0907     | 12499465.8542
------------------------------------------------------------------------------------------------------------------------
3     | 272.05     | -20.20     | 17       | 41.18      | 18.12      | -0.51      | 26.40      | 0.0908     | 10058551.1352
------------------------------------------------------------------------------------------------------------------------
4     | 258.66     | -18.92     | 18       | 38.89      | 18.12      | -0.73      | 18.15      | 0.0897     | 6398876.0704
------------------------------------------------------------------------------------------------------------------------
5     | 234.10     | -18.17     | 17       | 41.18      | 18.12      | -0.60      | 17.00      | 0.1098     | 2971798.6731
------------------------------------------------------------------------------------------------------------------------
6     | 227.54     | -19.43     | 32       | 43.75      | 10.52      | -3.30      | 4.10       | 0.3647     | 3796589.7573
------------------------------------------------------------------------------------------------------------------------
7     | 219.35     | -20.03     | 32       | 43.75      | 10.52      | -3.30      | 3.87       | 0.3789     | 2789361.8928
------------------------------------------------------------------------------------------------------------------------
8     | 213.38     | -19.77     | 31       | 45.16      | 10.52      | -3.30      | 3.97       | 0.3923     | 2253653.8398
------------------------------------------------------------------------------------------------------------------------
9     | 195.16     | -19.76     | 32       | 46.88      | 10.52      | -3.30      | 3.67       | 0.4045     | 1104826.8639
------------------------------------------------------------------------------------------------------------------------
10    | 173.78     | -18.45     | 32       | 46.88      | 10.52      | -3.30      | 3.67       | 0.4794     | 492809.2928
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 6173         | 1.06       | 2201       | ANOMALY
1     | SHORT      | 8374         | 6.60       | 3150       | ABSORPTION
2     | LONG       | 11524        | -0.26      | 396        | ABSORPTION
2     | LONG       | 11524        | -0.00      | 397        | ABSORPTION
3     | LONG       | 12043        | -0.40      | 1171       | RANGE_OSC
3     | LONG       | 12043        | -0.00      | 1172       | RANGE_OSC
4     | LONG       | 13340        | -0.49      | 370        | ABSORPTION
5     | LONG       | 13833        | 26.29      | 13407      | ABSORPTION
6     | SHORT      | 27240        | -0.39      | 1586       | ABSORPTION
7     | SHORT      | 28952        | 49.43      | 2987       | ABSORPTION
8     | LONG       | 31940        | 10.17      | 349        | ANOMALY
9     | LONG       | 32378        | -1.14      | 874        | ANOMALY
10    | LONG       | 33378        | 48.98      | 3335       | ANOMALY
11    | SHORT      | 36713        | -1.33      | 683        | ABSORPTION
11    | SHORT      | 36713        | -0.00      | 686        | ABSORPTION
12    | SHORT      | 37521        | -1.30      | 484        | RANGE_OSC
13    | SHORT      | 38127        | 31.06      | 5114       | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<