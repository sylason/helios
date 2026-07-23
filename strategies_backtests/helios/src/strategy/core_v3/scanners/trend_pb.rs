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
        let kama_upper = kama * (1.0 + pb_cfg.kama_tolerance_pct);
        let kama_lower = kama * (1.0 - pb_cfg.kama_tolerance_pct);
        
        if dir > 0.5 { // Bullish Trend
            // Tìm cơ hội mua khi giá hồi về KAMA và Stoch Oversold
            if close <= kama_upper && stoch_k < stoch_pb && stoch_k > stoch_d {
                raw_score = 0.8 + (stoch_pb - stoch_k) / 150.0; // Điểm cộng nếu càng oversold sâu
                signal_type = Some(SignalType::Long);
            }
        } else if dir < -0.5 { // Bearish Trend
            // Tìm cơ hội bán khi giá hồi lên KAMA và Stoch Overbought
            if close >= kama_lower && stoch_k > (100.0 - stoch_pb) && stoch_k < stoch_d {
                raw_score = 0.8 + (stoch_k - (100.0 - stoch_pb)) / 150.0;
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
