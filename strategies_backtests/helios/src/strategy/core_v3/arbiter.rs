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

#[derive(Debug, Clone)]
pub struct Proposal {
    /// Loại tín hiệu
    pub signal_type: SignalType,
    /// Điểm tự tin do Scanner tự chấm dựa trên độ hoàn hảo của Pattern (0.0 đến 1.0)
    pub raw_score: f64,
    /// Tên của Scanner
    pub scanner_id: &'static str,
    pub suggested_sl: Option<f64>,
    pub suggested_tp: Option<f64>,
}

pub struct ArbiterDecision {
    pub is_approved: bool,
    pub final_signal: Option<SignalType>,
    pub winning_scanner: Option<&'static str>,
    pub final_confidence: f64,
    pub final_sl: Option<f64>,
    pub final_tp: Option<f64>,
}

pub trait TacticalScanner {
    /// Tên định danh của Scanner
    fn id(&self) -> &'static str;
    
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
    pub min_approval_confidence: f64,
}

impl HeliosArbiter {
    pub fn new(min_approval_confidence: f64) -> Self {
        Self {
            scanners: Vec::new(),
            min_approval_confidence,
        }
    }

    pub fn add_scanner(&mut self, scanner: Box<dyn TacticalScanner>) {
        self.scanners.push(scanner);
    }

    pub fn process_tick(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3, debug_mode: bool) -> ArbiterDecision {
        let mut best_proposal: Option<(Proposal, f64)> = None;
        let mut max_confidence = 0.0;
        
        let i = tick.i;
        
        // 1. Thu thập Proposal từ tất cả Scanner
        for scanner in &self.scanners {
            if let Some(proposal) = scanner.scan(tick, sensors, matrix, config) {
                // Tùy thuộc vào Scanner ID, áp dụng Multiplier tương ứng từ Matrix
                let multiplier = match scanner.id() {
                    "TREND_PB" => matrix.trend_multiplier[i],
                    "MOMENTUM_IGN" => matrix.volatility_expansion_multiplier[i],
                    "RANGE_OSC" => matrix.range_multiplier[i],
                    "ANOMALY" => matrix.anomaly_multiplier[i],
                    "REVERSAL" => matrix.reversal_multiplier[i],
                    "ABSORPTION" => matrix.absorption_multiplier[i],
                    _ => 1.0,
                };
                
                // [EXPERIMENTAL FEATURE] The Additive Logic & Hard Veto
                // 1. Hard Veto: Nếu môi trường hoàn toàn không phù hợp (multiplier < 0.2), chặn ngay lập tức.
                let confidence = if multiplier < 0.2 {
                    0.0
                } else {
                    // 2. Additive Weighted Scoring: 
                    // Phép cộng trọng số (60% Scanner, 40% Môi trường) để tránh cối xay thịt thập phân.
                    (proposal.raw_score * 0.6) + (multiplier * 0.4)
                };
                
                // if debug_mode {
                //     println!("[Bar {}] Arbiter: Scanner {} proposed {:?} | Raw: {:.2} | Multiplier: {:.2} | Final: {:.2}", 
                //              tick.i, scanner.id(), proposal.signal_type, proposal.raw_score, multiplier, confidence);
                // }
                
                // Chọn Proposal có niềm tin cao nhất
                if confidence > max_confidence {
                    max_confidence = confidence;
                    best_proposal = Some((proposal, confidence));
                }
            }
        }
        
        // 2. Phê duyệt (Approval Gate)
        if let Some((proposal, confidence)) = best_proposal {
            if confidence >= self.min_approval_confidence {
                // if debug_mode {
                //     println!("[Bar {}] Arbiter: APPROVED {} {:?} with confidence {:.2} (Threshold: {:.2})", 
                //              tick.i, proposal.scanner_id, proposal.signal_type, confidence, self.min_approval_confidence);
                // }
                return ArbiterDecision {
                    is_approved: true,
                    final_signal: Some(proposal.signal_type),
                    winning_scanner: Some(proposal.scanner_id),
                    final_confidence: confidence,
                    final_sl: proposal.suggested_sl,
                    final_tp: proposal.suggested_tp,
                };
            } else if debug_mode {
                // println!("[Bar {}] Arbiter: REJECTED {} {:?} | Confidence {:.2} < Threshold {:.2}", 
                //          tick.i, proposal.scanner_id, proposal.signal_type, confidence, self.min_approval_confidence);
            }
        }
        
        // Không có lệnh nào đủ tốt
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
