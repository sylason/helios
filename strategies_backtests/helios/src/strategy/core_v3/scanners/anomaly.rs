//! # ANOMALY FADE SCANNER (ANOMALY)
//!
//! ## 1. Mục đích:
//! Bắt các dị biệt cực hạn của hành vi giá (Anomaly / Extreme Wick Rejection).
//! Khi xuất hiện cây nến có bóng nến (râu nến) dài bất thường từ chối một vùng giá cực hạn,
//! kết hợp với độ lệch chuẩn Z-Score cao, Scanner này mở vị thế ngược lại để ăn nhịp hồi phục nhanh (Fade).
//!
//! ## 2. Logic Hoạt Động Cốt Lõi:
//! - **Chỉ báo sử dụng:**
//!   + `z_score`: Đo độ lệch chuẩn giá so với trung bình quá khứ.
//!   + `upper_wick` ($High - \max(Open, Close)$): Chiều dài râu nến trên.
//!   + `lower_wick` ($\min(Open, Close) - Low$): Chiều dài râu nến dưới.
//!   + `atr_sl`: Chuẩn hóa chiều dài râu nến qua ATR.
//! - **Cơ chế Chấm Điểm:**
//!   + Long: $0.60 \times f_{\text{wick\_long}} + 0.40 \times f_z$ khi `z_score < 0.0`.
//!   + Short: $0.60 \times f_{\text{wick\_short}} + 0.40 \times f_z$ khi `z_score > 0.0`.
//! - **Thiết lập Vị thế:**
//!   + Long: `suggested_sl` neo dưới đáy râu nến: `low - wick_buffer`.
//!   + Short: `suggested_sl` neo trên đỉnh râu nến: `high + wick_buffer`.
//!   + Take Profit: `None` (để Trailing SL tự động xử lý).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;

pub struct AnomalyScanner;

impl TacticalScanner for AnomalyScanner {
    fn id(&self) -> ScannerId {
        ScannerId::AnomalyFade
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(AnomalyScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, _matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let ano_cfg = &config.tactical_sniper.anomaly;
        
        let z_score = sensors.z_score[i];
        let current_atr = sensors.atr_sl[i];
        
        let open = tick.opens()[i];
        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let close = tick.closes()[i];

        if z_score.is_nan() || current_atr.is_nan() || current_atr <= 0.0 {
            return None;
        }

        let upper_wick = high - open.max(close);
        let lower_wick = open.min(close) - low;
        
        let wick_thresh = ano_cfg.wick_atr_threshold.max(0.1);
        let z_thresh = ano_cfg.z_score_threshold.max(0.1);
        let min_score = ano_cfg.min_activation_score;

        let f_z = (z_score.abs() / z_thresh).tanh();
        let wick_buffer = current_atr * ano_cfg.wick_sl_buffer_atr;

        // Long Anomaly (Lower Pinbar Rejection khi Z-Score âm)
        if z_score < 0.0 {
            let f_wick_long = (lower_wick / (wick_thresh * current_atr)).tanh();
            let score_long = 0.60 * f_wick_long + 0.40 * f_z;

            if score_long >= min_score {
                return Some(Proposal {
                    signal_type: SignalType::Long,
                    raw_score: score_long.min(1.0),
                    scanner_id: self.id(),
                    suggested_sl: Some(low - wick_buffer),
                    suggested_tp: None,
                });
            }
        }
        // Short Anomaly (Upper Pinbar Rejection khi Z-Score dương)
        else if z_score > 0.0 {
            let f_wick_short = (upper_wick / (wick_thresh * current_atr)).tanh();
            let score_short = 0.60 * f_wick_short + 0.40 * f_z;

            if score_short >= min_score {
                return Some(Proposal {
                    signal_type: SignalType::Short,
                    raw_score: score_short.min(1.0),
                    scanner_id: self.id(),
                    suggested_sl: Some(high + wick_buffer),
                    suggested_tp: None,
                });
            }
        }

        None
    }
}
