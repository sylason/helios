use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct AnomalyScanner;

impl TacticalScanner for AnomalyScanner {
    fn id(&self) -> &'static str {
        "ANOMALY"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let ano_cfg = &config.tactical_sniper.anomaly;
        
        let z_score = matrix.price_z_score[i];
        let rsi = sensors.rsi[i];
        
        let open = tick.opens()[i];
        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let close = tick.closes()[i];
        
        let total_range = high - low;
        
        let mut raw_score = 0.0;
        let mut signal_type = None;
        
        let z_thresh = ano_cfg.z_score_threshold; 
        
        if total_range > 0.0 {
            let upper_wick = high - open.max(close);
            let lower_wick = open.min(close) - low;
            
            let upper_rejection = upper_wick / total_range;
            let lower_rejection = lower_wick / total_range;
            
            let rsi_high = ano_cfg.rsi_climax * 100.0;
            let rsi_low = (1.0 - ano_cfg.rsi_climax) * 100.0;
            
            // [EXPERIMENTAL FEATURE] Nới lỏng Anomaly: Chuyển Rejection từ điều kiện bắt buộc (VETO) sang điểm cộng (SCORE).
            // Giữ lại linh hồn bắt râu bằng cách cộng điểm nếu có râu.
            
            // Short Anomaly (Flash pump and reject)
            if z_score > z_thresh && rsi > rsi_high {
                raw_score = 0.8 + upper_rejection * 0.2;
                signal_type = Some(SignalType::Short);
            }
            // Long Anomaly (Flash crash and reject)
            else if z_score < -z_thresh && rsi < rsi_low {
                raw_score = 0.8 + lower_rejection * 0.2;
                signal_type = Some(SignalType::Long);
            }
        }
        
        if let Some(sig) = signal_type {
            Some(Proposal {
                signal_type: sig,
                raw_score: raw_score.min(1.0).max(0.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp: None,
            })
        } else {
            None
        }
    }
}
