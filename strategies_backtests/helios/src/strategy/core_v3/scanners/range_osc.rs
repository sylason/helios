//! # RANGE BOUND SCANNER (FIXED STRUCTURAL BOX VAH/VAL/POC)
//!
//! ## 1. Mục đích:
//! Khai thác trạng thái thị trường đi ngang, tích lũy không có xu hướng (Range-Bound / Choppy Market)
//! dựa trên mô hình Hộp Cấu Trúc Cố Định (Fixed Structural Box) theo chuẩn Auction Market Theory.
//! Thay thế hoàn toàn dải Bollinger Bands động 20 nến co giãn méo mó bằng biên trên VAH, biên dưới VAL
//! và trục giá trị công bằng POC.
//!
//! ## 2. Logic Hoạt Động & Hệ Thống Phòng Ngự Định Lượng:
//! - **Xác lập Hộp:** Quét cực trị N = 40 nến: VAH = max(High), VAL = min(Low), POC = (VAH + VAL) / 2.
//! - **Chốt chặn 1 (Sàn Chi Phí Sinh Tồn):** Chiều cao hộp box_height = VAH - VAL >= 2.0 * ATR.
//! - **Chốt chặn 2 (Dead Zone Exclusion):** Cấm giao dịch trong vùng 50% quanh trục giữa POC (|delta_norm| < 0.50).
//! - **Chốt chặn 3 (Vector Lực Nến CLV):** Đòi hỏi CLV >= +0.35 (Long tại VAL) hoặc CLV <= -0.35 (Short tại VAH).
//! - **Chốt chặn 4 (Chống Cản Tàu / Bắt Dao Rơi):** Cấm Long khi KAMA slope dốc xuống, cấm Short khi KAMA slope dốc lên.
//! - **Chốt Lời & Cắt Lỗ:** TP hướng sang biên đối diện hộp (VAL -> VAH, VAH -> VAL) đạt R:R >= 1.50; SL neo dưới chân râu nến vi mô.

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::risk::RiskManager;

pub struct RangeOscScanner;

