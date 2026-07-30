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
        let mom_cfg = &config.tactical_sniper.momentum;
        let vol_thresh = mom_cfg.volume_spike_min;
        let max_vol_mult = mom_cfg.vol_max_multiplier;
        let v_w = mom_cfg.vol_vs_break_weight.max(0.0).min(1.0);
        let b_w = 1.0 - v_w;
        
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let kc_u = sensors.kc_upper[i];
        let kc_l = sensors.kc_lower[i];
        let close = tick.closes()[i];
        
        // 1. Phải đang có sự nén (Squeeze) hoặc vừa thoát nén
        // Đo lường độ bục của BB ra khỏi KC theo % chiều rộng Keltner Channel
        let bb_break_dist = (bb_u - kc_u).max(0.0).max((kc_l - bb_l).max(0.0));
        let kc_width = kc_u - kc_l;
        let break_activation = if kc_width > 0.0 { (bb_break_dist / (kc_width * 0.5)).min(1.0) } else { 0.0 };
        
        // 2. Volume Activation
        let v = tick.volumes()[i];
        let ma = sensors.vol_ma_series[i];
        let vol_raw = if ma > 0.0 { v / ma } else { 1.0 };
        
        let vol_activation = if max_vol_mult > vol_thresh { ((vol_raw - vol_thresh) / (max_vol_mult - vol_thresh)).max(0.0).min(1.0) } else { 0.0 };
        
        let raw_score = (vol_activation * v_w) + (break_activation * b_w);
        
        if raw_score > 0.0 {
            let mut signal_type = None;
            
            let rsi = sensors.rsi[i];
            let rsi_pct = rsi / 100.0;

            // Bullish Breakout (Yêu cầu RSI đạt ngưỡng đà tăng nhưng chưa quá cực đoan Climax)
            if close > kc_u && rsi_pct >= mom_cfg.rsi_threshold && rsi_pct <= mom_cfg.rsi_climax {
                signal_type = Some(SignalType::Long);
            } 
            // Bearish Breakout
            else if close < kc_l && (1.0 - rsi_pct) >= mom_cfg.rsi_threshold && (1.0 - rsi_pct) <= mom_cfg.rsi_climax {
                signal_type = Some(SignalType::Short);
            }
            
            if let Some(sig) = signal_type {
                return Some(Proposal {
                    signal_type: sig,
                    raw_score: raw_score.min(1.0).max(0.0),
                    scanner_id: self.id(),
                    suggested_sl: None,
                    suggested_tp: None,
                });
            }
        }
        
        None
    }
}
