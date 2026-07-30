    Finished `release` profile [optimized] target(s) in 0.76s
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
[Prospector] ✅ Precomputation finished in 41.80ms
Workers: 7
[Prospector] 🎲 Random Restart triggered (1/2). Scout Job #2 queued.
[Prospector] Scout Job #2 done in 1.18s
[Prospector] 🎲 Random Restart triggered (2/2). Scout Job #4 queued.
[Prospector] Scout Job #4 done in 794.71ms
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
--- Prospector Finished in 857.28s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
[Prospector] Database at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db' cleaned up.

=== TOP 10 HELIOS V3 CANDIDATES ===
Rank  | Return %   | Max DD %   | Trades   | Win Rate   | Best %     | Worst %    | ProfitF    | Sharpe     | Sortino
------------------------------------------------------------------------------------------------------------------------
1     | 341.25     | -20.89     | 63       | 53.97      | 8.29       | -2.07      | 3.51       | 0.1122     | 116626317.0299
------------------------------------------------------------------------------------------------------------------------
2     | 287.90     | -17.27     | 37       | 62.16      | 8.41       | -2.56      | 6.58       | 0.1242     | 22830423.4628
------------------------------------------------------------------------------------------------------------------------
3     | 239.29     | -26.19     | 92       | 41.30      | 8.29       | -5.22      | 2.96       | 0.1023     | 3354744.9515
------------------------------------------------------------------------------------------------------------------------
4     | 210.62     | -23.20     | 33       | 57.58      | 8.41       | -2.44      | 4.34       | 0.1205     | 1582305.2409
------------------------------------------------------------------------------------------------------------------------
5     | 181.00     | -37.34     | 85       | 40.00      | 8.29       | -2.07      | 2.26       | 0.0628     | 249847.6713
------------------------------------------------------------------------------------------------------------------------
6     | 180.50     | -11.20     | 23       | 65.22      | 15.17      | -2.99      | 8.92       | 0.4133     | 622998.2260
------------------------------------------------------------------------------------------------------------------------
7     | 144.42     | -18.02     | 72       | 55.56      | 7.32       | -3.95      | 3.64       | 0.4800     | 259657.7369
------------------------------------------------------------------------------------------------------------------------
8     | 132.24     | -23.14     | 30       | 53.33      | 8.41       | -3.33      | 2.85       | 0.0840     | 35640.2497
------------------------------------------------------------------------------------------------------------------------
9     | 121.84     | -18.03     | 66       | 62.12      | 7.32       | -3.95      | 3.74       | 0.4458     | 65620.4459
------------------------------------------------------------------------------------------------------------------------
10    | 101.58     | -15.22     | 23       | 56.52      | 12.97      | -2.05      | 4.53       | 0.4884     | 11767.7331
------------------------------------------------------------------------------------------------------------------------

=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===

