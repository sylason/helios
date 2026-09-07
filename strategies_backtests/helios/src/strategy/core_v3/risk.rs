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
        tick: &mut TickContext,
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

            // =========================================================================
            // 🛑 TÁI THIẾT KẾ: GIẢI PHÓNG VỐN NGÂM (STAGNANT CAPITAL RELEASE ĐỘNG)
            // =========================================================================
            // Lệnh chỉ bị đóng do ngâm vốn khi thỏa mãn ĐỒNG THỜI 2 điều kiện:
            // 1. Số nến nắm giữ (holding bars) >= max_stagnant_hold_bars (120 -> 2880 bars)
            // 2. Tỷ lệ lãi trên giá vào lệnh KHÔNG đạt mức tối thiểu (trade_pnl_pct < stagnant_min_profit_pct)
            let bars_held = i.saturating_sub(entry_bar);
            let current_close = tick.closes()[i];
            let trade_price_pnl_pct = if trade.entry_price > 0.0 {
                if trade.is_long() {
                    (current_close - trade.entry_price) / trade.entry_price
                } else {
                    (trade.entry_price - current_close) / trade.entry_price
                }
            } else {
                0.0
            };

            let max_hold = risk_cfg.max_stagnant_hold_bars;
            if max_hold > 0 && bars_held >= max_hold {
                // Nếu sau thời gian ngâm tối đa mà tỷ lệ lãi chưa đạt ngưỡng tối thiểu
                if trade_price_pnl_pct < risk_cfg.stagnant_min_profit_pct {
                    if _debug_mode {
                        tracing::info!(
                            "⏳ [STAGNANT CAPITAL RELEASE] Bar: {} | Trade #{:?} | Held: {} bars >= max: {} | PnL: {:+.2}% < min: {:.2}% -> Closed to release capital",
                            i, trade.id, bars_held, max_hold, trade_price_pnl_pct * 100.0, risk_cfg.stagnant_min_profit_pct * 100.0
                        );
                    }
                    tick.close(None);
                    continue;
                }
            }

            // Zombie Trade Purge: Lệnh ngâm quá 2x max_stagnant_hold_bars (tối thiểu 2880 nến = 48 tiếng) -> Cưỡng bức dọn dẹp
            let zombie_limit = (max_hold * 2).max(2880);
            if max_hold > 0 && bars_held >= zombie_limit {
                if _debug_mode {
                    tracing::warn!(
                        "🧟 [ZOMBIE TRADE PURGE] Bar: {} | Trade #{:?} | Held: {} bars >= zombie limit: {} -> Force closed",
                        i, trade.id, bars_held, zombie_limit
                    );
                }
                tick.close(None);
                continue;
            }

            let trade_margin = (trade.entry_price * trade.size.abs()) / leverage;
            let current_roe = if trade_margin > 0.0 { current_pnl / trade_margin } else { 0.0 };
            let peak_roe = if trade.entry_price > 0.0 {
                if trade.is_long() {
                    ((peak_val - trade.entry_price) / trade.entry_price) * leverage
                } else {
                    ((trade.entry_price - peak_val) / trade.entry_price) * leverage
                }
            } else {
                0.0
            };

            // 1. Kiểm tra chế độ Siêu Sóng Mega Trend
            let mega_trend_trigger_pnl = risk_cfg.mega_trend_pnl_trigger * trade.entry_price * trade.size.abs();
            let is_mega_trend = current_pnl > 0.0 && (current_pnl >= mega_trend_trigger_pnl || current_roe >= 0.50);

            // 2. Tính toán hệ số nhân Multiplier hiện tại
            let mut current_multiplier = if is_mega_trend {
                // CHẾ ĐỘ THẢ DIỀU SIÊU SÓNG: Dùng tsl_atr_loose (8.0 -> 15.0 ATR) để cho sóng bay cao
                risk_cfg.tsl_atr_loose
            } else if is_volatile && current_pnl > 0.0 {
                if base_multiplier < risk_cfg.tsl_atr_loose {
                    risk_cfg.tsl_atr_loose
                } else {
                    base_multiplier
                }
            } else {
                base_multiplier
            };

            if decay_cycles > 0 && !is_mega_trend {
                let progressive_factor = risk_cfg.tighten_step_multiplier.powi(decay_cycles as i32);
                current_multiplier *= progressive_factor;
                
                if current_multiplier < risk_cfg.tsl_atr_tight {
                    current_multiplier = risk_cfg.tsl_atr_tight;
                }
            }

            let lev_cap_pct = (playbook_max_sl_pct).min(0.80 / leverage);

            let mut calculated_sl = if trade.is_long() {
                let raw_sl = peak_val - (current_atr * current_multiplier);
                let min_long_sl = trade.entry_price * (1.0 - lev_cap_pct);
                raw_sl.max(min_long_sl)
            } else {
                let raw_sl = peak_val + (current_atr * current_multiplier);
                let max_short_sl = trade.entry_price * (1.0 + lev_cap_pct);
                raw_sl.min(max_short_sl)
            };

            // [ENHANCEMENT 1: SQUEEZE-AWARE DYNAMIC BREAK-EVEN TRIGGER]
            // Không hardcode: Ngưỡng kích hoạt tự động giãn nở theo độ nén Squeeze của Matrix và DNA base_be_roe_trigger
            let sqz_multiplier = matrix.squeeze_multiplier[i].max(0.0);
            let base_be_atr_mult = risk_cfg.base_sl_atr.max(1.0);
            let dynamic_be_atr = current_atr * base_be_atr_mult * (1.0 + sqz_multiplier);
            let base_be_roe = if risk_cfg.base_be_roe_trigger > 0.0 { risk_cfg.base_be_roe_trigger } else { 0.08 };
            let be_roe_trigger = base_be_roe * (1.0 + sqz_multiplier);
            let be_price_buffer_pct = 0.0008; // 0.08% buffer bù đắp phí sàn 0.04% Maker/Taker
            let is_be_triggered = current_pnl > 0.0 && (current_roe >= be_roe_trigger || current_pnl >= (dynamic_be_atr * trade.size.abs()));

            if is_be_triggered && !is_mega_trend {
                if trade.is_long() {
                    let be_sl = trade.entry_price * (1.0 + be_price_buffer_pct);
                    calculated_sl = calculated_sl.max(be_sl);
                } else {
                    let be_sl = trade.entry_price * (1.0 - be_price_buffer_pct);
                    calculated_sl = calculated_sl.min(be_sl);
                }
            }

            // [ENHANCEMENT 2: MEGA TREND HARD PROFIT FLOOR]
            // Khi siêu sóng đạt đỉnh cao: Khóa cứng lợi nhuận đỉnh theo tỷ lệ DNA mega_trend_lock_ratio
            if is_mega_trend {
                let lock_ratio = if risk_cfg.mega_trend_lock_ratio > 0.0 { risk_cfg.mega_trend_lock_ratio } else { 0.50 };
                let min_lock_roe = if risk_cfg.mega_trend_min_lock_roe > 0.0 { risk_cfg.mega_trend_min_lock_roe } else { 0.25 };
                let locked_roe = (peak_roe * lock_ratio).max(min_lock_roe);
                if trade.is_long() {
                    let floor_sl = trade.entry_price * (1.0 + (locked_roe / leverage));
                    calculated_sl = calculated_sl.max(floor_sl);
                } else {
                    let floor_sl = trade.entry_price * (1.0 - (locked_roe / leverage));
                    calculated_sl = calculated_sl.min(floor_sl);
                }
            }

            // [ENHANCEMENT 3: MONOTONIC RATCHET GUARD]
            // Bảo toàn Stop Loss 1 chiều tuyệt đối: SL chỉ được phép dời theo hướng bảo vệ vốn (Tiến, cấm Lùi).
            // Chỉ kích hoạt khi vị thế có lãi danh nghĩa (current_pnl > 0.0).
            if current_pnl > 0.0 {
                let should_update = if trade.is_long() {
                    calculated_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
                } else {
                    calculated_sl < trade.sl_price.unwrap_or(f64::INFINITY)
                };

                if should_update {
                    tick.update_trade_sl_tp(trade.id, Some(calculated_sl), trade.tp_price);
                }
            }
        }
    }
}
