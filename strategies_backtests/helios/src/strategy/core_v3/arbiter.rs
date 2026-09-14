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

    /// Tính điểm tự tin đấu thầu chuẩn Quant (Weighted Geometric Mean + Zero Guard).
    /// Triệt tiêu hoàn toàn lỗi 0^0 = 1.0, không chia cho 0, không bị bẫy bù trừ (Compensatory Trap),
    /// và mở khóa toàn bộ dải động min_confidence [0.01 .. 0.99] của DNA.
    #[inline]
    pub fn compute_confidence(
        raw_score: f64,
        regime_weight: f64,
        multiplier: f64,
        scanner_weight: f64,
    ) -> f64 {
        let context_product = regime_weight * multiplier;
        if raw_score <= 0.0 || context_product <= 0.0 {
            0.0
        } else {
            let context_score = context_product.sqrt();
            let w = scanner_weight.max(0.01).min(0.99);
            raw_score.powf(w) * context_score.powf(1.0 - w)
        }
    }

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, _debug_mode: bool) -> ArbiterDecision {
        let i = tick.i;
        
        // 1. TÍNH NGƯỠNG PHÊ DUYỆT ĐỘNG CONTINUOUS (DYNAMIC APPROVAL THRESHOLD 4 TRẠNG THÁI)
        let w_t = matrix.w_trend_norm[i];
        let w_r = matrix.w_range_norm[i];
        let w_c = matrix.w_comp_norm[i];
        let w_ch = matrix.w_chaos_norm[i];

        let required_confidence = w_t * config.arbiter.min_confidence_trending
                                + w_r * config.arbiter.min_confidence_ranging
                                + w_c * config.arbiter.min_confidence_compression
                                + w_ch * config.arbiter.min_confidence_chaos;

        let mut best_proposal: Option<(Proposal, f64)> = None;
        let mut max_confidence = 0.0;
        
        // 2. VÒNG LẶP THU THẬP & ĐẤU THẦU ĐA BỐI CẢNH (CONFIDENCE AUCTION)
        // Mọi Scanner đều được đối xử bình đẳng, không bị khóa cứng nhị phân
        for scanner in &self.scanners {
            let id = scanner.id();

            if let Some(proposal) = scanner.scan(tick, sensors, matrix, config) {
                // Tra cứu Trọng số Bối cảnh Simplex Đội Nhà (O(1)) 4 Trạng Thái
                let regime_weight = match id {
                    ScannerId::TrendPullback | ScannerId::MomentumIgnition => w_t,
                    ScannerId::RangeOscillator                             => w_r,
                    ScannerId::AbsorptionScout                             => w_c,
                    ScannerId::AnomalyFade | ScannerId::ClimaxReversal     => w_ch,
                };

                // Tra cứu Multiplier vi mô của Matrix
                let multiplier = match id {
                    ScannerId::TrendPullback => matrix.trend_multiplier[i],
                    ScannerId::MomentumIgnition => matrix.volatility_expansion_multiplier[i],
                    ScannerId::RangeOscillator => matrix.range_multiplier[i],
                    ScannerId::AnomalyFade => matrix.anomaly_multiplier[i],
                    ScannerId::ClimaxReversal => matrix.reversal_multiplier[i],
                    ScannerId::AbsorptionScout => matrix.absorption_multiplier[i],
                };
                
                let min_veto = config.arbiter.min_multiplier_veto;
                
                // Màng lọc Phủ quyết (Veto Check)
                if multiplier < min_veto {
                    continue;
                }

                // Điểm Tự Tin Đấu Thầu (Weighted Geometric Mean Chuẩn Quant + Zero Guard)
                let confidence = Self::compute_confidence(
                    proposal.raw_score,
                    regime_weight,
                    multiplier,
                    config.arbiter.scanner_weight,
                );
                
                // Cạnh tranh Ngôi Quán Quân (Max Tracking trên thanh ghi CPU)
                if confidence > max_confidence {
                    max_confidence = confidence;
                    best_proposal = Some((proposal, confidence));
                }
            }
        }
        
        // 3. HỘI ĐỒNG TRỌNG TÀI PHÊ DUYỆT THEO NGƯỠNG ĐỘNG
        if let Some((proposal, confidence)) = best_proposal {
            if confidence >= required_confidence {
                return ArbiterDecision {
                    is_approved: true,
                    final_signal: Some(proposal.signal_type),
                    winning_scanner: Some(proposal.scanner_id),
                    final_confidence: confidence,
                    final_sl: proposal.suggested_sl,
                    final_tp: proposal.suggested_tp,
                };
            }
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_guard_prevents_false_signals() {
        // Kịch bản 1: Scanner không có tín hiệu (raw_score <= 0.0)
        assert_eq!(HeliosArbiter::compute_confidence(0.0, 0.5, 0.8, 0.5), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(-0.5, 0.5, 0.8, 0.5), 0.0);

        // Kịch bản 2: Bối cảnh phủ quyết hoàn toàn (regime_weight <= 0.0 hoặc multiplier <= 0.0)
        assert_eq!(HeliosArbiter::compute_confidence(0.9, 0.0, 0.8, 0.5), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.9, 0.5, 0.0, 0.5), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.9, -0.1, 0.8, 0.5), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.9, 0.5, -0.2, 0.5), 0.0);

        // Kịch bản 3: Cả hai đều bằng 0 (triệt tiêu lỗi 0^0 = 1.0)
        assert_eq!(HeliosArbiter::compute_confidence(0.0, 0.0, 0.0, 0.5), 0.0);
    }

    #[test]
    fn test_full_dynamic_range_coverage() {
        // Kiểm chứng khả năng vươn tới các mức tự tin cao (> 0.50 đến > 0.80)
        // để kích hoạt toàn bộ dải DNA min_confidence (1% .. 99%)
        let conf_mid = HeliosArbiter::compute_confidence(0.75, 0.40, 0.65, 0.5);
        assert!(conf_mid > 0.50, "conf_mid ({}) phải > 0.50", conf_mid);

        let conf_high = HeliosArbiter::compute_confidence(0.85, 0.55, 0.80, 0.5);
        assert!(conf_high > 0.70, "conf_high ({}) phải > 0.70", conf_high);

        let conf_max = HeliosArbiter::compute_confidence(0.95, 0.70, 0.90, 0.5);
        assert!(conf_max > 0.80, "conf_max ({}) phải > 0.80", conf_max);
    }

    #[test]
    fn test_scanner_weight_activation() {
        let raw_score = 0.90;
        let regime_weight = 0.30;
        let multiplier = 0.50;
        // context_score = sqrt(0.15) ≈ 0.3873

        // Khi scanner_weight cao (0.85), ưu tiên tín hiệu scanner (0.90) -> confidence cao hơn
        let conf_scanner_heavy = HeliosArbiter::compute_confidence(raw_score, regime_weight, multiplier, 0.85);

        // Khi scanner_weight thấp (0.15), ưu tiên bối cảnh vĩ mô (0.3873) -> confidence thấp hơn
        let conf_context_heavy = HeliosArbiter::compute_confidence(raw_score, regime_weight, multiplier, 0.15);

        assert!(
            conf_scanner_heavy > conf_context_heavy,
            "Scanner-heavy ({}) phải lớn hơn Context-heavy ({})",
            conf_scanner_heavy,
            conf_context_heavy
        );
    }

    #[test]
    fn test_fair_auction_balance_scanners() {
        // Kiểm chứng tính công bằng của 4-State Simplex trong Đấu thầu Arbiter (FLAW-TSL.9):
        // Khi thị trường có Trend rõ rệt (w_t = 0.65, w_c = 0.10):
        let w_t = 0.65;
        let w_c = 0.10;
        let trend_mul = 0.80;
        let abs_mul = 0.50;
        let scanner_weight = 0.50;

        // Giả sử Absorption có raw_score rất cao (0.90), còn TrendPullback là 0.85
        let conf_trend = HeliosArbiter::compute_confidence(0.85, w_t, trend_mul, scanner_weight);
        let conf_abs = HeliosArbiter::compute_confidence(0.90, w_c, abs_mul, scanner_weight);

        // TrendPullback phải chiến thắng hoàn toàn nhờ bối cảnh Đội Nhà (w_t) vượt trội
        assert!(
            conf_trend > conf_abs,
            "TrendPullback ({:.4}) phải thắng thầu áp đảo Absorption ({:.4}) khi thị trường có Trend!",
            conf_trend, conf_abs
        );
        assert!(conf_trend > 0.70, "Điểm tự tin của Trend ({:.4}) phải > 0.70", conf_trend);
        assert!(conf_abs < 0.50, "Điểm tự tin của Absorption ({:.4}) phải bị kiềm chế < 0.50", conf_abs);
    }
}
