//! # RANGE OSCILLATOR SCANNER (RANGE_OSC)
//!
//! ## 1. Mục đích:
//! Khai thác trạng thái thị trường đi ngang, tích lũy không có xu hướng (Range-Bound / Choppy Market).
//! Thực hiện giao dịch hồi quy về giá trị trung bình (Mean Reversion): Mua tại biên dưới Bollinger Bands
//! và Bán tại biên trên, chốt lời khi giá quay về đường trung bình SMA 20 (BB Middle).
//!
//! ## 2. Logic Hoạt Động & Hệ Thống Phòng Ngự 5 Tầng:
//! - **Chỉ báo sử dụng:**
//!   + Dải Bollinger Bands (`bb_upper`, `bb_middle`, `bb_lower`, `bb_width_norm`): Xác định biên độ kênh dao động.
//!   + `volume_force`: Phát hiện dòng tiền tin tức bất thường.
//!   + `trend_strength` & `bb_middle_slope_adj`: Nhận diện độ dốc và xung lực xu hướng để tránh cản tàu.
//!   + Stochastic Range (`stoch_range_k`, `stoch_range_d`): Đo đạc tín hiệu quá mua / quá bán và điểm giao cắt đảo chiều.
//! - **Các Chốt Chặn Phòng Ngự:**
//!   + Chốt chặn 1: Bề rộng dải tối thiểu `bb_width_norm >= min_width_pct` & Khối lượng thấp (không có tin tức xả hàng).
//!   + Chốt chặn 2: Sức mạnh xu hướng `trend_strength <= max_trend` (cấm tuyệt đối giao dịch Range khi thị trường đang có Trend).
//!   + Chốt chặn 3: Trục giữa BB Middle phẳng, không dốc (`|bb_slope| <= max_bb_slope`).
//!   + Chốt chặn 4: Nến đảo chiều phản ứng biên (Rejection Candle: Hammer rút chân cho Long, Shooting Star râu trên cho Short).
//! - **Thiết lập Vị thế:**
//!   + Stop Loss: Neo ngoài biên dải Bollinger Bands (`low.min(bb_l) - buffer` cho Long, `high.max(bb_u) + buffer` cho Short).
//!   + Take Profit: Hướng về trục giữa BB Middle (`target_price = bb_l + (bb_u - bb_l) * tp_ratio`).

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
        let open = tick.opens()[i];
        let high = tick.highs()[i];
        let low = tick.lows()[i];

        let bb_u = sensors.bb_upper[i];
        let bb_m = sensors.bb_middle[i];
        let bb_l = sensors.bb_lower[i];
        let bb_w_norm = matrix.bb_width_norm[i];
        let vol_force = matrix.volume_force[i];
        
        if bb_u.is_nan() || bb_m.is_nan() || bb_l.is_nan() || close <= 0.0 {
            return None;
        }

        // 1. Kiểm tra bề rộng tối thiểu Bollinger Bands & Volume Max Threshold
        if bb_w_norm < range_cfg.min_width_pct || vol_force > range_cfg.volume_max_threshold {
            return None;
        }

        // 1.1 BỘ LỌC XU HƯỚNG QUANT: Cấm đánh Range khi thị trường có Trend mạnh
        let max_trend = range_cfg.max_trend_strength.max(0.01);
        if matrix.trend_strength[i] > max_trend {
            return None;
        }

        // 1.2 🛡️ FLAW-REG.10: Kiểm tra độ dốc trục giữa BB Middle (SMA 20)
        let bb_slope = matrix.bb_middle_slope_adj.get(i).copied().unwrap_or(0.0);
        let max_bb_slope = range_cfg.max_bb_slope.max(0.01);

        let stoch_k = sensors.stoch_range_k[i];
        let stoch_d = sensors.stoch_range_d[i];
        if stoch_k.is_nan() || stoch_d.is_nan() {
            return None;
        }

        let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
        let buffer = current_atr * range_cfg.struct_sl_buffer_atr.max(0.05);
        let bb_prox = (range_cfg.bb_proximity_pct * close).max(1e-5);
        let two_sigma_sq = 2.0 * bb_prox * bb_prox;

        // 2. Chấm điểm Mẫu Hình LONG:
        // Giá bắt buộc phải nằm ở nửa dưới dải (close <= bb_m) và không bị đâm thủng dải dưới (close >= bb_l)
        let f_price_long = if close <= bb_m && close >= bb_l {
            let dist_l = close - bb_l;
            (- (dist_l * dist_l) / two_sigma_sq).exp()
        } else {
            0.0 // Đâm thủng dải dưới (Breakdown) hoặc nằm ở nửa trên dải -> Triệt tiêu!
        };
        let score_long = if f_price_long > 0.0 {
            let f_stoch_long = 1.0 / (1.0 + (-0.1 * (range_cfg.stoch_oversold - stoch_k)).exp());
            let f_hook_long = 1.0 / (1.0 + (-0.5 * (stoch_k - stoch_d)).exp());
            0.50 * f_price_long + 0.30 * f_stoch_long + 0.20 * f_hook_long
        } else {
            0.0
        };

        // 3. Chấm điểm Mẫu Hình SHORT:
        // Giá bắt buộc phải nằm ở nửa trên dải (close >= bb_m) và không bị đâm thủng dải trên (close <= bb_u)
        let f_price_short = if close >= bb_m && close <= bb_u {
            let dist_u = bb_u - close;
            (- (dist_u * dist_u) / two_sigma_sq).exp()
        } else {
            0.0 // Đâm thủng dải trên (Breakout) hoặc nằm ở nửa dưới dải -> Triệt tiêu!
        };
        let score_short = if f_price_short > 0.0 {
            let f_stoch_short = 1.0 / (1.0 + (-0.1 * (stoch_k - range_cfg.stoch_overbought)).exp());
            let f_hook_short = 1.0 / (1.0 + (-0.5 * (stoch_d - stoch_k)).exp());
            0.50 * f_price_short + 0.30 * f_stoch_short + 0.20 * f_hook_short
        } else {
            0.0
        };

        let min_score = range_cfg.min_activation_score;
        let max_adverse = range_cfg.max_adverse_slope.max(0.01);
        let trend_dir = matrix.trend_dir[i];
        let tp_ratio = range_cfg.tp_target_ratio.max(0.05).min(0.95);

        if score_long >= min_score && score_long > score_short {
            // 🛡️ CHỐNG BẮT DAO RƠI & TRÔI DỐC:
            // 1. KAMA slope vĩ mô không được dốc xuống
            if trend_dir < -max_adverse {
                return None;
            }
            // 2. Trục giữa BB Middle không được dốc cắm đầu
            if bb_slope < -max_bb_slope {
                return None;
            }
            // 🛡️ KHỐI 2: Nến xác nhận phản ứng (Rejection Candle)
            let is_green = close >= open;
            let lower_wick = (open.min(close) - low).max(0.0);
            let upper_wick = (high - open.max(close)).max(0.0);
            let is_hammer = lower_wick >= upper_wick * 1.2 && close > (high + low) / 2.0;
            if !is_green && !is_hammer {
                return None;
            }

            // 🎯 KHỐI 3: Take Profit thực tế hướng về Trục Giữa bb_middle
            let target_price = bb_l + (bb_u - bb_l) * tp_ratio;
            let valid_tp = target_price.max(close + buffer.max(current_atr * 0.20));

            // 🛡️ FLAW-TSL.6: Structural SL neo ngoài biên dưới dải Bollinger Bands
            let base_support = low.min(bb_l);
            let mut sl = base_support - buffer;
            if sl >= close {
                sl = close - buffer.max(current_atr * 0.20);
            }

            Some(Proposal {
                signal_type: SignalType::Long,
                raw_score: score_long.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: Some(valid_tp),
            })
        } else if score_short >= min_score && score_short > score_long {
            // 🛡️ CHỐNG CẢN TÀU & TRÔI DỐC LÊN:
            if trend_dir > max_adverse {
                return None;
            }
            if bb_slope > max_bb_slope {
                return None;
            }
            // 🛡️ KHỐI 2: Nến xác nhận phản ứng (Rejection Candle)
            let is_red = close <= open;
            let lower_wick = (open.min(close) - low).max(0.0);
            let upper_wick = (high - open.max(close)).max(0.0);
            let is_shooting_star = upper_wick >= lower_wick * 1.2 && close < (high + low) / 2.0;
            if !is_red && !is_shooting_star {
                return None;
            }

            // 🎯 KHỐI 3: Take Profit thực tế hướng về Trục Giữa bb_middle
            let target_price = bb_u - (bb_u - bb_l) * tp_ratio;
            let valid_tp = target_price.min(close - buffer.max(current_atr * 0.20));

            // 🛡️ FLAW-TSL.6: Structural SL neo ngoài biên trên dải Bollinger Bands
            let base_resistance = high.max(bb_u);
            let mut sl = base_resistance + buffer;
            if sl <= close {
                sl = close + buffer.max(current_atr * 0.20);
            }

            Some(Proposal {
                signal_type: SignalType::Short,
                raw_score: score_short.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: Some(valid_tp),
            })
        } else {
            None
        }
    }
}
