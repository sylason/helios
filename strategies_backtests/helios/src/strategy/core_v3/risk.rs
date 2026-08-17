// strategies_backtests/helios/src/strategy/core_v3/risk.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::sensors::HeliosSensors;

pub struct RiskManager;

impl RiskManager {
    /// Tính toán Size dựa trên Fixed Percentage (Fraction của Ký quỹ) nhưng đảm bảo Min Order Size.
    /// Trả về một giá trị < 1.0 (ví dụ 0.1 = 10%) để `ferro_trade` dùng làm Initial Margin.
    pub fn calculate_size(equity: f64, risk_pct: f64, min_order_usd: f64, leverage: f64) -> f64 {
        let mut target_margin = equity * risk_pct;
        let mut order_size_usd = target_margin * leverage;
        
        if order_size_usd < min_order_usd {
            order_size_usd = min_order_usd;
            target_margin = order_size_usd / leverage;
        }
        
        let size_fraction = target_margin / equity;
        size_fraction.min(0.20)
    }

    /// Hàm khống chế trần SL ban đầu theo % tối đa cho phép và đòn bẩy (Leverage Safety Bounds)
    pub fn clamp_sl_price(price: f64, sl_price: f64, is_long: bool, max_initial_sl_pct: f64, min_initial_sl_pct: f64, leverage: f64) -> f64 {
        let lev = if leverage <= 0.0 { 1.0 } else { leverage };
        let max_sl_by_lev = 0.80 / lev;
        let cap_pct = (if max_initial_sl_pct <= 0.0 { 0.02 } else { max_initial_sl_pct }).min(max_sl_by_lev);
        let min_pct = if min_initial_sl_pct <= 0.0 { 0.002 } else { min_initial_sl_pct };

        if is_long {
            let max_allowed_sl = price * (1.0 - min_pct);
            let min_allowed_sl = price * (1.0 - cap_pct);
            sl_price.min(max_allowed_sl).max(min_allowed_sl)
        } else {
            let min_allowed_sl = price * (1.0 + min_pct);
            let max_allowed_sl = price * (1.0 + cap_pct);
            sl_price.max(min_allowed_sl).min(max_allowed_sl)
        }
    }

    /// Tính giá cắt lỗ khởi tạo với Hard Max Cap % để khống chế rủi ro tối đa
    pub fn get_initial_sl(
        tick: &TickContext,
        sensors: &HeliosSensors,
        is_long: bool,
        multiplier: f64,
        max_initial_sl_pct: f64,
        min_initial_sl_pct: f64,
        leverage: f64,
    ) -> f64 {
        let i = tick.i;
        let current_atr = sensors.atr_sl[i];
        let price = tick.closes()[i];
        let raw_sl = if is_long {
            price - (current_atr * multiplier)
        } else {
            price + (current_atr * multiplier)
        };

        Self::clamp_sl_price(price, raw_sl, is_long, max_initial_sl_pct, min_initial_sl_pct, leverage)
    }

