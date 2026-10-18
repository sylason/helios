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
//!   + `vol_ratio`: Đo lường tỷ lệ khối lượng thực tế so với SMA khối lượng 20 phiên.
//!   + `mfi`: Đo lường sự kiệt sức của dòng tiền (quá bán / quá mua).
//!   + `clv`: Close Location Value đo lường lực rút chân/đè đầu của nến đảo chiều.
//!   + `atr_sl`: Chuẩn hóa khoảng cách ly giác (Displacement) và đệm Stop Loss.
//! - **Chốt chặn Bảo vệ Cứng (Hard Gates):**
//!   + Remaining Displacement Guard: abs(close - kama_mid) >= min_reversal_displacement * current_atr.
//!   + CLV Guard (Anti-Falling Knife): Long yêu cầu CLV >= min_clv; Short yêu cầu CLV <= -min_clv.
//!   + MFI Exhaustion Filter: Long yêu cầu MFI <= mfi_thresh; Short yêu cầu MFI >= 100 - mfi_thresh.
//! - **Cơ chế Chấm Điểm 3 Thành Phần (Convex Normalization):**
//!   Score = norm_w_z * f_z + norm_w_disp * f_disp + norm_w_vol * f_vol
//! - **Thiết lập Vị thế:**
//!   + Long: Khi z_score < 0.0, close < kama_mid, CLV >= min_clv, MFI <= mfi_thresh. SL: low - wick_buffer, TP: Some(kama_mid).
//!   + Short: Khi z_score > 0.0, close > kama_mid, CLV <= -min_clv, MFI >= 100 - mfi_thresh. SL: high + wick_buffer, TP: Some(kama_mid).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::normalization::{sat_unbounded, convex_combine};

pub struct ReversalScanner;

impl TacticalScanner for ReversalScanner {
    fn id(&self) -> ScannerId {
        ScannerId::ClimaxReversal
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(ReversalScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, _matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let rev_cfg = &config.tactical_sniper.reversal;
        
        let z_score = sensors.z_score[i];
        let kama_mid = sensors.kama_mid[i];
        let current_atr = sensors.atr_sl[i];
        let high_price = tick.highs()[i];
        let low_price = tick.lows()[i];
        let close_price = tick.closes()[i];
        let vol_current = tick.volumes()[i];
        let vol_sma_raw = sensors.vol_sma.get(i).copied().unwrap_or(vol_current);
        let vol_sma = if vol_sma_raw.is_nan() || vol_sma_raw <= 0.0 { vol_current } else { vol_sma_raw };
        let mfi = sensors.mfi.get(i).copied().unwrap_or(50.0);

        // 🛡️ Task REV-01: Vá Lỗ Hổng Bảo Vệ NaN & Dữ liệu không hợp lệ
        if z_score.is_nan() || kama_mid.is_nan() || current_atr.is_nan() || current_atr <= 0.0 || mfi.is_nan() {
            return None;
        }

        // 🛡️ Task REV-05: Chốt Chặn Khoảng Cách Hồi Quy Tối Thiểu (Remaining Displacement Guard)
        // Giá nến đóng cửa phải còn cách KAMA Midline một khoảng tối thiểu để đảm bảo tiềm năng sóng hồi
        let displacement = (close_price - kama_mid).abs();
        let min_disp = if rev_cfg.min_reversal_displacement <= 0.0 { 0.50 } else { rev_cfg.min_reversal_displacement.max(0.1) };
        let min_disp_dist = min_disp * current_atr;
        if displacement < min_disp_dist {
            return None;
        }

        // 🛡️ Task REV-03: Chốt Chặn Nến Xác Nhận Đảo Chiều (Anti-Falling Knife CLV Guard)
        // Tính Close Location Value phi thứ nguyên [-1.0, +1.0]
        let candle_range = (high_price - low_price).max(1e-5);
        let clv = (2.0 * close_price - high_price - low_price) / candle_range;
        let min_clv = if rev_cfg.min_reversal_clv <= 0.0 { 0.20 } else { rev_cfg.min_reversal_clv.clamp(0.05, 0.90) };

        // 🛡️ Task REV-04: Bộ Lọc Dòng Tiền Kiệt Sức MFI (Money Flow Exhaustion Filter)
        let mfi_thresh = if rev_cfg.mfi_reversal_thresh <= 0.0 { 30.0 } else { rev_cfg.mfi_reversal_thresh.clamp(10.0, 50.0) };

        // Xác định hướng tín hiệu và kiểm tra đồng thời CLV + MFI
        let signal_type = if z_score < 0.0 && close_price < kama_mid {
            // Long Reversal: Cần nến rút chân (CLV >= min_clv) và dòng tiền bán kiệt sức (MFI <= mfi_thresh)
            if clv < min_clv || mfi > mfi_thresh {
                return None;
            }
            SignalType::Long
        } else if z_score > 0.0 && close_price > kama_mid {
            // Short Reversal: Cần nến đè đầu (CLV <= -min_clv) và dòng tiền mua quá tải (MFI >= 100 - mfi_thresh)
            if clv > -min_clv || mfi < (100.0 - mfi_thresh) {
                return None;
            }
            SignalType::Short
        } else {
            return None;
        };

        // 📊 Task REV-04, REV-07 & TODO-AUDIT-H-01: Chấm Điểm Mềm 3 Thành Phần với Trọng Số Lồi (Convex Normalization)
        let z_thresh = if rev_cfg.z_score_threshold <= 0.0 { 2.0 } else { rev_cfg.z_score_threshold.max(0.1) };
        let vol_thresh = if rev_cfg.climax_vol_threshold <= 0.0 { 1.50 } else { rev_cfg.climax_vol_threshold.max(0.50) };

        let f_z = sat_unbounded(z_score.abs(), z_thresh);
        let f_disp = sat_unbounded(displacement, min_disp_dist);
        let vol_ratio = if vol_sma > 0.0 { vol_current / vol_sma.max(1e-5) } else { 1.0 };
        let f_vol = sat_unbounded(vol_ratio, vol_thresh);

        // Trọng số lồi (Convex Weights)
        let w_z = if rev_cfg.w_z_score <= 0.0 { 0.40 } else { rev_cfg.w_z_score.clamp(0.10, 0.80) };
        let w_disp = if rev_cfg.w_displacement <= 0.0 { 0.40 } else { rev_cfg.w_displacement.clamp(0.10, 0.80) };
        let w_vol = (1.0 - w_z - w_disp).max(0.05);

        let score = convex_combine(&[f_z, f_disp, f_vol], &[w_z, w_disp, w_vol]);
        let min_score = rev_cfg.min_activation_score;

        // Kiểm tra an toàn kép cho điểm số
        if score.is_nan() || score < min_score {
            return None;
        }

        let wick_buffer_mult = if rev_cfg.wick_sl_buffer_atr <= 0.0 { 0.50 } else { rev_cfg.wick_sl_buffer_atr.max(0.10) };
        let wick_buffer = current_atr * wick_buffer_mult;
        let suggested_tp = Some(kama_mid);

        match signal_type {
            SignalType::Long => Some(Proposal {
                signal_type: SignalType::Long,
                raw_score: score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(low_price - wick_buffer),
                suggested_tp,
            }),
            SignalType::Short => Some(Proposal {
                signal_type: SignalType::Short,
                raw_score: score.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(high_price + wick_buffer),
                suggested_tp,
            }),
        }
    }
}
