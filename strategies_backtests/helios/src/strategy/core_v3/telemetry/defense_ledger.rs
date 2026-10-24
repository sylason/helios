// strategies_backtests/helios/src/strategy/core_v3/telemetry/defense_ledger.rs

//! # 🛡️ Defense Matrix Ledger (Sổ Cái Quản Trị Phòng Thủ Vốn)
//!
//! Mô-đun chuyên biệt quản trị và kiểm toán 3 Tầng Ma Trận Phòng Thủ Vốn:
//! - Tầng 1: Regime Gatekeeper (Chaos Whipsaw & Volatility Flatline, State Transition Edge Trigger).
//! - Tầng 2: Boundary Guard & Chop Zone (TrendPB Overbought / Oversold / Squeeze).
//! - Tầng 3: Stop Loss Cap 15% ROE & Sniper Entry Guard (Structural Risk Veto & SL Clamping).
//!
//! Tách biệt 100% tầng logic tính toán với việc thu thập số liệu kiểm toán.

use crate::strategy::core_v3::gatekeeper::{GateEvaluation, GateVetoReason, MarketRegime};
use crate::strategy::core_v3::arbiter::{ArbiterDecision, AuctionStatus};

/// Helper chuẩn hóa định dạng timestamp mili-giây UTC sang chuỗi hiển thị YYYY-MM-DD HH:MM:SS
fn format_utc_ts(ts_ms: i64) -> String {
    if ts_ms <= 0 {
        return "N/A".to_string();
    }
    let secs = ts_ms / 1000;
    let nsecs = ((ts_ms % 1000) * 1_000_000) as u32;
    chrono::DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatekeeperZoneRecord {
    pub start_bar: usize,
    pub end_bar: usize,
    pub start_ts: i64,
    pub end_ts: i64,
    pub reason: String,
}

#[derive(Debug, Default, Clone)]
pub struct DefenseLedger {
    // -------------------------------------------------------------------------
    // 1. TẦNG 1: REGIME GATEKEEPER
    // -------------------------------------------------------------------------
    /// Số thanh nến được né bão biến động hỗn loạn (Chaos Whipsaw)
    pub chaos_whipsaw_bars: usize,
    /// ⚡ Số thanh nến bị phong tỏa bởi Cầu Dao Khẩn Cấp Flash Crash & Liquidity Vacuum
    pub flash_crash_freeze_bars: usize,
    /// Số thanh nến bị chặn vì sàn biến động chết / tắt thanh khoản (Flatline)
    pub volatility_flatline_bars: usize,
    /// Tổng số lần chuyển đổi trạng thái (Vào bão / Ra khỏi bão)
    pub gatekeeper_state_transitions: usize,
    /// Trạng thái Veto trước đó để bắt sự kiện biên (Edge Detection)
    pub prev_gate_veto: Option<GateVetoReason>,
    /// Mốc thanh nến bắt đầu rơi vào bão
    pub veto_start_bar: usize,
    /// Mốc thời gian bắt đầu rơi vào bão (ms)
    pub veto_start_ts: i64,
    /// Mốc thanh nến cuối cùng được quan sát
    pub last_seen_bar: usize,
    /// Mốc thời gian cuối cùng được quan sát (ms)
    pub last_seen_ts: i64,
    /// Mốc thanh nến của lần phát Heartbeat gần nhất
    pub last_heartbeat_bar: usize,
    /// Mốc thời gian của lần phát Heartbeat gần nhất (ms UTC)
    pub last_heartbeat_ts: i64,
    /// Danh sách các vùng bảo vệ đã hoàn thành để phục vụ Visual Report
    pub zones: Vec<GatekeeperZoneRecord>,

    // -------------------------------------------------------------------------
    // 2. TẦNG 2: BOUNDARY GUARD & CHOP ZONE (TrendPB)
    // -------------------------------------------------------------------------
    /// Số lần chặn mua đu đỉnh phân phối (Long khi Z > max_z)
    pub trend_pb_boundary_long_vetos: usize,
    /// Số lần chặn bán tháo hoảng loạn (Short khi Z < -max_z)
    pub trend_pb_boundary_short_vetos: usize,
    /// Số lần chặn cào lệnh trong vùng co thắt (Squeeze > max_squeeze)
    pub trend_pb_chop_zone_vetos: usize,

    // -------------------------------------------------------------------------
    // 3. TẦNG 3: STOP LOSS CAP & SNIPER ENTRY GUARD
    // -------------------------------------------------------------------------
    /// Số proposal bị bắn hạ vì SL cấu trúc vượt quá 15% ROE
    pub sniper_structural_sl_vetos: usize,
    /// Số lệnh được duyệt nhưng SL thô bị kẹp cứng về trần toàn cục 15% ROE
    pub sl_clamped_to_hard_cap: usize,
    /// Số lệnh được duyệt nhưng SL thô bị kẹp về trần Playbook DNA
    pub sl_clamped_to_playbook_cap: usize,
}

impl DefenseLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Theo dõi trạng thái Gatekeeper trên từng nến, phát hiện chuyển đổi trạng thái (Edge Trigger)
    /// Trả về thông điệp cảnh báo nếu có sự kiện Vào Bão hoặc Hết Bão.
    pub fn on_gatekeeper_tick(
        &mut self,
        current_bar: usize,
        current_ts: i64,
        gate_eval: &GateEvaluation,
    ) -> Option<String> {
        self.last_seen_bar = current_bar;
        self.last_seen_ts = current_ts;

        if !gate_eval.passed {
            let reason = gate_eval.veto_reason?;

            // Nếu chuyển đổi giữa các loại veto khác nhau trong lúc đang né bão
            if let Some(old_reason) = self.prev_gate_veto {
                if old_reason != reason {
                    self.zones.push(GatekeeperZoneRecord {
                        start_bar: self.veto_start_bar,
                        end_bar: current_bar,
                        start_ts: self.veto_start_ts,
                        end_ts: current_ts,
                        reason: old_reason.as_str().to_string(),
                    });
                    self.veto_start_bar = current_bar;
                    self.veto_start_ts = current_ts;
                    self.prev_gate_veto = Some(reason);
                    self.gatekeeper_state_transitions += 1;
                }
            }

            let time_str = format_utc_ts(current_ts);
            let msg = match reason {
                GateVetoReason::FlashCrashFreeze => {
                    self.flash_crash_freeze_bars += 1;
                    if self.prev_gate_veto.is_none() {
                        self.prev_gate_veto = Some(reason);
                        self.veto_start_bar = current_bar;
                        self.veto_start_ts = current_ts;
                        self.gatekeeper_state_transitions += 1;
                        Some(format!(
                            "⚡ [GATEKEEPER ENGAGED] [{}] Bar: {} | Veto: FlashCrashFreeze | Kinetic Shock detected in lookback! | Entering Shelter!",
                            time_str, current_bar
                        ))
                    } else {
                        None
                    }
                }
                GateVetoReason::ChaosWhipsaw => {
                    self.chaos_whipsaw_bars += 1;
                    if self.prev_gate_veto.is_none() {
                        self.prev_gate_veto = Some(reason);
                        self.veto_start_bar = current_bar;
                        self.veto_start_ts = current_ts;
                        self.gatekeeper_state_transitions += 1;
                        Some(format!(
                            "🛡️ [GATEKEEPER ENGAGED] [{}] Bar: {} | Veto: ChaosWhipsaw | Chaos: {:.3} >= Threshold: {:.3} (Hysteresis: {}) | NATR: {:.2}% | Entering Shelter!",
                            time_str, current_bar, gate_eval.current_chaos, gate_eval.chaos_threshold, gate_eval.is_hysteresis_active, gate_eval.market_natr_pct
                        ))
                    } else {
                        None
                    }
                }
                GateVetoReason::MacroFlatline => {
                    self.volatility_flatline_bars += 1;
                    if self.prev_gate_veto.is_none() {
                        self.prev_gate_veto = Some(reason);
                        self.veto_start_bar = current_bar;
                        self.veto_start_ts = current_ts;
                        self.gatekeeper_state_transitions += 1;
                        Some(format!(
                            "🛡️ [GATEKEEPER ENGAGED] [{}] Bar: {} | Veto: MacroFlatline | Macro NATR: {:.2}% < Req: {:.2}% (FeeFloor: {:.2}% + DNAAlpha: {:.2}%) | Entering Shelter!",
                            time_str, current_bar, gate_eval.market_natr_pct, gate_eval.required_min_natr_pct, gate_eval.fee_floor_pct, gate_eval.min_alpha_pct
                        ))
                    } else {
                        None
                    }
                }
                GateVetoReason::MicroFlatline => {
                    self.volatility_flatline_bars += 1;
                    if self.prev_gate_veto.is_none() {
                        self.prev_gate_veto = Some(reason);
                        self.veto_start_bar = current_bar;
                        self.veto_start_ts = current_ts;
                        self.gatekeeper_state_transitions += 1;
                        Some(format!(
                            "🛡️ [GATEKEEPER ENGAGED] [{}] Bar: {} | Veto: MicroFlatline | Micro NATR: {:.2}% < MicroFeeFloor: {:.2}% | Entering Shelter!",
                            time_str, current_bar, gate_eval.micro_natr_pct, gate_eval.micro_fee_floor_pct
                        ))
                    } else {
                        None
                    }
                }
                GateVetoReason::DataAnomaly => {
                    if self.prev_gate_veto.is_none() {
                        self.prev_gate_veto = Some(reason);
                        self.veto_start_bar = current_bar;
                        self.veto_start_ts = current_ts;
                        self.gatekeeper_state_transitions += 1;
                        Some(format!(
                            "🛡️ [GATEKEEPER ENGAGED] [{}] Bar: {} | Veto: DataAnomaly | Entering Shelter!",
                            time_str, current_bar
                        ))
                    } else {
                        None
                    }
                }
            };
            msg
        } else {
            if let Some(prev) = self.prev_gate_veto.take() {
                // Sự kiện cạnh xuống (Falling Edge): Bão vừa tan, thị trường ổn định trở lại
                let duration_bars = current_bar.saturating_sub(self.veto_start_bar);
                let duration_ms = current_ts.saturating_sub(self.veto_start_ts);
                let duration_mins = duration_ms / 60_000;
                let duration_hrs = duration_ms as f64 / 3_600_000.0;
                let time_str = format_utc_ts(current_ts);

                self.gatekeeper_state_transitions += 1;
                self.zones.push(GatekeeperZoneRecord {
                    start_bar: self.veto_start_bar,
                    end_bar: current_bar,
                    start_ts: self.veto_start_ts,
                    end_ts: current_ts,
                    reason: prev.as_str().to_string(),
                });

                Some(format!(
                    "✅ [GATEKEEPER DISENGAGED] [{}] Bar: {} | Market Stabilized from {:?} | Chaos: {:.3} | NATR: {:.2}% | Storm Duration: {} bars ({} mins, {:.1} hrs)",
                    time_str, current_bar, prev, gate_eval.current_chaos, gate_eval.market_natr_pct, duration_bars, duration_mins, duration_hrs
                ))
            } else {
                None
            }
        }
    }

    /// Xuất danh sách các vùng bảo vệ dạng JSON cho Visual Report
    pub fn export_zones_json(&self) -> String {
        let mut all_zones = self.zones.clone();
        if let Some(active_reason) = self.prev_gate_veto {
            all_zones.push(GatekeeperZoneRecord {
                start_bar: self.veto_start_bar,
                end_bar: self.last_seen_bar,
                start_ts: self.veto_start_ts,
                end_ts: self.last_seen_ts,
                reason: active_reason.as_str().to_string(),
            });
        }
        serde_json::to_string(&all_zones).unwrap_or_else(|_| "[]".to_string())
    }

    /// Kiểm tra và phát nhịp đập thị trường (Hourly Heartbeat) mỗi 60 phút (dựa trên mốc thời gian thực tế ms hoặc bars)
    pub fn check_hourly_heartbeat(
        &mut self,
        current_bar: usize,
        current_ts: i64,
        current_price: f64,
        gate_eval: &GateEvaluation,
        kama_slope: f64,
        squeeze: f64,
    ) -> Option<String> {
        let is_due = if self.last_heartbeat_ts > 0 && current_ts > 0 {
            current_ts.saturating_sub(self.last_heartbeat_ts) >= 60 * 60 * 1000
        } else if self.last_heartbeat_bar > 0 {
            current_bar.saturating_sub(self.last_heartbeat_bar) >= 60
        } else {
            self.last_heartbeat_bar = current_bar;
            self.last_heartbeat_ts = current_ts;
            return None;
        };

        if is_due {
            self.last_heartbeat_bar = current_bar;
            self.last_heartbeat_ts = current_ts;

            let time_str = format_utc_ts(current_ts);
            let regime_str = if gate_eval.passed {
                gate_eval.regime.as_str()
            } else {
                gate_eval.veto_reason.map(|r| r.as_str()).unwrap_or(gate_eval.regime.as_str())
            };
            let gate_status = if gate_eval.passed {
                "PASS".to_string()
            } else {
                format!("VETO({:?})", gate_eval.veto_reason)
            };

            let eligible_str = match gate_eval.regime {
                MarketRegime::Trending => "[TrendPB, MomIGN, AbsScout]",
                MarketRegime::RangeBound => "[RangeOsc, ClimaxRev, AbsScout, AnomalyFade]",
                MarketRegime::Breakout => "[MomIGN, AbsScout]",
                MarketRegime::VolatilityFlatline | MarketRegime::TurbulentChaos | MarketRegime::DataAnomaly => "[] (All Blocked)",
            };

            Some(format!(
                "💓 [MARKET HEARTBEAT {}] Bar: {} | Price: {:.2} | Regime: {} | KAMA_Slope: {:+.2} | Squeeze: {:.2} | Chaos: {:.3} (Max: {:.2}) | NATR: {:.2}% (Req: {:.2}%) | Gate: {} | Eligible: {}",
                time_str, current_bar, current_price, regime_str, kama_slope, squeeze, gate_eval.current_chaos, gate_eval.chaos_threshold, gate_eval.market_natr_pct, gate_eval.required_min_natr_pct, gate_status, eligible_str
            ))
        } else {
            None
        }
    }

    /// Cập nhật số liệu từ kết quả đấu thầu của Arbiter
    pub fn on_auction_decision(&mut self, decision: &ArbiterDecision) {
        for candidate in &decision.auction_ledger {
            if candidate.status == AuctionStatus::StructuralRiskVeto {
                self.sniper_structural_sl_vetos += 1;
            }
        }
    }

    /// Theo dõi và ghi nhận chốt chặn Tầng 2 (Boundary Guard & Chop Zone) cho TrendPB
    pub fn on_trend_pb_tick(
        &mut self,
        squeeze: f64,
        max_squeeze: f64,
        kama_slope: f64,
        slope_thresh: f64,
        z_score: f64,
        max_boundary_z: f64,
    ) {
        if squeeze > max_squeeze {
            self.trend_pb_chop_zone_vetos += 1;
        }
        if kama_slope > slope_thresh && z_score > max_boundary_z {
            self.trend_pb_boundary_long_vetos += 1;
        } else if kama_slope < -slope_thresh && z_score < -max_boundary_z {
            self.trend_pb_boundary_short_vetos += 1;
        }
    }

    /// Ghi nhận lệnh có SL thô bị kẹp về trần
    pub fn on_sl_clamped(&mut self, is_hard_cap: bool) {
        if is_hard_cap {
            self.sl_clamped_to_hard_cap += 1;
        } else {
            self.sl_clamped_to_playbook_cap += 1;
        }
    }

    /// Định dạng Bảng Sổ Cái Tổng Kết Ma Trận Phòng Thủ Vốn (FINAL DEFENSE LEDGER)
    pub fn format_summary_table(&self) -> String {
        let chaos_hours = self.chaos_whipsaw_bars as f64 / 60.0;
        let freeze_hours = self.flash_crash_freeze_bars as f64 / 60.0;
        let flatline_hours = self.volatility_flatline_bars as f64 / 60.0;

        let mut lines = Vec::new();
        lines.push("╔═══════════════════════════════════════════════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║ 🛡️ CAPITAL DEFENSE MATRIX: OBSERVABILITY & AUDIT LEDGER                                              ║".to_string());
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 1. TẦNG 1: REGIME GATEKEEPER                                                                         ║".to_string());
        lines.push(format!(
            "║    - Chaos Whipsaw Storm Sheltered : {:>8} bars ({:>5.1} hrs né bão hỗn loạn)                        ║",
            self.chaos_whipsaw_bars, chaos_hours
        ));
        lines.push(format!(
            "║    - Flash Crash Shock Frozen      : {:>8} bars ({:>5.1} hrs né sốc sập nến)                         ║",
            self.flash_crash_freeze_bars, freeze_hours
        ));
        lines.push(format!(
            "║    - Volatility Flatline Blocked   : {:>8} bars ({:>5.1} hrs né sàn chết thanh khoản)                ║",
            self.volatility_flatline_bars, flatline_hours
        ));
        lines.push(format!(
            "║    - Total Gatekeeper State Shifts : {:>8} times                                                     ║",
            self.gatekeeper_state_transitions
        ));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 2. TẦNG 2: BOUNDARY GUARD & CHOP ZONE (TrendPB)                                                       ║".to_string());
        lines.push(format!(
            "║    - Overbought Long Blocked (Z>max): {:>7} signals (Chặn mua đu đỉnh phân phối)                    ║",
            self.trend_pb_boundary_long_vetos
        ));
        lines.push(format!(
            "║    - Oversold Short Blocked (Z<-max): {:>7} signals (Chặn bán tháo hoảng loạn)                     ║",
            self.trend_pb_boundary_short_vetos
        ));
        lines.push(format!(
            "║    - Squeeze Chop Zone Blocked     : {:>7} signals (Chặn cào lệnh khi thị trường nén)               ║",
            self.trend_pb_chop_zone_vetos
        ));
        lines.push("╠═══════════════════════════════════════════════════════════════════════════════════════════════════════╣".to_string());
        lines.push("║ 3. TẦNG 3: STOP LOSS CAP & SNIPER ENTRY GUARD                                                         ║".to_string());
        lines.push(format!(
            "║    - Sniper Structural SL Vetoed   : {:>7} proposals (Bắn hạ lệnh có SL > 15% ROE)                  ║",
            self.sniper_structural_sl_vetos
        ));
        lines.push(format!(
            "║    - Initial SL Clamped to Hard Cap: {:>7} orders    (Kẹp cứng SL về trần 15% ROE)                 ║",
            self.sl_clamped_to_hard_cap
        ));
        lines.push(format!(
            "║    - Initial SL Clamped to Playbook: {:>7} orders    (Kẹp SL theo trần Playbook DNA)                ║",
            self.sl_clamped_to_playbook_cap
        ));
        lines.push("╚═══════════════════════════════════════════════════════════════════════════════════════════════════════╝".to_string());

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::core_v3::gatekeeper::GateVetoReason;
    use crate::strategy::core_v3::arbiter::{AuctionCandidate, ScannerId, SignalType};

    #[test]
    fn test_gatekeeper_state_transition_edge_detection() {
        let mut ledger = DefenseLedger::new();

        // 1. Bình thường (PASS)
        let eval_pass = GateEvaluation {
            passed: true,
            veto_reason: None,
            current_chaos: 0.15,
            market_natr_pct: 0.20,
            is_hysteresis_active: false,
            ..Default::default()
        };
        assert!(ledger.on_gatekeeper_tick(100, 100_000, &eval_pass).is_none());

        // 2. Cạnh lên (Rising Edge): Bắt đầu vào bão Chaos
        let eval_chaos = GateEvaluation {
            passed: false,
            veto_reason: Some(GateVetoReason::ChaosWhipsaw),
            current_chaos: 0.35,
            market_natr_pct: 0.20,
            is_hysteresis_active: false,
            chaos_threshold: 0.65,
            ..Default::default()
        };
        let msg_engaged = ledger.on_gatekeeper_tick(101, 101_000, &eval_chaos);
        assert!(msg_engaged.is_some());
        assert!(msg_engaged.unwrap().contains("[GATEKEEPER ENGAGED]"));
        assert_eq!(ledger.chaos_whipsaw_bars, 1);
        assert_eq!(ledger.gatekeeper_state_transitions, 1);

        // 3. Trong bão (No spam): Bar 102 tiếp tục bão
        let msg_stay = ledger.on_gatekeeper_tick(102, 102_000, &eval_chaos);
        assert!(msg_stay.is_none(), "Trong bão không được spam log!");
        assert_eq!(ledger.chaos_whipsaw_bars, 2);
        assert_eq!(ledger.gatekeeper_state_transitions, 1);

        // 4. Cạnh xuống (Falling Edge): Hết bão ở Bar 150
        let msg_disengaged = ledger.on_gatekeeper_tick(150, 150_000, &eval_pass);
        assert!(msg_disengaged.is_some());
        let disengage_str = msg_disengaged.unwrap();
        assert!(disengage_str.contains("[GATEKEEPER DISENGAGED]"));
        assert!(disengage_str.contains("Storm Duration: 49 bars"));
        assert_eq!(ledger.gatekeeper_state_transitions, 2);

        // 5. Kiểm tra export zones
        assert_eq!(ledger.zones.len(), 1);
        assert_eq!(ledger.zones[0].start_bar, 101);
        assert_eq!(ledger.zones[0].end_bar, 150);
        assert_eq!(ledger.zones[0].start_ts, 101_000);
        assert_eq!(ledger.zones[0].end_ts, 150_000);
        assert!(ledger.export_zones_json().contains("CHAOS_WHIPSAW"));
    }

    #[test]
    fn test_gatekeeper_hourly_heartbeat() {
        let mut ledger = DefenseLedger::new();
        let eval_flatline = GateEvaluation {
            passed: false,
            veto_reason: Some(GateVetoReason::MacroFlatline),
            current_chaos: 0.10,
            market_natr_pct: 0.15,
            required_min_natr_pct: 0.25,
            ..Default::default()
        };

        // Lần đầu tiên gọi heartbeat (bar 100): chỉ lưu mốc ban đầu, chưa phát
        let hb_init = ledger.check_hourly_heartbeat(100, 0, 2500.0, &eval_flatline, 0.05, 1.20);
        assert!(hb_init.is_none());

        // Sau 30 bars (chưa đủ 60 bars): chưa phát
        let hb_early = ledger.check_hourly_heartbeat(130, 0, 2502.0, &eval_flatline, 0.05, 1.20);
        assert!(hb_early.is_none());

        // Sau 60 bars (bar 160): phát ra heartbeat!
        let hb_60 = ledger.check_hourly_heartbeat(160, 0, 2505.0, &eval_flatline, 0.05, 1.20);
        assert!(hb_60.is_some());
        let msg = hb_60.unwrap();
        assert!(msg.contains("[MARKET HEARTBEAT"));
        assert!(msg.contains("MACRO_FLATLINE"));
        assert!(msg.contains("NATR: 0.15% (Req: 0.25%)"));
    }

    #[test]
    fn test_hourly_heartbeat_timestamp_based_in_actor_sliding_window() {
        let mut ledger = DefenseLedger::new();
        let eval_trending = GateEvaluation {
            passed: true,
            regime: MarketRegime::Trending,
            veto_reason: None,
            current_chaos: 0.12,
            market_natr_pct: 1.85,
            required_min_natr_pct: 1.20,
            chaos_threshold: 0.35,
            ..Default::default()
        };

        // Giả lập môi trường Live/Paper Actor: current_bar luôn cố định là 57599
        let fixed_bar = 57599;
        let start_ts = 1_700_000_000_000i64; // Epoch ms

        // 1. Tick đầu tiên: khởi tạo mốc thời gian
        let hb_init = ledger.check_hourly_heartbeat(fixed_bar, start_ts, 2650.0, &eval_trending, 0.15, 0.40);
        assert!(hb_init.is_none());

        // 2. Sau 59 phút (59 * 60_000 ms): Chưa đủ 60 phút -> chưa phát
        let ts_59m = start_ts + 59 * 60 * 1000;
        let hb_59m = ledger.check_hourly_heartbeat(fixed_bar, ts_59m, 2655.0, &eval_trending, 0.16, 0.38);
        assert!(hb_59m.is_none(), "Chưa đủ 60 phút không được phát heartbeat!");

        // 3. Đúng 60 phút (60 * 60_000 ms): Đủ 60 phút -> PHẢI PHÁT HEARTBEAT dù current_bar vẫn là 57599!
        let ts_60m = start_ts + 60 * 60 * 1000;
        let hb_60m = ledger.check_hourly_heartbeat(fixed_bar, ts_60m, 2660.0, &eval_trending, 0.18, 0.35);
        assert!(hb_60m.is_some(), "Sau 60 phút timestamp thực tế, heartbeat bắt buộc phải nổ!");
        
        let msg = hb_60m.unwrap();
        assert!(msg.contains("[MARKET HEARTBEAT"));
        assert!(msg.contains("Regime: TRENDING"));
        assert!(msg.contains("KAMA_Slope: +0.18"));
        assert!(msg.contains("Gate: PASS"));
        assert!(msg.contains("Eligible: [TrendPB, MomIGN, AbsScout]"));
    }

    #[test]
    fn test_tier2_trend_pb_counters() {
        let mut ledger = DefenseLedger::new();

        // Chop zone triggered
        ledger.on_trend_pb_tick(0.85, 0.75, 0.05, 0.02, 0.30, 0.50);
        assert_eq!(ledger.trend_pb_chop_zone_vetos, 1);

        // Boundary Guard Long triggered (Z > max_z)
        ledger.on_trend_pb_tick(0.60, 0.75, 0.08, 0.02, 0.70, 0.50);
        assert_eq!(ledger.trend_pb_boundary_long_vetos, 1);

        // Boundary Guard Short triggered (Z < -max_z)
        ledger.on_trend_pb_tick(0.60, 0.75, -0.08, 0.02, -0.70, 0.50);
        assert_eq!(ledger.trend_pb_boundary_short_vetos, 1);
    }

    #[test]
    fn test_tier3_sniper_veto_and_clamping_counters() {
        let mut ledger = DefenseLedger::new();

        use crate::strategy::core_v3::gatekeeper::GateEvaluation;

        let decision = ArbiterDecision {
            is_approved: false,
            final_signal: None,
            winning_scanner: None,
            final_confidence: 0.0,
            final_sl: None,
            final_tp: None,
            chaos_val: 0.15,
            chaos_penalty: 0.0,
            gate_veto: None,
            gate_eval: GateEvaluation::default(),
            auction_ledger: vec![
                AuctionCandidate {
                    scanner_id: ScannerId::AbsorptionScout,
                    signal_type: Some(SignalType::Long),
                    raw_score: 0.75,
                    confidence: 0.75,
                    status: AuctionStatus::StructuralRiskVeto,
                    reason: "Structural SL exceeds ROE cap".to_string(),
                }
            ],
        };

        ledger.on_auction_decision(&decision);
        assert_eq!(ledger.sniper_structural_sl_vetos, 1);

        ledger.on_sl_clamped(true);
        ledger.on_sl_clamped(false);
        assert_eq!(ledger.sl_clamped_to_hard_cap, 1);
        assert_eq!(ledger.sl_clamped_to_playbook_cap, 1);

        let summary = ledger.format_summary_table();
        assert!(summary.contains("CAPITAL DEFENSE MATRIX: OBSERVABILITY & AUDIT LEDGER"));
        assert!(summary.contains("Sniper Structural SL Vetoed"));
        assert!(summary.contains("Flash Crash Shock Frozen"));
    }

    #[test]
    fn test_gatekeeper_flash_crash_freeze_edge_detection() {
        let mut ledger = DefenseLedger::new();

        let eval_shock = GateEvaluation {
            passed: false,
            veto_reason: Some(GateVetoReason::FlashCrashFreeze),
            current_chaos: 0.30,
            market_natr_pct: 0.80,
            is_shock_freeze_active: true,
            ..Default::default()
        };

        let msg_start = ledger.on_gatekeeper_tick(100, 100_000, &eval_shock);
        assert!(msg_start.is_some());
        let alert = msg_start.unwrap();
        assert!(alert.contains("FlashCrashFreeze"));
        assert!(alert.contains("Kinetic Shock detected in lookback"));
        assert_eq!(ledger.flash_crash_freeze_bars, 1);
        assert_eq!(ledger.gatekeeper_state_transitions, 1);

        // Nến kế tiếp vẫn bị freeze: không phát lại alert nhưng tăng số bar
        let msg_cont = ledger.on_gatekeeper_tick(101, 101_000, &eval_shock);
        assert!(msg_cont.is_none());
        assert_eq!(ledger.flash_crash_freeze_bars, 2);

        // Hết freeze: quay lại PASS
        let eval_pass = GateEvaluation {
            passed: true,
            veto_reason: None,
            current_chaos: 0.20,
            market_natr_pct: 0.50,
            is_shock_freeze_active: false,
            ..Default::default()
        };
        let msg_end = ledger.on_gatekeeper_tick(120, 120_000, &eval_pass);
        assert!(msg_end.is_some());
        assert_eq!(ledger.gatekeeper_state_transitions, 2);
        assert_eq!(ledger.zones.len(), 1);
        assert_eq!(ledger.zones[0].reason, "FLASH_CRASH_FREEZE");
    }
}
