// strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct TrendPBScanner;

impl TacticalScanner for TrendPBScanner {
    fn id(&self) -> ScannerId {
        ScannerId::TrendPullback
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, _matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let pb_cfg = &config.tactical_sniper.pullback;
        
        let stoch_k = sensors.stoch_trend_k[i];
        let rsi = sensors.rsi_trend[i];
        let slope_adj = sensors.kama_slope_adj[i];

        if slope_adj.is_nan() || stoch_k.is_nan() || rsi.is_nan() {
            return None;
        }

        let slope_thresh = pb_cfg.kama_slope_threshold.max(1e-4);
        let f_slope = (slope_adj.abs() / slope_thresh).tanh();

        let stoch_pb_thresh = pb_cfg.stoch_pullback_thresh;
        let rsi_buy_thresh = pb_cfg.rsi_buy_thresh;

        let min_score = pb_cfg.min_activation_score;

        // 1. Chấm điểm Mềm Mẫu Hình LONG (KAMA Slope Dương + Stoch Pullback + RSI Safe)
        if slope_adj > 0.0 {
            let f_stoch_long = 1.0 / (1.0 + (-0.1 * (stoch_pb_thresh - stoch_k)).exp());
            let f_rsi_long = 1.0 / (1.0 + (-0.1 * (rsi_buy_thresh - rsi)).exp());
            let score_long = 0.40 * f_slope + 0.40 * f_stoch_long + 0.20 * f_rsi_long;

            if score_long >= min_score {
                return Some(Proposal {
                    signal_type: SignalType::Long,
                    raw_score: score_long.min(1.0),
                    scanner_id: self.id(),
                    suggested_sl: None,
                    suggested_tp: None,
                });
            }
        }
        // 2. Chấm điểm Mềm Mẫu Hình SHORT (KAMA Slope Âm + Stoch Rally + RSI Overheat)
        else if slope_adj < 0.0 {
            let f_stoch_short = 1.0 / (1.0 + (-0.1 * (stoch_k - (100.0 - stoch_pb_thresh))).exp());
            let f_rsi_short = 1.0 / (1.0 + (-0.1 * (rsi - (100.0 - rsi_buy_thresh))).exp());
            let score_short = 0.40 * f_slope + 0.40 * f_stoch_short + 0.20 * f_rsi_short;

            if score_short >= min_score {
                return Some(Proposal {
                    signal_type: SignalType::Short,
                    raw_score: score_short.min(1.0),
                    scanner_id: self.id(),
                    suggested_sl: None,
                    suggested_tp: None,
                });
            }
        }

        None
    }
}
