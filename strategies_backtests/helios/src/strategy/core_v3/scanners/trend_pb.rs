// strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};

pub struct TrendPBScanner;

impl TacticalScanner for TrendPBScanner {
    fn id(&self) -> &'static str {
        "TREND_PB"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3) -> Option<Proposal> {
        let i = tick.i;
        
        let close = tick.closes()[i];
        let kama = sensors.kama_mid[i];
        let stoch_k = sensors.stoch_k[i];
        let stoch_d = sensors.stoch_d[i];
        
        // 1. Điều kiện tiên quyết: Hướng xu hướng
        let dir = matrix.trend_dir[i];
        
        // 2. Chấm điểm raw pattern
        let mut raw_score = 0.0;
        let mut signal_type = None;
        
        if dir > 0.5 { // Bullish Trend
            // Tìm cơ hội mua khi giá hồi về KAMA và Stoch Oversold
            if close <= kama * 1.005 && stoch_k < 30.0 && stoch_k > stoch_d {
                raw_score = 0.8 + (30.0 - stoch_k) / 150.0; // Điểm cộng nếu càng oversold sâu
                signal_type = Some(SignalType::Long);
            }
        } else if dir < -0.5 { // Bearish Trend
            // Tìm cơ hội bán khi giá hồi lên KAMA và Stoch Overbought
            if close >= kama * 0.995 && stoch_k > 70.0 && stoch_k < stoch_d {
                raw_score = 0.8 + (stoch_k - 70.0) / 150.0;
                signal_type = Some(SignalType::Short);
            }
        }
        
        // Trả về Proposal nếu có
        if let Some(sig) = signal_type {
            Some(Proposal {
                signal_type: sig,
                raw_score: raw_score.min(1.0),
                scanner_id: self.id(),
            })
        } else {
            None
        }
    }
}
