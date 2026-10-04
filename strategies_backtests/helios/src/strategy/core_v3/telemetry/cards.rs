// strategies_backtests/helios/src/strategy/core_v3/telemetry/cards.rs

//! # 🎴 Decision & Defense Telemetry Cards Formatter
//!
//! Mô-đun chuyên biệt định dạng các thẻ kiểm toán quyết định (Audit Cards):
//! 1. `format_approved_card`: Thẻ xanh khi lệnh được Arbiter & Risk Manager phê duyệt.
//! 2. `format_veto_card`: Thẻ đỏ cảnh báo khi proposal bị Ma Trận Phòng Thủ Vốn (Tầng 1 hoặc Tầng 3) VETO.
//!
//! Tách biệt 100% tầng trình diễn (Presentation Layer) khỏi tầng logic chiến lược (Domain Core).

use ferro_trade::strategy::TickContext;
use crate::strategy::core_v3::arbiter::{ArbiterDecision, AuctionStatus, SignalType};
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;

pub struct DecisionCardFormatter;

impl DecisionCardFormatter {
    /// Định dạng thẻ kiểm toán quyết định phê duyệt mở lệnh (ORDER APPROVED)
    pub fn format_approved_card(
        tick: &TickContext,
        sensors: &HeliosSensors,
        matrix: &MarketMatrixV3,
        signal: SignalType,
        decision: &ArbiterDecision,
        price: f64,
        initial_sl: f64,
        size: f64,
        leverage: f64,
        sl_clamped_note: Option<&str>,
    ) -> String {
        let i = tick.i;
        let is_long = signal == SignalType::Long;
        let action_str = if is_long { "BUY (LONG)" } else { "SELL (SHORT)" };
        let time_str = tick
            .data_view
            .raw_cache()
            .datetime
            .get(i)
            .and_then(|ts| {
                let secs = ts / 1_000_000_000;
                let nsecs = (ts % 1_000_000_000) as u32;
                chrono::DateTime::from_timestamp(secs, nsecs)
            })
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("Bar #{}", i));

        let winner_str = decision
            .winning_scanner
            .map(|s| s.as_str())
            .unwrap_or("UNKNOWN");
        let winner_candidate = decision
            .auction_ledger
            .iter()
            .find(|c| c.status == AuctionStatus::Won);
        let raw_score = winner_candidate
            .map(|c| c.raw_score)
            .unwrap_or(decision.final_confidence);

        let sl_dist_pct = if is_long {
            (initial_sl - price) / price * 100.0
        } else {
            (price - initial_sl) / price * 100.0
        };
        let sl_roe_pct = sl_dist_pct * leverage;

        let tp_str = decision
            .final_tp
            .map(|tp| {
                let dist = if is_long {
                    (tp - price) / price * 100.0
                } else {
                    (price - tp) / price * 100.0
                };
                format!("{:.2} ({:+.2}%)", tp, dist)
            })
            .unwrap_or_else(|| "Dynamic / Structural KAMA TP".to_string());

        let notional_usd = size * price;

        // Sensors snapshot
        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let candle_range = (high - low).max(1e-5);
        let clv = (2.0 * price - high - low) / candle_range;

        let price_z = sensors.z_score.get(i).copied().unwrap_or(0.0);
        let vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
        let mfi = sensors.mfi.get(i).copied().unwrap_or(0.0);
        let macro_natr = sensors.natr.get(i).copied().unwrap_or(0.0);
        let atr_sl = sensors.atr_sl.get(i).copied().unwrap_or(0.0);
        let micro_natr = if price > 0.0 { (atr_sl / price) * 100.0 } else { 0.0 };

        // Matrix snapshot
        let kama_slope_atr = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
        let squeeze_intensity = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);
        let trend_strength = matrix.trend_strength.get(i).copied().unwrap_or(0.0);
        let chaos_index = matrix.chaos_index.get(i).copied().unwrap_or(0.0);
        let bb_slope = matrix.bb_middle_slope_adj.get(i).copied().unwrap_or(0.0);

        let mut lines = Vec::new();
        lines.push("╔═══════════════════════════════════════════════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║ 🎯 QUANT DECISION TELEMETRY: ORDER APPROVED                                                          ║".to_string());
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push(format!(
            "║ Bar: {:<6} | Time: {:<19} | Action: {:<18}                              ║",
            i, time_str, action_str
        ));
        lines.push(format!(
            "║ Winner: {:<14} | Side: {:<5}                                                                ║",
            winner_str, format!("{:?}", signal)
        ));
        let conf_line = format!(
            "Decision Confidence: {:.3} (Raw: {:.3} | Gatekeeper: PASS | Chaos: {:.3})",
            decision.final_confidence, raw_score, decision.chaos_val
        );
        lines.push(format!("║ {:<99} ║", conf_line));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ ⚖️ ARBITER AUCTION LEDGER:                                                                            ║".to_string());

        for candidate in &decision.auction_ledger {
            let tag = match candidate.status {
                AuctionStatus::Won => "🏆 [WON]          ",
                AuctionStatus::RunnerUp => "🥈 [RUNNER-UP]    ",
                AuctionStatus::LowConfidence => "❌ [LOW-CONF]     ",
                AuctionStatus::ConflictEliminated => "⚔️ [CONFLICT-FLAT]",
                AuctionStatus::StructuralRiskVeto => "🛑 [RISK-VETO]    ",
                AuctionStatus::NotTriggered => "⚪ [NOT-TRIGGERED]",
            };
            let side_str = candidate
                .signal_type
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "None".to_string());
            lines.push(format!(
                "║   {} {:<12} | Side: {:<5} | Raw: {:.3} | Conf: {:.3} | {:<28} ║",
                tag,
                candidate.scanner_id.as_str(),
                side_str,
                candidate.raw_score,
                candidate.confidence,
                candidate.reason
            ));
        }

        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 📊 SENSORS SNAPSHOT:                                                                                  ║".to_string());
        let sensor_l1 = format!(
            "Price: {:.2} | Price_Z: {:+.2} | Vol_Z: {:+.2} | MFI: {:.1} | CLV: {:+.2}",
            price, price_z, vol_z, mfi, clv
        );
        let sensor_l2 = format!(
            "NATR_Micro (Flatline): {:.2}% | NATR_Macro (Chaos): {:.2}% | ATR_SL: {:.4}",
            micro_natr, macro_natr, atr_sl
        );
        lines.push(format!("║   {:<99} ║", sensor_l1));
        lines.push(format!("║   {:<99} ║", sensor_l2));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 🧭 MATRIX STATE:                                                                                      ║".to_string());
        lines.push(format!(
            "║   KAMA_Slope_ATR: {:+.2} | Squeeze: {:.2} | Trend_Str: {:.2} | Chaos_Idx: {:.2} | BB_Slope: {:+.2}    ║",
            kama_slope_atr, squeeze_intensity, trend_strength, chaos_index, bb_slope
        ));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 🛡️ RISK PARAMETERS:                                                                                   ║".to_string());
        
        let sl_line = if let Some(note) = sl_clamped_note {
            format!(
                "║   Entry: {:.2} | Initial SL: {:.2} ({:+.2}% / ROE: {:+.1}%) [{:<20}]      ║",
                price, initial_sl, sl_dist_pct, sl_roe_pct, note
            )
        } else {
            format!(
                "║   Entry: {:.2} | Initial SL: {:.2} ({:+.2}% / ROE: {:+.1}%) | Dynamic TP: {:<27} ║",
                price, initial_sl, sl_dist_pct, sl_roe_pct, tp_str
            )
        };
        lines.push(sl_line);

        lines.push(format!(
            "║   Position Size: {:.4} (${:.2}) | Leverage: {:.0}x | Margin Mode: ISOLATED                           ║",
            size, notional_usd, leverage
        ));
        lines.push("╚═══════════════════════════════════════════════════════════════════════════════════════════════════════╝".to_string());

        lines.join("\n")
    }

    /// Định dạng thẻ đỏ cảnh báo khi proposal bị Ma Trận Phòng Thủ Vốn VETO (ORDER DEFENSE VETO)
    pub fn format_veto_card(
        tick: &TickContext,
        sensors: &HeliosSensors,
        matrix: &MarketMatrixV3,
        decision: &ArbiterDecision,
        price: f64,
        leverage: f64,
    ) -> String {
        let i = tick.i;
        let time_str = tick
            .data_view
            .raw_cache()
            .datetime
            .get(i)
            .and_then(|ts| {
                let secs = ts / 1_000_000_000;
                let nsecs = (ts % 1_000_000_000) as u32;
                chrono::DateTime::from_timestamp(secs, nsecs)
            })
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("Bar #{}", i));

        let veto_tier_str = if decision.gate_veto.is_some() {
            "TIER 1 (REGIME GATEKEEPER)"
        } else if decision.auction_ledger.iter().any(|c| c.status == AuctionStatus::StructuralRiskVeto) {
            "TIER 3 (SNIPER ENTRY GUARD)"
        } else {
            "ARBITER AUCTION FILTER"
        };

        // Sensors snapshot
        let price_z = sensors.z_score.get(i).copied().unwrap_or(0.0);
        let vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
        let mfi = sensors.mfi.get(i).copied().unwrap_or(0.0);
        let macro_natr = sensors.natr.get(i).copied().unwrap_or(0.0);
        let atr_sl = sensors.atr_sl.get(i).copied().unwrap_or(0.0);
        let micro_natr = if price > 0.0 { (atr_sl / price) * 100.0 } else { 0.0 };

        // Matrix snapshot
        let kama_slope_atr = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
        let squeeze_intensity = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);
        let trend_strength = matrix.trend_strength.get(i).copied().unwrap_or(0.0);
        let chaos_index = matrix.chaos_index.get(i).copied().unwrap_or(0.0);

        let mut lines = Vec::new();
        lines.push("╔═══════════════════════════════════════════════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║ 🛡️ QUANT DECISION TELEMETRY: ORDER DEFENSE VETO                                                      ║".to_string());
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push(format!(
            "║ Bar: {:<6} | Time: {:<19} | Veto Tier: {:<45} ║",
            i, time_str, veto_tier_str
        ));

        if let Some(gate_veto) = decision.gate_veto {
            let veto_info = format!(
                "GATEKEEPER VETO: {:<18} | Chaos: {:.3} | NATR_Mic: {:.2}% | NATR_Mac: {:.2}%",
                gate_veto.as_str(), decision.chaos_val, micro_natr, macro_natr
            );
            lines.push(format!("║ 🛑 {:<97} ║", veto_info));
        }

        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ ⚖️ VETOED CANDIDATES & AUCTION LEDGER:                                                                ║".to_string());

        for candidate in &decision.auction_ledger {
            let tag = match candidate.status {
                AuctionStatus::Won => "🏆 [WON]          ",
                AuctionStatus::RunnerUp => "🥈 [RUNNER-UP]    ",
                AuctionStatus::LowConfidence => "❌ [LOW-CONF]     ",
                AuctionStatus::ConflictEliminated => "⚔️ [CONFLICT-FLAT]",
                AuctionStatus::StructuralRiskVeto => "🛑 [RISK-VETO]    ",
                AuctionStatus::NotTriggered => "⚪ [NOT-TRIGGERED]",
            };
            let side_str = candidate
                .signal_type
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "None".to_string());
            lines.push(format!(
                "║   {} {:<12} | Side: {:<5} | Raw: {:.3} | Conf: {:.3} | {:<28} ║",
                tag,
                candidate.scanner_id.as_str(),
                side_str,
                candidate.raw_score,
                candidate.confidence,
                candidate.reason
            ));
        }

        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 📊 SENSORS SNAPSHOT:                                                                                  ║".to_string());
        let sensor_l1 = format!(
            "Price: {:.2} | Price_Z: {:+.2} | Vol_Z: {:+.2} | MFI: {:.1} | Lev: {:.0}x",
            price, price_z, vol_z, mfi, leverage
        );
        let sensor_l2 = format!(
            "NATR_Micro (Flatline): {:.2}% | NATR_Macro (Chaos): {:.2}% | ATR_SL: {:.4}",
            micro_natr, macro_natr, atr_sl
        );
        lines.push(format!("║   {:<99} ║", sensor_l1));
        lines.push(format!("║   {:<99} ║", sensor_l2));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 🧭 MATRIX STATE:                                                                                      ║".to_string());
        lines.push(format!(
            "║   KAMA_Slope_ATR: {:+.2} | Squeeze: {:.2} | Trend_Str: {:.2} | Chaos_Idx: {:.2}                         ║",
            kama_slope_atr, squeeze_intensity, trend_strength, chaos_index
        ));
        lines.push("╚═══════════════════════════════════════════════════════════════════════════════════════════════════════╝".to_string());

        lines.join("\n")
    }
}