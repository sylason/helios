# Implementation Plan: Helios Port

## Phase 1: Infrastructure & Types [DONE]
- [x] Define core enums (Regime, SubStrategy, etc.).
- [x] Implement `Position` struct with `entry_reason` and `initial_sl`.
- [x] Implement Tag translation layer (Enum <-> String).

## Phase 2: Fixing Build & Dependencies [DONE]
- [x] Resolve `hashbrown` simsimd error.
- [x] Add `yata` dependency (Removed in favor of internal mapping).
- [x] Fix Polars 0.52 API changes (Column vs Series, Quantile).
- [x] Fix Import paths (Actions, StrongMovePhase).
- [x] Fix Chrono/DateTime conversion errors.
- [x] **Milestone:** Project compiles successfully.

## Phase 3: Runtime Verification (Quick Tests) [IN PROGRESS]
- [ ] Create `run_helios.rs` (Single run with real data).
- [ ] Create `run_helios_optimize.rs` (GA run with minimal config).
- [ ] Verify Order/Trade execution flow (Logs check).

## Phase 4: Core Logic Validation (Deep Dive)
- [ ] Validate MarketAnalyzer logic (compare indicators with Python).
- [ ] Validate RiskManager calculations.
- [ ] Implement/Refine Early Exit logic in Orchestrator.
- [ ] Compare signals against Dymate Python.
