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
//!   + Khối lượng dòng tiền (`flow_force` / `volume_force`): Xác nhận có dòng tiền lớn tham gia.
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

pub struct MomentumIgnitionScanner;

impl TacticalScanner for MomentumIgnitionScanner {
    fn id(&self) -> ScannerId {
        ScannerId::MomentumIgnition
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let mom_cfg = &config.tactical_sniper.momentum;
        
        let rsi_mom = sensors.rsi_momentum[i];
        let bb_u = sensors.bb_upper[i];
        let bb_l = sensors.bb_lower[i];
        let close = tick.closes()[i];
        let atr = sensors.atr_sl[i];
        let flow_force = matrix.flow_axis.get(i).copied().unwrap_or(matrix.volume_force[i]);

        if rsi_mom.is_nan() || bb_u.is_nan() || bb_l.is_nan() || atr.is_nan() || atr <= 0.0 {
            return None;
        }

        let rsi_thresh = mom_cfg.rsi_threshold;
        let vol_thresh = mom_cfg.vol_z_threshold.max(0.01);
        let f_vol = (flow_force / vol_thresh).tanh().max(0.0).min(1.0);

        // 1. Chấm điểm Mềm Mẫu Hình LONG (BB Upper Breakout + RSI Surge + Volume Force)
        let f_break_long = 1.0 / (1.0 + (-2.0 * (close - bb_u) / atr).exp());
        let f_rsi_long = 1.0 / (1.0 + (-0.15 * (rsi_mom - rsi_thresh)).exp());
        let score_long = 0.40 * f_break_long + 0.35 * f_rsi_long + 0.25 * f_vol;

        // 2. Chấm điểm Mềm Mẫu Hình SHORT (BB Lower Breakdown + RSI Plummet + Volume Force)
        let f_break_short = 1.0 / (1.0 + (-2.0 * (bb_l - close) / atr).exp());
        let f_rsi_short = 1.0 / (1.0 + (-0.15 * ((100.0 - rsi_thresh) - rsi_mom)).exp());
        let score_short = 0.40 * f_break_short + 0.35 * f_rsi_short + 0.25 * f_vol;

        let min_score = mom_cfg.min_activation_score;

        let low = tick.lows()[i];
        let high = tick.highs()[i];
        let buffer = atr * mom_cfg.struct_sl_buffer_atr.max(0.05);

        if score_long >= min_score && score_long > score_short {
            // 🛡️ FLAW-TSL.6: Structural SL neo dưới đáy nến bùng nổ xung lực (Low[i] - buffer)
            let mut sl = low - buffer;
            if sl >= close {
                sl = close - buffer.max(close * 0.002);
            }
            Some(Proposal {
                signal_type: SignalType::Long,
                raw_score: score_long.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: None,
            })
        } else if score_short >= min_score && score_short > score_long {
            // 🛡️ FLAW-TSL.6: Structural SL neo trên đỉnh nến bùng nổ xung lực (High[i] + buffer)
            let mut sl = high + buffer;
            if sl <= close {
                sl = close + buffer.max(close * 0.002);
            }
            Some(Proposal {
                signal_type: SignalType::Short,
                raw_score: score_short.min(1.0),
                scanner_id: self.id(),
                suggested_sl: Some(sl),
                suggested_tp: None,
            })
        } else {
            None
        }
    }
}