>>> [RANK 1] Re-running for full trade extraction...
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | LONG       | 18166        | 7.64       | 514        | ABSORPTION
1     | SHORT      | 18743        | -4.11      | 499        | ABSORPTION
2     | LONG       | 19256        | 0.46       | 514        | ABSORPTION
3     | LONG       | 19771        | 5.07       | 514        | RANGE_OSC
4     | LONG       | 20286        | 0.68       | 514        | TREND_PB
5     | LONG       | 20801        | -1.84      | 514        | TREND_PB
6     | SHORT      | 21330        | -3.72      | 233        | REVERSAL
7     | SHORT      | 21639        | 0.01       | 514        | TREND_PB
8     | SHORT      | 22154        | -0.77      | 514        | TREND_PB
9     | LONG       | 22686        | -0.15      | 1          | REVERSAL
10    | LONG       | 22687        | -0.15      | 1          | REVERSAL
11    | LONG       | 22688        | -0.15      | 1          | REVERSAL
12    | LONG       | 22690        | -1.59      | 247        | ANOMALY
13    | SHORT      | 22950        | -1.26      | 514        | TREND_PB
14    | LONG       | 23478        | 3.44       | 514        | ABSORPTION
15    | LONG       | 23993        | -3.50      | 514        | TREND_PB
16    | SHORT      | 24521        | -0.92      | 514        | TREND_PB
16    | SHORT      | 24521        | -0.00      | 515        | TREND_PB
17    | SHORT      | 25060        | -1.16      | 514        | ABSORPTION
18    | LONG       | 25633        | 3.47       | 514        | TREND_PB
19    | LONG       | 26148        | 0.89       | 514        | ANOMALY
20    | LONG       | 26663        | -1.13      | 514        | ABSORPTION
21    | SHORT      | 27201        | 5.82       | 514        | ABSORPTION
22    | SHORT      | 27719        | 1.83       | 514        | ABSORPTION
23    | SHORT      | 28247        | -0.45      | 514        | ABSORPTION
24    | LONG       | 28788        | -0.14      | 514        | ABSORPTION
25    | SHORT      | 29316        | 5.30       | 514        | ABSORPTION
26    | SHORT      | 29831        | 1.24       | 514        | TREND_PB
27    | LONG       | 30374        | 2.95       | 345        | REVERSAL
28    | LONG       | 30720        | -2.37      | 514        | ABSORPTION
29    | SHORT      | 31248        | 11.51      | 514        | ABSORPTION
30    | SHORT      | 31763        | 15.76      | 199        | TREND_PB
31    | SHORT      | 31969        | 8.42       | 514        | TREND_PB
32    | LONG       | 32485        | 1.80       | 514        | RANGE_OSC
33    | SHORT      | 33000        | 6.42       | 514        | ABSORPTION
34    | LONG       | 33515        | 7.82       | 514        | TREND_PB
35    | LONG       | 34222        | 30.99      | 514        | ANOMALY
36    | LONG       | 34895        | 4.09       | 514        | RANGE_OSC
37    | LONG       | 35411        | -9.87      | 258        | ABSORPTION
38    | SHORT      | 35683        | -9.27      | 110        | ABSORPTION
39    | LONG       | 35806        | 12.95      | 514        | ABSORPTION
40    | LONG       | 36406        | -0.37      | 1          | REVERSAL
41    | LONG       | 36407        | -0.37      | 1          | REVERSAL
42    | LONG       | 36414        | -0.37      | 1          | REVERSAL
43    | LONG       | 36432        | -0.36      | 1          | REVERSAL
44    | SHORT      | 36482        | 25.27      | 514        | ABSORPTION
45    | LONG       | 37045        | 17.40      | 332        | REVERSAL
46    | LONG       | 37377        | 5.73       | 514        | ABSORPTION
47    | SHORT      | 37892        | -12.36     | 483        | ABSORPTION
48    | SHORT      | 38395        | 14.15      | 219        | REVERSAL
49    | SHORT      | 38614        | 16.36      | 514        | TREND_PB
50    | LONG       | 39143        | 7.42       | 514        | REVERSAL
50    | LONG       | 39143        | 0.00       | 515        | REVERSAL
51    | LONG       | 39659        | -0.84      | 514        | ABSORPTION
52    | SHORT      | 40187        | 25.15      | 514        | TREND_PB
53    | SHORT      | 40710        | -0.60      | 1          | REVERSAL
54    | SHORT      | 40716        | -0.59      | 1          | REVERSAL
55    | LONG       | 40772        | -15.32     | 373        | ABSORPTION
56    | SHORT      | 41158        | 9.08       | 514        | ABSORPTION
57    | LONG       | 41673        | 13.17      | 514        | ANOMALY
58    | LONG       | 42188        | 11.22      | 514        | RANGE_OSC
59    | LONG       | 42703        | -5.86      | 514        | TREND_PB
60    | LONG       | 43232        | 0.81       | 9          | RANGE_OSC
--- End of Rank 1 ---

>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<