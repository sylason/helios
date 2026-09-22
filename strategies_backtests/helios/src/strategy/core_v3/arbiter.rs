// strategies_backtests/helios/src/strategy/core_v3/arbiter.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::config::HeliosConfigV3;

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

pub struct ArbiterDecision {
    pub is_approved: bool,
    pub final_signal: Option<SignalType>,
    pub winning_scanner: Option<ScannerId>,
    pub final_confidence: f64,
    pub final_sl: Option<f64>,
    pub final_tp: Option<f64>,
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

    /// Tính điểm tự tin đấu thầu phi tuyến bậc 2 (Quadratic) với Phạt Mềm Có Chặn Trần (Soft Capped Penalty):
    /// Confidence = raw_score * (1.0 - penalty)
    /// Trong đó: penalty = min(cap, cap * chaos_index^2) với cap = max_chaos_penalty
    /// - Khi thị trường chớm nhiễu (chaos_index = 0.50): mức phạt rất êm ái (chỉ 12.5% với cap 0.50),
    ///   giúp bảo vệ các setup chất lượng khi thị trường mới chỉ hơi gợn sóng.
    /// - Khi thị trường bão Whipsaw cực đại (chaos_index = 1.0), điểm tự tin bị trừ tối đa đúng bằng cap (DNA [0.10, 0.90]).
    /// - An toàn tuyệt đối với NaN: Trả về 0.0 nếu raw_score, chaos_index hoặc max_chaos_penalty là NaN.
    #[inline]
    pub fn compute_confidence(raw_score: f64, chaos_index: f64, max_chaos_penalty: f64) -> f64 {
        if raw_score.is_nan() || chaos_index.is_nan() || max_chaos_penalty.is_nan() || raw_score <= 0.0 {
            0.0
        } else {
            let cap = max_chaos_penalty.clamp(0.0, 1.0);
            let c_clamped = chaos_index.clamp(0.0, 1.0);
            let penalty = (cap * c_clamped * c_clamped).min(cap);
            (raw_score * (1.0 - penalty)).clamp(0.0, 1.0)
        }
    }

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, _debug_mode: bool) -> ArbiterDecision {
        let i = tick.i;
        
        let chaos_val = matrix.chaos_index.get(i).copied().unwrap_or(0.0);
        let min_threshold = config.arbiter.min_confidence_threshold;
        let conflict_margin = config.arbiter.conflict_confidence_margin;
        let max_chaos_penalty = config.arbiter.max_chaos_penalty;

        let mut approved_proposals: Vec<(Proposal, f64)> = Vec::new();

        // 1. THU THẬP & CHẤM ĐIỂM TỰ TIN PHI TUYẾN O(1) CHO CÁC PROPOSALS
        for scanner in &self.scanners {
            if let Some(proposal) = scanner.scan(tick, sensors, matrix, config) {
                let confidence = Self::compute_confidence(proposal.raw_score, chaos_val, max_chaos_penalty);
                if confidence >= min_threshold {
                    approved_proposals.push((proposal, confidence));
                }
            }
        }

        if approved_proposals.is_empty() {
            return ArbiterDecision {
                is_approved: false,
                final_signal: None,
                winning_scanner: None,
                final_confidence: 0.0,
                final_sl: None,
                final_tp: None,
            };
        }

        // 2. GIẢI QUYẾT XUNG ĐỘT TÍN HIỆU QUA TWO-STAGE AUCTION (O(N), COMMUTATIVE, CONFIDENCE-FIRST)
        let winner = Self::resolve_conflicts(approved_proposals, conflict_margin);

        if let Some((proposal, confidence)) = winner {
            ArbiterDecision {
                is_approved: true,
                final_signal: Some(proposal.signal_type),
                winning_scanner: Some(proposal.scanner_id),
                final_confidence: confidence,
                final_sl: proposal.suggested_sl,
                final_tp: proposal.suggested_tp,
            }
        } else {
            ArbiterDecision {
                is_approved: false,
                final_signal: None,
                winning_scanner: None,
                final_confidence: 0.0,
                final_sl: None,
                final_tp: None,
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
        assert_eq!(HeliosArbiter::compute_confidence(0.8, f64::NAN, 0.50), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.8, 0.2, f64::NAN), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(f64::NAN, f64::NAN, f64::NAN), 0.0);
    }

    #[test]
    fn test_soft_chaos_penalty_preserves_strong_signals() {
        // Thị trường êm ả (chaos = 0.0): confidence = raw_score
        let conf_calm = HeliosArbiter::compute_confidence(0.80, 0.0, 0.50);
        assert_eq!(conf_calm, 0.80);

        // Bắt đầu nhiễu (chaos = 0.50): penalty = 0.50 * (0.50)^2 = 0.125 (giảm 12.5% tự tin)
        // conf = 0.80 * (1.0 - 0.125) = 0.80 * 0.875 = 0.70
        let conf_choppy = HeliosArbiter::compute_confidence(0.80, 0.50, 0.50);
        assert!((conf_choppy - 0.70).abs() < 1e-6);

        // Bão Whipsaw cực đại (chaos = 1.0): penalty chặn trần ở 50%, confidence vẫn còn 0.40 >= 0.20
        let conf_storm = HeliosArbiter::compute_confidence(0.80, 1.0, 0.50);
        assert!((conf_storm - 0.40).abs() < 1e-6);
        assert!(conf_storm >= 0.20, "Setup đẹp (0.80) khi bão cực đại vẫn phải sống sót!");

        // Setup cực hạn (chaos = 1.5): penalty vẫn bị kẹp ở 50%
        let conf_extreme = HeliosArbiter::compute_confidence(0.90, 1.5, 0.50);
        assert!((conf_extreme - 0.45).abs() < 1e-6);

        // Kiểm tra với trần phạt 90% (max_chaos_penalty = 0.90):
        // Khi chaos = 1.0 -> penalty = 0.90 -> conf = 0.80 * (1.0 - 0.90) = 0.08
        let conf_severe = HeliosArbiter::compute_confidence(0.80, 1.0, 0.90);
        assert!((conf_severe - 0.08).abs() < 1e-6);
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
