//! # CLIMAX REVERSAL SCANNER (REVERSAL)
//!
//! ## 1. Mục đích:
//! Bắt các điểm kiệt sức của sóng giá sau những cú đẩy cực hạn (Climax Run / Capitulation Dump).
//! Khi giá bị kéo căng quá mức khỏi giá trị cân bằng và thanh khoản bị cạn kiệt, Scanner này
//! tìm kiếm cơ hội đảo chiều hồi quy về đường trung vị KAMA (KAMA Midline).
//!
//! ## 2. Logic Hoạt Động Cốt Lõi:
//! - **Chỉ báo sử dụng:**
//!   + `z_score`: Độ lệch chuẩn số học của giá so với trung bình (nhận diện độ căng dãn giá).
//!   + `kama_mid`: Trục trung vị Kaufman biểu thị mức giá cân bằng dài hạn.
//!   + `volume_force`: Đo lường sự suy kiệt khối lượng (Volume Exhaustion).
//!   + `atr_sl`: Chuẩn hóa khoảng cách ly giác (Displacement) và đệm Stop Loss.
//! - **Cơ chế Chấm Điểm 3 Thành Phần:**
//!   $$\text{Score} = 0.40 \times f_z + 0.40 \times f_{\text{disp}} + 0.20 \times f_{\text{vol\_exhaustion}}$$
//! - **Thiết lập Vị thế:**
//!   + Long: Khi `z_score < 0.0` và `close < kama_mid`. SL: `low - wick_buffer`, TP: `Some(kama_mid)`.
//!   + Short: Khi `z_score > 0.0` và `close > kama_mid`. SL: `high + wick_buffer`, TP: `Some(kama_mid)`.

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct ReversalScanner;

impl TacticalScanner for ReversalScanner {
    fn id(&self) -> ScannerId {
        ScannerId::ClimaxReversal
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let rev_cfg = &config.tactical_sniper.reversal;
        
        let z_score = sensors.z_score[i];
        let kama_mid = sensors.kama_mid[i];
        let current_atr = sensors.atr_sl[i];
        let close_price = tick.closes()[i];
        let vol_force = matrix.volume_force[i];

        if z_score.is_nan() || kama_mid.is_nan() || current_atr.is_nan() || current_atr <= 0.0 {
            return None;
        }

        let z_thresh = rev_cfg.z_score_threshold.max(0.1);
        let min_disp = rev_cfg.min_reversal_displacement.max(0.1);
        let exhaust_drop = rev_cfg.exhaustion_vol_drop.max(0.01);

        // 1. Chấm điểm Mềm 3 Thành Phần (Z-Score + KAMA Displacement + Volume Exhaustion)
        let f_z = (z_score.abs() / z_thresh).tanh();
        let displacement = (close_price - kama_mid).abs();
        let f_disp = (displacement / (min_disp * current_atr)).tanh();
        let f_vol = (1.0 - (vol_force / exhaust_drop).tanh()).max(0.0).min(1.0);

        let score = 0.40 * f_z + 0.40 * f_disp + 0.20 * f_vol;
        let min_score = rev_cfg.min_activation_score;

        if score < min_score {
            return None;
        }

        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let wick_buffer = current_atr * rev_cfg.wick_sl_buffer_atr;
        let suggested_tp = Some(kama_mid);

        // Long Reversal khi giá dưới trung vị KAMA và Z-Score âm
        if z_score < 0.0 && close_price < kama_mid {
            Some(Proposal {
                signal_type: SignalType::Long,
                raw_score: score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(low - wick_buffer),
                suggested_tp,
            })
        }
        // Short Reversal khi giá trên trung vị KAMA và Z-Score dương
        else if z_score > 0.0 && close_price > kama_mid {
            Some(Proposal {
                signal_type: SignalType::Short,
                raw_score: score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(high + wick_buffer),
                suggested_tp,
            })
        } else {
            None
        }
    }
}
