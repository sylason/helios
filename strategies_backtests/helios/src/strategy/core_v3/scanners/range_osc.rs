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

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let range_cfg = &config.tactical_sniper.range;
        
        let close = tick.closes()[i];
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let bb_m = (bb_u + bb_l) / 2.0;
        
        // 1. Kiểm tra bề rộng tối thiểu Bollinger Bands (tránh nén quá chặt sắp breakout)
        let bb_width_pct = if bb_m > 0.0 { (bb_u - bb_l) / bb_m } else { 0.0 };
        if bb_width_pct < range_cfg.min_width_pct {
            return None;
        }

        // 2. Kiểm tra Volume Force tối đa (Tránh giao dịch Range khi volume quá lớn báo hiệu Breakout)
        if matrix.volume_force[i] > range_cfg.volume_max_threshold {
            return None;
        }
        
        let stoch_k = sensors.stoch_k[i];
        let stoch_d = sensors.stoch_d[i];
        
        let mut raw_score = 0.0;
        let mut signal_type = None;
        let mut suggested_tp = None;
        
        let stoch_os = range_cfg.stoch_oversold * 100.0;
        let stoch_ob = range_cfg.stoch_overbought * 100.0;
        
        // [EXPERIMENTAL FEATURE] Soft Activation
        let bb_long_dist = if close > bb_l { close - bb_l } else { 0.0 };
        let bb_short_dist = if close < bb_u { bb_u - close } else { 0.0 };
        
        let tol_pct = range_cfg.range_bb_tolerance_pct;
        let bb_w = range_cfg.bb_vs_stoch_weight.max(0.0).min(1.0);
        let st_w = 1.0 - bb_w;

        let bb_range = bb_u - bb_l;
        let bb_tol = if bb_range > 0.0 { bb_range * tol_pct } else { close * 0.005 };
        
        let bb_long_activation = if bb_tol > 0.0 { (1.0 - (bb_long_dist / bb_tol)).max(0.0).min(1.0) } else { 0.0 };
        let bb_short_activation = if bb_tol > 0.0 { (1.0 - (bb_short_dist / bb_tol)).max(0.0).min(1.0) } else { 0.0 };
        
        let stoch_long_activation = if stoch_os > 0.0 { ((stoch_os - stoch_k) / stoch_os).max(0.0).min(1.0) } else { 0.0 };
        let stoch_short_activation = if 100.0 - stoch_ob > 0.0 { ((stoch_k - stoch_ob) / (100.0 - stoch_ob)).max(0.0).min(1.0) } else { 0.0 };
        
        let long_score = (bb_long_activation * bb_w) + (stoch_long_activation * st_w);
        let short_score = (bb_short_activation * bb_w) + (stoch_short_activation * st_w);
        
        // Cần đạt tối thiểu điểm kích hoạt, chéo Stoch_K cắt Stoch_D
        if long_score > 0.0 && stoch_k > stoch_d {
            raw_score = long_score;
            signal_type = Some(SignalType::Long);
            suggested_tp = Some(bb_u);
        } else if short_score > 0.0 && stoch_k < stoch_d {
            raw_score = short_score;
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
