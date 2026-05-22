// strategies_backtests/helios/src/strategy/logic/risk.rs

use crate::types::*;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;
use ferro_trade::strategy::TickContext;

/// QUẢN TRỊ RỦI RO & TRAILING THÍCH NGHI (Level 3-4: Risk Management)
/// Helios sử dụng cơ chế "Elastic Trailing" - Co giãn khoảng cách dừng lỗ
/// dựa trên thế trận (Regime) và độ dốc (Angle) để cân bằng giữa Tấn công và Phòng thủ.
pub fn manage_trailing_stop(
    config: &HeliosConfig,
    memory: &mut TacticalMemory,
    tick: &TickContext,
    regime: MarketRegime,
    trend_angle: f64,
    _atr_val: f64,
) {
    let risk_cfg = &config.risk_management.stop_management;
    let base_risk = &config.risk_management.risk_base;
    
    // --- 🧬 GENE ỔN ĐỊNH: Dùng ATR mượt (Benchmark) (FIX #19) ---
    // Sử dụng ATR trung bình để mức dừng lỗ không bị giật (Noise) bởi 1-2 nến cá biệt.
    let smooth_atr = memory.volatility_benchmark;

    // --- 🧭 CHIẾN THUẬT CO GIÃN (ELASTIC LOGIC) ---
    // Mục tiêu: Nới rộng khi bùng nổ để "nuôi" trend, siết chặt khi hưng phấn quá đà để khóa lời.
    let multiplier = match regime {
        
        // ⚡ ANOMALY (FLASH CRASH / SPIKE): Nới rộng để tránh bị văng bởi râu nến quét thanh khoản.
        MarketRegime::Anomaly => risk_cfg.tsl_atr_loose,

        // 🌪️ SQUEEZE BREAKOUT: Nới rộng để giá có không gian "retest" vùng phá vỡ.
        MarketRegime::Squeeze => risk_cfg.tsl_atr_loose,

        // 🌊 TRENDING: Điều tiết theo độ dốc xu hướng
        MarketRegime::Trending => {
            if trend_angle.abs() > config.market_regime.trend.angle_parabolic {
                // 🚀 PARABOLIC: Giá bay quá dốc (>75°), hưng phấn quá đà -> Siết chặt (TIGHT) để bảo vệ lời.
                risk_cfg.tsl_atr_tight
            } else {
                // 📈 HEALTHY TREND: Sóng chạy bền vững -> Dùng mức bình thường (NORMAL).
                risk_cfg.tsl_atr_normal
            }
        },

        // 🧱 DEFAULT / ABSORPTION / RANGING: Bảo vệ vốn an toàn.
        _ => risk_cfg.tsl_atr_normal,
    };

    // --- 🛠️ THỰC THI DỜI SL (ELASTIC PEAK LOCK) ---
    // Lặp qua mọi lệnh đang mở và cập nhật SL nếu có lợi hơn.
    for trade in tick.trades.iter().filter(|t| t.is_open()) {
        let entry_price = trade.entry_price;
        
        // Sử dụng giá đỉnh đạt được (Peak Price) thay vì Close hiện tại (FIX #16)
        // Giúp SL bám theo "tầm cao mới" mà không bị lùi lại khi giá rút râu.
        let peak = memory.peak_price.unwrap_or(entry_price);
        
        // 1. Logic Breakeven (Dời về hòa vốn) (FIX #18)
        if base_risk.use_breakeven > 0.5 && !memory.is_breakeven_active {
            let sl_dist = (trade.entry_price - trade.sl_price.unwrap_or(0.0)).abs();
            if sl_dist > 0.0 {
                let current_profit_rr = if trade.is_long() {
                    (peak - entry_price) / sl_dist
                } else {
                    (entry_price - peak) / sl_dist
                };

                // Đạt mục tiêu RR (ví dụ 1.5R) -> Khóa ngay tại điểm vào lệnh
                if current_profit_rr >= base_risk.be_profit_target_rr {
                    tick.update_trade_sl_tp(trade.id, Some(entry_price), trade.tp_price);
                    memory.is_breakeven_active = true;
                    continue; 
                }
            }
        }

        // 2. Logic Trailing Stop dựa trên Peak Price (FIX #16)
        let new_sl = if trade.is_long() {
            peak - (smooth_atr * multiplier)
        } else {
            peak + (smooth_atr * multiplier)
        };

        // Quy tắc bất biến: Chỉ dời SL theo chiều có lợi (Lock-in profit)
        let should_update = if trade.is_long() {
            new_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
        } else {
            new_sl < trade.sl_price.unwrap_or(f64::INFINITY)
        };

        if should_update {
            tick.update_trade_sl_tp(trade.id, Some(new_sl), trade.tp_price);
        }
    }
}
