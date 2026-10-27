//! # TREND PULLBACK SCANNER (TREND_PB)
//!
//! ## 1. Mục đích:
//! Bắt các nhịp hồi quy có kiểm soát (Pullback / Retracement) thuận theo trục xu hướng vĩ mô
//! được dẫn dắt bởi KAMA (Kaufman Adaptive Moving Average). Mục tiêu là "Mua khi giá giảm trong Trend tăng,
//! Bán khi giá tăng trong Trend giảm" tại các vùng hỗ trợ/kháng cự động.
//!
//! ## 2. Logic Hoạt Động & Bộ Lọc 3 Tầng An Toàn:
//! - **Chỉ báo sử dụng:**
//!   + `kama_slope_atr`: Độ dốc chuẩn hóa của KAMA làm la bàn xu hướng.
//!   + `kama_mid`: Đường trung vị KAMA đóng vai trò hỗ trợ/kháng cự động.
//!   + `stoch_trend_k`: Stochastic nhận diện nhịp hạ nhiệt tạm thời của giá.
//!   + `rsi_trend`: Trần/Sàn RSI bảo vệ chống mua đu đỉnh hoặc bán đu đáy.
//!   + `clv` (Close Location Value): Đạo hàm vị trí nến xác nhận nến phản ứng thuận chiều.
//! - **Quy trình Chấm Điểm & Phê Duyệt:**
//!   + Bước 1: Kiểm tra độ dốc KAMA đủ mạnh (`|kama_slope| >= slope_thresh`).
//!   + Bước 2: Kiểm tra khoảng cách `delta_kama` nằm trong hộp an toàn `[-inval_floor, max_dist]`.
//!   + Bước 3: Stochastic đã hạ nhiệt + RSI chưa chạm trần cực đại.
//!   + Bước 4: CLV xác nhận đóng nến thuận chiều (Long cấm nến đỏ, Short cấm nến xanh).
//! - **Thiết lập Vị thế:**
//!   + Long: `suggested_sl` neo dưới đáy nhịp hồi hoặc hỗ trợ KAMA: `low.min(kama_mid) - buffer`.
//!   + Short: `suggested_sl` neo trên đỉnh nhịp hồi hoặc kháng cự KAMA: `high.max(kama_mid) + buffer`.
//!   + Take Profit: `None` (để Trailing Stop Loss trong `RiskManager` tự động thả diều lợi nhuận).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::normalization::{sat_unbounded, sat_proximity, convex_combine};
use crate::strategy::core_v3::risk::RiskManager;

pub struct TrendPBScanner;

