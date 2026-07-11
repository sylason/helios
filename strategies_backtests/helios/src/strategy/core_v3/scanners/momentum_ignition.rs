// strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};

pub struct MomentumIgnitionScanner;

impl TacticalScanner for MomentumIgnitionScanner {
    fn id(&self) -> &'static str {
        "MOMENTUM_IGN"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3) -> Option<Proposal> {
        let i = tick.i;
        
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let kc_u = sensors.kc_upper[i];
        let kc_l = sensors.kc_lower[i];
        let close = tick.closes()[i];
        
        // 1. Phải đang có sự nén (Squeeze) hoặc vừa thoát nén
        // Khi BB bung ra ngoài KC, đó là Breakout Squeeze
        let is_breakout = bb_u > kc_u || bb_l < kc_l;
        
        // 2. Volume phải đột biến
        let vol_force = matrix.volume_force[i];
        
        if is_breakout && vol_force > 1.5 {
            let mut raw_score = 0.0;
            let mut signal_type = None;
            
            // Bullish Breakout
            if close > kc_u {
                raw_score = 0.7 + (vol_force - 1.5) * 0.1;
                signal_type = Some(SignalType::Long);
            } 
            // Bearish Breakout
            else if close < kc_l {
                raw_score = 0.7 + (vol_force - 1.5) * 0.1;
                signal_type = Some(SignalType::Short);
            }
            
            if let Some(sig) = signal_type {
                return Some(Proposal {
                    signal_type: sig,
                    raw_score: raw_score.min(1.0),
                    scanner_id: self.id(),
                });
            }
        }
        
        None
    }
}
