PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios> cargo check
   Compiling pyo3-build-config v0.25.1
   Compiling pyo3-ffi v0.25.1
   Compiling pyo3-macros-backend v0.25.1
   Compiling pyo3 v0.25.1
   Compiling pyo3-macros v0.25.1
    Checking ferro_trade v0.1.0 (E:\Projects\Automations\trading_backtest\ferro_trade)
    Checking helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
error[E0609]: no field `playbook` on type `&HeliosConfig`
  --> src\strategy\core_v2\playbook.rs:83:37
   |
83 |             let rsi_thresh = config.playbook.momentum_ignition.rsi_threshold;
   |                                     ^^^^^^^^ unknown field
   |
   = note: available fields are: `indicators`, `matrix`, `environment_fuse`, `market_regime`, `tactical_sniper`, `risk_management`

error[E0609]: no field `playbook` on type `&HeliosConfig`
  --> src\strategy\core_v2\playbook.rs:84:37
   |
84 |             let rsi_climax = config.playbook.momentum_ignition.rsi_climax;
   |                                     ^^^^^^^^ unknown field
   |
   = note: available fields are: `indicators`, `matrix`, `environment_fuse`, `market_regime`, `tactical_sniper`, `risk_management`

warning: unused variable: `tp`
   --> src\strategy\core_v2\tactical.rs:442:21
    |
442 |                 let tp = current_close + (sl_dist * 2.0);
    |                     ^^ help: if this is intentional, prefix it with an underscore: `_tp`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `price`
   --> src\strategy\core_v3\mod.rs:151:21
    |
151 |                 let price = tick.closes()[i];
    |                     ^^^^^ help: if this is intentional, prefix it with an underscore: `_price`

warning: unused variable: `size`
   --> src\strategy\core_v3\mod.rs:155:21
    |
155 |                 let size = RiskManager::calculate_size(tick.equity, 0.10, 21.0, 20.0);
    |                     ^^^^ help: if this is intentional, prefix it with an underscore: `_size`

For more information about this error, try `rustc --explain E0609`.
warning: `helios` (lib) generated 3 warnings
error: could not compile `helios` (lib) due to 2 previous errors; 3 warnings emitted
PS E:\Projects\Automations\trading_backtest\strategies_backtests\helios>