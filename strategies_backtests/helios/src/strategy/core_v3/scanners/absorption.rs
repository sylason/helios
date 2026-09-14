// strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct AbsorptionScanner;

impl TacticalScanner for AbsorptionScanner {
    fn id(&self) -> ScannerId {
        ScannerId::AbsorptionScout
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let abs_cfg = &config.tactical_sniper.absorption;
        
        let vol_force = matrix.volume_force[i];
        let squeeze = matrix.squeeze_intensity[i];
        let vwma = sensors.vwma[i];
        let bb_middle = sensors.bb_middle[i];
        let bb_lower = sensors.bb_lower[i];
        let bb_upper = sensors.bb_upper[i];

        if vwma.is_nan() || bb_middle.is_nan() || bb_lower.is_nan() || bb_upper.is_nan() {
            return None;
        }

        let sqz_thresh = abs_cfg.squeeze_threshold.max(0.01);
        let vol_thresh = abs_cfg.vol_z_threshold.max(0.01);

        // 1. Chấm điểm Mềm Cường độ Nén Squeeze & Lực gom Volume
        let f_sqz = (squeeze / sqz_thresh).tanh();
        let f_vol = (vol_force / vol_thresh).tanh();
        let raw_score = (0.50 * f_sqz + 0.50 * f_vol).min(1.0);

        if raw_score >= abs_cfg.min_activation_score {
            let close = tick.closes()[i];

            // 🛡️ FLAW-REG.9 TẦNG 1: Price Action & Mean Alignment
            // Phải có sự đồng thuận giữa dòng tiền (VWMA) và hành động giá (Close) so với trục giữa BB:
            // - Long (Accumulation): Cả VWMA và Close đều phải nằm trên hoặc bằng BB Middle
            // - Short (Distribution): Cả VWMA và Close đều phải nằm dưới hoặc bằng BB Middle
            // Nếu có phân kỳ (ví dụ VWMA >= bb_middle nhưng Close < bb_middle do nến đỏ cắm đầu), bỏ qua không vào lệnh!
            let signal_type = if vwma >= bb_middle && close >= bb_middle {
                SignalType::Long
            } else if vwma < bb_middle && close <= bb_middle {
                SignalType::Short
            } else {
                return None;
            };

            // 🛡️ FLAW-REG.9 TẦNG 2: Macro Trend Direction Alignment (100% Dynamic qua DNA)
            // Ngăn chặn triệt để bẫy bắt dao rơi khi xả lũ hoặc cản tàu khi bơm dốc:
            // - Long: Nếu trend_dir < -max_adverse_slope -> VETO cấm bắt dao rơi!
            // - Short: Nếu trend_dir > max_adverse_slope -> VETO cấm cản tàu!
            let trend_dir = matrix.trend_dir.get(i).copied().unwrap_or(0.0);
            let max_adverse = abs_cfg.max_adverse_slope.max(0.01);
            match signal_type {
                SignalType::Long => {
                    if trend_dir < -max_adverse {
                        return None;
                    }
                }
                SignalType::Short => {
                    if trend_dir > max_adverse {
                        return None;
                    }
                }
            }
            let low = tick.lows()[i];
            let high = tick.highs()[i];
            let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
            let buffer = current_atr * abs_cfg.struct_sl_buffer_atr.max(0.05);

            // 🛡️ FLAW-TSL.6: Structural SL neo dưới dải BB Lower (Long) hoặc trên dải BB Upper (Short) vùng nén
            let sl = match signal_type {
                SignalType::Long => {
                    let base_support = low.min(bb_lower);
                    let mut s = base_support - buffer;
                    if s >= close {
                        s = close - buffer.max(close * 0.002);
                    }
                    s
                }
                SignalType::Short => {
                    let base_resistance = high.max(bb_upper);
                    let mut s = base_resistance + buffer;
                    if s <= close {
                        s = close + buffer.max(close * 0.002);
                    }
                    s
                }
            };

            return Some(Proposal {
                signal_type,
                raw_score,
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: None,
            });
        }
        
        None
    }
}