    /// Quản lý Trailing Stop cho tất cả các lệnh đang mở (Adaptive Loose Trailing & Stagnant Exit)
    pub fn manage_trailing_stop(
        config: &HeliosConfigV3,
        matrix: &MarketMatrixV3,
        tick: &TickContext,
        sensors: &HeliosSensors,
        _debug_mode: bool,
    ) {
        let i = tick.i;
        let risk_cfg = &config.risk_management.stop_management;
        let leverage = config.risk_management.risk_base.leverage.max(1.0);
        
        let current_atr = sensors.atr_sl[i];
        if current_atr.is_nan() || current_atr <= 0.0 {
            return;
        }

        let anomaly_threshold = config.tactical_sniper.anomaly.z_score_threshold;
        let current_anomaly = matrix.anomaly_multiplier[i]; 
        let is_volatile = current_anomaly > anomaly_threshold;

        for trade in tick.trades.iter().filter(|t| t.is_open()) {
            let entry_bar = trade.entry_bar;
            
            // 1. Base Multiplier & Playbook Max SL Pct
            let (base_multiplier, playbook_max_sl_pct) = if let Some(ref tag_str) = trade.tag {
                match tag_str.as_str() {
                    "RANGE_OSC" => (risk_cfg.tsl_atr_tight, config.tactical_sniper.range.max_initial_sl_pct),
                    "MOMENTUM_IGN" => (risk_cfg.tsl_atr_loose, config.tactical_sniper.momentum.max_initial_sl_pct),
                    "ANOMALY" => (risk_cfg.tsl_atr_loose, config.tactical_sniper.anomaly.max_initial_sl_pct),
                    "TREND_PB" => (risk_cfg.tsl_atr_normal, config.tactical_sniper.pullback.max_initial_sl_pct),
                    "REVERSAL" => (risk_cfg.tsl_atr_tight, config.tactical_sniper.reversal.max_initial_sl_pct),
                    "ABSORPTION" => (risk_cfg.tsl_atr_normal, config.tactical_sniper.absorption.max_initial_sl_pct),
                    _ => (risk_cfg.tsl_atr_normal, 0.05),
                }
            } else {
                (risk_cfg.tsl_atr_normal, 0.05)
            };

            // 2. Tìm đỉnh lợi nhuận cao nhất (High Watermark) kể từ nến thứ 2
            let highs = tick.highs();
            let lows = tick.lows();
            
            let start_b = (entry_bar + 1).min(i);
            let (peak_val, peak_bar) = if trade.is_long() {
                let mut max_high = highs[start_b];
                let mut max_bar = start_b;
                for j in start_b..=i {
                    if highs[j] > max_high { 
                        max_high = highs[j]; 
                        max_bar = j;
                    }
                }
                (max_high, max_bar)
            } else {
                let mut min_low = lows[start_b];
                let mut min_bar = start_b;
                for j in start_b..=i {
                    if lows[j] < min_low { 
                        min_low = lows[j]; 
                        min_bar = j;
                    }
                }
                (min_low, min_bar)
            };

            // 3. Đếm số nến đi ngang kể từ đỉnh
            let effective_stagnant_bars = if i >= peak_bar { i - peak_bar } else { 0 };

            let time_countdown_bars = risk_cfg.time_countdown_bars.max(1);
            let decay_cycles = effective_stagnant_bars / time_countdown_bars;
            
            let current_pnl = trade.pl(Some(tick.closes()[i]));

            // STAGNANT CAPITAL RELEASE MỞ RỘNG:
            // 1. Lệnh lỗ ngâm quá 1 chu kỳ decay -> Đóng ngay cắt lỗ nhỏ
            if decay_cycles > 0 && current_pnl <= 0.0 {
                tick.close(None);
                continue;
            }

            // 2. Lệnh ngâm quá 4 chu kỳ decay (Zombie Trade) -> Đóng chủ động giải phóng vốn ký quỹ
            if decay_cycles >= 4 {
                tick.close(None);
                continue;
            }

            // Tính toán hệ số nhân Multiplier hiện tại (Progressive Decay)
            let mut current_multiplier = if is_volatile && current_pnl > 0.0 {
                if base_multiplier < risk_cfg.tsl_atr_loose {
                    risk_cfg.tsl_atr_loose
                } else {
                    base_multiplier
                }
            } else {
                base_multiplier
            };

            let is_trend_or_momentum = if let Some(ref tag_str) = trade.tag {
                tag_str == "TREND_PB" || tag_str == "MOMENTUM_IGN" || tag_str == "ABSORPTION"
            } else {
                false
            };

            if decay_cycles > 0 {
                let progressive_factor = risk_cfg.tighten_step_multiplier.powi(decay_cycles as i32);
                current_multiplier *= progressive_factor;
                
                let mega_pnl_trigger = risk_cfg.mega_trend_pnl_trigger;
                let max_mega_cycles = risk_cfg.mega_trend_max_cycles;
                let min_tight = if is_trend_or_momentum && current_pnl > mega_pnl_trigger && decay_cycles <= max_mega_cycles {
                    risk_cfg.tsl_atr_normal
                } else {
                    risk_cfg.tsl_atr_tight
                };

                if current_multiplier < min_tight {
                    current_multiplier = min_tight;
                }
            }

            let lev_cap_pct = (playbook_max_sl_pct).min(0.80 / leverage);

            let new_sl = if trade.is_long() {
                let calculated_sl = peak_val - (current_atr * current_multiplier);
                let min_long_sl = trade.entry_price * (1.0 - lev_cap_pct);
                calculated_sl.max(min_long_sl)
            } else {
                let calculated_sl = peak_val + (current_atr * current_multiplier);
                let max_short_sl = trade.entry_price * (1.0 + lev_cap_pct);
                calculated_sl.min(max_short_sl)
            };

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
}
