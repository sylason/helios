=== HELIOS STRATEGY: RANK 1 DIAGNOSTIC RUN (BIAS ANALYSIS) ===
Loaded 43243 bars.
[Prospector] 🔄 Resuming from Job ID #1
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
[Prospector] ✅ Precomputation finished in 19.25ms

--- RANK 1 DECODED PARAMETERS (Verification) ---
Lookbacks: Trend=94, Sqz=211, Mom=403, Vol=418, Z=12
Thresholds: Min Score=0.5153, Env ER=0.0225, Anomaly Z=0.6345
Playbook Pullback RSI Buy Thresh: 0.3068

>>> STARTING BACKTEST WITH CHAMPION DNA <<<

=== BACKTEST RESULTS ===
Time: 302.4817ms
Return: 160.17%
Trades: 39
Max DD: -30.58%

=== TRADE LISTING ===
ID    | Type       | Entry Bar    | Pn         | Duration   | Playbook Tag
-------------------------------------------------------------------------------------
0     | LONG       | 16254        | -2.11      | 290        | Trend_PB_L_S88I95
1     | LONG       | 16552        | -2.74      | 281        | Trend_PB_L_S87I95
2     | LONG       | 16879        | -1.99      | 276        | Trend_PB_L_S67I74
3     | LONG       | 17181        | 0.94       | 423        | Trend_PB_L_S86I95
4     | LONG       | 17648        | 11.01      | 2383       | Trend_PB_L_S85I95
5     | LONG       | 20039        | 6.02       | 809        | Trend_PB_L_S78I95
6     | SHORT      | 20848        | -0.92      | 531        | Absorp_S_S78I95
7     | LONG       | 21379        | 0.73       | 209        | Absorp_L_S78I95
8     | SHORT      | 21641        | -4.36      | 83         | Absorp_S_S77I95
9     | SHORT      | 22013        | -0.00      | 1587       | Absorp_S_S52I63
10    | LONG       | 23600        | 2.80       | 431        | Absorp_L_S79I95
11    | SHORT      | 24031        | 0.69       | 876        | Absorp_S_S78I95
12    | LONG       | 24909        | -3.75      | 36         | Absorp_L_S78I95
13    | LONG       | 24977        | 7.82       | 1437       | Absorp_L_S74I95
14    | SHORT      | 26418        | -2.37      | 610        | Absorp_S_S75I95
15    | LONG       | 27105        | -2.13      | 51         | Absorp_L_S66I85
16    | SHORT      | 27215        | 7.20       | 1082       | Absorp_S_S77I95
17    | LONG       | 28297        | -3.37      | 373        | Absorp_L_S78I94
18    | SHORT      | 28670        | -5.21      | 141        | Absorp_S_S78I94
19    | LONG       | 28812        | -2.76      | 483        | Absorp_L_S77I94
20    | SHORT      | 29295        | 4.96       | 680        | Absorp_S_S78I94
21    | LONG       | 30121        | -5.60      | 114        | Trend_PB_L_S53I94
22    | SHORT      | 30237        | -4.89      | 317        | Absorp_S_S76I94
23    | LONG       | 30564        | -3.25      | 991        | Absorp_L_S74I94
24    | SHORT      | 31557        | 29.23      | 376        | Absorp_S_S78I94
25    | LONG       | 31955        | 12.19      | 10         | Trend_PB_L_S55I92
26    | LONG       | 31966        | 7.56       | 6          | Trend_PB_L_S54I92
27    | LONG       | 31973        | -1.53      | 9          | Trend_PB_L_S55I92
28    | LONG       | 31993        | -6.49      | 57         | Trend_PB_L_S56I90
29    | SHORT      | 32525        | 0.16       | 1204       | Absorp_S_S56I69
30    | LONG       | 33729        | 34.32      | 1940       | Absorp_L_S73I90
31    | LONG       | 35738        | 3.52       | 852        | Trend_PB_L_S60I91
32    | SHORT      | 36591        | 10.50      | 689        | Absorp_S_S73I91
33    | LONG       | 37280        | 6.60       | 1262       | Absorp_L_S74I90
34    | SHORT      | 38546        | 4.49       | 1170       | Absorp_S_S56I70
35    | LONG       | 39721        | -5.39      | 443        | Absorp_L_S71I91
36    | SHORT      | 40177        | 4.37       | 636        | Absorp_S_S72I91
37    | LONG       | 40813        | -8.71      | 331        | Absorp_L_S75I91
38    | LONG       | 41639        | 8.59       | 1602       | Trend_PB_L_S59I90