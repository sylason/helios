error[E0425]: cannot find value `now` in this scope
   --> src\optimizer\prospector_runner.rs:166:33
    |
166 |                     updated_at: now,
    |                                 ^^^ help: a function with a similar name exists: `not`
    |
   ::: C:\Users\PC\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\polars-plan-0.52.0\src\dsl\functions\syntactic_sugar.rs:41:1
    |
 41 | pub fn not(expr: Expr) -> Expr {
    | ------------------------------ similarly named function `not` defined here

warning: unused import: `debug`
 --> src\execution\paper_trader.rs:1:21
  |
1 | use tracing::{info, debug};
  |                     ^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

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

warning: unused import: `error`
 --> src\optimizer\prospector_runner.rs:5:27
  |
5 | use tracing::{info, warn, error};
  |                           ^^^^^

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
5 |     TradeApi, NewOrderParams, NewOrderSideEnum,
  |     ^^^^^^^^

warning: unused variable: `weight_monitor`
   --> src\execution\ws_manager.rs:138:13
    |
138 |         let weight_monitor = self.weight_monitor.clone();
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_weight_monitor`

warning: unused variable: `symbol`
   --> src\optimizer\helios_bridge.rs:151:9
    |
151 |         symbol: &str,
    |         ^^^^^^ help: if this is intentional, prefix it with an underscore: `_symbol`

For more information about this error, try `rustc --explain E0425`.
warning: `bot_engine` (lib) generated 10 warnings
error: could not compile `bot_engine` (lib) due to 1 previous error; 10 warnings emitted
PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine>