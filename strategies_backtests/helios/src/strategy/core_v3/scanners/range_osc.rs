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
        let min_score = range_cfg.min_activation_score.max(0.05);
        let kama_slope = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
        let max_adverse = range_cfg.max_adverse_slope.max(0.01);

        // 4. Định tuyến tín hiệu Long/Short
        if delta_norm <= -0.70 && clv >= 0.35 {
            // 🛡️ CHỐNG BẮT DAO RƠI: Cấm Long khi KAMA slope dốc xuống mạnh
            if kama_slope < -max_adverse {
                return None;
            }

            let raw_score = (0.5 * delta_norm.abs() + 0.5 * clv).clamp(0.0, 1.0);
            if raw_score < min_score {
                return None;
            }

            let raw_sl = low - buffer;
            let fallback_buf = buffer.max(current_atr * 0.20);
            let sl = RiskManager::sanitize_structural_sl(true, close, raw_sl, 0.0015, fallback_buf);
            let tp = vah; // Chốt lời sang biên đối diện hộp (Value Area High)

            return Some(Proposal {
                signal_type: SignalType::Long,
                raw_score,
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: Some(tp),
            });
        } else if delta_norm >= 0.70 && clv <= -0.35 {
            // 🛡️ CHỐNG CẢN TÀU: Cấm Short khi KAMA slope dốc lên mạnh
            if kama_slope > max_adverse {
                return None;
            }

            let raw_score = (0.5 * delta_norm.abs() + 0.5 * (-clv)).clamp(0.0, 1.0);
            if raw_score < min_score {
                return None;
            }

            let raw_sl = high + buffer;
            let fallback_buf = buffer.max(current_atr * 0.20);
            let sl = RiskManager::sanitize_structural_sl(false, close, raw_sl, 0.0015, fallback_buf);
            let tp = val; // Chốt lời sang biên đối diện hộp (Value Area Low)

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
}
