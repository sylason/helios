# kết quả khi có đầy dủ nội dung hàm next() hàm init 

    Finished `release` profile [optimized] target(s) in 52.14s
     Running `target\release\examples\v3_run_helios.exe`
=== HELIOS STRATEGY V3: DIAGNOSTIC RUN ===
Loaded 43243 bars.
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
[Prospector] ✅ Precomputation finished in 26.95ms

=== BACKTEST RESULTS V3 ===
Time: 103.6679ms
Return: -0.51%
Trades: 93
Max DD: -1.17%

# kết quả khi comment out hàm next(), chỉ có nội dung hàm init 
=== BACKTEST RESULTS V3 ===
Time: 88.0048ms
Return: 0.00%
Trades: 0
Max DD: -0.00%