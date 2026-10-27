//! # ABSORPTION SCOUT SCANNER (ABSORPTION)
//!
//! ## 1. Mục đích:
//! Nhận diện vùng nén biến động (Volatility Squeeze) nơi có sự xuất hiện của khối lượng giao dịch
//! hấp thụ lớn (Volume Absorption). Đây là vùng mà dòng tiền tổ chức âm thầm gom hàng (Accumulation)
//! hoặc phân phối (Distribution) trước khi đẩy giá bứt phá.
//!
//! ## 2. Logic Hoạt Động & Bộ Lọc Phòng Ngự 2 Tầng:
//! - **Chỉ báo sử dụng:**
//!   + `squeeze_intensity`: Cường độ nén của biến động thị trường.
//!   + `vol_z_score`: Khối lượng giao dịch gia tăng bất thường (Z-Score).
//!   + `bb_middle`, `bb_lower`, `bb_upper`: Dải Bollinger Bands bao bọc vùng nén.
//!   + `kama_slope_atr`: Độ dốc xu hướng KAMA chuẩn hóa theo ATR để tránh bẫy cản tàu hoặc bắt dao rơi.
//! - **Hệ Thống Phòng Ngự 2 Tầng:**
//!   + Tầng 1 (Price Action & Wick Clearance): Quét râu (%B_low / %B_high) kết hợp lực nến rút chân/đè đầu CLV.
//!     - Long: Râu nến quét sâu vào vùng đáy tích lũy (%B_low <= max_bound) và CLV >= min_clv, close <= bb_middle.
//!     - Short: Râu nến quét cao vào vùng đỉnh phân phối (%B_high >= 1 - max_bound) và CLV <= -min_clv, close >= bb_middle.
//!   + Tầng 2 (Macro Trend Alignment): VETO chặn bắt dao rơi khi `kama_slope_atr` dốc ngược chiều vượt ngưỡng `max_adverse_slope`.
//! - **Thiết lập Vị thế:**
//!   + Long: SL neo dưới đáy nến Spring vi mô: `low - buffer`.
//!   + Short: SL neo trên đỉnh nến Upthrust vi mô: `high + buffer`.
//!   + Take Profit: `None` (để Trailing SL tự động bảo toàn lợi nhuận).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::normalization::{sat_unbounded, sat_ratio_damped, convex_combine};
use crate::strategy::core_v3::risk::RiskManager;

pub struct AbsorptionScanner;

