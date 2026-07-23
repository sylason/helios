// strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct MomentumIgnitionScanner;

impl TacticalScanner for MomentumIgnitionScanner {
    fn id(&self) -> &'static str {
        "MOMENTUM_IGN"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, _matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        // [NOTE] Momentum Ignition đang mượn config `break_confirm_mult` của Squeeze (Thay vì tự dùng `volume_spike_min` của nó).
        // Tuy nhiên, tỷ lệ mượn này (0.5 đến 2.5) lại vô tình khớp hoàn hảo với phép tính `v / ma` bên dưới và 
        // đang tạo ra PnL cao nhất cho hệ thống. 
        // Do đó theo nguyên tắc "If it ain't broke, don't fix it", ta TUYỆT ĐỐI GIỮ NGUYÊN logic này.
        let vol_thresh = config.tactical_sniper.squeeze.break_confirm_mult;
        
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let kc_u = sensors.kc_upper[i];
        let kc_l = sensors.kc_lower[i];
        let close = tick.closes()[i];
        
        // 1. Phải đang có sự nén (Squeeze) hoặc vừa thoát nén
        // Khi BB bung ra ngoài KC, đó là Breakout Squeeze
        let is_breakout = bb_u > kc_u || bb_l < kc_l;
        
        // 2. Volume phải đột biến
        let v = tick.volumes()[i];
        let ma = sensors.vol_ma_series[i];
        let vol_raw = if ma > 0.0 { v / ma } else { 1.0 };
        
        if is_breakout && vol_raw > vol_thresh {
            let mut raw_score = 0.0;
            let mut signal_type = None;
            
            // Bullish Breakout
            if close > kc_u {
                raw_score = 0.7 + (vol_raw - vol_thresh) * 0.1;
                signal_type = Some(SignalType::Long);
            } 
            // Bearish Breakout
            else if close < kc_l {
                raw_score = 0.7 + (vol_raw - vol_thresh) * 0.1;
                signal_type = Some(SignalType::Short);
            }
            
            if let Some(sig) = signal_type {
                return Some(Proposal {
                    signal_type: sig,
                    raw_score: raw_score.min(1.0),
                    scanner_id: self.id(),
                    suggested_sl: None,
                    suggested_tp: None,
                });
            }
        }
        
        None
    }
}
