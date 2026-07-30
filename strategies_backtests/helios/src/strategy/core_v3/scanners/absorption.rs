use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct AbsorptionScanner;

impl TacticalScanner for AbsorptionScanner {
    fn id(&self) -> &'static str {
        "ABSORPTION"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let abs_cfg = &config.tactical_sniper.absorption;
        
        let vol_force = matrix.volume_force[i];
        let squeeze = matrix.squeeze_intensity[i];
        let trend_mul = matrix.trend_multiplier[i];

        // Nếu Trend đang quá mạnh, từ chối Absorption để nhường vị thế cho các Playbook Trend/Reversal
        if trend_mul > abs_cfg.trend_filter_threshold {
            return None;
        }
        
        // [EXPERIMENTAL FEATURE] Soft Activation (Probabilistic Scoring)
        let vol_thresh = abs_cfg.vol_min_threshold;
        let vol_activation = if 1.0 > vol_thresh { ((vol_force - vol_thresh) / (1.0 - vol_thresh)).max(0.0).min(1.0) } else { 0.0 };
        let squeeze_activation = squeeze.max(0.0).min(1.0);
        let v_w = abs_cfg.vol_vs_squeeze_weight.max(0.0).min(1.0);
        let sq_w = 1.0 - v_w;
        
        let raw_score = (vol_activation * v_w) + (squeeze_activation * sq_w);
        
        let min_activation = abs_cfg.min_activation_score;
        
        if raw_score >= min_activation && raw_score > 0.0 {
            let macd = sensors.macd_main[i];
            let macd_signal = sensors.macd_signal[i];
            
            let signal_type = if macd > macd_signal {
                SignalType::Long
            } else {
                SignalType::Short
            };
            
            return Some(Proposal {
                signal_type,
                raw_score: raw_score.min(1.0).max(0.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp: None,
            });
        }
        
        None
    }
}
