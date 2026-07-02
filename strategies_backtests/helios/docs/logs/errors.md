PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios> cargo run --release --example run_helios_optimize
   Compiling ferro_trade_derive v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade\ferro_trade_derive)
   Compiling ferro_trade v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade)
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
error[E0061]: this function takes 5 arguments but 6 arguments were supplied
   --> src\strategy\mod.rs:340:9
    |
340 |         logic::risk::manage_trailing_stop(
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
344 |             regime_for_trailing,
    |             ------------------- unexpected argument #4 of type `types::MarketRegime`
    |
note: function defined here
   --> src\strategy\logic\risk.rs:11:8
    |
 11 | pub fn manage_trailing_stop(
    |        ^^^^^^^^^^^^^^^^^^^^
help: remove the extra argument
    |
343 -             tick,
344 -             regime_for_trailing,
343 +             tick,
    |

warning: unused variable: `last_start`
  --> src\strategy\matrix.rs:39:17
   |
39 |             let last_start = len.saturating_sub(10);
   |                 ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_last_start`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `category`
  --> src\strategy\matrix.rs:36:24
   |
36 |         let trace_v = |category: &str, name: &str, data: &[f64]| {
   |                        ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_category`

warning: unused variable: `name`
  --> src\strategy\matrix.rs:36:40
   |
36 |         let trace_v = |category: &str, name: &str, data: &[f64]| {
   |                                        ^^^^ help: if this is intentional, prefix it with an underscore: `_name`

warning: unused variable: `raw_volumes`
  --> src\strategy\mod.rs:45:79
   |
45 |     fn run_init_diagnostics(&mut self, ctx: &InitContext, raw_closes: &[f64], raw_volumes: &[f64]) {
   |                                                                               ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_raw_volumes`

warning: unused variable: `tp`
   --> src\strategy\mod.rs:225:19
    |
225 |         let (mfi, tp, mf, pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
    |                   ^^ help: if this is intentional, prefix it with an underscore: `_tp`

warning: unused variable: `mf`
   --> src\strategy\mod.rs:225:23
    |
225 |         let (mfi, tp, mf, pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
    |                       ^^ help: if this is intentional, prefix it with an underscore: `_mf`

warning: unused variable: `pmf`
   --> src\strategy\mod.rs:225:27
    |
225 |         let (mfi, tp, mf, pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
    |                           ^^^ help: if this is intentional, prefix it with an underscore: `_pmf`

For more information about this error, try `rustc --explain E0061`.
warning: `helios` (lib) generated 7 warnings
error: could not compile `helios` (lib) due to 1 previous error; 7 warnings emitted