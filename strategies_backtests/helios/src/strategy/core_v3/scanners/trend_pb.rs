// strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct TrendPBScanner;

impl TacticalScanner for TrendPBScanner {
    fn id(&self) -> &'static str {
        "TREND_PB"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let pb_cfg = &config.tactical_sniper.pullback;
        
        let close = tick.closes()[i];
        let kama = sensors.kama_mid[i];
        let stoch_k = sensors.stoch_k[i];
        let stoch_d = sensors.stoch_d[i];
        
        // 1. Điều kiện tiên quyết: Hướng xu hướng
        let dir = matrix.trend_dir[i];
        
        // 2. Chấm điểm raw pattern
        let mut raw_score = 0.0;
        let mut signal_type = None;
        
        let stoch_pb = pb_cfg.stoch_pullback_thresh * 100.0;
        
        // [EXPERIMENTAL FEATURE] Soft Activation
        let stoch_long_activation = if stoch_pb > 0.0 { ((stoch_pb - stoch_k) / stoch_pb).max(0.0).min(1.0) } else { 0.0 };
        let stoch_short_activation = if 100.0 - stoch_pb < 100.0 { ((stoch_k - (100.0 - stoch_pb)) / stoch_pb).max(0.0).min(1.0) } else { 0.0 };
        
        let kama_long_dist = if close > kama { close - kama } else { 0.0 };
        let kama_short_dist = if close < kama { kama - close } else { 0.0 };
        
        let kama_tol_abs = kama * pb_cfg.kama_tolerance_pct;
        
        let kama_long_activation = if kama_tol_abs > 0.0 { (1.0 - (kama_long_dist / (kama_tol_abs * 2.0))).max(0.0).min(1.0) } else { 0.0 };
        let kama_short_activation = if kama_tol_abs > 0.0 { (1.0 - (kama_short_dist / (kama_tol_abs * 2.0))).max(0.0).min(1.0) } else { 0.0 };
        
        let dir_thresh = pb_cfg.trend_dir_threshold;
        let st_w = pb_cfg.stoch_vs_kama_weight.max(0.0).min(1.0);
        let k_w = 1.0 - st_w;
        
        let rsi = sensors.rsi[i];
        let rsi_buy_max = pb_cfg.rsi_buy_thresh * 100.0;
        let rsi_sell_min = (1.0 - pb_cfg.rsi_buy_thresh) * 100.0;

        if dir > dir_thresh { // Bullish Trend
            let score = (stoch_long_activation * st_w) + (kama_long_activation * k_w);
            if score > 0.0 && stoch_k > stoch_d && rsi <= rsi_buy_max {
                raw_score = score;
                signal_type = Some(SignalType::Long);
            }
        } else if dir < -dir_thresh { // Bearish Trend
            let score = (stoch_short_activation * st_w) + (kama_short_activation * k_w);
            if score > 0.0 && stoch_k < stoch_d && rsi >= rsi_sell_min {
                raw_score = score;
                signal_type = Some(SignalType::Short);
            }
        }
        
        // Trả về Proposal nếu có
        if let Some(sig) = signal_type {
            Some(Proposal {
                signal_type: sig,
                raw_score: raw_score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp: None,
            })
        } else {
            None
        }
    }
}
