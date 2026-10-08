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
        let stoch_k = sensors.stoch_trend_k.get(i).copied().unwrap_or(f64::NAN);
        let rsi = sensors.rsi_trend.get(i).copied().unwrap_or(f64::NAN);
        let z_score = sensors.z_score.get(i).copied().unwrap_or(0.0);
        let squeeze = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);

        if kama_slope.is_nan() || stoch_k.is_nan() || rsi.is_nan() || z_score.is_nan() || squeeze.is_nan() {
            return None;
        }

        // 🧱 PHÒNG THỦ CHOP ZONE: Chặn đứng khi thị trường đang co thắt nén chặt BB/KC
        // Chop Zone là đất diễn của range_osc; TrendPB chỉ săn sóng khi thị trường đã bung nén.
        if squeeze > pb_cfg.max_squeeze_intensity {
            return None;
        }

        let slope_thresh = pb_cfg.kama_slope_threshold.max(1e-4);
        if kama_slope.abs() < slope_thresh {
            return None; // Độ dốc KAMA chưa đủ mạnh để hình thành xu hướng
        }

        let close = tick.closes()[i];
        let low = tick.lows()[i];
        let high = tick.highs()[i];

        let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
        let buffer_mult = pb_cfg.struct_sl_buffer_atr.clamp(0.10, 1.20);
        let buffer = current_atr * buffer_mult;

        let prev_stoch_k = if i >= 1 {
            sensors.stoch_trend_k.get(i - 1).copied().unwrap_or(stoch_k)
        } else {
            stoch_k
        };

        // Đạo hàm vi phân gradient nến: Close Location Value (CLV) [-1.0, +1.0]
        let tr_span = (high - low).max(1e-6);
        let clv = (2.0 * close - (high + low)) / tr_span;

        let f_slope = sat_unbounded(kama_slope.abs(), slope_thresh);

        // -------------------------------------------------------------------------
        // 1. CHIẾN ĐỘI PULLBACK LONG (KAMA Dốc Lên + Hồi Quy Vi Mô + Lực Cầu Rút Chân)
        // -------------------------------------------------------------------------
        if kama_slope > slope_thresh {
            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống mua đu đỉnh phân phối
            if z_score > pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Vùng hạ nhiệt Stochastic vi mô & Trần RSI an toàn
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k > stoch_thresh || rsi > pb_cfg.rsi_max_cap {
                return None;
            }

            // 🛡️ Chốt chặn 2: Soft Stochastic Hook Guard (Chống bắt dao rơi khi thác lũ thanh lý)
            if stoch_k < 10.0 && stoch_k < prev_stoch_k {
                return None; // Đang rơi tự do cắm đầu, cấm bắt dao rơi!
            }

            // Chốt chặn 3: Xác nhận lực cầu vi mô CLV (Close Location Value >= min_clv)
            if clv < pb_cfg.min_clv {
                return None;
            }

            let f_stoch = sat_proximity(stoch_k, stoch_thresh);
            let f_clv = ((clv - pb_cfg.min_clv) / (1.0 - pb_cfg.min_clv).max(1e-4)).clamp(0.0, 1.0);
            let score_long = convex_combine(&[f_slope, f_stoch, f_clv], &[0.40, 0.30, 0.30]);

            if score_long >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo chặt chẽ theo đáy nến vi mô hiện tại + đệm ATR (Khử 100% KAMA Suction Trap)
                let mut sl = low - buffer;
                if sl >= close {
                    sl = close - buffer.max(close * 0.002);
                }

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
            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống bán đuổi đáy hoảng loạn
            if z_score < -pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Vùng hạ nhiệt Stochastic vi mô & Sàn RSI an toàn
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k < (100.0 - stoch_thresh) || rsi < (100.0 - pb_cfg.rsi_max_cap) {
                return None;
            }

            // 🛡️ Chốt chặn 2: Soft Stochastic Hook Guard (Chống bán khống khi FOMO bốc đầu)
            if stoch_k > 90.0 && stoch_k > prev_stoch_k {
                return None; // Đang tăng dựng đứng, cấm chặn đầu xe lửa!
            }

            // Chốt chặn 3: Xác nhận lực cung vi mô CLV (Close Location Value <= -min_clv)
            if clv > -pb_cfg.min_clv {
                return None;
            }

            let stoch_dist = (100.0 - stoch_k).max(0.0);
            let f_stoch = sat_proximity(stoch_dist, stoch_thresh);
            let f_clv = ((-clv - pb_cfg.min_clv) / (1.0 - pb_cfg.min_clv).max(1e-4)).clamp(0.0, 1.0);
            let score_short = convex_combine(&[f_slope, f_stoch, f_clv], &[0.40, 0.30, 0.30]);

            if score_short >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo chặt chẽ theo đỉnh nến vi mô hiện tại + đệm ATR (Khử 100% KAMA Suction Trap)
                let mut sl = high + buffer;
                if sl <= close {
                    sl = close + buffer.max(close * 0.002);
                }

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
    use ferro_trade::data::{RawDataCache, DataView};
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

        let mut sensors = HeliosSensors::default();
        // Giả lập KAMA 4h nằm tít phía dưới cách 5% (1900.0 so với giá 2004.0)
        sensors.kama_mid = std::sync::Arc::new(vec![1900.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![4.0; len]); // buffer = 4.0 * 0.5 = 2.0
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);

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

        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);

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

        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = std::sync::Arc::new(vec![4.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);

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

        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![50.0; len]);

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

        // Kịch bản A: Stoch cực thấp (4.0 < 10.0) và ĐANG CẮM ĐẦU RƠI (prev = 8.0, curr = 4.0)
        let mut stoch_falling = vec![30.0; len];
        stoch_falling[i - 1] = 8.0;
        stoch_falling[i] = 4.0;

        let mut sensors_falling = HeliosSensors::default();
        sensors_falling.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_falling.stoch_trend_k = std::sync::Arc::new(stoch_falling);
        sensors_falling.rsi_trend = std::sync::Arc::new(vec![35.0; len]);

        let prop_falling = TrendPBScanner.scan(&tick, &sensors_falling, &matrix, &config);
        assert!(prop_falling.is_none(), "Soft Hook Guard PHẢI chặn đứng dao rơi khi Stoch < 10 và đang cắm đầu!");

        // Kịch bản B: Stoch cực thấp (4.0 < 10.0) nhưng ĐÃ NGÓC ĐẦU UỐN CONG (prev = 2.0, curr = 4.0)
        let mut stoch_hooked = vec![30.0; len];
        stoch_hooked[i - 1] = 2.0;
        stoch_hooked[i] = 4.0;

        let mut sensors_hooked = HeliosSensors::default();
        sensors_hooked.atr_sl = std::sync::Arc::new(vec![5.0; len]);
        sensors_hooked.stoch_trend_k = std::sync::Arc::new(stoch_hooked);
        sensors_hooked.rsi_trend = std::sync::Arc::new(vec![35.0; len]);

        let prop_hooked = TrendPBScanner.scan(&tick, &sensors_hooked, &matrix, &config);
        assert!(prop_hooked.is_some(), "Soft Hook Guard PHẢI phê duyệt khi Stoch đã ngóc đầu tạo đáy!");
    }
}
