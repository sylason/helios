    Finished `release` profile [optimized] target(s) in 20.37s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
✅ Found existing DB at 'E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db'. Resuming...
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.
Config Loaded:
  Workers: Pop=80, Gen=10
  Scout Method: lhs
  Exploration Strategy: random_restart
  Max Threads: Some(7)
[Prospector] ⚠️ Recovered 2 interrupted jobs from storage.
[Prospector] 🔄 Resuming from Job ID #2
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
[Prospector] ✅ Precomputation finished in 26.33ms
Workers: 7

thread '<unnamed>' (26592) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\indicators\factory\bulk.rs:111:99:
called `Result::unwrap()` on an `Err` value: Calculation("Insufficient data points for the requested calculation")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread '<unnamed>' (26736) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\indicators\factory\bulk.rs:111:99:
called `Result::unwrap()` on an `Err` value: Calculation("Insufficient data points for the requested calculation")
Rayon: detected unexpected panic; aborting
error: process didn't exit successfully: `target\release\examples\v3_run_helios_optimize.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>