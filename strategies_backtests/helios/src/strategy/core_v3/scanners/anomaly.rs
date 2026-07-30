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
            
            // [EXPERIMENTAL FEATURE] Soft Activation (Probabilistic Scoring)
            // Chấm điểm liên tục thay vì dùng lệnh AND cắt cụt.
            
            let z_short_activation = if 1.0 > z_thresh { ((z_score - z_thresh) / (1.0 - z_thresh)).max(0.0).min(1.0) } else { 0.0 };
            let rsi_short_activation = if 100.0 > rsi_high { ((rsi - rsi_high) / (100.0 - rsi_high)).max(0.0).min(1.0) } else { 0.0 };
            
            let z_long_activation = if 1.0 > z_thresh { (((-z_score) - z_thresh) / (1.0 - z_thresh)).max(0.0).min(1.0) } else { 0.0 };
            let rsi_long_activation = if rsi_low > 0.0 { ((rsi_low - rsi) / rsi_low).max(0.0).min(1.0) } else { 0.0 };
            
            let z_w = ano_cfg.z_vs_rsi_weight.max(0.0).min(1.0);
            let rsi_w = 1.0 - z_w;
            let wick_buffer_mult = ano_cfg.wick_sl_buffer_atr;

            let short_base = (z_short_activation * z_w) + (rsi_short_activation * rsi_w);
            let long_base = (z_long_activation * z_w) + (rsi_long_activation * rsi_w);
            
            let mut suggested_sl = None;
            let current_atr = sensors.atr_sl[i];

            let pattern_w = ano_cfg.pattern_vs_wick_weight.max(0.0).min(1.0);
            let wick_w = 1.0 - pattern_w;

            let min_rej = ano_cfg.min_rejection_pct;

            if short_base > long_base && short_base > 0.0 && upper_rejection >= min_rej {
                raw_score = short_base * pattern_w + upper_rejection * wick_w;
                signal_type = Some(SignalType::Short);
                if !current_atr.is_nan() && current_atr > 0.0 {
                    suggested_sl = Some(high + current_atr * wick_buffer_mult);
                }
            } else if long_base > short_base && long_base > 0.0 && lower_rejection >= min_rej {
                raw_score = long_base * pattern_w + lower_rejection * wick_w;
                signal_type = Some(SignalType::Long);
                if !current_atr.is_nan() && current_atr > 0.0 {
                    suggested_sl = Some(low - current_atr * wick_buffer_mult);
                }
            }
            
            if let Some(sig) = signal_type {
                return Some(Proposal {
                    signal_type: sig,
                    raw_score: raw_score.min(1.0).max(0.0),
                    scanner_id: self.id(),
                    suggested_sl,
                    suggested_tp: None,
                });
            }
        }
        
        None
    }
}
