PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios> cargo run --release --example v3_run_helios_optimize
warning: constant `RISK_SIZE_RATIO` is never used
  --> examples\common.rs:14:11
   |
14 | pub const RISK_SIZE_RATIO: f64 = 0.2;  // Vào lệnh 1/5 tổng tài sản
   |           ^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: constant `MIN_TRADE_AMOUNT` is never used
  --> examples\common.rs:15:11
   |
15 | pub const MIN_TRADE_AMOUNT: f64 = 10.0; // Tiền vào lệnh tối thiểu $10
   |           ^^^^^^^^^^^^^^^^

warning: `helios` (example "v3_run_helios_optimize") generated 2 warnings
    Finished `release` profile [optimized] target(s) in 0.73s
     Running `target\release\examples\v3_run_helios_optimize.exe`
=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===
🆕 Starting fresh optimization run for Helios V3.
Loading data: E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv
Loaded 43243 rows.

thread 'main' (23644) panicked at examples\v3_run_helios_optimize.rs:45:76:
JSON format error: Error("missing field `mode`", line: 33, column: 5)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\release\examples\v3_run_helios_optimize.exe` (exit code: 101)