impl TacticalScanner for AbsorptionScanner {
    fn id(&self) -> ScannerId {
        ScannerId::AbsorptionScout
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(AbsorptionScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        if i < 1 {
            return None;
        }
        let abs_cfg = &config.tactical_sniper.absorption;
        
        let raw_vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(0.0);
        let squeeze = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);
        let bb_middle = sensors.bb_middle.get(i).copied().unwrap_or(f64::NAN);
        let bb_lower = sensors.bb_lower.get(i).copied().unwrap_or(f64::NAN);
        let bb_upper = sensors.bb_upper.get(i).copied().unwrap_or(f64::NAN);

        if bb_middle.is_nan() || bb_lower.is_nan() || bb_upper.is_nan() || raw_vol_z.is_nan() || squeeze.is_nan() {
            return None;
        }

        let sqz_thresh = abs_cfg.squeeze_threshold.max(0.01);
        let vol_thresh = abs_cfg.vol_z_threshold.max(0.01);

        let high = tick.highs()[i];
        let low = tick.lows()[i];
        let close = tick.closes()[i];
        let candle_range = (high - low).max(1e-5);
        let raw_atr = sensors.atr_sl.get(i).copied().unwrap_or(0.0);
        if !close.is_finite() || close <= 1e-12 || !raw_atr.is_finite() || raw_atr <= 0.0 {
            return None;
        }
        let current_atr = raw_atr.max(1e-12);

        // 🛡️ CHỐT CHẶN BẤT BIẾN (TODO-AUDIT-NEW-36): Bắt buộc Volume phải vượt mức trung bình
        // Triệt tiêu 100% nguy cơ mở lệnh gom hàng khi thị trường chết thanh khoản lúc nửa đêm
        if raw_vol_z <= 0.0 {
            return None;
        }

        // 🛡️ TODO-AUDIT-NEW-176 (ABS-SUB-FRICTIONAL-FILTER): Chốt chặn hộp giá nén nhỏ hơn (1 nến ATR thực tế + 2 nửa hộp phí sàn 2 chiều)
        // Tự động co giãn theo biến động thực tế của từng tài sản/thời điểm mà không dùng số ma thuật 2.5 cứng,
        // đồng thời bảo vệ bb_width > 0.0 tuyệt đối từ đầu hàm cho phép chia %B trên mọi mệnh giá tài sản.
        let bb_width = bb_upper - bb_lower;
        let risk_base = &config.risk_management.risk_base;
        let roundtrip_friction = (2.0 * risk_base.taker_fee_rate + risk_base.slippage_allowance_pct).max(0.0);
        let min_required_width = raw_atr + 2.0 * roundtrip_friction * close;
        if bb_width <= 0.0 || bb_width < min_required_width {
            return None;
        }

        // 🛡️ TODO-AUDIT-NEW-36: Chuẩn hóa biên độ nến qua ATR (phi thứ nguyên, khử số ma thuật sàn 0.10)
        let norm_spread = candle_range / current_atr;
        let f_vol = sat_ratio_damped(raw_vol_z, norm_spread, vol_thresh);

        // 1. Chấm điểm Mềm Cường độ Nén Squeeze & Lực gom Volume qua trọng số DNA squeeze_weight
        let f_sqz = sat_unbounded(squeeze, sqz_thresh);
        let w_sqz = if abs_cfg.squeeze_weight <= 0.0 { 0.50 } else { abs_cfg.squeeze_weight.clamp(0.0, 1.0) };
        let raw_score = convex_combine(&[f_sqz, f_vol], &[w_sqz, 1.0 - w_sqz]);

        if raw_score >= abs_cfg.min_activation_score {
            // 🛡️ TODO-AUDIT-NEW-34 & NEW-35: Dynamic Quant (%B Râu nến + CLV Spring/Upthrust)
            // Khắc phục triệt để nghịch lý hình học nến và lệch pha MTF nến 1m vs BB 15m.
            // Loại bỏ hoàn toàn bộ lọc trễ pha vwma < bb_middle theo TODO-SCN-02.
            let pct_b_close = (close - bb_lower) / bb_width;
            let pct_b_low   = (low - bb_lower) / bb_width;
            let pct_b_high  = (high - bb_lower) / bb_width;
            let clv = (2.0 * close - high - low) / candle_range;

            let spring_tol = abs_cfg.spring_tolerance.max(0.0);
            let min_clv = abs_cfg.min_clv.max(0.0);
            let max_bound = if abs_cfg.max_boundary_pct_b <= 0.0 { 0.40 } else { abs_cfg.max_boundary_pct_b.clamp(0.05, 0.50) };

            // 🛡️ TODO-AUDIT-NEW-49 (ABS-SEQUENCE -> QUANT KINETIC & FLOW DYNAMICS):
            // 1. Gia tốc hãm phanh (Kinetic Deceleration): Vận tốc xu hướng KAMA phải giảm tốc (a > 0 khi Long, a < 0 khi Short)
            let curr_kama_slope = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
            let prev_kama_slope = matrix.kama_slope_atr.get(i - 1).copied().unwrap_or(0.0);
            let kama_deceleration = curr_kama_slope - prev_kama_slope;

            // 2. Dòng tiền đảo chiều (Money Flow Vector Flip): MFI phải ở vùng kiệt sức và bắt đầu quay đầu
            let curr_mfi = sensors.mfi.get(i).copied().unwrap_or(50.0);
            let prev_mfi = sensors.mfi.get(i - 1).copied().unwrap_or(curr_mfi);
            let mfi_thresh = if abs_cfg.mfi_absorption_thresh <= 0.0 { 35.0 } else { abs_cfg.mfi_absorption_thresh.clamp(10.0, 50.0) };

            // - Long (Accumulation Spring / Liquidity Sweep):
            //   + Quét râu: Râu nến (low) nhúng sâu vào vùng đáy tích lũy: pct_b_low <= max_bound
            //   + Lực nến: CLV >= min_clv (rút chân tăng chứng minh cá mập gom hấp thụ)
            //   + 🛡️ TODO-AUDIT-NEW-50 (Wall Defense Reclaim Gate): Đóng cửa phải giữ vững bên trong hoặc trên dải đáy (pct_b_close >= 0.0)
            //   + Chặn trần mua đu đỉnh: pct_b_close <= (max_bound + spring_tol) && close <= bb_middle
            //   + Động lực học: Dòng tiền MFI kiệt sức và bắt đầu tăng (curr_mfi <= mfi_thresh && curr_mfi > prev_mfi)
            //   + Cơ học hãm phanh: Gia tốc KAMA dương (kama_deceleration > 0.0)
            // - Short (Distribution Upthrust / Liquidity Sweep):
            //   + Quét râu: Râu nến (high) nhô cao vào vùng đỉnh phân phối: pct_b_high >= (1.0 - max_bound)
            //   + Lực nến: CLV <= -min_clv (đè đầu giảm chứng minh cá mập xả hàng)
            //   + 🛡️ TODO-AUDIT-NEW-50 (Wall Defense Reclaim Gate): Đóng cửa phải giữ vững bên trong hoặc dưới dải đỉnh (pct_b_close <= 1.0)
            //   + Chặn sàn bán đu đáy: pct_b_close >= (1.0 - max_bound - spring_tol) && close >= bb_middle
            //   + Động lực học: Dòng tiền MFI quá tải và bắt đầu giảm (curr_mfi >= 100 - mfi_thresh && curr_mfi < prev_mfi)
            //   + Cơ học hãm phanh: Gia tốc KAMA âm (kama_deceleration < 0.0)
            let signal_type = if pct_b_low <= max_bound 
                && clv >= min_clv 
                && pct_b_close >= 0.0 
                && pct_b_close <= (max_bound + spring_tol) 
                && close <= bb_middle 
                && curr_mfi <= mfi_thresh 
                && curr_mfi > prev_mfi 
                && kama_deceleration > 0.0
            {
                SignalType::Long
            } else if pct_b_high >= (1.0 - max_bound) 
                && clv <= -min_clv 
                && pct_b_close <= 1.0 
                && pct_b_close >= (1.0 - max_bound - spring_tol) 
                && close >= bb_middle 
                && curr_mfi >= (100.0 - mfi_thresh) 
                && curr_mfi < prev_mfi 
                && kama_deceleration < 0.0
            {
                SignalType::Short
            } else {
                return None;
            };

            // 🛡️ FLAW-REG.9 TẦNG 2: Macro Trend Direction Alignment (100% Dynamic qua DNA)
            // Ngăn chặn triệt để bẫy bắt dao rơi khi xả lũ hoặc cản tàu khi bơm dốc:
            // - Long: Nếu kama_slope < -max_adverse_slope -> VETO cấm bắt dao rơi!
            // - Short: Nếu kama_slope > max_adverse_slope -> VETO cấm cản tàu!
            let kama_slope = matrix.kama_slope_atr.get(i).copied().unwrap_or(0.0);
            let max_adverse = abs_cfg.max_adverse_slope.max(0.01);
            match signal_type {
                SignalType::Long => {
                    if kama_slope < -max_adverse {
                        return None;
                    }
                }
                SignalType::Short => {
                    if kama_slope > max_adverse {
                        return None;
                    }
                }
            }

            let buffer = current_atr * abs_cfg.struct_sl_buffer_atr.max(0.05);

            // 🛡️ TASK-SEN-02: Structural SL neo trực tiếp theo đáy nến Spring (Long) hoặc đỉnh Upthrust (Short) vi mô
            // Xóa bỏ hoàn toàn .min(bb_lower) và .max(bb_upper) để không bị kéo giãn SL theo dải BB khung 1h/4h
            let raw_sl = match signal_type {
                SignalType::Long => low - buffer,
                SignalType::Short => high + buffer,
            };
            let sl = RiskManager::sanitize_structural_sl(
                signal_type == SignalType::Long,
                close,
                raw_sl,
                0.0015,
                buffer,
            );

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_absorption_sl_spring_geometry() {
        // Giả lập nến Spring vi mô 1m:
        let low: f64 = 2500.0;
        let close: f64 = 2508.0;
        let current_atr: f64 = 3.0; // ATR vi mô 1m
        let struct_sl_buffer_atr: f64 = 1.0;
        let buffer: f64 = current_atr * struct_sl_buffer_atr; // 3.0

        // Giả sử dải Bollinger Bands khung lớn 1h (bb_lower = 2400.0) nằm rất xa
        let bb_lower_macro: f64 = 2400.0;

        // CÔNG THỨC MỚI: SL neo trực tiếp vào low của nến Spring và được chuẩn hóa qua RiskManager
        let raw_sl = low - buffer;
        let sl_new = RiskManager::sanitize_structural_sl(true, close, raw_sl, 0.0015, buffer);

        // CÔNG THỨC CŨ (BỊ BẪY BBANDS SUCTION):
        let base_support_old = low.min(bb_lower_macro); // = 2400.0
        let sl_old = base_support_old - buffer; // = 2397.0

        // Kiểm chứng:
        assert_eq!(sl_new, 2497.0, "SL mới phải neo sau đáy nến 1m $2500 - $3 = $2497");
        let loss_dist_new = close - sl_new; // 2508 - 2497 = 11 giá
        let roe_loss_20x_new = 20.0 * (loss_dist_new / close); // ~ 8.77% ROE
        assert!(roe_loss_20x_new <= 0.10, "Mức lỗ ROE mới (8.77%) phải nằm trong dải an toàn <= 10%");

        // Đối chiếu công thức cũ:
        let loss_dist_old = close - sl_old; // 2508 - 2397 = 111 giá
        let roe_loss_20x_old = 20.0 * (loss_dist_old / close); // ~ 88.5% ROE!
        assert!(roe_loss_20x_old > 0.15, "Công thức cũ bị bẫy SL phình to 88.5% ROE (chắc chắn bị Sniper Guard VETO)");
    }

    #[test]
    fn test_absorption_early_exit_on_sub_zero_volume() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2000.0; len]),
            high: std::sync::Arc::new(vec![2002.0; len]),
            low: std::sync::Arc::new(vec![1998.0; len]),
            close: std::sync::Arc::new(vec![2000.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();
        let config = HeliosConfigV3::default();

        // Volume âm (dưới trung bình)
        sensors.vol_z_score = std::sync::Arc::new(vec![-0.5; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![2.0; len]);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.80; len]);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi volume Z <= 0.0, scanner bắt buộc phải thoát sớm trả về None!");
    }

