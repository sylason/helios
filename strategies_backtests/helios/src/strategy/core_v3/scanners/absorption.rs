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
        
        if vol_force >= abs_cfg.vol_min_threshold && squeeze > 0.5 {
            let macd = sensors.macd_main[i];
            let macd_signal = sensors.macd_signal[i];
            
            let raw_score = 0.7 + ((vol_force - abs_cfg.vol_min_threshold) / 2.0);
            
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