impl TacticalScanner for RangeOscScanner {
    fn id(&self) -> ScannerId {
        ScannerId::RangeOscillator
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(RangeOscScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let lookback = 40.min(i);
        if lookback < 10 {
            return None;
        }

        let close = tick.closes()[i];
        let high = tick.highs()[i];
        let low = tick.lows()[i];

        if close <= 0.0 || high.is_nan() || low.is_nan() || close.is_nan() {
            return None;
        }

        let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
        let highs = tick.highs();
        let lows = tick.lows();

        // 1. Xác lập Hộp Cấu Trúc Cố Định (Fixed Structural Box) từ cực trị N nến
        let mut vah = f64::NEG_INFINITY;
        let mut val = f64::INFINITY;
        for idx in (i - lookback)..i {
            let h = highs[idx];
            let l = lows[idx];
            if !h.is_nan() && h > vah { vah = h; }
            if !l.is_nan() && l < val { val = l; }
        }

        if !vah.is_finite() || !val.is_finite() || vah <= val {
            return None;
        }

        let box_height = vah - val;
        // 🛡️ CHỐT CHẶN 1: Sàn chi phí sinh tồn (Minimum Economic Height >= 2.0 ATR)
        // Ngăn chặn các hộp quá hẹp bị phí sàn và spread bào mòn vốn
        if box_height < 2.0 * current_atr {
            return None;
        }

        let poc = (vah + val) * 0.5;

        // 2. Tọa độ phân vị phi thứ nguyên trong hộp: delta_norm in [-1.0, 1.0]
        let delta_norm = ((close - poc) / (0.5 * box_height)).clamp(-1.0, 1.0);

        // 🛡️ CHỐT CHẶN 2: Vùng cấm Dead Zone 50% quanh trục giữa POC
        // Triệt tiêu hoàn toàn nguy cơ cào lệnh ở giữa hộp
        if delta_norm.abs() < 0.50 {
            return None;
        }

        // 3. Vector lực từ chối giá phi thứ nguyên (Close Location Value - CLV in [-1.0, 1.0])
        let candle_range = (high - low).max(1e-5);
        let clv = ((2.0 * close - high - low) / candle_range).clamp(-1.0, 1.0);

        let range_cfg = &config.tactical_sniper.range;
        let buffer = current_atr * range_cfg.struct_sl_buffer_atr.max(0.10);
        // Đường biên tuyến tính liên tục (Continuous Coupling Frontier): 0.5*|delta_norm| + 0.5*|CLV| >= 0.50
        let effective_min_score = range_cfg.min_activation_score.max(0.50);
        let kama_slope = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
        let max_adverse = range_cfg.max_adverse_slope.max(0.01);

        // 🛡️ CHỐT CHẶN 5: Bộ Đếm Chu Kỳ Sống Của Hộp (Boundary Touch Counter: Chỉ Đánh Lần 2, 3, 4; Cấm Lần 5+)
        // & Anti-Breakout Directional Volume Veto (TODO-AUDIT-NEW-197)
        let zone_lower = val + 0.15 * box_height;
        let reset_lower = val + 0.30 * box_height;
        let zone_upper = vah - 0.15 * box_height;
        let reset_upper = vah - 0.30 * box_height;

        let vols = tick.volumes();
        let mut sum_vol = 0.0;
        let mut count_vol = 0usize;
        for idx in (i - lookback)..i {
            let v = vols[idx];
            if !v.is_nan() && v > 0.0 {
                sum_vol += v;
                count_vol += 1;
            }
        }
        let avg_box_vol = if count_vol > 0 { sum_vol / (count_vol as f64) } else { 1.0 };
        let start_breakout_idx = i.saturating_sub(2);
        let closes = tick.closes();

        // 4. Định tuyến tín hiệu Long/Short
        if delta_norm <= -0.50 && close >= val && clv >= 0.35 {
            // 🛡️ BỘ ĐẾM CHU KỲ SỐNG: Đếm chu kỳ chạm đáy VAL bằng Schmitt Trigger Hysteresis 15%/30%
            let mut lower_touches = 0usize;
            let mut in_lower_zone = false;
            for idx in (i - lookback)..=i {
                let l = lows[idx];
                if !l.is_nan() && l <= zone_lower {
                    if !in_lower_zone {
                        lower_touches += 1;
                        in_lower_zone = true;
                    }
                } else if !l.is_nan() && l > reset_lower {
                    in_lower_zone = false;
                }
            }

            // Khóa hoàn toàn giao dịch nảy biên khi hộp đã lão hóa (chạm từ lần thứ 5 trở đi)
            if lower_touches > 4 {
                return None;
            }

            // 🛡️ ANTI-BREAKOUT DIRECTIONAL VOLUME VETO: Quét cụm 3 nến gần nhất (i-2..=i)
            // Ngăn chặn cản đầu xe tăng khi có nến xả thủng đáy kèm Volume đột biến
            for idx in start_breakout_idx..=i {
                let vol_z = sensors.vol_z_score.get(idx).copied().unwrap_or(0.0);
                let vol_ratio = if avg_box_vol > 0.0 { vols[idx] / avg_box_vol } else { 0.0 };
                let is_vol_spike = vol_z >= 2.0 || vol_ratio >= 2.5;

                if is_vol_spike {
                    let rng = (highs[idx] - lows[idx]).max(1e-5);
                    let clv_idx = ((2.0 * closes[idx] - highs[idx] - lows[idx]) / rng).clamp(-1.0, 1.0);
                    let is_downward_breakdown = (clv_idx <= -0.70 && closes[idx] <= reset_lower) || closes[idx] < val;
                    if is_downward_breakdown {
                        return None;
                    }
                }
            }

            // 🛡️ CHỐNG BẮT DAO RƠI: Cấm Long khi KAMA slope dốc xuống mạnh
            if kama_slope < -max_adverse {
                return None;
            }

            let raw_score = (0.5 * delta_norm.abs() + 0.5 * clv).clamp(0.0, 1.0);
            if raw_score < effective_min_score {
                return None;
            }

            let raw_sl = low - buffer;
            let fallback_buf = buffer.max(current_atr * 0.20);
            let sl = RiskManager::sanitize_structural_sl(true, close, raw_sl, 0.0015, fallback_buf);
            let tp = vah; // Chốt lời sang biên đối diện hộp (Value Area High)

            // 🛡️ BỘ LỌC KỲ VỌNG TOÁN HỌC: Bắt buộc R:R >= 1.50 (Tự động lọc nến quét râu quá sâu / dao rơi)
            let reward = (tp - close).max(0.0);
            let risk = (close - sl).max(1e-5);
            if reward / risk < 1.50 {
                return None;
            }

            return Some(Proposal {
                signal_type: SignalType::Long,
                raw_score,
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: Some(tp),
            });
        } else if delta_norm >= 0.50 && close <= vah && clv <= -0.35 {
            // 🛡️ BỘ ĐẾM CHU KỲ SỐNG: Đếm chu kỳ chạm đỉnh VAH bằng Schmitt Trigger Hysteresis 15%/30%
            let mut upper_touches = 0usize;
            let mut in_upper_zone = false;
            for idx in (i - lookback)..=i {
                let h = highs[idx];
                if !h.is_nan() && h >= zone_upper {
                    if !in_upper_zone {
                        upper_touches += 1;
                        in_upper_zone = true;
                    }
                } else if !h.is_nan() && h < reset_upper {
                    in_upper_zone = false;
                }
            }

            // Khóa hoàn toàn giao dịch nảy biên khi hộp đã lão hóa (chạm từ lần thứ 5 trở đi)
            if upper_touches > 4 {
                return None;
            }

            // 🛡️ ANTI-BREAKOUT DIRECTIONAL VOLUME VETO: Quét cụm 3 nến gần nhất (i-2..=i)
            // Ngăn chặn cản đầu tên lửa khi có nến mua bứt phá đỉnh kèm Volume đột biến
            for idx in start_breakout_idx..=i {
                let vol_z = sensors.vol_z_score.get(idx).copied().unwrap_or(0.0);
                let vol_ratio = if avg_box_vol > 0.0 { vols[idx] / avg_box_vol } else { 0.0 };
                let is_vol_spike = vol_z >= 2.0 || vol_ratio >= 2.5;

                if is_vol_spike {
                    let rng = (highs[idx] - lows[idx]).max(1e-5);
                    let clv_idx = ((2.0 * closes[idx] - highs[idx] - lows[idx]) / rng).clamp(-1.0, 1.0);
                    let is_upward_breakout = (clv_idx >= 0.70 && closes[idx] >= reset_upper) || closes[idx] > vah;
                    if is_upward_breakout {
                        return None;
                    }
                }
            }

            // 🛡️ CHỐNG CẢN TÀU: Cấm Short khi KAMA slope dốc lên mạnh
            if kama_slope > max_adverse {
                return None;
            }

            let raw_score = (0.5 * delta_norm.abs() + 0.5 * (-clv)).clamp(0.0, 1.0);
            if raw_score < effective_min_score {
                return None;
            }

            let raw_sl = high + buffer;
            let fallback_buf = buffer.max(current_atr * 0.20);
            let sl = RiskManager::sanitize_structural_sl(false, close, raw_sl, 0.0015, fallback_buf);
            let tp = val; // Chốt lời sang biên đối diện hộp (Value Area Low)

            // 🛡️ BỘ LỌC KỲ VỌNG TOÁN HỌC: Bắt buộc R:R >= 1.50 (Tự động lọc nến quét râu quá sâu / tên lửa)
            let reward = (close - tp).max(0.0);
            let risk = (sl - close).max(1e-5);
            if reward / risk < 1.50 {
                return None;
            }

            return Some(Proposal {
                signal_type: SignalType::Short,
                raw_score,
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: Some(tp),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use ferro_trade::data::{DataView, RawDataCache};
    use ferro_trade::backtest::IndicatorManager;
    use ferro_trade::broker::position::Position;
    use ferro_trade::broker::actions::Action;
    use ferro_trade::broker::trade::Trade;
    use smallvec::SmallVec;

    fn setup_mock_tick(
        opens: Vec<f64>,
        highs: Vec<f64>,
        lows: Vec<f64>,
        closes: Vec<f64>,
        _i: usize,
    ) -> (DataView, IndicatorManager, SmallVec<[Action; 4]>, Vec<Trade>) {
        let len = opens.len();
        let cache = RawDataCache {
            datetime: Arc::new(vec![0i64; len]),
            open: Arc::new(opens),
            high: Arc::new(highs),
            low: Arc::new(lows),
            close: Arc::new(closes),
            volume: Arc::new(vec![100.0; len]),
            custom_columns: std::collections::HashMap::new(),
        };
        let dv = DataView::new(cache);
        let im = IndicatorManager::new();
        let act = SmallVec::new();
        let tr = Vec::new();
        (dv, im, act, tr)
    }

    #[test]
    fn test_range_bound_approves_rebound_at_val() {
        let len = 50;
        let i = len - 1;
        // Hộp cấu trúc: VAH = 2100.0, VAL = 2000.0, POC = 2050.0 (box_height = 100.0 >= 2.0 * atr)
        let mut opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        
        // Thiết lập cực trị trong cửa sổ 40 nến:
        highs[10] = 2100.0; // VAH
        lows[15] = 2000.0;  // VAL

        // Nến hiện tại: nảy tại sàn VAL (Close = 2012 -> delta_norm = (2012 - 2050)/50 = -0.76 <= -0.70)
        // Lực nến CLV: High = 2015, Low = 2000, Close = 2012 -> CLV = (4024 - 2015 - 2000) / 15 = 9 / 15 = +0.60 >= 0.35
        opens[i] = 2002.0;
        highs[i] = 2015.0;
        lows[i] = 2000.0;
        closes[i] = 2012.0;

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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]); // box_height 100.0 >= 2 * 10.0

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]); // Phẳng

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeBound PHẢI duyệt Proposal Long tại sàn VAL khi có lực nảy CLV");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert_eq!(p.suggested_tp, Some(2100.0), "TP Long phải hướng về biên trên VAH");
    }

    #[test]
    fn test_range_bound_approves_rebound_at_vah() {
        let len = 50;
        let i = len - 1;
        let mut opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        highs[10] = 2100.0; // VAH
        lows[15] = 2000.0;  // VAL

        // Nến hiện tại: nảy tại trần VAH (Close = 2088 -> delta_norm = (2088 - 2050)/50 = +0.76 >= 0.70)
        // Lực nến CLV: High = 2100, Low = 2085, Close = 2088 -> CLV = (4176 - 2100 - 2085) / 15 = -9 / 15 = -0.60 <= -0.35
        opens[i] = 2098.0;
        highs[i] = 2100.0;
        lows[i] = 2085.0;
        closes[i] = 2088.0;

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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeBound PHẢI duyệt Proposal Short tại trần VAH khi có lực đè CLV");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Short);
        assert_eq!(p.suggested_tp, Some(2000.0), "TP Short phải hướng về biên dưới VAL");
    }

    #[test]
    fn test_range_bound_vetoes_dead_zone_near_poc() {
        let len = 50;
        let i = len - 1;
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        highs[10] = 2100.0; // VAH
        lows[15] = 2000.0;  // VAL

        // Giá lơ lửng tại 2045 (delta_norm = (2045 - 2050)/50 = -0.10, |delta_norm| < 0.50)
        closes[i] = 2045.0;

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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let matrix = MarketMatrixV3::default();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "RangeBound PHẢI VETO khi giá lơ lửng trong Dead Zone 50% quanh POC!");
    }

