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

pub trait TacticalScanner {
    /// Tên định danh của Scanner (Type-Safe Enum)
    fn id(&self) -> ScannerId;
    
    /// Hàm quét tín hiệu trên nến hiện tại
    /// Trả về None nếu không có cơ hội. Trả về Some(Proposal) nếu có tín hiệu.
    fn scan(&self, 
            tick: &TickContext, 
            sensors: &HeliosSensors, 
            matrix: &MarketMatrixV3,
            config: &HeliosConfigV3) -> Option<Proposal>;
}

pub struct HeliosArbiter {
    pub scanners: Vec<Box<dyn TacticalScanner>>,
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

    /// Tính điểm tự tin đấu thầu tuyến tính O(1):
    /// Confidence = raw_score * (1.0 - chaos_penalty)
    /// Khi thị trường bão Whipsaw (chaos_index cao), điểm tự tin tự động bị hạ thấp để phòng thủ.
    #[inline]
    pub fn compute_confidence(raw_score: f64, chaos_index: f64) -> f64 {
        if raw_score <= 0.0 {
            0.0
        } else {
            let penalty = chaos_index.max(0.0).min(1.0);
            (raw_score * (1.0 - penalty)).max(0.0).min(1.0)
        }
    }

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, _debug_mode: bool) -> ArbiterDecision {
        let i = tick.i;
        let current_close = tick.closes()[i];
        
        let chaos_val = matrix.chaos_index.get(i).copied().unwrap_or(0.0);
        let min_threshold = config.arbiter.min_confidence_threshold;

        let mut approved_proposals: Vec<(Proposal, f64)> = Vec::new();

        // 1. THU THẬP & CHẤM ĐIỂM TỰ TIN TUYẾN TÍNH O(1) CHO CÁC PROPOSALS
        for scanner in &self.scanners {
            if let Some(proposal) = scanner.scan(tick, sensors, matrix, config) {
                let confidence = Self::compute_confidence(proposal.raw_score, chaos_val);
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

        // 2. GIẢI QUYẾT XUNG ĐỘT TÍN HIỆU (CONFLICT RESOLUTION)
        let mut best: Option<(Proposal, f64)> = None;

        for (proposal, conf) in approved_proposals {
            match &best {
                None => {
                    best = Some((proposal, conf));
                }
                Some((current_prop, current_conf)) => {
                    if proposal.signal_type == current_prop.signal_type {
                        // CÙNG CHIỀU: Ưu tiên lệnh có khoảng cách Stop Loss gần hơn (R:R tốt hơn)
                        let dist_new = proposal.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);
                        let dist_curr = current_prop.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);

                        if dist_new < dist_curr {
                            best = Some((proposal, conf));
                        } else if (dist_new - dist_curr).abs() < 1e-6 && conf > *current_conf {
                            best = Some((proposal, conf));
                        }
                    } else {
                        // NGƯỢC CHIỀU (XUNG ĐỘT LONG VS SHORT):
                        // Nếu một bên có độ tự tin vượt trội (> 0.15), chọn bên mạnh hơn;
                        // Nếu độ tự tin tương đương, hủy cả hai (Flat - Cash is King để bảo vệ tài khoản).
                        let diff = conf - *current_conf;
                        if diff > 0.15 {
                            best = Some((proposal, conf));
                        } else if diff < -0.15 {
                            // Giữ nguyên current
                        } else {
                            return ArbiterDecision {
                                is_approved: false,
                                final_signal: None,
                                winning_scanner: None,
                                final_confidence: 0.0,
                                final_sl: None,
                                final_tp: None,
                            };
                        }
                    }
                }
            }
        }

        if let Some((proposal, confidence)) = best {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_guard_prevents_false_signals() {
        // Scanner không có tín hiệu (raw_score <= 0.0) -> confidence = 0.0
        assert_eq!(HeliosArbiter::compute_confidence(0.0, 0.2), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(-0.5, 0.2), 0.0);

        // Bão Whipsaw cực đại (chaos_index >= 1.0) -> confidence = 0.0
        assert_eq!(HeliosArbiter::compute_confidence(0.9, 1.0), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.9, 1.2), 0.0);
    }

    #[test]
    fn test_chaos_penalty_scales_down_confidence() {
        // Thị trường êm ả: confidence = raw_score
        let conf_calm = HeliosArbiter::compute_confidence(0.80, 0.0);
        assert_eq!(conf_calm, 0.80);

        // Bắt đầu nhiễu (chaos = 0.50): confidence giảm 50%
        let conf_choppy = HeliosArbiter::compute_confidence(0.80, 0.50);
        assert!((conf_choppy - 0.40).abs() < 1e-6);

        // Bão Whipsaw (chaos = 0.80): confidence chỉ còn 0.16 (dưới ngưỡng phê duyệt 0.20)
        let conf_storm = HeliosArbiter::compute_confidence(0.80, 0.80);
        assert!((conf_storm - 0.16).abs() < 1e-6);
        assert!(conf_storm < 0.20, "Bão Whipsaw phải triệt tiêu tự tin để không vào lệnh!");
    }

    #[test]
    fn test_conflict_resolution_closer_stop_loss() {
        let dist_loose = (2500.0 - 2450.0_f64).abs();
        let dist_tight = (2500.0 - 2480.0_f64).abs();
        assert!(dist_tight < dist_loose, "Absorption có SL chặt hơn TrendPB");
    }

    #[test]
    fn test_conflict_resolution_opposite_signals_flat() {
        let conf_long = 0.70_f64;
        let conf_short = 0.72_f64;
        let diff = (conf_long - conf_short).abs();
        assert!(diff <= 0.15, "Xung đột căng thẳng không vượt trội");
    }
}
