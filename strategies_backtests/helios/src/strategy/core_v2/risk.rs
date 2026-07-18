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
    current_regime: MarketRegime, // [FIXED]: Nhận diện Regime hiện tại thay vì trend_angle
    _atr_val: f64,
) {
    let risk_cfg = &config.risk_management.stop_management;
    let base_risk = &config.risk_management.risk_base;
    
    // --- 🧬 GENE ỔN ĐỊNH: Dùng ATR mượt (Benchmark) (FIX #19) ---
    // Sử dụng ATR trung bình để mức dừng lỗ không bị giật (Noise) bởi 1-2 nến cá biệt.
    let smooth_atr = memory.volatility_benchmark;

    // --- 🧭 CHIẾN THUẬT CO GIÃN THEO REGIME (ADAPTIVE LOGIC) ---
    // [FIXED]: TSL lật ngược logic, liên tục điều chỉnh theo MarketRegime hiện tại.
    let mut multiplier = match current_regime {
        // 🚀 PARABOLIC: Giá bay quá dốc, râu nến dài -> Nới lỏng (LOOSE) để sống sót qua nhiễu.
        MarketRegime::Parabolic => risk_cfg.tsl_atr_loose,
        
        // 📈 TRENDING: Sóng chạy bền vững, ít nhiễu -> Mức bình thường (NORMAL).
        MarketRegime::Trending => risk_cfg.tsl_atr_normal,
        
        // 🌪️ SQUEEZE: Nén biên độ, dễ có quét thanh khoản 2 đầu (Fakeout) -> Mức bình thường/rộng để tránh bị quét.
        MarketRegime::Squeeze => risk_cfg.tsl_atr_normal,
        
        // 🧱 RANGING / ABSORPTION / ANOMALY: Chạm biên quay đầu hoặc rủi ro cao -> Siết chặt (TIGHT) để ăn ngay.
        MarketRegime::Ranging | MarketRegime::Absorption | MarketRegime::Anomaly => risk_cfg.tsl_atr_tight,
    };

    // --- LỆNH ZOMBIE (TIME DECAY SL) ---
    // Ép vị thế đóng nếu ngâm quá lâu
    if memory.holding_bars_counter > risk_cfg.time_decay_bars {
        multiplier = multiplier.min(risk_cfg.tsl_atr_tight * 0.5).max(0.1); 
    }

    // --- 🛠️ THỰC THI DỜI SL (ELASTIC PEAK LOCK) ---
    for trade in tick.trades.iter().filter(|t| t.is_open()) {
        let entry_price = trade.entry_price;
        
        // Sử dụng giá đỉnh đạt được (Peak Price) thay vì Close hiện tại (FIX #16)
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
        let mut new_sl = if trade.is_long() {
            peak - (smooth_atr * multiplier)
        } else {
            peak + (smooth_atr * multiplier)
        };

        let current_sl = trade.sl_price.unwrap_or(if trade.is_long() { f64::NEG_INFINITY } else { f64::INFINITY });

        // --- 🛡️ ELASTIC RATCHET (MỞ KHÓA BÁNH CÓC BẢO VỆ VỐN) ---
        // Cho phép SL nới lỏng khi thị trường cần thở (ví dụ từ Ranging -> Parabolic), 
        // nhưng TUYỆT ĐỐI chặn ở Entry Price (Hòa vốn) để không gia tăng rủi ro so với ban đầu.
        let should_update = if trade.is_long() {
            if new_sl > current_sl {
                true // Siết chặt hơn (Lock-in profit) -> Luôn cho phép
            } else {
                // Nới lỏng ra (Ratchet Release)
                if current_sl > entry_price {
                    new_sl = new_sl.max(entry_price); // Chặn ở mức hòa vốn, không cho lỗ lại
                    new_sl < current_sl // Nếu sau khi chặn mà vẫn nới lỏng được so với SL hiện tại thì cho phép
                } else {
                    false // Đang lỗ hoặc mới hòa vốn, tuyệt đối không nới rộng thêm rủi ro ban đầu
                }
            }
        } else {
            if new_sl < current_sl {
                true // Siết chặt hơn (Lock-in profit) -> Luôn cho phép
            } else {
                // Nới lỏng ra (Ratchet Release)
                if current_sl < entry_price {
                    new_sl = new_sl.min(entry_price); // Chặn ở mức hòa vốn, không cho lỗ lại
                    new_sl > current_sl 
                } else {
                    false // Đang lỗ hoặc mới hòa vốn, tuyệt đối không nới rộng thêm rủi ro ban đầu
                }
            }
        };

        if should_update {
            tick.update_trade_sl_tp(trade.id, Some(new_sl), trade.tp_price);
        }
    }
}
