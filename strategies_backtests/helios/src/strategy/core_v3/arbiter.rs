// strategies_backtests/helios/src/strategy/core_v3/arbiter.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::gatekeeper::{RegimeGatekeeper, GateVetoReason};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Long,
    Short,
}

/// [TYPE-SAFE SCANNER ID]
/// Định danh duy nhất cho từng Tactical Scanner theo chuẩn Idiomatic Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScannerId {
    TrendPullback,      // TREND_PB
    MomentumIgnition,   // MOMENTUM_IGN
    RangeOscillator,    // RANGE_OSC
    ClimaxReversal,     // REVERSAL
    AnomalyFade,        // ANOMALY
    AbsorptionScout,    // ABSORPTION
}

impl ScannerId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScannerId::TrendPullback => "TREND_PB",
            ScannerId::MomentumIgnition => "MOMENTUM_IGN",
            ScannerId::RangeOscillator => "RANGE_OSC",
            ScannerId::ClimaxReversal => "REVERSAL",
            ScannerId::AnomalyFade => "ANOMALY",
            ScannerId::AbsorptionScout => "ABSORPTION",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proposal {
    /// Loại tín hiệu
    pub signal_type: SignalType,
    /// Điểm tự tin do Scanner tự chấm dựa trên độ hoàn hảo của Pattern (0.0 đến 1.0)
    pub raw_score: f64,
    /// Tên định danh của Scanner (Type-Safe Enum)
    pub scanner_id: ScannerId,
    pub suggested_sl: Option<f64>,
    pub suggested_tp: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionStatus {
    Won,
    RunnerUp,
    LowConfidence,
    ConflictEliminated,
    NotTriggered,
}

#[derive(Debug, Clone)]
pub struct AuctionCandidate {
    pub scanner_id: ScannerId,
    pub signal_type: Option<SignalType>,
    pub raw_score: f64,
    pub confidence: f64,
    pub status: AuctionStatus,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ArbiterDecision {
    pub is_approved: bool,
    pub final_signal: Option<SignalType>,
    pub winning_scanner: Option<ScannerId>,
    pub final_confidence: f64,
    pub final_sl: Option<f64>,
    pub final_tp: Option<f64>,
    pub chaos_val: f64,
    pub chaos_penalty: f64,
    pub gate_veto: Option<GateVetoReason>,
    pub auction_ledger: Vec<AuctionCandidate>,
}

pub trait TacticalScanner: Send + Sync {
    /// Tên định danh của Scanner (Type-Safe Enum)
    fn id(&self) -> ScannerId;
    
    /// Hàm quét tín hiệu trên nến hiện tại
    /// Trả về None nếu không có cơ hội. Trả về Some(Proposal) nếu có tín hiệu.
    fn scan(&self, 
            tick: &TickContext, 
            sensors: &HeliosSensors, 
            matrix: &MarketMatrixV3,
            config: &HeliosConfigV3) -> Option<Proposal>;

    /// Hỗ trợ clone trait object theo chuẩn idiomatic Rust
    fn clone_box(&self) -> Box<dyn TacticalScanner>;
}

impl Clone for Box<dyn TacticalScanner> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct HeliosArbiter {
    pub scanners: Vec<Box<dyn TacticalScanner>>,
}

impl std::fmt::Debug for HeliosArbiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scanner_names: Vec<&'static str> = self.scanners.iter().map(|s| s.id().as_str()).collect();
        f.debug_struct("HeliosArbiter")
            .field("scanners", &scanner_names)
            .finish()
    }
}

impl Default for HeliosArbiter {
    fn default() -> Self {
        Self::new()
    }
}

impl HeliosArbiter {
    pub fn new() -> Self {
        Self {
            scanners: Vec::new(),
        }
    }

    pub fn add_scanner(&mut self, scanner: Box<dyn TacticalScanner>) {
        self.scanners.push(scanner);
    }

