// strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct RangeOscScanner;

impl TacticalScanner for RangeOscScanner {
    fn id(&self) -> ScannerId {
        ScannerId::RangeOscillator
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let range_cfg = &config.tactical_sniper.range;
        
        let close = tick.closes()[i];
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let bb_w_norm = matrix.bb_width_norm[i];
        let vol_force = matrix.volume_force[i];
        
        if bb_u.is_nan() || bb_l.is_nan() || close <= 0.0 {
            return None;
        }

        // 1. Kiểm tra bề rộng tối thiểu Bollinger Bands & Volume Max Threshold
        if bb_w_norm < range_cfg.min_width_pct || vol_force > range_cfg.volume_max_threshold {
            return None;
        }

        // 1.1 BỘ LỌC XU HƯỚNG QUANT: Cấm đánh Range khi thị trường đang có Trend mạnh
        let max_trend = if range_cfg.max_trend_strength > 0.0 { range_cfg.max_trend_strength } else { 0.60 };
        if matrix.trend_strength[i] > max_trend {
            return None;
        }

        let stoch_k = sensors.stoch_range_k[i];
        let stoch_d = sensors.stoch_range_d[i];

        if stoch_k.is_nan() || stoch_d.is_nan() {
            return None;
        }

        let bb_prox = (range_cfg.bb_proximity_pct * close).max(1e-5);
        let two_sigma_sq = 2.0 * bb_prox * bb_prox;

        // 2. Chấm điểm Mềm Mẫu Hình LONG (Gaussian Price + Sigmoid Stoch + Hook)
        let dist_l = (close - bb_l).abs();
        let f_price_long = (- (dist_l * dist_l) / two_sigma_sq).exp();
        let f_stoch_long = 1.0 / (1.0 + (-0.1 * (range_cfg.stoch_oversold - stoch_k)).exp());
        let f_hook_long = 1.0 / (1.0 + (-0.5 * (stoch_k - stoch_d)).exp());
        let score_long = 0.50 * f_price_long + 0.30 * f_stoch_long + 0.20 * f_hook_long;

        // 3. Chấm điểm Mềm Mẫu Hình SHORT (Gaussian Price + Sigmoid Stoch + Hook)
        let dist_u = (close - bb_u).abs();
        let f_price_short = (- (dist_u * dist_u) / two_sigma_sq).exp();
        let f_stoch_short = 1.0 / (1.0 + (-0.1 * (stoch_k - range_cfg.stoch_overbought)).exp());
        let f_hook_short = 1.0 / (1.0 + (-0.5 * (stoch_d - stoch_k)).exp());
        let score_short = 0.50 * f_price_short + 0.30 * f_stoch_short + 0.20 * f_hook_short;

        let min_score = range_cfg.min_activation_score;
        let max_slope = if range_cfg.max_adverse_slope > 0.0 { range_cfg.max_adverse_slope } else { 0.20 };
        let trend_dir = matrix.trend_dir[i];

        if score_long >= min_score && score_long > score_short {
            // 🛡️ CHỐNG BẮT DAO RƠI: Nếu Trend đang đổ dốc mạnh (trend_dir < -max_slope) -> Cấm Long!
            if trend_dir < -max_slope {
                return None;
            }
            // 🎯 SỬA FLAW-4.2: Take Profit của Long bắt buộc phải cao hơn giá Close ít nhất 0.2%
            let valid_tp = bb_u.max(close * 1.002);
            Some(Proposal {
                signal_type: SignalType::Long,
                raw_score: score_long.min(1.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp: Some(valid_tp),
            })
        } else if score_short >= min_score && score_short > score_long {
            // 🛡️ CHỐNG CẢN TÀU: Nếu Trend đang dốc đứng đi lên (trend_dir > +max_slope) -> Cấm Short!
            if trend_dir > max_slope {
                return None;
            }
            // 🎯 SỬA FLAW-4.2: Take Profit của Short bắt buộc phải thấp hơn giá Close ít nhất 0.2%
            let valid_tp = bb_l.min(close * 0.998);
            Some(Proposal {
                signal_type: SignalType::Short,
                raw_score: score_short.min(1.0),
                scanner_id: self.id(),
                suggested_sl: None,
                suggested_tp: Some(valid_tp),
            })
        } else {
            None
        }
    }
}
