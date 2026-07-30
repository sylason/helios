use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct ReversalScanner;

impl TacticalScanner for ReversalScanner {
    fn id(&self) -> &'static str {
        "REVERSAL"
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let rev_cfg = &config.tactical_sniper.reversal;
        
        let z_score = matrix.price_z_score[i];
        let rsi = sensors.rsi[i];
        
        let mut raw_score = 0.0;
        let mut signal_type = None;
        
        let rsi_high = rev_cfg.rsi_extreme * 100.0;
        let rsi_low = (1.0 - rev_cfg.rsi_extreme) * 100.0;
        
        let z_thresh = rev_cfg.z_score_threshold; 
        
        // 1. Reversal yêu cầu khối lượng phải cạn kiệt (Volume Force <= exhaustion_vol_drop) để tránh bắt dao rơi khi volume đang bùng nổ
        if matrix.volume_force[i] > rev_cfg.exhaustion_vol_drop {
            return None;
        }
        
        let z_short_activation = if 1.0 > z_thresh { ((z_score - z_thresh) / (1.0 - z_thresh)).max(0.0).min(1.0) } else { 0.0 };
        let rsi_short_activation = if 100.0 > rsi_high { ((rsi - rsi_high) / (100.0 - rsi_high)).max(0.0).min(1.0) } else { 0.0 };
        
        let z_long_activation = if 1.0 > z_thresh { (((-z_score) - z_thresh) / (1.0 - z_thresh)).max(0.0).min(1.0) } else { 0.0 };
        let rsi_long_activation = if rsi_low > 0.0 { ((rsi_low - rsi) / rsi_low).max(0.0).min(1.0) } else { 0.0 };
        
        let z_w = rev_cfg.z_vs_rsi_weight.max(0.0).min(1.0);
        let rsi_w = 1.0 - z_w;
        let wick_buffer_mult = rev_cfg.wick_sl_buffer_atr;

        let min_disp_mult = rev_cfg.min_reversal_displacement;
        let kama_mid = sensors.kama_mid[i];
        let close_price = tick.closes()[i];
        let current_atr = sensors.atr_sl[i];

        // 1. Kiểm tra Bắt buộc: Giá phải bị nén chệch xa trung vị kama_mid tối thiểu min_disp_mult * ATR
        if kama_mid.is_nan() || current_atr.is_nan() || current_atr <= 0.0 || kama_mid <= 0.0 {
            return None;
        }
        let displacement = (close_price - kama_mid).abs();
        if displacement < (current_atr * min_disp_mult) {
            return None;
        }

        let short_base = (z_short_activation * z_w) + (rsi_short_activation * rsi_w);
        let long_base = (z_long_activation * z_w) + (rsi_long_activation * rsi_w);
        
        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let mut suggested_sl = None;
        
        if short_base > long_base && short_base > 0.0 {
            raw_score = short_base; 
            signal_type = Some(SignalType::Short);
            suggested_sl = Some(high + current_atr * wick_buffer_mult);
        }
        else if long_base > short_base && long_base > 0.0 {
            raw_score = long_base;
            signal_type = Some(SignalType::Long);
            suggested_sl = Some(low - current_atr * wick_buffer_mult);
        }
        
        // LUÔN LUÔN GÁN TAKE PROFIT TẠI KAMA_MID (đã được đảm bảo khoảng cách >= min_disp_mult * ATR)
        let suggested_tp = Some(kama_mid);

        if let Some(sig) = signal_type {
            Some(Proposal {
                signal_type: sig,
                raw_score: raw_score.min(1.0).max(0.0),
                scanner_id: self.id(),
                suggested_sl,
                suggested_tp,
            })
        } else {
            None
        }
    }
}
