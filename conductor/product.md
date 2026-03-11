# Product Definition: Ferro-Trade Ecosystem

## 1. Vision & Goals
The **Ferro-Trade Ecosystem** is a high-performance, modular algorithmic trading platform designed for serious quantitative research and execution. The primary goal is to provide a 1:1 consistent, ultra-fast backtesting engine in Rust that matches the logic of Python's `backtesting.py`, while enabling advanced strategies like Helios (Dymate port).

## 2. Core Components

### 🏗️ Ferro-Trade (The Engine)
-   **Role:** High-performance Backtesting Engine & Optimizer.
-   **Language:** Rust (with Python bindings via PyO3).
-   **Key Features:** 
    -   Event-driven architecture.
    -   1:1 Logic parity with `backtesting.py`.
    -   Genetic Algorithm Optimizer (Prospector).
    -   Fast Data Processing with Polars.

### ☀️ Helios (The Strategy)
-   **Role:** Flagship Trading Strategy.
-   **Language:** Rust.
-   **Origin:** A direct port of the "Dymate" strategy (Python).
-   **Logic:** Multi-timeframe analysis, Regime detection, 4-step decision process.

### 🐍 Dymate (The Reference)
-   **Role:** Reference Implementation & Prototype.
-   **Language:** Python.
-   **Status:** Source of Truth for logic verification.

### 🤖 Trading Bot (The Executor)
-   **Role:** Live Trading Execution.
-   **Status:** Early stage/Planned.

## 3. Project Structure (Monorepo)
-   `ferro_trade/`: Core Engine code.
-   `strategies_backtests/helios/`: Rust implementation of Dymate.
-   `strategies_backtests/dymate/`: Python reference code.
-   `data/`: Shared OHLCV data.