impl TacticalScanner for TrendPBScanner {
    fn id(&self) -> ScannerId {
        ScannerId::TrendPullback
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(TrendPBScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let pb_cfg = &config.tactical_sniper.pullback;
        
        let kama_slope = matrix.kama_slope_atr.get(i).copied()
            .unwrap_or_else(|| {
                sensors.kama_slope_adj.get(i).copied()
                    .map(|s| (s / 0.20).clamp(-1.0, 1.0))
                    .unwrap_or(0.0)
            });
        let kama_mid = sensors.kama_mid.get(i).copied().unwrap_or(f64::NAN);
        let stoch_k = sensors.stoch_trend_k.get(i).copied().unwrap_or(f64::NAN);
        let rsi = sensors.rsi_trend.get(i).copied().unwrap_or(f64::NAN);
        let z_score = sensors.z_score.get(i).copied().unwrap_or(0.0);
        let squeeze = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);

        if kama_slope.is_nan() || kama_mid.is_nan() || stoch_k.is_nan() || rsi.is_nan() || z_score.is_nan() || squeeze.is_nan() {
            return None;
        }

        let slope_thresh = pb_cfg.kama_slope_threshold.max(1e-4);
        if kama_slope.abs() < slope_thresh {
            return None; // Độ dốc KAMA chưa đủ mạnh để hình thành xu hướng
        }

        let close = tick.closes()[i];
        let low = tick.lows()[i];
        let high = tick.highs()[i];

        // 🧱 PHÒNG THỦ CHOP ZONE & NGOẠI LỆ CỜ TIẾP DIỄN (HIGH TIGHT / CONTINUATION FLAG):
        // Chop Zone là đất diễn của range_osc; TrendPB chỉ vượt trần Squeeze khi xung lực dốc KAMA
        // vượt trội (>= 2x threshold) và giá nến giữ vững phía trên trục KAMA_mid (Long) hoặc dưới KAMA_mid (Short).
        let is_continuation_flag = kama_slope.abs() >= 2.0 * slope_thresh
            && ((kama_slope > 0.0 && close >= kama_mid) || (kama_slope < 0.0 && close <= kama_mid));

        if squeeze > pb_cfg.max_squeeze_intensity && !is_continuation_flag {
            return None;
        }

        let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
        let buffer_mult = pb_cfg.struct_sl_buffer_atr.clamp(0.10, 2.00);
        let buffer = current_atr * buffer_mult;

        // 🧭 Quy đổi ATR vi mô sang hệ quy chiếu khung thời gian KAMA (Scale-Invariant MTF ATR):
        // sqrt(T_kama / T_atr) bảo đảm delta_kama luôn nhất quán dù KAMA chạy ở 15m, 1h hay 4h (TODO-SEM-02 / TODO-AUDIT-NEW-38)
        let kama_tf_mins = config.indicators.kama.timeframe.to_minutes().max(1) as f64;
        let atr_tf_mins = config.indicators.atr.timeframe.to_minutes().max(1) as f64;
        let kama_equiv_atr = (current_atr * (kama_tf_mins / atr_tf_mins).max(1.0).sqrt()).max(1e-5);
        let max_dist = pb_cfg.max_pullback_dist_atr.max(0.10);
        let inval_floor = pb_cfg.invalidation_buffer_atr.max(0.05);

        let cmo = sensors.cmo_micro.get(i).copied().unwrap_or(0.0);
        let prev_cmo = if i >= 1 {
            sensors.cmo_micro.get(i - 1).copied().unwrap_or(cmo)
        } else {
            cmo
        };

        // Đạo hàm vi phân gradient nến: Close Location Value (CLV) [-1.0, +1.0]
        let tr_span = (high - low).max(1e-6);
        let clv = (2.0 * close - (high + low)) / tr_span;

        let f_slope = sat_unbounded(kama_slope.abs(), slope_thresh);

        // -------------------------------------------------------------------------
        // 1. CHIẾN ĐỘI PULLBACK LONG (KAMA Dốc Lên + Hồi Quy Vi Mô + Lực Cầu Rút Chân)
        // -------------------------------------------------------------------------
        if kama_slope > slope_thresh {
            // 🛡️ CHỐT CHẶN VETO CẤU TRÚC BẬC THANG DOW (TODO-AUDIT-NEW-52):
            // Bảo vệ đáy sóng xoay chiều Higher Low (Swing Low cũ). Nếu giá đâm thủng đáy này -> VETO!
            let dow_floor = sensors.dow_floor.get(i).copied().unwrap_or(f64::NAN);
            if dow_floor.is_nan() || close < dow_floor {
                return None;
            }

            // 🛡️ CHỐT CHẶN TỶ LỆ HỒI QUY CỰC ĐẠI (TODO-AUDIT-NEW-180 Part 1):
            // Đo biên độ sóng đẩy impulse_range = impulse_ceiling - impulse_floor
            // retrace_ratio = (impulse_ceiling - close) / impulse_range <= max_retracement_ratio
            let imp_floor = sensors.impulse_floor.get(i).copied().unwrap_or(f64::NAN);
            let imp_ceiling = sensors.impulse_ceiling.get(i).copied().unwrap_or(f64::NAN);
            if !imp_floor.is_finite() || !imp_ceiling.is_finite() || imp_ceiling <= imp_floor {
                return None;
            }
            let impulse_range = (imp_ceiling - imp_floor).max(current_atr * 0.1);
            let retrace_ratio = ((imp_ceiling - close) / impulse_range).clamp(0.0, 2.0);
            if retrace_ratio > pb_cfg.max_retracement_ratio {
                return None;
            }

            // 🛡️ CHỐT CHẶN BẤT ĐỐI XỨNG VẬN TỐC SÓNG (TODO-AUDIT-NEW-180 Part 2):
            // cmo_trend_wave [-1.0, 1.0]: Nếu nhịp hồi xả dốc đứng (thác đổ), cmo < min_wave_cmo -> VETO!
            let wave_cmo = sensors.cmo_trend_wave.get(i).copied().unwrap_or(f64::NAN);
            if !wave_cmo.is_finite() || wave_cmo < pb_cfg.min_wave_cmo {
                return None;
            }

            // 🛡️ CHỐT CHẶN DÒNG TIỀN TÍCH LŨY / CẠN CUNG NO-SUPPLY (TODO-AUDIT-NEW-179):
            // cmf_trend [-1.0, 1.0]: Nếu xuất hiện áp lực xả hàng độc hại (toxic flow), cmf < min_cmf_trend -> VETO!
            let cmf_val = sensors.cmf_trend.get(i).copied().unwrap_or(f64::NAN);
            if !cmf_val.is_finite() || cmf_val < pb_cfg.min_cmf_trend {
                return None;
            }

            // 🛡️ CHỐT CHẶN KHOẢNG CÁCH KAMA (TODO-AUDIT-NEW-38):
            // delta_kama < -inval_floor: Giá đâm thủng sâu qua hỗ trợ KAMA (Breakdown Trap) -> VETO!
            // delta_kama > max_dist: Giá lơ lửng trên không trung xa KAMA (Floating Ghost Pullback) -> VETO!
            let delta_kama = (close - kama_mid) / kama_equiv_atr;
            if delta_kama < -inval_floor || delta_kama > max_dist {
                return None;
            }

            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống mua đu đỉnh Climax phân phối (Z > max_boundary_z)
            if z_score > pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Vùng hạ nhiệt Stochastic vi mô & Trần RSI an toàn (Single Responsibility)
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k > stoch_thresh || rsi > pb_cfg.rsi_max_cap {
                return None;
            }

            // 🛡️ Chốt chặn 2: Xác nhận nảy lên thuận xu hướng vi mô (CMO 1m & CLV >= min_clv - TODO-AUDIT-NEW-52)
            if cmo < prev_cmo || clv < pb_cfg.min_clv {
                return None;
            }

            let f_dist = sat_proximity(delta_kama.abs(), max_dist);
            let f_stoch = sat_proximity(stoch_k, stoch_thresh);
            let f_clv = ((clv - pb_cfg.min_clv) / (1.0 - pb_cfg.min_clv).max(1e-4)).clamp(0.0, 1.0);
            let score_long = convex_combine(&[f_slope, f_dist, f_stoch, f_clv], &[0.35, 0.25, 0.20, 0.20]);

            if score_long >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo thích ứng ra ngoài đáy sóng xoay chiều Dow và đáy nến vi mô + đệm ATR
                let struct_floor = dow_floor.min(low);
                let sl = RiskManager::sanitize_structural_sl(true, close, struct_floor - buffer, 0.0015, buffer);

                return Some(Proposal {
                    signal_type: SignalType::Long,
                    raw_score: score_long,
                    scanner_id: self.id(),
                    suggested_sl: Some(sl),
                    suggested_tp: None,
                });
            }
        }
        // -------------------------------------------------------------------------
        // 2. CHIẾN ĐỘI PULLBACK SHORT (KAMA Dốc Xuống + Hồi Quy Vi Mô + Lực Cung Đè Đỉnh)
        // -------------------------------------------------------------------------
        else if kama_slope < -slope_thresh {
            // 🛡️ CHỐT CHẶN VETO CẤU TRÚC BẬC THANG DOW (TODO-AUDIT-NEW-52):
            // Bảo vệ đỉnh sóng xoay chiều Lower High (Swing High cũ). Nếu giá đâm thủng đỉnh này -> VETO!
            let dow_ceiling = sensors.dow_ceiling.get(i).copied().unwrap_or(f64::NAN);
            if dow_ceiling.is_nan() || close > dow_ceiling {
                return None;
            }

            // 🛡️ CHỐT CHẶN TỶ LỆ HỒI QUY CỰC ĐẠI (TODO-AUDIT-NEW-180 Part 1):
            // Đo biên độ sóng đẩy impulse_range = impulse_ceiling - impulse_floor
            // retrace_ratio = (close - impulse_floor) / impulse_range <= max_retracement_ratio
            let imp_floor = sensors.impulse_floor.get(i).copied().unwrap_or(f64::NAN);
            let imp_ceiling = sensors.impulse_ceiling.get(i).copied().unwrap_or(f64::NAN);
            if !imp_floor.is_finite() || !imp_ceiling.is_finite() || imp_ceiling <= imp_floor {
                return None;
            }
            let impulse_range = (imp_ceiling - imp_floor).max(current_atr * 0.1);
            let retrace_ratio = ((close - imp_floor) / impulse_range).clamp(0.0, 2.0);
            if retrace_ratio > pb_cfg.max_retracement_ratio {
                return None;
            }

            // 🛡️ CHỐT CHẶN BẤT ĐỐI XỨNG VẬN TỐC SÓNG (TODO-AUDIT-NEW-180 Part 2):
            // cmo_trend_wave [-1.0, 1.0]: Nếu nhịp hồi đẩy tăng tốc chớp nhoáng, wave_cmo > -min_wave_cmo -> VETO!
            let wave_cmo = sensors.cmo_trend_wave.get(i).copied().unwrap_or(f64::NAN);
            if !wave_cmo.is_finite() || wave_cmo > -pb_cfg.min_wave_cmo {
                return None;
            }

            // 🛡️ CHỐT CHẶN DÒNG TIỀN PHÂN PHỐI / CẠN CẦU NO-DEMAND (TODO-AUDIT-NEW-179):
            // cmf_trend [-1.0, 1.0]: Nếu xuất hiện lực gom hàng ngược xu hướng, cmf > -min_cmf_trend -> VETO!
            let cmf_val = sensors.cmf_trend.get(i).copied().unwrap_or(f64::NAN);
            if !cmf_val.is_finite() || cmf_val > -pb_cfg.min_cmf_trend {
                return None;
            }

            // 🛡️ CHỐT CHẶN KHOẢNG CÁCH KAMA (TODO-AUDIT-NEW-38):
            // delta_kama < -inval_floor: Giá đâm thủng sâu lên trên kháng cự KAMA (Breakout Invalidation) -> VETO!
            // delta_kama > max_dist: Giá lơ lửng ở đáy sâu xa KAMA (Floating Ghost Short) -> VETO!
            let delta_kama = (kama_mid - close) / kama_equiv_atr;
            if delta_kama < -inval_floor || delta_kama > max_dist {
                return None;
            }

            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống bán đuổi đáy Climax hoảng loạn (Z < -max_boundary_z)
            if z_score < -pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Vùng hạ nhiệt Stochastic vi mô & Sàn RSI an toàn (Single Responsibility)
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k < (100.0 - stoch_thresh) || rsi < (100.0 - pb_cfg.rsi_max_cap) {
                return None;
            }

            // 🛡️ Chốt chặn 2: Xác nhận cắm đầu thuận xu hướng vi mô (CMO 1m & CLV <= -min_clv - TODO-AUDIT-NEW-52)
            if cmo > prev_cmo || clv > -pb_cfg.min_clv {
                return None;
            }

            let delta_short = delta_kama.abs();
            let f_dist = sat_proximity(delta_short, max_dist);
            let stoch_dist = (100.0 - stoch_k).max(0.0);
            let f_stoch = sat_proximity(stoch_dist, stoch_thresh);
            let f_clv = ((-clv - pb_cfg.min_clv) / (1.0 - pb_cfg.min_clv).max(1e-4)).clamp(0.0, 1.0);
            let score_short = convex_combine(&[f_slope, f_dist, f_stoch, f_clv], &[0.35, 0.25, 0.20, 0.20]);

            if score_short >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo thích ứng ra ngoài đỉnh sóng xoay chiều Dow và đỉnh nến vi mô + đệm ATR
                let struct_ceil = dow_ceiling.max(high);
                let sl = RiskManager::sanitize_structural_sl(false, close, struct_ceil + buffer, 0.0015, buffer);

                return Some(Proposal {
                    signal_type: SignalType::Short,
                    raw_score: score_short,
                    scanner_id: self.id(),
                    suggested_sl: Some(sl),
                    suggested_tp: None,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_trade::data::{RawDataCache, DataView, Timeframe};
    use ferro_trade::backtest::manager::IndicatorManager;
    use ferro_trade::broker::position::Position;
    use smallvec::SmallVec;
    use std::collections::HashMap;

    fn setup_mock_tick(
        opens: Vec<f64>,
        highs: Vec<f64>,
        lows: Vec<f64>,
        closes: Vec<f64>,
        _i: usize,
    ) -> (DataView, IndicatorManager, SmallVec<[ferro_trade::broker::actions::Action; 4]>, Vec<ferro_trade::broker::trade::Trade>) {
        let len = closes.len();
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        (DataView::new(raw_cache), IndicatorManager::new(), SmallVec::new(), Vec::new())
    }

    fn mock_pb_sensors_long(len: usize) -> HeliosSensors {
        let mut s = HeliosSensors::default();
        s.impulse_floor = std::sync::Arc::new(vec![1950.0; len]);
        s.impulse_ceiling = std::sync::Arc::new(vec![2050.0; len]);
        s.cmo_trend_wave = std::sync::Arc::new(vec![0.20; len]);
        s.cmf_trend = std::sync::Arc::new(vec![0.10; len]);
        s
    }

    fn mock_pb_sensors_short(len: usize) -> HeliosSensors {
        let mut s = HeliosSensors::default();
        s.impulse_floor = std::sync::Arc::new(vec![1950.0; len]);
        s.impulse_ceiling = std::sync::Arc::new(vec![2050.0; len]);
        s.cmo_trend_wave = std::sync::Arc::new(vec![-0.20; len]);
        s.cmf_trend = std::sync::Arc::new(vec![-0.10; len]);
        s
    }

    #[test]
    fn test_trend_pb_sl_geometry_no_kama_suction() {
        let len = 10;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2005.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2004.0; len];
        opens[i] = 2000.0;
        highs[i] = 2005.0;
        lows[i] = 1998.0;
        closes[i] = 2004.0; // CLV = (2*2004 - 2005 - 1998)/(2005-1998) = 5/7 = 0.71

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 0.50;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;

        let mut sensors = mock_pb_sensors_long(len);
        // Giả lập KAMA 15m nằm dưới giá (1998.0 so với giá 2004.0 -> delta_kama = (2004 - 1998)/(4 * sqrt(3)) = 6/6.93 = 0.86 <= 2.0)
        sensors.kama_mid = std::sync::Arc::new(vec![1998.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![4.0; len]); // buffer = 4.0 * 0.5 = 2.0
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1998.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.25; len]); // Uptrend mạnh

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "Trend_PB phải kích hoạt Long!");
        let p = prop.unwrap();
        let sl = p.suggested_sl.unwrap();

        // Với low=1998 và buffer=2.0 -> SL = 1996.0
        // Khẳng định SL không bị kéo tụt xuống KAMA 1900.0!
        assert!((sl - 1996.0).abs() < 1e-4, "SL phải neo tại 1996.0, thực tế nhận được: {}", sl);
        assert!(sl > 1950.0, "KAMA Suction Trap bị triệt tiêu hoàn toàn!");
    }

