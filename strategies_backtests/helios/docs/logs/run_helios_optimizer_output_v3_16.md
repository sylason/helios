    Finished `release` profile [optimized] target(s) in 1m 15s
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
[Prospector] ✅ Precomputation finished in 21.80ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 798.41ms
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 870.43ms
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
[Prospector] ✋ Random Restart limit reached (12/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (13/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (14/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (15/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (16/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (17/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (18/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (19/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (20/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (21/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (22/2). No more exploration.
[Prospector] ✋ Random Restart limit reached (23/2). No more exploration.
--- Prospector Finished in 1842.88s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 312.94     | -20.34     | 19       | 47.37      | 23.38      | -2.70      | 8.50       | 0.0605     | 27298021.5499
------------------------------------------------------------------------------------------------------------------------
2     | 308.31     | -20.54     | 19       | 47.37      | 23.38      | -2.73      | 8.23       | 0.0596     | 23471397.4708
------------------------------------------------------------------------------------------------------------------------
3     | 307.08     | -20.58     | 19       | 47.37      | 23.38      | -2.73      | 8.22       | 0.0598     | 22636365.3460
------------------------------------------------------------------------------------------------------------------------
4     | 301.39     | -20.78     | 19       | 47.37      | 23.24      | -2.73      | 8.07       | 0.0601     | 18874297.5065
------------------------------------------------------------------------------------------------------------------------
5     | 267.15     | -20.78     | 19       | 47.37      | 23.24      | -2.73      | 6.16       | 0.0584     | 6212666.4964
------------------------------------------------------------------------------------------------------------------------
6     | 233.89     | -21.11     | 19       | 47.37      | 22.80      | -2.86      | 6.12       | 0.0575     | 2245313.8940
------------------------------------------------------------------------------------------------------------------------
7     | 208.25     | -17.40     | 30       | 36.67      | 19.96      | -2.44      | 5.22       | 0.1235     | 939978.1211
------------------------------------------------------------------------------------------------------------------------
8     | 202.65     | -17.23     | 31       | 35.48      | 19.96      | -2.44      | 4.65       | 0.1295     | 765180.6986
------------------------------------------------------------------------------------------------------------------------
9     | 199.75     | -17.24     | 33       | 33.33      | 19.96      | -2.55      | 4.22       | 0.1282     | 676474.6205
------------------------------------------------------------------------------------------------------------------------
10    | 193.03     | -20.92     | 19       | 47.37      | 16.71      | -3.04      | 10.29      | 0.1987     | 744603.5294
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | SHORT      | 3280         | 12.18      | 1793       | ABSORPTION
1     | LONG       | 5362         | -0.21      | 2292       | TREND_PB
2     | SHORT      | 7781         | -1.11      | 485        | ABSORPTION
3     | SHORT      | 8388         | 10.54      | 2214       | ABSORPTION
4     | LONG       | 10602        | 10.81      | 6235       | ABSORPTION
5     | SHORT      | 16837        | -3.94      | 676        | ABSORPTION
6     | LONG       | 17635        | 10.45      | 578        | RANGE_OSC
7     | LONG       | 18213        | 9.59       | 2851       | ABSORPTION
8     | SHORT      | 21064        | -8.78      | 466        | ABSORPTION
9     | SHORT      | 21724        | -1.56      | 1904       | ABSORPTION
10    | SHORT      | 23938        | -0.19      | 1771       | ANOMALY
11    | SHORT      | 25865        | -3.63      | 71         | ANOMALY
12    | SHORT      | 26100        | 65.99      | 5839       | ANOMALY
13    | LONG       | 31940        | 7.46       | 243        | ANOMALY
14    | LONG       | 32224        | -0.86      | 1050       | ANOMALY
15    | LONG       | 33560        | 49.06      | 3154       | ABSORPTION
16    | SHORT      | 36714        | -0.51      | 682        | ABSORPTION
17    | SHORT      | 37518        | 37.81      | 5094       | ANOMALY
18    | LONG       | 42612        | -5.34      | 629        | ABSORPTION
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>
