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

        if vwma.is_nan() || bb_middle.is_nan() {
            return None;
        }

        let sqz_thresh = abs_cfg.squeeze_threshold.max(0.01);
        let vol_thresh = abs_cfg.vol_z_threshold.max(0.01);

        // 1. Chấm điểm Mềm Cường độ Nén Squeeze & Lực gom Volume
        let f_sqz = (squeeze / sqz_thresh).tanh();
        let f_vol = (vol_force / vol_thresh).tanh();
        let raw_score = (0.50 * f_sqz + 0.50 * f_vol).min(1.0);

        if raw_score >= abs_cfg.min_activation_score {
            // 2. Xác định hướng bùng nổ bằng VWMA vs SMA middle:
            // VWMA > BB_Middle: Volume lớn tập trung nến mua -> Accumulation (Long)
            // VWMA < BB_Middle: Volume lớn tập trung nến bán -> Distribution (Short)
            let signal_type = if vwma >= bb_middle {
                SignalType::Long
            } else {
                SignalType::Short
            };

            let close = tick.closes()[i];
            let low = tick.lows()[i];
            let high = tick.highs()[i];
            let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
            let buffer = current_atr * abs_cfg.struct_sl_buffer_atr.max(0.05);

            // 🛡️ FLAW-TSL.6: Structural SL neo dưới/trên vùng nén hấp thụ thanh khoản (BB Middle)
            let sl = match signal_type {
                SignalType::Long => {
                    let base_support = low.min(bb_middle);
                    let mut s = base_support - buffer;
                    if s >= close {
                        s = close - buffer.max(close * 0.002);
                    }
                    s
                }
                SignalType::Short => {
                    let base_resistance = high.max(bb_middle);
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
