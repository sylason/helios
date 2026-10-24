//! # MOMENTUM IGNITION SCANNER (MOMENTUM_IGN)
//!
//! ## 1. Mục đích:
//! Bắt các cú bứt phá xung lực nến mạnh (Momentum Ignition / Volatility Breakout) khi thị trường
//! thoát khỏi vùng tích lũy với sự gia tăng đột biến của biên độ nến và khối lượng giao dịch.
//!
//! ## 2. Logic Hoạt Động Cốt Lõi:
//! - **Chỉ báo sử dụng:**
//!   + Dải Bollinger Bands (`bb_upper`, `bb_lower`): Nhận diện giá vượt biên biến động.
//!   + RSI Momentum (`rsi_momentum`): Đo đạc sức mạnh gia tốc dòng tiền ngắn hạn.
//!   + Khối lượng dòng tiền (`sensors.vol_z_score`): Xác nhận bùng nổ xung lực qua Volume Z-Score chuẩn hóa.
//!   + ATR (`atr_sl`): Chuẩn hóa khoảng cách giá và làm đệm Stop Loss.
//! - **Cơ chế Chấm Điểm (Sigmoid / Tanh Smooth Scoring):**
//!   + Long: $0.40 \times f_{\text{break\_long}} + 0.35 \times f_{\text{rsi\_long}} + 0.25 \times f_{\text{vol}}$
//!   + Short: $0.40 \times f_{\text{break\_short}} + 0.35 \times f_{\text{rsi\_short}} + 0.25 \times f_{\text{vol}}$
//! - **Thiết lập Vị thế:**
//!   + Long: Đặt `suggested_sl` dưới đáy nến bùng nổ: `Low[i] - buffer`.
//!   + Short: Đặt `suggested_sl` trên đỉnh nến bùng nổ: `High[i] + buffer`.
//!   + Take Profit: `None` (ủy quyền hoàn toàn cho động cơ Trailing Stop Loss trong `RiskManager`).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::normalization::{sat_unbounded, sat_penetration, convex_combine};
use crate::strategy::core_v3::risk::RiskManager;

pub struct MomentumIgnitionScanner;

