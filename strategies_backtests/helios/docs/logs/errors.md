PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run
warning: methods `print_full_config`, `run_init_diagnostics`, and `print_indicator_samples` are never used
   --> E:\Projects\Automations\trading_backtest\strategies_backtests\v2_trading_bot_demo\src\strategy\mod.rs:38:8
    |
 36 | impl HeliosStrategy {
    | ------------------- methods in this implementation
 37 |     /// [DEBUG] In toàn bộ cấu hình để kiểm tra DNA nạp vào có đúng không.
 38 |     fn print_full_config(&self) {
    |        ^^^^^^^^^^^^^^^^^
...
 48 |     fn run_init_diagnostics(&mut self, ctx: &InitContext, raw_closes: &[f64], _raw_volumes: &[f64]) {
    |        ^^^^^^^^^^^^^^^^^^^^
...
169 |     fn print_indicator_samples(&self, name: &str, data: &[f64], start: usize, count: usize) {
    |        ^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: `helios` (lib) generated 1 warning
   Compiling bot_engine v0.1.0 (E:\Projects\Automations\trading_backtest\trading_bot\bot_engine)
error[E0433]: failed to resolve: use of undeclared type `Action`
   --> src\controller\asset_actor.rs:274:32
    |
274 |                         if let Action::Buy { .. } = action {
    |                                ^^^^^^ use of undeclared type `Action`
    |
help: an enum with a similar name exists
    |
274 -                         if let Action::Buy { .. } = action {
274 +                         if let Option::Buy { .. } = action {
    |
help: consider importing one of these items
    |
  1 + use ferro_trade::broker::actions::Action;
    |
  1 + use reqwest::redirect::Action;
    |

error[E0433]: failed to resolve: use of undeclared type `Action`
   --> src\controller\asset_actor.rs:279:39
    |
279 |                         } else if let Action::Sell { .. } = action {
    |                                       ^^^^^^ use of undeclared type `Action`
    |
help: an enum with a similar name exists
    |
279 -                         } else if let Action::Sell { .. } = action {
279 +                         } else if let Option::Sell { .. } = action {
    |
help: consider importing one of these items
    |
  1 + use ferro_trade::broker::actions::Action;
    |
  1 + use reqwest::redirect::Action;
    |

warning: unused import: `error`
 --> src\optimizer\prospector_runner.rs:5:27
  |
5 | use tracing::{info, warn, error};
  |                           ^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unused import: `debug`
 --> src\execution\paper_trader.rs:1:21
  |
1 | use tracing::{info, debug};
  |                     ^^^^^

warning: unused imports: `mpsc` and `watch`
 --> src\execution\ws_manager.rs:2:19
  |
2 | use tokio::sync::{mpsc, watch, broadcast};
  |                   ^^^^  ^^^^^

warning: unused imports: `NewOrderParams` and `NewOrderSideEnum`
  --> src\execution\ws_manager.rs:12:21
   |
12 |     websocket_api::{NewOrderParams, NewOrderSideEnum},
   |                     ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
  --> src\main.rs:18:5
   |
18 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `HeliosStrategy`
  --> src\main.rs:19:24
   |
19 | use helios::strategy::{HeliosStrategy, QuickCrossStrategy};
   |                        ^^^^^^^^^^^^^^

warning: unused variable: `connection`
   --> src\execution\ws_manager.rs:244:24
    |
244 |                     Ok(connection) => {
    |                        ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_connection`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused import: `TradeApi`
 --> src\execution\account_manager.rs:4:30
  |
4 |         RestApi, AccountApi, TradeApi,
  |                              ^^^^^^^^

warning: unused import: `AccountApi`
 --> src\execution\account_manager.rs:4:18
  |
4 |         RestApi, AccountApi, TradeApi,
  |                  ^^^^^^^^^^

warning: unused import: `TradeApi`
 --> src\execution\order_executor.rs:5:5
  |
5 |     TradeApi, NewOrderParams, NewOrderSideEnum
  |     ^^^^^^^^

warning: unused import: `ferro_trade::strategy::Strategy`
  --> src\main.rs:20:5
   |
20 | use ferro_trade::strategy::Strategy;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `weight_monitor`
   --> src\execution\ws_manager.rs:138:13
    |
138 |         let weight_monitor = self.weight_monitor.clone();
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_weight_monitor`

For more information about this error, try `rustc --explain E0433`.
warning: `bot_engine` (bin "bot_engine") generated 12 warnings
error: could not compile `bot_engine` (bin "bot_engine") due to 2 previous errors; 12 warnings emitted