    /// [LEVEL 4: PURE CONFIDENCE (FLAW-RISK.11 Refactor)]
    /// Sau khi Cầu Dao Gatekeeper (Tầng 1) đã thẩm định thị trường an toàn (Tradable Regime),
    /// điểm tự tin phản ánh 100% độ hoàn hảo của mẫu hình (Pattern Quality) từ chính Scanner.
    /// Loại bỏ hình phạt mềm (quadratic chaos penalty) cũ để tránh tình trạng "phạt 2 lần"
    /// và ngăn chặn các setup kém chất lượng lọt qua cửa khi bão bùng phát.
    /// 
    /// - An toàn tuyệt đối với NaN: Trả về 0.0 nếu raw_score là NaN hoặc <= 0.0.
    #[inline]
    pub fn compute_confidence(raw_score: f64, _chaos_index: f64, _max_chaos_penalty: f64) -> f64 {
        if raw_score.is_nan() || raw_score <= 0.0 {
            0.0
        } else {
            raw_score.clamp(0.0, 1.0)
        }
    }

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, debug_mode: bool) -> ArbiterDecision {
        let i = tick.i;
        
        let chaos_val = matrix.chaos_index.get(i).copied().unwrap_or(0.0);
        let min_threshold = config.arbiter.min_confidence_threshold;
        let conflict_margin = config.arbiter.conflict_confidence_margin;
        let max_chaos_penalty = config.arbiter.max_chaos_penalty;

        // 🛡️ TẦNG 1: CẦU DAO ĐIỀU HƯỚNG CHẾ ĐỘ THỊ TRƯỜNG (REGIME GATEKEEPER)
        // Thoát sớm O(1) nếu thị trường rơi vào tử địa (Bão Whipsaw hoặc Sàn biến động chết Flatline).
        // Triệt tiêu 100% lãng phí CPU chạy Scanners khi thị trường không thể giao dịch (EV < 0).
        let gate_eval = RegimeGatekeeper::evaluate(tick, sensors, matrix, config);
        if !gate_eval.passed {
            if debug_mode {
                info!(
                    "🛡️ [GATEKEEPER VETO] Bar: {} | Veto: {:?} | Chaos: {:.3} (Hysteresis: {}) | NATR: {:.2}%",
                    i,
                    gate_eval.veto_reason,
                    gate_eval.current_chaos,
                    gate_eval.is_hysteresis_active,
                    gate_eval.current_natr
                );
            }
            return ArbiterDecision {
                is_approved: false,
                final_signal: None,
                winning_scanner: None,
                final_confidence: 0.0,
                final_sl: None,
                final_tp: None,
                chaos_val: gate_eval.current_chaos,
                chaos_penalty: 0.0,
                gate_veto: gate_eval.veto_reason,
                auction_ledger: Vec::new(),
            };
        }

        // Tính chaos penalty thực tế (0.0 dưới mô hình Pure Confidence)
        let chaos_penalty = 0.0;

        let mut scanned_info: Vec<(ScannerId, Option<Proposal>, f64, bool)> = Vec::with_capacity(self.scanners.len());
        let mut approved_proposals: Vec<(Proposal, f64)> = Vec::new();

        // 1. THU THẬP & CHẤM ĐIỂM TỰ TIN PHI TUYẾN O(1) CHO CÁC PROPOSALS
        for scanner in &self.scanners {
            let id = scanner.id();
            let opt_prop = scanner.scan(tick, sensors, matrix, config);
            let (conf, passed) = if let Some(ref proposal) = opt_prop {
                let confidence = Self::compute_confidence(proposal.raw_score, chaos_val, max_chaos_penalty);
                let ok = confidence >= min_threshold;
                if ok {
                    approved_proposals.push((proposal.clone(), confidence));
                }
                (confidence, ok)
            } else {
                (0.0, false)
            };
            scanned_info.push((id, opt_prop, conf, passed));
        }

        // TELEMETRY: In biên bản đấu thầu nếu có ít nhất 1 proposal và debug_mode được bật
        if debug_mode {
            let triggered_proposals: Vec<_> = scanned_info.iter().filter_map(|(_, opt_p, conf, passed)| {
                opt_p.as_ref().map(|p| (p, *conf, *passed))
            }).collect();

            if !triggered_proposals.is_empty() {
                info!(
                    "⚖️ [ARBITER AUCTION] Bar: {} | Scanned Proposals: {} | Approved: {} | Chaos: {:.3} | Min Conf Thresh: {:.3} | Margin: {:.3}",
                    i, triggered_proposals.len(), approved_proposals.len(), chaos_val, min_threshold, conflict_margin
                );
                for (p, conf, passed) in &triggered_proposals {
                    info!(
                        "   - [{:<8}] Scanner: {:<12} | Side: {:<5} | Raw: {:.3} | Conf: {:.3} | SL: {:?} | TP: {:?}",
                        if *passed { "APPROVED" } else { "REJECTED" },
                        p.scanner_id.as_str(),
                        format!("{:?}", p.signal_type),
                        p.raw_score,
                        conf,
                        p.suggested_sl,
                        p.suggested_tp
                    );
                }
            }
        }

        if approved_proposals.is_empty() {
            if debug_mode && scanned_info.iter().any(|(_, opt_p, _, _)| opt_p.is_some()) {
                info!("   => [ARBITER RESULT] REJECTED: All proposals failed min_confidence_threshold ({:.3})", min_threshold);
            }
            let auction_ledger = scanned_info
                .into_iter()
                .map(|(id, opt_p, conf, _)| {
                    let (sig, raw, status, reason) = if let Some(p) = opt_p {
                        (
                            Some(p.signal_type),
                            p.raw_score,
                            AuctionStatus::LowConfidence,
                            format!("Confidence {:.3} < min_threshold {:.3}", conf, min_threshold),
                        )
                    } else {
                        (
                            None,
                            0.0,
                            AuctionStatus::NotTriggered,
                            "Pattern conditions not met".to_string(),
                        )
                    };
                    AuctionCandidate {
                        scanner_id: id,
                        signal_type: sig,
                        raw_score: raw,
                        confidence: conf,
                        status,
                        reason,
                    }
                })
                .collect();

            return ArbiterDecision {
                is_approved: false,
                final_signal: None,
                winning_scanner: None,
                final_confidence: 0.0,
                final_sl: None,
                final_tp: None,
                chaos_val,
                chaos_penalty,
                gate_veto: None,
                auction_ledger,
            };
        }

        // 2. GIẢI QUYẾT XUNG ĐỘT TÍN HIỆU QUA TWO-STAGE AUCTION (O(N), COMMUTATIVE, CONFIDENCE-FIRST)
        let winner = Self::resolve_conflicts(approved_proposals, conflict_margin);

        if let Some((ref proposal, confidence)) = winner {
            if debug_mode {
                info!(
                    "   => [ARBITER RESULT] ✅ APPROVED! Winner: {} ({:?}) | Final Conf: {:.3} | SL: {:?} | TP: {:?}",
                    proposal.scanner_id.as_str(), proposal.signal_type, confidence, proposal.suggested_sl, proposal.suggested_tp
                );
            }

            let winner_id = proposal.scanner_id;
            let winner_conf = confidence;
            let runner_up_conf = scanned_info
                .iter()
                .filter(|(id, _, _, passed)| *id != winner_id && *passed)
                .map(|(_, _, conf, _)| *conf)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let auction_ledger = scanned_info
                .into_iter()
                .map(|(id, opt_p, conf, passed)| {
                    let sig = opt_p.as_ref().map(|p| p.signal_type);
                    let raw = opt_p.as_ref().map(|p| p.raw_score).unwrap_or(0.0);
                    let (status, reason) = if id == winner_id {
                        let reason_str = if let Some(r_conf) = runner_up_conf {
                            format!("Won auction with confidence {:.3} (Lead: +{:.3} over runner-up)", winner_conf, winner_conf - r_conf)
                        } else {
                            format!("Won auction with confidence {:.3} (Sole approved proposal)", winner_conf)
                        };
                        (AuctionStatus::Won, reason_str)
                    } else if passed {
                        let reason_str = if sig == Some(proposal.signal_type) {
                            format!("Subordinate to winner on same side (Conf: {:.3} < {:.3})", conf, winner_conf)
                        } else {
                            format!("Defeated in inter-side face-off (Conf: {:.3} vs {:.3})", conf, winner_conf)
                        };
                        (AuctionStatus::RunnerUp, reason_str)
                    } else if opt_p.is_some() {
                        (AuctionStatus::LowConfidence, format!("Confidence {:.3} < min_threshold {:.3}", conf, min_threshold))
                    } else {
                        (AuctionStatus::NotTriggered, "Pattern conditions not met".to_string())
                    };

                    AuctionCandidate {
                        scanner_id: id,
                        signal_type: sig,
                        raw_score: raw,
                        confidence: conf,
                        status,
                        reason,
                    }
                })
                .collect();

            ArbiterDecision {
                is_approved: true,
                final_signal: Some(proposal.signal_type),
                winning_scanner: Some(proposal.scanner_id),
                final_confidence: confidence,
                final_sl: proposal.suggested_sl,
                final_tp: proposal.suggested_tp,
                chaos_val,
                chaos_penalty,
                gate_veto: None,
                auction_ledger,
            }
        } else {
            if debug_mode {
                info!("   => [ARBITER RESULT] 🛡️ FLAT: Conflict deadlock between Long and Short (|diff| <= margin {:.3})", conflict_margin);
            }

            let auction_ledger = scanned_info
                .into_iter()
                .map(|(id, opt_p, conf, passed)| {
                    let sig = opt_p.as_ref().map(|p| p.signal_type);
                    let raw = opt_p.as_ref().map(|p| p.raw_score).unwrap_or(0.0);
                    let (status, reason) = if passed {
                        (AuctionStatus::ConflictEliminated, format!("Opposing signals deadlock (|diff| <= margin {:.3})", conflict_margin))
                    } else if opt_p.is_some() {
                        (AuctionStatus::LowConfidence, format!("Confidence {:.3} < min_threshold {:.3}", conf, min_threshold))
                    } else {
                        (AuctionStatus::NotTriggered, "Pattern conditions not met".to_string())
                    };

                    AuctionCandidate {
                        scanner_id: id,
                        signal_type: sig,
                        raw_score: raw,
                        confidence: conf,
                        status,
                        reason,
                    }
                })
                .collect();

            ArbiterDecision {
                is_approved: false,
                final_signal: None,
                winning_scanner: None,
                final_confidence: 0.0,
                final_sl: None,
                final_tp: None,
                chaos_val,
                chaos_penalty,
                gate_veto: None,
                auction_ledger,
            }
        }
    }

    /// ⚔️ TWO-STAGE AUCTION: Giải quyết xung đột tín hiệu O(N), Commutative, Confidence-First
    /// Tầng 1: Đấu thầu nội bộ từng chiều (Intra-Side Auction: tìm Long mạnh nhất và Short mạnh nhất)
    /// Tầng 2: Phân xử đối đầu liên chiều (Inter-Side Face-Off: Long vs Short)
    pub fn resolve_conflicts(approved_proposals: Vec<(Proposal, f64)>, conflict_margin: f64) -> Option<(Proposal, f64)> {
        let mut best_long: Option<(Proposal, f64)> = None;
        let mut best_short: Option<(Proposal, f64)> = None;

        // TẦNG 1: ĐẤU THẦU NỘI BỘ TỪNG CHIỀU (INTRA-SIDE AUCTION)
        for (proposal, conf) in approved_proposals {
            let target_slot = match proposal.signal_type {
                SignalType::Long => &mut best_long,
                SignalType::Short => &mut best_short,
            };

            match target_slot {
                None => {
                    *target_slot = Some((proposal, conf));
                }
                Some((current_prop, current_conf)) => {
                    // Ưu tiên 1: Confidence cao hơn (vượt sai số 1e-6)
                    if conf > *current_conf + 1e-6 {
                        *target_slot = Some((proposal, conf));
                    } else if (conf - *current_conf).abs() <= 1e-6 && proposal.raw_score > current_prop.raw_score {
                        // Ưu tiên 2 (Tie-breaking): raw_score cao hơn nếu confidence hòa
                        *target_slot = Some((proposal, conf));
                    }
                }
            }
        }

        // TẦNG 2: PHÂN XỬ LIÊN CHIỀU (INTER-SIDE FACE-OFF: LONG VS SHORT)
        match (best_long, best_short) {
            (Some(long_candidate), None) => Some(long_candidate),
            (None, Some(short_candidate)) => Some(short_candidate),
            (Some((l_prop, l_conf)), Some((s_prop, s_conf))) => {
                let diff = l_conf - s_conf;
                if diff > conflict_margin {
                    Some((l_prop, l_conf))
                } else if diff < -conflict_margin {
                    Some((s_prop, s_conf))
                } else {
                    // Xung đột giằng co không bên nào vượt trội (|diff| <= conflict_margin):
                    // Triệt tiêu cả 2 (Flat - Cash is King để bảo vệ tài khoản)
                    None
                }
            }
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_guard_prevents_false_signals() {
        // Scanner không có tín hiệu (raw_score <= 0.0) -> confidence = 0.0
        assert_eq!(HeliosArbiter::compute_confidence(0.0, 0.2, 0.50), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(-0.5, 0.2, 0.50), 0.0);
    }

    #[test]
    fn test_nan_safety_in_confidence() {
        assert_eq!(HeliosArbiter::compute_confidence(f64::NAN, 0.2, 0.50), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(f64::NAN, f64::NAN, f64::NAN), 0.0);
    }

    #[test]
    fn test_pure_confidence_score() {
        // Dưới mô hình Pure Confidence (FLAW-RISK.11), điểm tự tin phản ánh 100% Pattern Quality
        let conf_calm = HeliosArbiter::compute_confidence(0.80, 0.0, 0.50);
        assert_eq!(conf_calm, 0.80);

        let conf_choppy = HeliosArbiter::compute_confidence(0.80, 0.50, 0.50);
        assert_eq!(conf_choppy, 0.80);

        let conf_storm = HeliosArbiter::compute_confidence(0.80, 1.0, 0.50);
        assert_eq!(conf_storm, 0.80);

        // Kẹp trần 1.0
        let conf_over = HeliosArbiter::compute_confidence(1.20, 0.20, 0.50);
        assert_eq!(conf_over, 1.0);
    }

    #[test]
    fn test_conflict_resolution_higher_confidence_wins() {
        let margin = 0.15;
        // Đề xuất 1: TrendPB Long, confidence 0.85, SL xa ($2450)
        let p1 = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.85,
            scanner_id: ScannerId::TrendPullback,
            suggested_sl: Some(2450.0),
            suggested_tp: None,
        };
        // Đề xuất 2: Absorption Long, confidence 0.50, SL hẹp ($2480)
        let p2 = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.50,
            scanner_id: ScannerId::AbsorptionScout,
            suggested_sl: Some(2480.0),
            suggested_tp: None,
        };

        // Theo nguyên tắc Confidence-First: p1 (0.85) phải thắng p2 (0.50) dù SL của p1 xa hơn!
        let winner = HeliosArbiter::resolve_conflicts(vec![(p1, 0.85), (p2, 0.50)], margin);
        assert!(winner.is_some());
        let (winning_prop, winning_conf) = winner.unwrap();
        assert_eq!(winning_prop.scanner_id, ScannerId::TrendPullback);
        assert_eq!(winning_conf, 0.85);
    }

    #[test]
    fn test_conflict_resolution_tie_breaker_uses_raw_score() {
        let margin = 0.15;
        let p1 = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.82,
            scanner_id: ScannerId::TrendPullback,
            suggested_sl: Some(2450.0),
            suggested_tp: None,
        };
        let p2 = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.88,
            scanner_id: ScannerId::AbsorptionScout,
            suggested_sl: Some(2480.0),
            suggested_tp: None,
        };

        // Cả 2 cùng có confidence = 0.80 sau khi trừ penalty, nhưng p2 có raw_score cao hơn (0.88 > 0.82)
        let winner = HeliosArbiter::resolve_conflicts(vec![(p1, 0.80), (p2, 0.80)], margin);
        assert!(winner.is_some());
        let (winning_prop, _) = winner.unwrap();
        assert_eq!(winning_prop.scanner_id, ScannerId::AbsorptionScout);
    }

    #[test]
    fn test_conflict_resolution_opposite_signals_flat() {
        let margin = 0.15;
        let p_long = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.70,
            scanner_id: ScannerId::TrendPullback,
            suggested_sl: Some(2450.0),
            suggested_tp: None,
        };
        let p_short = Proposal {
            signal_type: SignalType::Short,
            raw_score: 0.72,
            scanner_id: ScannerId::ClimaxReversal,
            suggested_sl: Some(2550.0),
            suggested_tp: None,
        };

        // Xung đột giằng co (0.72 - 0.70 = 0.02 <= 0.15) -> Triệt tiêu (Flat)
        let winner = HeliosArbiter::resolve_conflicts(vec![(p_long, 0.70), (p_short, 0.72)], margin);
        assert!(winner.is_none(), "Xung đột căng thẳng không vượt trội phải hủy lệnh để bảo toàn vốn!");
    }

    #[test]
    fn test_conflict_resolution_opposite_signals_decisive_win() {
        let margin = 0.15;
        let p_long = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.85,
            scanner_id: ScannerId::TrendPullback,
            suggested_sl: Some(2450.0),
            suggested_tp: None,
        };
        let p_short = Proposal {
            signal_type: SignalType::Short,
            raw_score: 0.60,
            scanner_id: ScannerId::ClimaxReversal,
            suggested_sl: Some(2550.0),
            suggested_tp: None,
        };

        // Long vượt trội Short (0.85 - 0.60 = 0.25 > 0.15) -> Long chiến thắng
        let winner = HeliosArbiter::resolve_conflicts(vec![(p_long, 0.85), (p_short, 0.60)], margin);
        assert!(winner.is_some());
        let (winning_prop, winning_conf) = winner.unwrap();
        assert_eq!(winning_prop.signal_type, SignalType::Long);
        assert_eq!(winning_conf, 0.85);
    }

    #[test]
    fn test_two_stage_auction_commutativity() {
        let margin = 0.15;
        let p_long = Proposal {
            signal_type: SignalType::Long,
            raw_score: 0.90,
            scanner_id: ScannerId::TrendPullback,
            suggested_sl: Some(2450.0),
            suggested_tp: None,
        };
        let p_short = Proposal {
            signal_type: SignalType::Short,
            raw_score: 0.60,
            scanner_id: ScannerId::ClimaxReversal,
            suggested_sl: Some(2550.0),
            suggested_tp: None,
        };

        // Thứ tự 1: [Long, Short]
        let w1 = HeliosArbiter::resolve_conflicts(vec![(p_long.clone(), 0.90), (p_short.clone(), 0.60)], margin);
        // Thứ tự 2: [Short, Long]
        let w2 = HeliosArbiter::resolve_conflicts(vec![(p_short, 0.60), (p_long, 0.90)], margin);

        assert!(w1.is_some());
        assert!(w2.is_some());
        assert_eq!(w1.as_ref().unwrap().0.scanner_id, w2.as_ref().unwrap().0.scanner_id);
        assert_eq!(w1.unwrap().0.scanner_id, ScannerId::TrendPullback);
    }
}