impl TacticalScanner for MomentumIgnitionScanner {
    fn id(&self) -> ScannerId {
        ScannerId::MomentumIgnition
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(MomentumIgnitionScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let mom_cfg = &config.tactical_sniper.momentum;
        
        let bb_u = sensors.bb_upper.get(i).copied().unwrap_or(f64::NAN);
        let bb_l = sensors.bb_lower.get(i).copied().unwrap_or(f64::NAN);
        let close = tick.closes()[i];

        if bb_u.is_nan() || bb_l.is_nan() || close.is_nan() {
            return None;
        }

        // 🛡️ VẤN ĐỀ 2 (TODO-P0-04): Early Exit Guard (Khử 100% điểm ảo Sigmoid)
        // Giá bắt buộc phải thực sự vượt biên dải Bollinger Bands mới được coi là Breakout
        let is_long_candidate = close > bb_u;
        let is_short_candidate = close < bb_l;
        if !is_long_candidate && !is_short_candidate {
            return None; // Giá nằm trong dải -> Dứt khoát không có Breakout
        }

        let atr = sensors.atr_sl.get(i).copied().unwrap_or(f64::NAN);
        let rsi_mom = sensors.rsi_momentum.get(i).copied().unwrap_or(f64::NAN);
        // 🚀 TASK-MAT-01 (Occam's Scalpel): Đọc trực tiếp Volume Z-Score tinh khiết từ sensors
        let raw_vol_z = sensors.vol_z_score.get(i).copied().unwrap_or(f64::NAN);

        if rsi_mom.is_nan() || atr.is_nan() || atr <= 0.0 || raw_vol_z.is_nan() {
            return None;
        }

        let open = tick.opens()[i];
        let high = tick.highs()[i];
        let low = tick.lows()[i];

        // 🛡️ VẤN ĐỀ 1 (TODO-P0-02): Màu nến & CLV Guard (Khử 100% bẫy râu nến Fakeout)
        let candle_range = (high - low).max(1e-5);
        let clv = (2.0 * close - high - low) / candle_range;
        let min_clv = if mom_cfg.min_clv <= 0.0 { 0.20 } else { mom_cfg.min_clv.clamp(0.05, 0.90) };

        let signal_type = if is_long_candidate && close > open && clv >= min_clv {
            SignalType::Long
        } else if is_short_candidate && close < open && clv <= -min_clv {
            SignalType::Short
        } else {
            return None; // Nến rút chân / râu ngược (Shooting Star / Hammer) -> Loại bỏ bẫy fakeout
        };

        // 🛡️ VẤN ĐỀ 3 (TODO-P0-05 / TODO-STAB-04): Macro Trend Veto (Chống bắt dao rơi & cản đầu xe tăng)
        let kama_slope = matrix.kama_slope_atr.get(i).copied()
            .or_else(|| sensors.kama_slope_adj.get(i).copied())
            .unwrap_or(0.0);
        if !kama_slope.is_finite() {
            return None;
        }
        let max_adverse = mom_cfg.max_adverse_slope.clamp(0.0, 0.05);
        match signal_type {
            SignalType::Long => {
                if kama_slope < -max_adverse {
                    return None; // KAMA dốc xuống -> Cấm Long bùng nổ ngược thác lũ
                }
            }
            SignalType::Short => {
                if kama_slope > max_adverse {
                    return None; // KAMA dốc lên -> Cấm Short bùng nổ cản đầu xe tăng
                }
            }
        }

        // 🛡️ VẤN ĐỀ 4 (TODO-P0-06): Chuẩn hóa điểm số phi thứ nguyên qua Tanh & Trọng số DNA
        let vol_thresh = if mom_cfg.vol_z_threshold <= 0.0 { 1.50 } else { mom_cfg.vol_z_threshold.max(0.10) };
        let rsi_thresh = if mom_cfg.rsi_threshold <= 0.0 { 60.0 } else { mom_cfg.rsi_threshold.clamp(50.0, 90.0) };

        let f_vol = sat_unbounded(raw_vol_z, vol_thresh);
        // Đặt mục tiêu bứt phá chuẩn qua BB là 0.25 ATR
        let target_breakout = atr * 0.25;

        let (f_break, f_rsi) = match signal_type {
            SignalType::Long => {
                let dist = (close - bb_u).max(0.0);
                let f_b = sat_penetration(dist, target_breakout);
                let rsi_excess = (rsi_mom - rsi_thresh).max(0.0);
                let f_r = sat_unbounded(rsi_excess, 15.0);
                (f_b, f_r)
            }
            SignalType::Short => {
                let dist = (bb_l - close).max(0.0);
                let f_b = sat_penetration(dist, target_breakout);
                let rsi_excess = ((100.0 - rsi_thresh) - rsi_mom).max(0.0);
                let f_r = sat_unbounded(rsi_excess, 15.0);
                (f_b, f_r)
            }
        };

        // Tổ hợp lồi 3 thành phần động từ DNA
        let w_break = if mom_cfg.w_breakout <= 0.0 { 0.40 } else { mom_cfg.w_breakout.clamp(0.10, 0.80) };
        let w_vol = if mom_cfg.w_volume <= 0.0 { 0.30 } else { mom_cfg.w_volume.clamp(0.10, 0.80) };
        let w_rsi = (1.0 - w_break - w_vol).max(0.05);
        let raw_score = convex_combine(&[f_break, f_vol, f_rsi], &[w_break, w_vol, w_rsi]);

        if raw_score < mom_cfg.min_activation_score {
            return None;
        }

        // 🛡️ VẤN ĐỀ 4 (TODO-P1-02): Stop Loss cấu trúc sạch sẽ (loại bỏ hardcode 0.002 và 0.05)
        let buffer_mult = if mom_cfg.struct_sl_buffer_atr <= 0.0 { 0.50 } else { mom_cfg.struct_sl_buffer_atr.max(0.10) };
        let buffer = atr * buffer_mult;

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

        Some(Proposal {
            signal_type,
            raw_score,
            scanner_id: self.id(),
            suggested_sl: Some(sl),
            suggested_tp: None,
        })
    }
}
