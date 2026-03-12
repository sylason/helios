# Research Notes: Trading Bot Analysis

## 1. Current Status (Python Implementation)
- **Data Acquisition:** Stable WebSocket connection to Binance `aggTrade`.
- **Data Processing:** 
    - Backfills 1000 candles from REST API.
    - Aggregates real-time trades into 1m OHLCV using Polars.
    - Stores data in `market_data.duckdb` (`kline_1m` and `raw_agg_trades`).
- **UI:** Basic Dear PyGui dashboard.

## 2. Structural Gaps
- **Missing Controller:** No event bus or central orchestrator yet.
- **Strategy Engine:** Planned as a Python class, but now needs to interface with Rust `ferro_trade`.
- **Execution Module:** Not implemented.
- **Performance/Backtest:** Not implemented in the bot code (logic resides in `ferro_trade`).

## 3. Integration Points (Rust Backend)
- **Data Feed:** Rust backend (`ferro_trade`) needs 1m OHLCV data from DuckDB or direct from the Data Engine.
- **Strategy Execution:** Helios strategy (Rust) will receive data and produce `Signal` objects.
- **Communication:** Need an efficient way to pass `Signals` from Rust to the UI (Python or Rust UI).

## 4. Observations for Prototype
- The current Python Data Engine is functional but might be a bottleneck if we want "ultra-low latency".
- Moving the Orchestrator to Rust early on would simplify integration with `ferro_trade` and `helios`.
