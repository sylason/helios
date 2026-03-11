# Specification: Helios Strategy Port (Dymate)

## Objective
Port the Python DyMATE strategy to Rust (Helios) with 1:1 logic parity, running on the `ferro_trade` engine.

## Core Features to Port
1.  **Multi-Timeframe Analysis**: Using `resample_apply` from ferro_trade.
2.  **Regime Detection**: Trending, Ranging, Squeeze.
3.  **4-Step Synthesis**: Logic for Confidence Score and Tradeable Zone.
4.  **Advanced Management**:
    *   Trailing Stop Loss (Dynamic multipliers).
    *   Break-Even logic.
    *   Early Exit on Regime Change (Requires Tagging/Entry Reason).
    *   Pyramiding (Requires Initial SL data).

## Data Synchronization
*   **Helios Position** must track:
    *   `entry_reason` (mapped from Trade Tag).
    *   `initial_stop_loss` (mapped from RiskPolicy).