    #[test]
    fn test_trend_pb_allows_red_hammer() {
        let len = 10;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2005.0; len];
        let mut lows = vec![1995.0; len];
        let mut closes = vec![1999.0; len];
        // Nến thân đỏ nhẹ: Open = 2000.0, Close = 1999.0 (Close < Open)
        // High = 2001.0, Low = 1995.0 (Tr_span = 6.0)
        // Close = 1999.5 -> 2*1999.5 - (2001 + 1995) = 3999 - 3996 = 3 -> CLV = 3/6 = 0.50 >= 0.40!
        opens[i] = 2000.0;
        highs[i] = 2001.0;
        lows[i] = 1995.0;
        closes[i] = 1999.5;

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![1998.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1995.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "Nến Hammer thân đỏ rút chân mạnh CLV=0.50 PHẢI được phê duyệt mở Long!");
    }

    #[test]
    fn test_trend_pb_single_bar_sl_anchoring() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2005.0; len];
        let mut lows = vec![1998.0; len];
        let closes = vec![2004.0; len];

        // Nến i: low = 2000.0
        // Nến i-1: giả lập quét râu sâu low = 1990.0
        lows[i - 1] = 1990.0;
        lows[i] = 2000.0;

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 0.50; // buffer = 4.0 * 0.5 = 2.0

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![4.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![2000.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some());
        let sl = prop.unwrap().suggested_sl.unwrap();

        // SL neo theo đáy nến hiện tại low=2000.0 - 2.0 = 1998.0 (KHÔNG nhìn ngược về 1990.0)
        assert!((sl - 1998.0).abs() < 1e-4, "SL phải neo chuẩn xác theo low nến i: 2000.0 - 2.0 = 1998.0, thực tế: {}", sl);
    }

    #[test]
    fn test_trend_pb_blocks_marubozu() {
        let len = 10;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2005.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1998.0; len];

        // Nến đỏ Marubozu xả đặc ruột: Open = 2005, High = 2005, Low = 1990, Close = 1990 (CLV = -1.0)
        opens[i] = 2005.0;
        highs[i] = 2005.0;
        lows[i] = 1990.0;
        closes[i] = 1990.0;

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.min_clv = 0.20;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![1998.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Nến đỏ Marubozu CLV=-1.0 PHẢI bị chặn đứng, không cho phép mở Long!");
    }

    #[test]
    fn test_trend_pb_soft_hook_guard_blocks_falling_knife() {
        let len = 10;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2002.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1998.0; len];
        opens[i] = 1995.0;
        highs[i] = 2000.0;
        lows[i] = 1992.0;
        closes[i] = 1998.0; // CLV dương: (2*1998 - 2000 - 1992)/(2000-1992) = 4/8 = 0.50 >= min_clv(0.20)

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.min_clv = 0.20;

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        // Kịch bản A: Động năng vi mô CMO đang cắm đầu rơi (cmo < prev_cmo) -> VETO!
        let mut cmo_falling = vec![0.10; len];
        cmo_falling[i - 1] = 0.20;
        cmo_falling[i] = 0.05;

        let mut sensors_falling = mock_pb_sensors_long(len);
        sensors_falling.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_falling.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_falling.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors_falling.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_falling.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors_falling.cmo_micro = std::sync::Arc::new(cmo_falling);

        let prop_falling = TrendPBScanner.scan(&tick, &sensors_falling, &matrix, &config);
        assert!(prop_falling.is_none(), "CMO Micro Guard PHẢI chặn đứng dao rơi khi CMO đang cắm đầu (cmo < prev_cmo)!");

        // Kịch bản B: Động năng vi mô CMO đã uốn cong ngóc đầu nảy lên (cmo >= prev_cmo) -> APPROVED!
        let mut cmo_hooked = vec![0.10; len];
        cmo_hooked[i - 1] = 0.05;
        cmo_hooked[i] = 0.20;

        let mut sensors_hooked = mock_pb_sensors_long(len);
        sensors_hooked.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_hooked.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_hooked.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors_hooked.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_hooked.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors_hooked.cmo_micro = std::sync::Arc::new(cmo_hooked);

        let prop_hooked = TrendPBScanner.scan(&tick, &sensors_hooked, &matrix, &config);
        assert!(prop_hooked.is_some(), "CMO Micro Guard PHẢI phê duyệt khi CMO đã ngóc đầu tạo đà nảy lên!");
    }

    #[test]
    fn test_trend_pb_hook_guard_blocks_zero_and_hundred_saturation() {
        let len = 10;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2002.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1998.0; len];
        opens[i] = 1995.0;
        highs[i] = 2000.0;
        lows[i] = 1992.0;
        closes[i] = 1998.0; // CLV = 0.50

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.min_clv = 0.20;

        // Long bão hòa sàn 0.0: K = 0.0, D = 0.0
        let mut sensors_zero = mock_pb_sensors_long(len);
        sensors_zero.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_zero.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_zero.stoch_trend_k = std::sync::Arc::new(vec![0.0; len]);
        sensors_zero.stoch_trend_d = std::sync::Arc::new(vec![0.0; len]);
        sensors_zero.rsi_trend = std::sync::Arc::new(vec![35.0; len]);

        let mut matrix_long = MarketMatrixV3::default();
        matrix_long.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop_zero = TrendPBScanner.scan(&tick, &sensors_zero, &matrix_long, &config);
        assert!(prop_zero.is_none(), "Hook Guard PHẢI chặn đứng sàn bão hòa xả lũ K=D=0.0!");

        // Short bão hòa trần 100.0: K = 100.0, D = 100.0
        let mut sensors_hundred = mock_pb_sensors_long(len);
        sensors_hundred.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_hundred.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_hundred.stoch_trend_k = std::sync::Arc::new(vec![100.0; len]);
        sensors_hundred.stoch_trend_d = std::sync::Arc::new(vec![100.0; len]);
        sensors_hundred.rsi_trend = std::sync::Arc::new(vec![65.0; len]);

        let mut matrix_short = MarketMatrixV3::default();
        matrix_short.kama_slope_atr = std::sync::Arc::new(vec![-0.20; len]);

        // Tạo nến CLV âm cho Short
        let mut closes_short = vec![1998.0; len];
        closes_short[i] = 1993.0; // CLV = -0.75 <= -0.20
        let (data_view_s, ind_mgr_s, mut actions_s, trades_s) = setup_mock_tick(
            vec![2000.0; len], vec![2000.0; len], vec![1992.0; len], closes_short, i
        );
        let tick_short = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_s,
            indicator_manager: &ind_mgr_s,
            trades: &trades_s,
            last_closed_trade: None,
            actions: &mut actions_s,
            position: Position::default(),
        };

        let prop_hundred = TrendPBScanner.scan(&tick_short, &sensors_hundred, &matrix_short, &config);
        assert!(prop_hundred.is_none(), "Hook Guard PHẢI chặn đứng trần bão hòa FOMO K=D=100.0!");
    }

    #[test]
    fn test_trend_pb_hook_guard_blocks_deadzone_falling() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2000.0; len];
        let lows = vec![1992.0; len];
        let closes = vec![1998.0; len]; // CLV = 0.50

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 35.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.min_clv = 0.20;

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        // Kịch bản A: Giá đâm thủng DowFloor (close 1998.0 < dow_floor 2002.0) -> VETO!
        let mut sensors_deadzone = mock_pb_sensors_long(len);
        sensors_deadzone.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_deadzone.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_deadzone.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors_deadzone.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_deadzone.dow_floor = std::sync::Arc::new(vec![2002.0; len]); // 2002 > close 1998 -> VETO
        sensors_deadzone.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let prop_deadzone = TrendPBScanner.scan(&tick, &sensors_deadzone, &matrix, &config);
        assert!(prop_deadzone.is_none(), "Dow Structure Guard PHẢI VETO khi giá đâm thủng đáy Higher Low cũ (close < dow_floor)!");

        // Kịch bản B: Giá giữ vững DowFloor (close 1998.0 >= dow_floor 1990.0) -> APPROVED!
        let mut sensors_rebound = mock_pb_sensors_long(len);
        sensors_rebound.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_rebound.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_rebound.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors_rebound.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_rebound.dow_floor = std::sync::Arc::new(vec![1990.0; len]); // 1990 <= close 1998 -> PASS
        sensors_rebound.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let prop_rebound = TrendPBScanner.scan(&tick, &sensors_rebound, &matrix, &config);
        assert!(prop_rebound.is_some(), "Dow Structure Guard PHẢI cho phép mở Long khi giá giữ vững trên đáy DowFloor!");
    }

    #[test]
    fn test_trend_pb_hook_guard_climax_dual_confirmation() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2000.0; len];
        let lows = vec![1992.0; len];
        let closes = vec![1998.0; len]; // CLV = 0.50

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 35.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.min_clv = 0.20;

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        // Kịch bản A: Nến chưa rút chân đạt chuẩn (min_clv = 0.60 > CLV = 0.50) -> VETO!
        let mut config_high_clv = config.clone();
        config_high_clv.tactical_sniper.pullback.min_clv = 0.60;

        let mut sensors_fake = mock_pb_sensors_long(len);
        sensors_fake.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_fake.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_fake.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors_fake.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_fake.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors_fake.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let prop_fake = TrendPBScanner.scan(&tick, &sensors_fake, &matrix, &config_high_clv);
        assert!(prop_fake.is_none(), "Micro CLV Guard PHẢI chặn đứng khi nến chưa rút chân đạt min_clv!");

        // Kịch bản B: Nến rút chân đạt chuẩn (min_clv = 0.20 <= CLV = 0.50) -> APPROVED!
        let mut sensors_confirmed = mock_pb_sensors_long(len);
        sensors_confirmed.kama_mid = std::sync::Arc::new(vec![1996.0; len]);
        sensors_confirmed.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_confirmed.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors_confirmed.rsi_trend = std::sync::Arc::new(vec![35.0; len]);
        sensors_confirmed.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors_confirmed.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let prop_confirmed = TrendPBScanner.scan(&tick, &sensors_confirmed, &matrix, &config);
        assert!(prop_confirmed.is_some(), "Micro CLV Guard PHẢI phê duyệt khi nến rút chân đạt min_clv!");
    }

    #[test]
    fn test_trend_pb_kama_distance_box_and_mtf_scaling() {
        let len = 10;
        let i = len - 1;

        // Cấu hình cơ sở: KAMA 15m, ATR 5m => scale = sqrt(15/5) = sqrt(3) ~ 1.73205
        // ATR = 10.0 => kama_equiv_atr = 17.3205
        // kama_mid = 2000.0, max_pullback_dist_atr = 2.0, invalidation_buffer_atr = 0.50
        // Hộp an toàn cho Long: delta_kama in [-0.50, +2.0]
        // Ngưỡng giá tuyệt đối tương ứng:
        // Cực dưới (invalidation floor): 2000.0 - 0.50 * 17.3205 = 1991.34
        // Cực trên (parabolic chasing ceiling): 2000.0 + 2.0 * 17.3205 = 2034.64
        let mut config = HeliosConfigV3::default();
        config.indicators.kama.timeframe = Timeframe::T15m;
        config.indicators.atr.timeframe = Timeframe::T5m;
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.50;
        config.tactical_sniper.pullback.min_clv = 0.05;

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.25; len]); // Uptrend mạnh

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        let mut stoch_k_vec = vec![30.0; len];
        stoch_k_vec[i - 1] = 20.0;
        stoch_k_vec[i] = 25.0; // Hook up: stoch_k > prev
        sensors.stoch_trend_k = std::sync::Arc::new(stoch_k_vec);
        sensors.stoch_trend_d = std::sync::Arc::new(vec![22.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1970.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        // --- Kịch bản 1: Shallow Piercing Hợp lệ (Nằm an toàn trong đệm gãy nông) ---
        // Close = 1998.0, Low = 1990.0, High = 2002.0 => CLV = (2*1998 - 3992) / 12 = +4/12 = +0.333 >= 0.05
        // delta_kama = (1998 - 2000) / 17.3205 = -0.1155 (>= -0.50) => APPROVED!
        {
            let mut opens = vec![2000.0; len];
            let mut highs = vec![2005.0; len];
            let mut lows = vec![1990.0; len];
            let mut closes = vec![2000.0; len];
            opens[i] = 1992.0;
            highs[i] = 2002.0;
            lows[i] = 1990.0;
            closes[i] = 1998.0;

            let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
            let tick = TickContext {
                i,
                equity: 1000.0,
                data_view: &dv,
                indicator_manager: &im,
                trades: &tr,
                last_closed_trade: None,
                actions: &mut act,
                position: Position::default(),
            };
            let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
            assert!(prop.is_some(), "Shallow Piercing delta_kama = -0.1155 in [-0.50, 2.0] PHẢI được phê duyệt!");
        }

        // --- Kịch bản 2: Breakdown Invalidation VETO (Xuyên thủng sâu quá ngưỡng cho phép) ---
        // Close = 1985.0, Low = 1980.0, High = 1988.0 => CLV = (2*1985 - 3968) / 8 = +0.25 >= 0.05
        // delta_kama = (1985 - 2000) / 17.3205 = -0.866 < -0.50 => VETO!
        {
            let mut opens = vec![2000.0; len];
            let mut highs = vec![2005.0; len];
            let mut lows = vec![1980.0; len];
            let mut closes = vec![2000.0; len];
            opens[i] = 1982.0;
            highs[i] = 1988.0;
            lows[i] = 1980.0;
            closes[i] = 1985.0;

            let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
            let tick = TickContext {
                i,
                equity: 1000.0,
                data_view: &dv,
                indicator_manager: &im,
                trades: &tr,
                last_closed_trade: None,
                actions: &mut act,
                position: Position::default(),
            };
            let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
            assert!(prop.is_none(), "Breakdown delta_kama = -0.866 < -0.50 PHẢI bị VETO chặn đứng!");
        }

        // --- Kịch bản 3: Parabolic Chasing VETO (Đu đỉnh quá xa trên KAMA) ---
        // Close = 2040.0, Low = 2035.0, High = 2043.0 => CLV = (2*2040 - 4078) / 8 = +0.25 >= 0.05
        // delta_kama = (2040 - 2000) / 17.3205 = +2.3094 > 2.0 => VETO!
        {
            let mut opens = vec![2000.0; len];
            let mut highs = vec![2045.0; len];
            let mut lows = vec![2000.0; len];
            let mut closes = vec![2000.0; len];
            opens[i] = 2036.0;
            highs[i] = 2043.0;
            lows[i] = 2035.0;
            closes[i] = 2040.0;

            let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
            let tick = TickContext {
                i,
                equity: 1000.0,
                data_view: &dv,
                indicator_manager: &im,
                trades: &tr,
                last_closed_trade: None,
                actions: &mut act,
                position: Position::default(),
            };
            let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
            assert!(prop.is_none(), "Parabolic chasing delta_kama = +2.31 > 2.0 PHẢI bị VETO chặn đứng!");
        }

        // --- Kịch bản 4: MTF Scale Invariance trên KAMA 4h ---
        // Đổi KAMA sang 4h (240m), ATR 5m => scale = sqrt(240/5) = sqrt(48) ~ 6.9282
        // kama_equiv_atr = 10.0 * 6.9282 = 69.282
        // Invalidation floor = 2000.0 - 0.50 * 69.282 = 1965.36
        // Thử giá Close = 1975.0 (cách KAMA -25.0):
        // Nếu dùng ATR 5m không scale (10.0): delta = -25 / 10 = -2.5 < -0.50 => SẼ BỊ VETO SAI LẦM!
        // Khi có MTF scale: delta_kama = -25 / 69.282 = -0.3608 in [-0.50, 2.0] => APPROVED!
        {
            let mut config_4h = config.clone();
            config_4h.indicators.kama.timeframe = Timeframe::T4h;

            let mut opens = vec![2000.0; len];
            let mut highs = vec![2005.0; len];
            let mut lows = vec![1970.0; len];
            let mut closes = vec![2000.0; len];
            opens[i] = 1972.0;
            highs[i] = 1978.0;
            lows[i] = 1970.0;
            closes[i] = 1975.0; // CLV = (2*1975 - 3948) / 8 = +0.25 >= 0.05

            let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
            let tick = TickContext {
                i,
                equity: 1000.0,
                data_view: &dv,
                indicator_manager: &im,
                trades: &tr,
                last_closed_trade: None,
                actions: &mut act,
                position: Position::default(),
            };
            let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config_4h);
            assert!(prop.is_some(), "KAMA 4h MTF Equiv ATR scaling PHẢI bảo vệ pullback rộng -25 điểm (delta_kama = -0.36) không bị VETO oan!");
        }
    }

    #[test]
    fn test_trend_pb_allows_entry_in_strong_macro_uptrend() {
        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2005.0; len];
        // Nến nảy hồi thuận xu hướng: High = 2010, Low = 2000, Close = 2006
        // CLV = (2 * 2006 - 2010 - 2000) / (2010 - 2000) = (4012 - 4010) / 10 = +0.20 >= min_clv (0.20)
        opens[i] = 2002.0;
        highs[i] = 2010.0;
        lows[i] = 2000.0;
        closes[i] = 2006.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv,
            indicator_manager: &im,
            trades: &tr,
            last_closed_trade: None,
            actions: &mut act,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default(); // default max_boundary_z = 1.60

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        // Stoch uốn cong ngóc đầu lên từ đáy: prev = 20.0, curr = 25.0
        let mut stoch_k = vec![30.0; len];
        stoch_k[i - 1] = 20.0;
        stoch_k[i] = 25.0;
        sensors.stoch_trend_k = std::sync::Arc::new(stoch_k);
        sensors.stoch_trend_d = std::sync::Arc::new(vec![22.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![52.0; len]);
        // Z-score trong sóng tăng 4h mạnh tự nhiên đạt +1.20 (vượt trần cũ 0.80 nhưng dưới trần mới 1.60)
        sensors.z_score = std::sync::Arc::new(vec![1.20; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1995.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.25; len]); // Uptrend mạnh

        // 1. Khẳng định: TrendPB PHẢI cho phép mở Long trong sóng tăng mạnh khi Z = 1.20 <= 1.60
        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "TrendPB PHẢI cấp Proposal Long khi nhịp hồi trong sóng tăng mạnh có Z = 1.20 <= 1.60!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);

        // 2. Khẳng định: Khi Z-score vọt lên đỉnh Climax phân phối (+1.80 > 1.60), Boundary Guard PHẢI VETO
        let mut sensors_climax = sensors.clone();
        sensors_climax.z_score = std::sync::Arc::new(vec![1.80; len]);
        let prop_climax = TrendPBScanner.scan(&tick, &sensors_climax, &matrix, &config);
        assert!(prop_climax.is_none(), "Boundary Guard PHẢI chặn đứng đu đỉnh Climax khi Z = 1.80 > 1.60!");
    }

    #[test]
    fn test_trend_pb_allows_short_entry_in_strong_macro_downtrend() {
        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2002.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1995.0; len];
        // Nến hồi lên rồi đè đầu cắm xuống thuận xu hướng giảm:
        // High = 2000, Low = 1990, Close = 1994
        // CLV = (2 * 1994 - 2000 - 1990) / (2000 - 1990) = (3988 - 3990) / 10 = -0.20 <= -min_clv (-0.20)
        opens[i] = 1998.0;
        highs[i] = 2000.0;
        lows[i] = 1990.0;
        closes[i] = 1994.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv,
            indicator_manager: &im,
            trades: &tr,
            last_closed_trade: None,
            actions: &mut act,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default(); // default max_boundary_z = 1.60

        let mut sensors = mock_pb_sensors_short(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        // Stoch uốn cong chúc đầu xuống từ đỉnh: prev = 80.0, curr = 75.0
        let mut stoch_k = vec![70.0; len];
        stoch_k[i - 1] = 80.0;
        stoch_k[i] = 75.0;
        sensors.stoch_trend_k = std::sync::Arc::new(stoch_k);
        sensors.stoch_trend_d = std::sync::Arc::new(vec![78.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![48.0; len]);
        // Z-score trong sóng giảm 4h mạnh tự nhiên đạt -1.20 (vượt trần âm cũ -0.80 nhưng trên trần mới -1.60)
        sensors.z_score = std::sync::Arc::new(vec![-1.20; len]);
        sensors.dow_ceiling = std::sync::Arc::new(vec![2005.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![-0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.25; len]); // Downtrend mạnh

        // 1. Khẳng định: TrendPB PHẢI cho phép mở Short trong sóng giảm mạnh khi Z = -1.20 >= -1.60
        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "TrendPB PHẢI cấp Proposal Short khi nhịp hồi trong sóng giảm mạnh có Z = -1.20 >= -1.60!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Short);

        // 2. Khẳng định: Khi Z-score sập sâu xuống đáy Climax hoảng loạn (-1.80 < -1.60), Boundary Guard PHẢI VETO
        let mut sensors_climax = sensors.clone();
        sensors_climax.z_score = std::sync::Arc::new(vec![-1.80; len]);
        let prop_climax = TrendPBScanner.scan(&tick, &sensors_climax, &matrix, &config);
        assert!(prop_climax.is_none(), "Boundary Guard PHẢI chặn đứng bán đuổi đáy Climax khi Z = -1.80 < -1.60!");
    }

    #[test]
    fn test_trend_pb_allows_entry_in_squeeze_continuation_flag() {
        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2005.0; len];
        // Nến nảy hồi thuận xu hướng: High = 2010, Low = 2000, Close = 2006 >= kama_mid (2000.0)
        // CLV = (2 * 2006 - 2010 - 2000) / (2010 - 2000) = (4012 - 4010) / 10 = +0.20 >= min_clv (0.20)
        opens[i] = 2002.0;
        highs[i] = 2010.0;
        lows[i] = 2000.0;
        closes[i] = 2006.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv,
            indicator_manager: &im,
            trades: &tr,
            last_closed_trade: None,
            actions: &mut act,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default(); // default max_squeeze_intensity = 0.75, slope_thresh = 0.05

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        let mut stoch_k = vec![30.0; len];
        stoch_k[i - 1] = 20.0;
        stoch_k[i] = 25.0;
        sensors.stoch_trend_k = std::sync::Arc::new(stoch_k);
        sensors.stoch_trend_d = std::sync::Arc::new(vec![22.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![52.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![1.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1995.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        // Squeeze nén cực chặt 0.85 > max_squeeze_intensity (0.75)
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.85; len]);
        // Độ dốc KAMA slope_atr = 0.15 >= 2 * 0.05 (thoả mãn ngoại lệ Cờ Tiếp Diễn)
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "TrendPB PHẢI cấp Proposal Long cho mô hình Cờ Tiếp Diễn dù Squeeze = 0.85 > 0.75!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
    }

    #[test]
    fn test_trend_pb_blocks_entry_in_squeeze_without_strong_slope() {
        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2005.0; len];
        opens[i] = 2002.0;
        highs[i] = 2010.0;
        lows[i] = 2000.0;
        closes[i] = 2006.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv,
            indicator_manager: &im,
            trades: &tr,
            last_closed_trade: None,
            actions: &mut act,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default();

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        let mut stoch_k = vec![30.0; len];
        stoch_k[i - 1] = 20.0;
        stoch_k[i] = 25.0;
        sensors.stoch_trend_k = std::sync::Arc::new(stoch_k);
        sensors.stoch_trend_d = std::sync::Arc::new(vec![22.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![52.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![1.0; len]);

        let mut matrix = MarketMatrixV3::default();
        // Squeeze nén cực chặt 0.85 > 0.75
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.85; len]);
        // Độ dốc KAMA slope_atr = 0.07 < 2 * 0.05 (chưa đủ gấp đôi để xem là cờ tiếp diễn)
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.07; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB PHẢI chặn đứng vị thế khi Squeeze = 0.85 > 0.75 mà độ dốc KAMA chưa đủ mạnh (Chop Zone)!");
    }

    #[test]
    fn test_trend_pb_sl_buffer_supports_up_to_2_0_atr() {
        // 🛡️ [TODO-AUDIT-NEW-184]: Kiểm chứng clamp struct_sl_buffer_atr đã nới lên 2.0 ATR (không còn bị bóp nghẹt ở 1.20 ATR)
        let len = 50;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        let closes = vec![2008.0; len];

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 2.00; // Đệm 2.0 ATR

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]); // ATR = 5.0 -> Buffer = 10.0
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some());
        let sl = prop.unwrap().suggested_sl.unwrap();

        // Low = 1990.0, Buffer = 5.0 * 2.0 = 10.0 -> SL phải là 1980.0 (nếu bị kẹp ở 1.20 cũ thì sẽ là 1990.0 - 6.0 = 1984.0)
        assert!((sl - 1980.0).abs() < 1e-4, "SL phải hỗ trợ buffer tới 2.0 ATR (1980.0), thực tế: {}", sl);
    }

    #[test]
    fn test_trend_pb_rejects_high_volume_distribution_retrace() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        let closes = vec![2008.0; len];

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.min_cmf_trend = -0.10;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        // CMF rất âm (-0.35 < -0.10): Xả hàng volume lớn (Toxic Flow)
        sensors.cmf_trend = std::sync::Arc::new(vec![-0.35; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Trend_PB PHẢI chặn nhịp hồi xả hàng volume lớn (CMF < min_cmf_trend)!");
    }

    #[test]
    fn test_trend_pb_accepts_volume_dry_up() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        let closes = vec![2008.0; len];

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.min_cmf_trend = -0.10;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        // CMF dương (+0.15 >= -0.10): Cạn cung lành mạnh (No-supply Pullback)
        sensors.cmf_trend = std::sync::Arc::new(vec![0.15; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "Trend_PB PHẢI chấp thuận nhịp hồi cạn cung lành mạnh (CMF >= min_cmf_trend)!");
    }

    #[test]
    fn test_trend_pb_rejects_deep_retracement_exceeding_ratio() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        // Giá hiện tại là 1995.0. Sóng đẩy từ 1980.0 lên 2060.0 (Impulse Range = 80.0)
        // Retracement = (2060.0 - 1995.0) / 80.0 = 65.0 / 80.0 = 0.8125 > max_ratio (0.75)
        let closes = vec![1995.0; len];

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.max_retracement_ratio = 0.75;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 5.0; // Mở rộng để không bị chặn bởi delta_kama
        config.tactical_sniper.pullback.invalidation_buffer_atr = 5.0;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![1995.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1980.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        // Cấu hình sóng đẩy: Floor 1980, Ceiling 2060 -> Range 80
        sensors.impulse_floor = std::sync::Arc::new(vec![1980.0; len]);
        sensors.impulse_ceiling = std::sync::Arc::new(vec![2060.0; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Trend_PB PHẢI chặn nhịp hồi quá sâu (retrace_ratio = 81.25% > 75%)!");
    }

    #[test]
    fn test_trend_pb_rejects_flash_velocity_inversion() {
        let len = 10;
        let i = len - 1;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        let closes = vec![2008.0; len];

        let (data_view, ind_mgr, mut actions, trades) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &ind_mgr,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.min_wave_cmo = -0.10;

        let mut sensors = mock_pb_sensors_long(len);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);
        sensors.dow_floor = std::sync::Arc::new(vec![1990.0; len]);
        sensors.cmo_micro = std::sync::Arc::new(vec![0.10; len]);

        // Vận tốc sóng bị đảo chiều âm cực mạnh (-0.45 < -0.10): Cú sập thác chớp nhoáng
        sensors.cmo_trend_wave = std::sync::Arc::new(vec![-0.45; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.20; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Trend_PB PHẢI chặn cú sập thác gia tốc giảm cực mạnh (wave_cmo < min_wave_cmo)!");
    }
}
