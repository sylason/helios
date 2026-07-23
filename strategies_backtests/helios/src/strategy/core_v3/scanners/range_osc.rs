// strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct RangeOscScanner;

impl TacticalScanner for RangeOscScanner {
    fn id(&self) -> &'static str {
        "RANGE_OSC"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, _matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let range_cfg = &config.tactical_sniper.range;
        
        let close = tick.closes()[i];
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let _bb_m = (bb_u + bb_l) / 2.0;
        
        let stoch_k = sensors.stoch_k[i];
        let stoch_d = sensors.stoch_d[i];
        
        let mut raw_score = 0.0;
        let mut signal_type = None;
        let mut suggested_tp = None;
        
        let stoch_os = range_cfg.stoch_oversold * 100.0;
        let stoch_ob = range_cfg.stoch_overbought * 100.0;
        
        // Mua khi giá chạm Band dưới và Stoch Oversold chéo lên
        if close <= bb_l * 1.002 && stoch_k < stoch_os && stoch_k > stoch_d {
            raw_score = 0.7 + (stoch_os - stoch_k) / 100.0;
            signal_type = Some(SignalType::Long);
            suggested_tp = Some(bb_u);
        }
        // Bán khi giá chạm Band trên và Stoch Overbought chéo xuống
        else if close >= bb_u * 0.998 && stoch_k > stoch_ob && stoch_k < stoch_d {
            raw_score = 0.7 + (stoch_k - stoch_ob) / 100.0;
            signal_type = Some(SignalType::Short);
            suggested_tp = Some(bb_l);
        }
        
        if let Some(sig) = signal_type {
            Some(Proposal {
                signal_type: sig,
                raw_score: raw_score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp,
            })
        } else {
            None
        }
    }
}