    #[test]
    fn test_absorption_raw_score_not_inflated_on_normal_volume() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        // Nến Spring tích lũy rút chân: close 1988, low 1982 (%B_low <= 0.40), high 1990
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1995.0; len];
        let mut highs = vec![2005.0; len];
        lows[i] = 1982.0;
        closes[i] = 1988.0;
        highs[i] = 1990.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1986.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();
        let config = HeliosConfigV3::default();

        // Volume Z = 1.8 (bình thường, chưa phải cá mập bùng nổ), dải BB width = 40.0
        sensors.vol_z_score = std::sync::Arc::new(vec![1.8; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]); // ATR = 10, candle_range = 8 -> norm_spread = 0.8
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.60; len]);
        let mut slope = vec![-0.05; len];
        slope[i] = -0.02; // kama_deceleration = -0.02 - (-0.05) = +0.03 > 0.0
        matrix.kama_slope_atr = std::sync::Arc::new(slope);
        let mut mfi = vec![28.0; len];
        mfi[i] = 30.0; // curr_mfi <= 35.0 && curr_mfi > prev_mfi
        sensors.mfi = std::sync::Arc::new(mfi);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_some(), "Nến Spring rút chân hợp lệ phải tạo proposal!");
        let prop = proposal.unwrap();
        // Điểm tự tin không được thổi phồng lên > 0.85
        assert!(prop.raw_score < 0.80, "Với volume bình thường 1.8 sigma, raw_score phải < 0.80, thực tế: {}", prop.raw_score);
    }

    #[test]
    fn test_absorption_rejects_sub_frictional_bandwidth() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        // Nến Spring rút chân: low 1998.6 (%B_low = 0.1/3.0 = 0.033 <= 0.40), close 1999.0, high 1999.2 (CLV = +0.33)
        lows[i] = 1998.6;
        closes[i] = 1999.0;
        highs[i] = 1999.2;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1998.8; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();
        let mut config = HeliosConfigV3::default();
        config.risk_management.risk_base.taker_fee_rate = 0.0004; // 0.04%
        config.risk_management.risk_base.slippage_allowance_pct = 0.0005; // 0.05% -> roundtrip = 0.13%
        config.tactical_sniper.absorption.min_activation_score = 0.05;
        config.tactical_sniper.absorption.min_clv = 0.20;

        sensors.vol_z_score = std::sync::Arc::new(vec![2.5; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        // Dải BB siêu hẹp: width = 3.0 (tức 0.15% giá), trong khi min_required = raw_atr (1.0) + 2 * 0.0013 * 1999.0 (5.197) = 6.197
        sensors.bb_lower = std::sync::Arc::new(vec![1998.5; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2001.5; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![1.0; len]);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.80; len]);
        let mut slope_frict = vec![-0.05; len];
        slope_frict[i] = -0.02; // kama_deceleration = +0.03 > 0.0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_frict);
        let mut mfi_frict = vec![28.0; len];
        mfi_frict[i] = 30.0; // curr_mfi <= 35.0 && curr_mfi > prev_mfi
        sensors.mfi = std::sync::Arc::new(mfi_frict);

        let scanner = AbsorptionScanner;

        // KỊCH BẢN 1: Dải BB quá hẹp (bb_width = 3.0 < 6.197) -> BẮT BUỘC TỪ CHỐI (None)!
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi dải nén nhỏ hơn (ATR + 2 * friction * close), scanner bắt buộc phải từ chối mở lệnh!");

        // KỊCH BẢN 2: Dải BB đủ rộng (bb_lower = 1990.0, bb_upper = 2010.0 -> width = 20.0 > 6.197)
        // Spring candle: low = 1992.0 (pct_b_low = 2/20 = 0.10 <= 0.40), close = 1995.0 (pct_b_close = 5/20 = 0.25 <= 0.40, close <= bb_middle 2000.0)
        // high = 1996.0 -> candle_range = 4.0, clv = (2*1995 - 1996 - 1992)/4.0 = 2/4 = +0.50 >= 0.20
        let mut lows_ok = vec![1999.0; len];
        let mut closes_ok = vec![1999.0; len];
        let mut highs_ok = vec![1999.5; len];
        lows_ok[i] = 1992.0;
        closes_ok[i] = 1995.0;
        highs_ok[i] = 1996.0;
        let raw_cache_ok = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1993.0; len]),
            high: std::sync::Arc::new(highs_ok),
            low: std::sync::Arc::new(lows_ok),
            close: std::sync::Arc::new(closes_ok),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_ok = DataView::new(raw_cache_ok);
        let tick_ok = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_ok,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        let proposal_ok = scanner.scan(&tick_ok, &sensors, &matrix, &config);
        assert!(proposal_ok.is_some(), "Khi dải BB đủ rộng, scanner phải duyệt nến Spring hợp lệ!");

        // KỊCH BẢN 3: Meme coin mệnh giá siêu nhỏ ($0.000010) khi dải BB phẳng lì (width = 0.0) -> BẢO VỆ CHỐNG CHIA 0 & TỪ CHỐI!
        let lows_meme = vec![0.000010; len];
        let closes_meme = vec![0.000010; len];
        let highs_meme = vec![0.000010; len];
        let raw_cache_meme = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![0.000010; len]),
            high: std::sync::Arc::new(highs_meme),
            low: std::sync::Arc::new(lows_meme),
            close: std::sync::Arc::new(closes_meme),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_meme = DataView::new(raw_cache_meme);
        let tick_meme = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_meme,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };
        sensors.bb_lower = std::sync::Arc::new(vec![0.000010; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![0.000010; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![0.000010; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![0.0000001; len]);
        let proposal_meme = scanner.scan(&tick_meme, &sensors, &matrix, &config);
        assert!(proposal_meme.is_none(), "Khi dải BB phẳng lì width = 0.0 trên coin mệnh giá siêu nhỏ, scanner phải từ chối an toàn!");
    }

    #[test]
    fn test_absorption_kinetic_and_mfi_flow_long_success() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        // Nến Spring rút chân: low 1992 (%B_low = 2/20 = 0.10 <= 0.40), close 1995, high 1996 (CLV = +0.50 >= 0.20)
        lows[i] = 1992.0;
        closes[i] = 1995.0;
        highs[i] = 1996.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1993.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.mfi_absorption_thresh = 35.0;

        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        // MFI kiệt sức ở 30 và quay đầu tăng lên 32
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i - 1] = 30.0;
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        // KAMA dốc xuống nhưng giảm tốc (phanh hãm dương): -0.08 -> -0.04 (a = +0.04 > 0)
        let mut slope_vec = vec![-0.08; len];
        slope_vec[i - 1] = -0.08;
        slope_vec[i] = -0.04;
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_some(), "Khi có hãm phanh dương (Delta v > 0) và MFI đảo chiều tăng từ vùng quá bán, scanner PHẢI phê duyệt Long!");
        let p = proposal.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.suggested_sl.is_some());
    }

    #[test]
    fn test_absorption_veto_when_falling_knife_accelerates() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        lows[i] = 1992.0;
        closes[i] = 1995.0;
        highs[i] = 1996.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1993.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i - 1] = 30.0;
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        // KAMA đang rơi tự do tăng tốc (dao rơi gia tốc âm): -0.04 -> -0.12 (a = -0.08 <= 0)
        let mut slope_vec = vec![-0.04; len];
        slope_vec[i - 1] = -0.04;
        slope_vec[i] = -0.12;
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi gia tốc rơi tiếp tục âm (dao rơi tăng tốc), scanner PHẢI VETO từ chối Long!");
    }

    #[test]
    fn test_absorption_veto_when_money_flow_continues_bleeding() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        lows[i] = 1992.0;
        closes[i] = 1995.0;
        highs[i] = 1996.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1993.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        // MFI vẫn đang tiếp tục sụt giảm (dòng tiền tiếp tục chảy máu): 30 -> 26 (Delta MFI <= 0)
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i - 1] = 30.0;
        mfi_vec[i] = 26.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        let mut slope_vec = vec![-0.08; len];
        slope_vec[i - 1] = -0.08;
        slope_vec[i] = -0.04;
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi MFI tiếp tục sụt giảm (dòng tiền chưa lật chiều), scanner PHẢI VETO từ chối Long!");
    }

    #[test]
    fn test_absorption_veto_when_limit_wall_shatters_breakdown() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        // Nến vỡ tường: low 1985, close 1989 < bb_lower 1990 (pct_b_close = -0.05 < 0.0), high 1991 (CLV = +0.33)
        lows[i] = 1985.0;
        closes[i] = 1989.0;
        highs[i] = 1991.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1990.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.mfi_absorption_thresh = 35.0;

        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i - 1] = 30.0;
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        let mut slope_vec = vec![-0.08; len];
        slope_vec[i - 1] = -0.08;
        slope_vec[i] = -0.04; // kama_deceleration = +0.04 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi nến đóng cửa xuyên thủng dải đáy (pct_b_close < 0.0), tường Limit vỡ vụn -> VETO cấm Long!");
    }

    #[test]
    fn test_absorption_approved_when_limit_wall_holds_spring_reclaim() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![1999.0; len];
        let mut closes = vec![1999.0; len];
        let mut highs = vec![1999.5; len];
        // Nến Spring giữ vững: low 1988 (%B_low = -0.10 <= 0.40), close 1993 (%B_close = +0.15 >= 0.0), high 1994 (CLV = +0.67)
        lows[i] = 1988.0;
        closes[i] = 1993.0;
        highs[i] = 1994.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1991.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.mfi_absorption_thresh = 35.0;

        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i - 1] = 30.0;
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        let mut slope_vec = vec![-0.08; len];
        slope_vec[i - 1] = -0.08;
        slope_vec[i] = -0.04;
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_some(), "Khi nến quét râu sâu và đóng cửa reclaim giữ vững trên sàn (pct_b_close >= 0.0), scanner PHẢI phê duyệt Long!");
        let p = proposal.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
    }

    #[test]
    fn test_absorption_short_veto_when_upper_wall_shatters_breakout() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 20;
        let i = len - 1;
        let mut lows = vec![2000.0; len];
        let mut closes = vec![2000.0; len];
        let mut highs = vec![2001.0; len];
        // Nến vỡ trần: high 2015, close 2011 > bb_upper 2010 (pct_b_close = 1.05 > 1.0), low 2009 (CLV = -0.33)
        lows[i] = 2009.0;
        closes[i] = 2011.0;
        highs[i] = 2015.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2010.0; len]),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.mfi_absorption_thresh = 35.0;

        let mut sensors = HeliosSensors::default();
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![5.0; len]);

        let mut mfi_vec = vec![70.0; len];
        mfi_vec[i - 1] = 70.0;
        mfi_vec[i] = 68.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::default();
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.70; len]);
        let mut slope_vec = vec![0.08; len];
        slope_vec[i - 1] = 0.08;
        slope_vec[i] = 0.04; // kama_deceleration = -0.04 < 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let scanner = AbsorptionScanner;
        let proposal = scanner.scan(&tick, &sensors, &matrix, &config);
        assert!(proposal.is_none(), "Khi nến đóng cửa vượt trần dải đỉnh (pct_b_close > 1.0), trần cản vỡ vụn -> VETO cấm Short!");
    }
}