    #[test]
    fn test_range_bound_vetoes_sub_frictional_micro_box() {
        let len = 50;
        let i = len - 1;
        // Hộp quá hẹp: VAH = 2010, VAL = 2000 -> box_height = 10.0 < 2.0 * ATR (20.0)
        let opens = vec![2005.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![2000.0; len];
        let mut closes = vec![2005.0; len];
        highs[10] = 2010.0;
        lows[15] = 2000.0;

        closes[i] = 2002.0;

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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]); // 2 * ATR = 20.0 > box_height (10.0)
        let matrix = MarketMatrixV3::default();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "RangeBound PHẢI VETO khi chiều cao hộp nhỏ hơn sàn chi phí 2.0 ATR!");
    }

    #[test]
    fn test_range_bound_spring_liquidity_sweep_reentry() {
        let len = 50;
        let i = len - 1;
        // Hộp: VAH = 2100, VAL = 2000, POC = 2050 (BoxHeight = 100 >= 2 * ATR)
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        highs[10] = 2100.0;
        lows[15] = 2000.0;

        // Kịch bản 1: Quét thanh khoản dưới VAL (Low = 1992 < 2000) rồi rút chân mạnh vào hộp
        // Close = 2022 -> delta_norm = -0.56 (nằm trong [-0.70, -0.50]), CLV = +0.714, R:R = 2.36 >= 1.50
        highs[i] = 2027.0;
        lows[i] = 1992.0;
        closes[i] = 2022.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeBound PHẢI duyệt cú quét thanh khoản rút chân mạnh vào vùng [-0.70, -0.50]!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert_eq!(p.suggested_tp, Some(2100.0));
        assert!(p.suggested_sl.unwrap() < 1992.0, "SL phải nằm dưới đáy râu quét 1992.0");

        // Kịch bản 2: Nến đâm thủng đóng ngoài hộp (Close = 1998 < VAL 2000) -> VETO!
        closes[i] = 1998.0;
        let (dv_brk, im_brk, mut act_brk, tr_brk) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
        let tick_brk = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_brk,
            indicator_manager: &im_brk,
            trades: &tr_brk,
            last_closed_trade: None,
            actions: &mut act_brk,
            position: Position::default(),
        };
        assert!(RangeOscScanner.scan(&tick_brk, &sensors, &matrix, &config).is_none(), "Phải VETO khi nến đóng ngoài hộp (Close < VAL)!");

        // Kịch bản 3: Râu quét quá sâu (Low = 1940) làm R:R < 1.50 -> VETO!
        lows[i] = 1940.0;
        closes[i] = 2022.0;
        let (dv_deep, im_deep, mut act_deep, tr_deep) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick_deep = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_deep,
            indicator_manager: &im_deep,
            trades: &tr_deep,
            last_closed_trade: None,
            actions: &mut act_deep,
            position: Position::default(),
        };
        assert!(RangeOscScanner.scan(&tick_deep, &sensors, &matrix, &config).is_none(), "Phải VETO khi râu quét quá sâu làm R:R < 1.50!");
    }

    #[test]
    fn test_range_bound_upthrust_reentry() {
        let len = 50;
        let i = len - 1;
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        highs[10] = 2100.0;
        lows[15] = 2000.0;

        // Kịch bản 1: Quét thanh khoản trên VAH (High = 2108 > 2100) rồi đóng cửa rút chân vào hộp
        // Close = 2078 -> delta_norm = +0.56, CLV = -0.714, R:R = 2.36 >= 1.50
        highs[i] = 2108.0;
        lows[i] = 2073.0;
        closes[i] = 2078.0;

        let (dv, im, mut act, tr) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
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
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeBound PHẢI duyệt cú quét thanh khoản trần VAH rút chân vào vùng [0.50, 0.70]!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Short);
        assert_eq!(p.suggested_tp, Some(2000.0));
        assert!(p.suggested_sl.unwrap() > 2108.0, "SL phải nằm trên đỉnh râu quét 2108.0");

        // Kịch bản 2: Nến đóng ngoài hộp trên VAH (Close = 2102 > 2100) -> VETO!
        closes[i] = 2102.0;
        let (dv_brk, im_brk, mut act_brk, tr_brk) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick_brk = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_brk,
            indicator_manager: &im_brk,
            trades: &tr_brk,
            last_closed_trade: None,
            actions: &mut act_brk,
            position: Position::default(),
        };
        assert!(RangeOscScanner.scan(&tick_brk, &sensors, &matrix, &config).is_none(), "Phải VETO khi nến đóng vượt ngoài đỉnh VAH!");
    }

    #[test]
    fn test_range_bound_targets_opposite_boundary_rr_ge_1_5() {
        let len = 50;
        let i = len - 1;
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        // Thiết lập hộp cấu trúc 40 nến: VAL = 2000.0, VAH = 2100.0, POC = 2050.0
        highs[10] = 2100.0;
        lows[15] = 2000.0;

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        // KỊCH BẢN 1 (LONG): Chạm biên dưới VAL nảy lên -> TP phải là VAH (2100.0) và R:R >= 1.50
        // Close = 2017 -> delta_norm = -0.66, CLV = +0.60
        lows[i] = 2005.0;
        highs[i] = 2020.0;
        closes[i] = 2017.0;
        let (dv_l, im_l, mut act_l, tr_l) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
        let tick_l = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_l,
            indicator_manager: &im_l,
            trades: &tr_l,
            last_closed_trade: None,
            actions: &mut act_l,
            position: Position::default(),
        };
        let prop_l = RangeOscScanner.scan(&tick_l, &sensors, &matrix, &config);
        assert!(prop_l.is_some(), "Long hợp lệ phải được duyệt");
        let p_l = prop_l.unwrap();
        assert_eq!(p_l.signal_type, SignalType::Long);
        assert_eq!(p_l.suggested_tp, Some(2100.0), "TP của Long PHẢI là biên trên VAH (2100.0)!");
        let sl_l = p_l.suggested_sl.unwrap();
        let rr_l = (2100.0 - 2017.0) / (2017.0 - sl_l);
        assert!(rr_l >= 1.50, "R:R của Long ({:.2}) phải >= 1.50", rr_l);

        // KỊCH BẢN 2 (SHORT): Chạm biên trên VAH dội xuống -> TP phải là VAL (2000.0) và R:R >= 1.50
        // Close = 2083 -> delta_norm = +0.66, CLV = -0.60
        lows[i] = 2080.0;
        highs[i] = 2095.0;
        closes[i] = 2083.0;
        let (dv_s, im_s, mut act_s, tr_s) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
        let tick_s = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_s,
            indicator_manager: &im_s,
            trades: &tr_s,
            last_closed_trade: None,
            actions: &mut act_s,
            position: Position::default(),
        };
        let prop_s = RangeOscScanner.scan(&tick_s, &sensors, &matrix, &config);
        assert!(prop_s.is_some(), "Short hợp lệ phải được duyệt");
        let p_s = prop_s.unwrap();
        assert_eq!(p_s.signal_type, SignalType::Short);
        assert_eq!(p_s.suggested_tp, Some(2000.0), "TP của Short PHẢI là biên dưới VAL (2000.0)!");
        let sl_s = p_s.suggested_sl.unwrap();
        let rr_s = (2083.0 - 2000.0) / (sl_s - 2083.0);
        assert!(rr_s >= 1.50, "R:R của Short ({:.2}) phải >= 1.50", rr_s);

        // KỊCH BẢN 3: R:R < 1.50 do SL quá xa (Low = 1965) -> PHẢI BỊ VETO!
        lows[i] = 1965.0;
        highs[i] = 2026.0;
        closes[i] = 2024.0;
        let (dv_bad, im_bad, mut act_bad, tr_bad) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick_bad = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_bad,
            indicator_manager: &im_bad,
            trades: &tr_bad,
            last_closed_trade: None,
            actions: &mut act_bad,
            position: Position::default(),
        };
        assert!(RangeOscScanner.scan(&tick_bad, &sensors, &matrix, &config).is_none(), "R:R < 1.50 bắt buộc phải bị VETO!");
    }

    #[test]
    fn test_range_bound_touch_counter_locks_after_4_touches() {
        let len = 50;
        let i = len - 1;
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        // Hộp: VAH = 2100.0, VAL = 2000.0, POC = 2050.0, box_height = 100.0 >= 2.0 * ATR
        // Lưu ý: Cửa sổ 40 nến tính từ (i - 40) = 9 đến i = 49. Đặt cực trị tại idx >= 10.
        highs[10] = 2100.0;
        lows[10] = 2000.0; // Chạm lần 1

        // Tạo thêm 3 chu kỳ chạm - nảy nữa tại các nến 15, 20, 25 (Tổng cộng đã chạm 4 lần trong quá khứ)
        // Zone threshold = 2000 + 0.15 * 100 = 2015; Reset threshold = 2000 + 0.30 * 100 = 2030
        // (lows mặc định 2040 > 2030 -> đã reset giữa các lần)
        lows[15] = 2005.0; // Chạm lần 2
        lows[20] = 2008.0; // Chạm lần 3
        lows[25] = 2010.0; // Chạm lần 4

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        // KỊCH BẢN A: Nến hiện tại i chạm đáy lần thứ 5 -> PHẢI BỊ KHÓA (VETO)!
        lows[i] = 2002.0;
        highs[i] = 2015.0;
        closes[i] = 2012.0;
        let (dv_5, im_5, mut act_5, tr_5) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
        let tick_5 = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_5,
            indicator_manager: &im_5,
            trades: &tr_5,
            last_closed_trade: None,
            actions: &mut act_5,
            position: Position::default(),
        };
        let prop_5 = RangeOscScanner.scan(&tick_5, &sensors, &matrix, &config);
        assert!(prop_5.is_none(), "RangeBound PHẢI VETO khi chạm biên đáy lần thứ 5 (hộp đã lão hóa)!");

        // KỊCH BẢN B: Xóa bớt 1 lần chạm (chỉ còn 3 lần quá khứ + nến i là lần thứ 4) -> PHẢI ĐƯỢC DUYỆT!
        lows[25] = 2040.0; // Hủy lần chạm 4 quá khứ
        let (dv_4, im_4, mut act_4, tr_4) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick_4 = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_4,
            indicator_manager: &im_4,
            trades: &tr_4,
            last_closed_trade: None,
            actions: &mut act_4,
            position: Position::default(),
        };
        let prop_4 = RangeOscScanner.scan(&tick_4, &sensors, &matrix, &config);
        assert!(prop_4.is_some(), "RangeBound PHẢI DUYỆT khi chạm biên đáy lần thứ 4 (hộp còn trong vòng đời)!");
        assert_eq!(prop_4.unwrap().signal_type, SignalType::Long);
    }

    #[test]
    fn test_range_bound_breakout_volume_invalidates_range() {
        let len = 50;
        let i = len - 1;
        let opens = vec![2050.0; len];
        let mut highs = vec![2060.0; len];
        let mut lows = vec![2040.0; len];
        let mut closes = vec![2050.0; len];
        highs[10] = 2100.0;
        lows[15] = 2000.0;

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.atr_sl = Arc::new(vec![10.0; len]);
        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = Arc::new(vec![0.0; len]);

        // KỊCH BẢN A: Cây nến ngay trước đó (i - 1) nổ Volume xả thủng đáy (Breakdown)
        // Nến i hồi nhẹ lờ đờ tại sàn VAL -> PHẢI BỊ VETO (chống bẫy Retest sau Breakout)!
        let mut vol_z = vec![0.0; len];
        vol_z[i - 1] = 2.8; // Nổ Volume Z-Score >= 2.0
        sensors.vol_z_score = Arc::new(vol_z);

        // Nến i - 1: Xả đâm thủng VAL (Close = 1986, High = 2010, Low = 1985 -> CLV = -0.92 <= -0.70)
        highs[i - 1] = 2010.0;
        lows[i - 1] = 1985.0;
        closes[i - 1] = 1986.0;

        // Nến i: Nhịp nảy nhẹ (Close = 2012, CLV = +0.60)
        lows[i] = 2000.0;
        highs[i] = 2015.0;
        closes[i] = 2012.0;

        let (dv_retest, im_retest, mut act_retest, tr_retest) = setup_mock_tick(opens.clone(), highs.clone(), lows.clone(), closes.clone(), i);
        let tick_retest = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_retest,
            indicator_manager: &im_retest,
            trades: &tr_retest,
            last_closed_trade: None,
            actions: &mut act_retest,
            position: Position::default(),
        };
        let prop_retest = RangeOscScanner.scan(&tick_retest, &sensors, &matrix, &config);
        assert!(prop_retest.is_none(), "RangeBound PHẢI VETO nhịp nảy nhẹ sau khi cây nến trước đó vừa xả thủng đáy với Volume lớn!");

        // KỊCH BẢN B: Nến i nổ Volume lớn nhưng là cú rút chân Spring (Close >= VAL, CLV >= +0.35)
        // -> BẢO TOÀN SPRING: PHẢI ĐƯỢC DUYỆT!
        let mut vol_z_spring = vec![0.0; len];
        vol_z_spring[i] = 3.2; // Volume cực lớn do cá mập gom hàng tại đáy
        sensors.vol_z_score = Arc::new(vol_z_spring);

        // Nến i - 1 bình thường
        highs[i - 1] = 2060.0;
        lows[i - 1] = 2040.0;
        closes[i - 1] = 2050.0;

        // Nến i là nến Spring: quét râu xuống 1995 rồi rút chân đóng 2012 bên trong hộp (CLV = +0.70)
        lows[i] = 1995.0;
        highs[i] = 2015.0;
        closes[i] = 2012.0;

        let (dv_spring, im_spring, mut act_spring, tr_spring) = setup_mock_tick(opens, highs, lows, closes, i);
        let tick_spring = TickContext {
            i,
            equity: 1000.0,
            data_view: &dv_spring,
            indicator_manager: &im_spring,
            trades: &tr_spring,
            last_closed_trade: None,
            actions: &mut act_spring,
            position: Position::default(),
        };
        let prop_spring = RangeOscScanner.scan(&tick_spring, &sensors, &matrix, &config);
        assert!(prop_spring.is_some(), "RangeBound PHẢI DUYỆT cú Spring rút chân nổ Volume gom hàng (Bảo toàn Task 195)!");
        assert_eq!(prop_spring.unwrap().signal_type, SignalType::Long);
    }
}
