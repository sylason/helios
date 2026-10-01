// strategies_backtests/helios/src/strategy/core_v3/telemetry/mod.rs

//! # 📡 Telemetry & Observability Subsystem
//!
//! Tầng quan sát và kiểm toán toàn diện cho Helios Core V3:
//! - `cards`: Định dạng thẻ kiểm toán quyết định phê duyệt (`format_approved_card`) và thẻ từ chối (`format_veto_card`).
//! - `defense_ledger`: Sổ cái tích lũy và kiểm toán Ma Trận 3 Tầng Phòng Thủ Vốn.

pub mod cards;
pub mod defense_ledger;

pub use cards::DecisionCardFormatter;
pub use defense_ledger::DefenseLedger;
