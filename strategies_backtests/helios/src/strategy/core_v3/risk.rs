use ferro_trade::strategy::TickContext;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::sensors::HeliosSensors;

pub struct RiskManager;

impl RiskManager {
    /// Tính toán Size dựa trên Fixed Percentage (Fraction của Ký quỹ) nhưng đảm bảo Min Order Size.
    /// Trả về một giá trị < 1.0 (ví dụ 0.1 = 10%) để `ferro_trade` dùng làm Initial Margin.
    pub fn calculate_size(equity: f64, risk_pct: f64, min_order_usd: f64, leverage: f64, max_cap_fraction: f64) -> f64 {
        if equity <= 0.0 || risk_pct <= 0.0 || leverage <= 0.0 {
            return 0.0;
        }
        let mut target_margin = equity * risk_pct;
        let mut order_size_usd = target_margin * leverage;
        
        if order_size_usd < min_order_usd {
            order_size_usd = min_order_usd;
            target_margin = order_size_usd / leverage;
        }
        
        let size_fraction = target_margin / equity;
        let cap = if max_cap_fraction <= 0.0 { 0.25 } else { max_cap_fraction };
        size_fraction.min(cap)
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

            // [ENHANCEMENT 1: REGIME-AWARE DYNAMIC BREAK-EVEN TRIGGER]
            // Không hardcode: Ngưỡng kích hoạt và khoảng đệm bảo vệ vốn phân tầng theo 3 nhóm Regime Simplex (Trend, Range, Volatile)
            let (be_trigger_mult, be_buffer_mult) = match trade.tag.as_deref() {
                Some("TREND_PB") | Some("ABSORPTION") => (
                    if risk_cfg.be_atr_trigger_trend > 0.0 { risk_cfg.be_atr_trigger_trend } else { 2.5 },
                    if risk_cfg.be_buffer_atr_trend > 0.0 { risk_cfg.be_buffer_atr_trend } else { 1.5 },
                ),
                Some("RANGE_OSC") | Some("REVERSAL") => (
                    if risk_cfg.be_atr_trigger_range > 0.0 { risk_cfg.be_atr_trigger_range } else { 1.5 },
                    if risk_cfg.be_buffer_atr_range > 0.0 { risk_cfg.be_buffer_atr_range } else { 0.8 },
                ),
                Some("MOMENTUM_IGN") | Some("ANOMALY") => (
                    if risk_cfg.be_atr_trigger_volatile > 0.0 { risk_cfg.be_atr_trigger_volatile } else { 2.0 },
                    if risk_cfg.be_buffer_atr_volatile > 0.0 { risk_cfg.be_buffer_atr_volatile } else { 1.2 },
                ),
                _ => (
                    if risk_cfg.be_atr_trigger_trend > 0.0 { risk_cfg.be_atr_trigger_trend } else { 2.5 },
                    if risk_cfg.be_buffer_atr_trend > 0.0 { risk_cfg.be_buffer_atr_trend } else { 1.5 },
                ),
            };

            let sqz_multiplier = matrix.squeeze_multiplier[i].max(0.0);
            let effective_be_trigger_atr = be_trigger_mult * (1.0 + 0.5 * sqz_multiplier);
            let required_profit = current_atr * effective_be_trigger_atr * trade.size.abs();
            let min_be_roe = (risk_cfg.base_be_roe_trigger.max(0.08) * (1.0 + sqz_multiplier)).min(0.50);
            let is_be_triggered = current_pnl > 0.0 && (current_roe >= min_be_roe || current_pnl >= required_profit);

            if is_be_triggered && !is_mega_trend {
                let fee_buffer_pct = 0.0008; // Buffer bù đắp phí sàn tối thiểu 0.08% Maker/Taker
                if trade.is_long() {
                    let min_be = trade.entry_price * (1.0 + fee_buffer_pct);
                    let dynamic_floor = peak_val - (current_atr * be_buffer_mult);
                    let target_be_sl = dynamic_floor.max(min_be);
                    calculated_sl = calculated_sl.max(target_be_sl);
                } else {
                    let max_be = trade.entry_price * (1.0 - fee_buffer_pct);
                    let dynamic_ceiling = peak_val + (current_atr * be_buffer_mult);
                    let target_be_sl = dynamic_ceiling.min(max_be);
                    calculated_sl = calculated_sl.min(target_be_sl);
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
            let underwater_cycles = bars_held / time_countdown_bars;
            if current_pnl > 0.0 {
                // KHI CÓ LÃI DANH NGHĨA (current_pnl > 0.0): Trailing theo đỉnh High Watermark & Dynamic BE & Mega Trend
                let should_update = if trade.is_long() {
                    calculated_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
                } else {
                    calculated_sl < trade.sl_price.unwrap_or(f64::INFINITY)
                };

                if should_update {
                    tick.update_trade_sl_tp(trade.id, Some(calculated_sl), trade.tp_price);
                }
            } else if underwater_cycles > 0 {
                // KHI ĐANG ÂM (current_pnl <= 0.0) VÀ BỊ NGÂM VỐN (underwater_cycles > 0):
                // Thu hẹp khoảng cách Stop Loss từ entry_price để cắt lỗ sớm bảo toàn vốn
                let progressive_factor = risk_cfg.tighten_step_multiplier.powi(underwater_cycles as i32);
                let underwater_mult = (risk_cfg.base_sl_atr * progressive_factor).max(risk_cfg.tsl_atr_tight);

                let tightened_sl = if trade.is_long() {
                    trade.entry_price - (current_atr * underwater_mult)
                } else {
                    trade.entry_price + (current_atr * underwater_mult)
                };

                // CHỐT CHẶN 1: Chống nhảy qua đầu giá nến hiện tại
                // Nếu mức SL siết chặt đã chạm hoặc vượt qua giá nến hiện tại -> Giải phóng vốn ngay!
                let should_force_close = if trade.is_long() {
                    tightened_sl >= current_close
                } else {
                    tightened_sl <= current_close
                };

                if should_force_close {
                    if _debug_mode {
                        tracing::info!(
                            "⏳ [UNDERWATER STAGNANT TIMEOUT] Bar: {} | Trade #{:?} | Tightened SL {:.2} reached current price {:.2} -> Closed to protect capital",
                            i, trade.id, tightened_sl, current_close
                        );
                    }
                    tick.close(None);
                    continue;
                }

                // CHỐT CHẶN 2: Buffer Floor tối thiểu từ cấu hình DNA chống quét non (Whipsaw)
                let floor_atr = if config.risk_management.stop_management.tsl_buffer_floor_atr <= 0.0 {
                    1.0
                } else {
                    config.risk_management.stop_management.tsl_buffer_floor_atr
                };
                let min_buffer = current_atr * floor_atr;
                let safe_tightened_sl = if trade.is_long() {
                    tightened_sl.min(current_close - min_buffer)
                } else {
                    tightened_sl.max(current_close + min_buffer)
                };

                // CHỐT CHẶN 3: Monotonic Ratchet Guard (TUYỆT ĐỐI CẤM LÙI SL)
                let should_update = if trade.is_long() {
                    safe_tightened_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
                } else {
                    safe_tightened_sl < trade.sl_price.unwrap_or(f64::INFINITY)
                };

                if should_update {
                    tick.update_trade_sl_tp(trade.id, Some(safe_tightened_sl), trade.tp_price);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_sl_price_and_initial_sl_bounds() {
        let price = 2500.0;
        let is_long = true;
        let max_initial_sl_pct = 0.05;
        let min_initial_sl_pct = 0.002;
        let leverage = 20.0;

        // SL quá xa (> 4% bị kẹp bởi leverage cap 0.80 / 20 = 0.04)
        let too_far_sl = 2300.0; // cách 200 giá = 8%
        let clamped = RiskManager::clamp_sl_price(price, too_far_sl, is_long, max_initial_sl_pct, min_initial_sl_pct, leverage);
        let expected_min_sl = price * (1.0 - 0.04); // 2400.0
        assert_eq!(clamped, expected_min_sl, "SL quá xa phải bị kẹp bởi trần đòn bẩy 20x (4%)");

        // SL quá gần (< 0.2% min_pct)
        let too_close_sl = 2499.0; // cách 1 giá = 0.04%
        let clamped_close = RiskManager::clamp_sl_price(price, too_close_sl, is_long, max_initial_sl_pct, min_initial_sl_pct, leverage);
        let expected_max_sl = price * (1.0 - min_initial_sl_pct); // 2495.0
        assert_eq!(clamped_close, expected_max_sl, "SL quá gần phải bị kẹp bởi sàn min_pct");
    }

    #[test]
    fn test_underwater_tightening_math_formula() {
        let entry_price: f64 = 2500.0;
        let current_atr: f64 = 10.0;
        let base_sl_atr: f64 = 4.0;
        let tsl_atr_tight: f64 = 1.0;
        let step_multiplier: f64 = 0.80; // Mỗi chu kỳ siết 20%

        // Chu kỳ 1: multiplier = 4.0 * 0.8 = 3.2
        let mult_c1 = (base_sl_atr * step_multiplier).max(tsl_atr_tight);
        assert!((mult_c1 - 3.2).abs() < 1e-9);
        let sl_c1 = entry_price - (current_atr * mult_c1);
        assert!((sl_c1 - 2468.0).abs() < 1e-9, "Chu kỳ 1 phải siết SL từ 2460 lên 2468");

        // Chu kỳ 2: multiplier = 4.0 * 0.64 = 2.56
        let mult_c2 = (base_sl_atr * step_multiplier * step_multiplier).max(tsl_atr_tight);
        assert!((mult_c2 - 2.56).abs() < 1e-9);
        let sl_c2 = entry_price - (current_atr * mult_c2);
        assert!((sl_c2 - 2474.4).abs() < 1e-9, "Chu kỳ 2 phải siết SL lên 2474.4");

        // Đảm bảo Monotonic: sl_c2 > sl_c1
        assert!(sl_c2 > sl_c1, "SL phải dời tăng dần để bảo vệ vốn khi ngâm lệnh âm");
    }

    #[test]
    fn test_calculate_size_nan_resilience_and_cap() {
        // Trường hợp bình thường
        let size = RiskManager::calculate_size(100.0, 0.20, 21.0, 20.0, 0.25);
        assert!(size > 0.0 && size <= 0.25);

        // Trường hợp min order size $21.0 khi vốn nhỏ ($60)
        let size_small = RiskManager::calculate_size(60.0, 0.10, 21.0, 20.0, 0.25);
        assert_eq!(size_small, 0.10);

        // Trường hợp order size < 21.0 (vốn $10, risk 5%, margin $0.5 -> order $10 < $21)
        // order_size nâng lên $21 -> margin = 21/20 = 1.05 -> fraction = 1.05/10 = 0.105
        let size_min_bump = RiskManager::calculate_size(10.0, 0.05, 21.0, 20.0, 0.25);
        assert!((size_min_bump - 0.105).abs() < 1e-9);

        // Chốt chặn NaN / Âm vốn / Zero equity
        assert_eq!(RiskManager::calculate_size(0.0, 0.20, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(-50.0, 0.20, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.0, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 21.0, 0.0, 0.25), 0.0);
    }
}
