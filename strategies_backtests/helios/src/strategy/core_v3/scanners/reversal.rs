use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct ReversalScanner;

impl TacticalScanner for ReversalScanner {
    fn id(&self) -> &'static str {
        "REVERSAL"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let rev_cfg = &config.tactical_sniper.reversal;
        
        let z_score = matrix.price_z_score[i];
        let rsi = sensors.rsi[i];
        
        let mut raw_score = 0.0;
        let mut signal_type = None;
        
        let rsi_high = rev_cfg.rsi_extreme * 100.0;
        let rsi_low = (1.0 - rev_cfg.rsi_extreme) * 100.0;
        
        // [EXPERIMENTAL FEATURE] Nới lỏng Reversal: Loại bỏ chặn Volume, giao lại việc lọc cho MarketMatrix.
        // Chỉ cần Z-Score lệch chuẩn và RSI cực đoan là nhả đạn.
        
        let z_thresh = rev_cfg.z_score_threshold; 
        
        if z_score > z_thresh && rsi > rsi_high {
            raw_score = 0.8 + ((z_score - z_thresh) / 2.0); 
            signal_type = Some(SignalType::Short);
        }
        else if z_score < -z_thresh && rsi < rsi_low {
            raw_score = 0.8 + (((-z_score) - z_thresh) / 2.0);
            signal_type = Some(SignalType::Long);
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
