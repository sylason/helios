# Tech Stack

## Core Technologies
-   **Rust:** Primary language for Engine (Ferro-Trade) and Strategies (Helios). Selected for performance, type safety, and memory safety.
-   **Python:** Used for Data Analysis, Visualization, and Reference implementations (Dymate).

## Key Libraries & Frameworks

### Rust
-   **Polars:** Ultra-fast DataFrame library for data manipulation.
-   **PyO3:** Rust bindings for Python (allowing Rust code to be called from Python).
-   **Serde:** Serialization/Deserialization framework.
-   **Strum:** Enum macros for cleaner code.
-   **Rayon:** Data parallelism library.
-   **Sled:** Embedded database for Optimizer state/checkpoints.

### Python
-   **Pandas:** Data manipulation.
-   **Backtesting.py:** The reference backtesting library being ported.
-   **TA-Lib:** Technical Analysis library (Reference).

## Development Tools
-   **Cargo:** Rust package manager.
-   **Conductor:** Workflow management (This system).
