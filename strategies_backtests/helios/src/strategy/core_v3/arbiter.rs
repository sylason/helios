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

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, _debug_mode: bool) -> ArbiterDecision {
        let i = tick.i;
        
        // 1. TÍNH NGƯỠNG PHÊ DUYỆT ĐỘNG CONTINUOUS (DYNAMIC APPROVAL THRESHOLD)
        let w_t = matrix.w_trend_norm[i];
        let w_r = matrix.w_range_norm[i];
        let w_v = matrix.w_volatile_norm[i];

        let required_confidence = w_t * config.arbiter.min_confidence_trending
                                + w_r * config.arbiter.min_confidence_ranging
                                + w_v * config.arbiter.min_confidence_volatile;

        let mut best_proposal: Option<(Proposal, f64)> = None;
        let mut max_confidence = 0.0;
        
        // 2. VÒNG LẶP THU THẬP & ĐẤU THẦU ĐA BỐI CẢNH (CONFIDENCE AUCTION)
        // Mọi Scanner đều được đối xử bình đẳng, không bị khóa cứng nhị phân
        for scanner in &self.scanners {
            let id = scanner.id();

            if let Some(proposal) = scanner.scan(tick, sensors, matrix, config) {
                // Tra cứu Trọng số Bối cảnh Simplex Đội Nhà (O(1))
                let regime_weight = match id {
                    ScannerId::TrendPullback | ScannerId::MomentumIgnition => w_t,
                    ScannerId::RangeOscillator | ScannerId::ClimaxReversal => w_r,
                    ScannerId::AnomalyFade | ScannerId::AbsorptionScout   => w_v,
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

                // Điểm Tự Tin Đấu Thầu (Tích Hỗ Tương: Điểm Mẫu Hình * Trọng Số Bối Cảnh * Multiplier Vi Mô)
                let confidence = proposal.raw_score * regime_weight * multiplier;
                
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
