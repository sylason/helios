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

    /// Hàm khống chế trần SL ban đầu theo % ROE tối đa cho phép và đòn bẩy động (FLAW-TSL.8)
    /// Min Distance Floor Guard bảo vệ chống kẹp ngược khi đòn bẩy cao.
    pub fn clamp_sl_price(
        price: f64,
        sl_price: f64,
        is_long: bool,
        max_sl_roe_cap: f64,
        min_initial_sl_pct: f64,
        leverage: f64,
    ) -> f64 {
        let lev = if leverage <= 0.0 { 1.0 } else { leverage };
        // Ngưỡng ROE tối đa (mặc định 20% ROE nếu config <= 0.0)
        let roe_cap = if max_sl_roe_cap <= 0.0 { 0.20 } else { max_sl_roe_cap };
        let min_pct = if min_initial_sl_pct <= 0.0 { 0.0015 } else { min_initial_sl_pct };

        // 1. Trần khoảng cách giá tối đa theo đòn bẩy thực tế (RÀNG BUỘC CỨNG BẤT KHẢ XÂM PHẠM)
        let cap_price_pct = roe_cap / lev;

        // 2. Sàn khoảng cách tối thiểu chống quét spread/noise (PHẢI LUÔN NẰM DƯỚI TRẦN)
        // Áp dụng tỷ lệ an toàn 50% trần: Sàn tối đa chỉ được chiếm 50% khoảng cách của Trần ROE.
        // Đảm bảo không bao giờ có nghịch lý Sàn >= Trần, mở ra khoảng [effective_min_pct, cap_price_pct] linh hoạt.
        let effective_min_pct = min_pct.min(cap_price_pct * 0.50);
        let effective_cap = cap_price_pct;

        if is_long {
            let max_allowed_sl = price * (1.0 - effective_min_pct); // SL gần nhất (chống cắn spread)
            let min_allowed_sl = price * (1.0 - effective_cap);     // SL xa nhất (khống chế lỗ tối đa)
            sl_price.min(max_allowed_sl).max(min_allowed_sl)
        } else {
            let min_allowed_sl = price * (1.0 + effective_min_pct); // SL gần nhất
            let max_allowed_sl = price * (1.0 + effective_cap);     // SL xa nhất
            sl_price.max(min_allowed_sl).min(max_allowed_sl)
        }
    }

    /// Tính giá cắt lỗ khởi tạo với Hard Max Cap ROE để khống chế rủi ro tối đa
    pub fn get_initial_sl(
        tick: &TickContext,
        sensors: &HeliosSensors,
        is_long: bool,
        multiplier: f64,
        max_sl_roe_cap: f64,
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

        Self::clamp_sl_price(price, raw_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage)
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
        let current_anomaly = matrix.chaos_index.get(i).copied().unwrap_or(0.0); 
        let is_volatile = current_anomaly > anomaly_threshold;

        for trade in tick.trades.iter().filter(|t| t.is_open()) {
            let entry_bar = trade.entry_bar;
            
            // 1. Base Multiplier & Playbook Max SL ROE (FLAW-TSL.8: Fallback Hierarchy)
            let (base_multiplier, playbook_max_sl_roe) = if let Some(ref tag_str) = trade.tag {
                match tag_str.as_str() {
                    "RANGE_OSC" => (risk_cfg.tsl_atr_tight, config.tactical_sniper.range.max_initial_sl_pct),
                    "MOMENTUM_IGN" => (risk_cfg.tsl_atr_loose, config.tactical_sniper.momentum.max_initial_sl_pct),
                    "ANOMALY" => (risk_cfg.tsl_atr_loose, config.tactical_sniper.anomaly.max_initial_sl_pct),
                    "TREND_PB" => (risk_cfg.tsl_atr_normal, config.tactical_sniper.pullback.max_initial_sl_pct),
                    "REVERSAL" => (risk_cfg.tsl_atr_tight, config.tactical_sniper.reversal.max_initial_sl_pct),
                    "ABSORPTION" => (risk_cfg.tsl_atr_normal, config.tactical_sniper.absorption.max_initial_sl_pct),
                    _ => (risk_cfg.tsl_atr_normal, risk_cfg.default_sl_roe_cap),
                }
            } else {
                (risk_cfg.tsl_atr_normal, risk_cfg.default_sl_roe_cap)
            };

            // 2. Tìm đỉnh lợi nhuận cao nhất (High Watermark) chuẩn hóa từ giá vào lệnh (FLAW-TSL.5)
            // Khởi tạo neo chuẩn từ entry_price để triệt tiêu 100% Lookahead bias của nến entry
            // và đảm bảo đỉnh không bao giờ bị ghi nhận âm dưới giá vốn nếu nến sau giảm.
            let highs = tick.highs();
            let lows = tick.lows();
            
            let (peak_val, peak_bar) = if trade.is_long() {
                let mut max_high = trade.entry_price;
                let mut max_bar = entry_bar;
                for j in (entry_bar + 1)..=i {
                    if highs[j] > max_high { 
                        max_high = highs[j]; 
                        max_bar = j;
                    }
                }
                (max_high, max_bar)
            } else {
                let mut min_low = trade.entry_price;
                let mut min_bar = entry_bar;
                for j in (entry_bar + 1)..=i {
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

            // =========================================================================
            // 🚀 [FLAW-TSL.1: HYBRID TTP - NHÁNH B: PEAK-CALLBACK TRAILING TAKE PROFIT]
            // =========================================================================
            // Áp dụng cho các Playbook xu hướng hoặc không có TP cố định (TREND_PB, MOMENTUM_IGN, ABSORPTION, ANOMALY)
            // Giúp thả diều tối đa lợi nhuận siêu sóng và chỉ chốt lời khi giá tạo đỉnh quay đầu (Callback).
            let tag_str = trade.tag.as_deref().unwrap_or("");
            let is_ttp_eligible = matches!(tag_str, "TREND_PB" | "MOMENTUM_IGN" | "ABSORPTION" | "ANOMALY") || trade.tp_price.is_none();
            if is_ttp_eligible {
                let ttp_act_roe = if risk_cfg.ttp_activation_roe > 0.0 { risk_cfg.ttp_activation_roe } else { 0.20 };
                let ttp_cb_atr = if risk_cfg.ttp_callback_atr > 0.0 { risk_cfg.ttp_callback_atr } else { 0.80 };
                let min_gain_pct = if risk_cfg.dynamic_tp_min_gain_pct > 0.0 { risk_cfg.dynamic_tp_min_gain_pct } else { 0.002 };

                if peak_roe >= ttp_act_roe {
                    if trade.is_long() {
                        let ttp_trigger_price = peak_val - (ttp_cb_atr * current_atr);
                        let min_exit_price = trade.entry_price * (1.0 + min_gain_pct);
                        if current_close <= ttp_trigger_price && current_close >= min_exit_price {
                            tracing::info!(
                                "🏆 [TTP TRIGGERED EXIT] Bar: {} | Trade #{:?} ({}) Long | Peak: {:.2} (ROE: {:.2}%) | Trigger: {:.2} | Close: {:.2} -> Locked profit via Trailing TP",
                                i, trade.id, tag_str, peak_val, peak_roe * 100.0, ttp_trigger_price, current_close
                            );
                            tick.close(None);
                            continue;
                        }
                    } else {
                        let ttp_trigger_price = peak_val + (ttp_cb_atr * current_atr);
                        let max_exit_price = trade.entry_price * (1.0 - min_gain_pct);
                        if current_close >= ttp_trigger_price && current_close <= max_exit_price {
                            tracing::info!(
                                "🏆 [TTP TRIGGERED EXIT] Bar: {} | Trade #{:?} ({}) Short | Peak: {:.2} (ROE: {:.2}%) | Trigger: {:.2} | Close: {:.2} -> Locked profit via Trailing TP",
                                i, trade.id, tag_str, peak_val, peak_roe * 100.0, ttp_trigger_price, current_close
                            );
                            tick.close(None);
                            continue;
                        }
                    }
                }
            }

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

            // [FLAW-TSL.8: DYNAMIC LEVERAGE ROE CAP & UNIVERSAL FLOOR GUARD]
            // Xóa bỏ vĩnh viễn magic number 0.80 / leverage (-80% ROE)
            let cap_price_pct = playbook_max_sl_roe / leverage;
            let effective_cap = cap_price_pct;

            let mut calculated_sl = if trade.is_long() {
                let raw_sl = peak_val - (current_atr * current_multiplier);
                let min_long_sl = trade.entry_price * (1.0 - effective_cap);
                raw_sl.max(min_long_sl)
            } else {
                let raw_sl = peak_val + (current_atr * current_multiplier);
                let max_short_sl = trade.entry_price * (1.0 + effective_cap);
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

            let sqz_multiplier = matrix.vol_axis.get(i).copied().unwrap_or(0.0).max(0.0);
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

            // =========================================================================
            // 🎯 [FLAW-TSL.1: HYBRID TTP - NHÁNH A: DYNAMIC STRUCTURAL TAKE PROFIT]
            // =========================================================================
            // Cho các Playbook RANGE_OSC (Bollinger Bands) và REVERSAL (KAMA Mid).
            // Dời TP bám sát cấu trúc thị trường động thay vì bị đóng băng (Frozen Static TP).
            let mut new_tp_to_update = trade.tp_price;
            if tag_str == "RANGE_OSC" || tag_str == "REVERSAL" {
                let raw_tp = if tag_str == "RANGE_OSC" {
                    let tp_ratio = config.tactical_sniper.range.tp_target_ratio.clamp(0.10, 0.90);
                    let bb_width = (sensors.bb_upper[i] - sensors.bb_lower[i]).max(0.0);
                    if trade.is_long() {
                        sensors.bb_lower[i] + bb_width * tp_ratio
                    } else {
                        sensors.bb_upper[i] - bb_width * tp_ratio
                    }
                } else {
                    // REVERSAL: Mục tiêu phục hồi sóng hồi về KAMA Mid
                    sensors.kama_mid[i]
                };

                if !raw_tp.is_nan() && raw_tp > 0.0 {
                    let min_gain_pct = if risk_cfg.dynamic_tp_min_gain_pct > 0.0 {
                        risk_cfg.dynamic_tp_min_gain_pct
                    } else {
                        0.002
                    };

                    // CHỐT CHẶN 1: Min Profit Floor Guard (Chống dải BB co thắt làm chốt lời thành chốt lỗ)
                    let guarded_tp = if trade.is_long() {
                        let min_tp_floor = trade.entry_price * (1.0 + min_gain_pct);
                        raw_tp.max(min_tp_floor)
                    } else {
                        let max_tp_ceiling = trade.entry_price * (1.0 - min_gain_pct);
                        raw_tp.min(max_tp_ceiling)
                    };

                    // CHỐT CHẶN 2: Spread Guard (Đảm bảo khoảng cách an toàn với SL >= 0.5 ATR)
                    let spread_buffer = 0.5 * current_atr;
                    let final_tp = if trade.is_long() {
                        guarded_tp.max(calculated_sl + spread_buffer)
                    } else {
                        guarded_tp.min(calculated_sl - spread_buffer)
                    };

                    if !final_tp.is_nan() && final_tp > 0.0 {
                        if let Some(old_tp) = trade.tp_price {
                            if (final_tp - old_tp).abs() > 1e-4 && _debug_mode {
                                tracing::debug!(
                                    "🎯 [DYNAMIC TP UPDATED] Bar: {} | Trade #{:?} ({}) | Old TP: {:.2} -> New TP: {:.2} | Close: {:.2}",
                                    i, trade.id, tag_str, old_tp, final_tp, current_close
                                );
                            }
                        }
                        new_tp_to_update = Some(final_tp);
                    }
                }
            }

            // Kiểm tra xem Take Profit có thay đổi cần đồng bộ sang Broker / PaperTrader không
            let tp_changed = match (trade.tp_price, new_tp_to_update) {
                (Some(old_tp), Some(new_tp)) => (old_tp - new_tp).abs() > 1e-4,
                (None, Some(_)) | (Some(_), None) => true,
                (None, None) => false,
            };

            // [ENHANCEMENT 3: MONOTONIC RATCHET GUARD]
            // Bảo toàn Stop Loss 1 chiều tuyệt đối: SL chỉ được phép dời theo hướng bảo vệ vốn (Tiến, cấm Lùi).
            let underwater_cycles = bars_held / time_countdown_bars;
            if current_pnl > 0.0 {
                // KHI CÓ LÃI DANH NGHĨA (current_pnl > 0.0): Trailing theo đỉnh High Watermark & Dynamic BE & Mega Trend
                let sl_changed = if trade.is_long() {
                    calculated_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
                } else {
                    calculated_sl < trade.sl_price.unwrap_or(f64::INFINITY)
                };

                if sl_changed || tp_changed {
                    let sl_to_send = if sl_changed { Some(calculated_sl) } else { trade.sl_price };
                    tick.update_trade_sl_tp(trade.id, sl_to_send, new_tp_to_update);
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
                let sl_changed = if trade.is_long() {
                    safe_tightened_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
                } else {
                    safe_tightened_sl < trade.sl_price.unwrap_or(f64::INFINITY)
                };

                if sl_changed || tp_changed {
                    let sl_to_send = if sl_changed { Some(safe_tightened_sl) } else { trade.sl_price };
                    tick.update_trade_sl_tp(trade.id, sl_to_send, new_tp_to_update);
                }
            } else if tp_changed {
                // KHI LỆNH CHƯA CÓ LÃI HOẶC CHƯA BỊ NGÂM VỐN NHƯNG DẢI TP CÓ SỰ THAY ĐỔI:
                // Cập nhật Dynamic TP bám sát thị trường ngay lập tức, giữ nguyên mức SL hiện tại
                tick.update_trade_sl_tp(trade.id, trade.sl_price, new_tp_to_update);
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
        let max_sl_roe_cap = 0.20; // 20% ROE
        let min_initial_sl_pct = 0.002; // 0.2% giá
        let leverage = 20.0;

        // 1. SL quá xa (Trần khoảng cách giá = 20% ROE / 20x = 1.0% giá = 25 giá)
        let too_far_sl = 2300.0; // cách 200 giá = 8%
        let clamped = RiskManager::clamp_sl_price(price, too_far_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        let expected_min_sl = price * (1.0 - 0.01); // 2475.0
        assert_eq!(clamped, expected_min_sl, "SL quá xa phải bị kẹp bởi trần ROE Cap 20% trên 20x (1.0% giá)");

        // 2. SL quá gần (< 0.2% min_pct = 5 giá)
        let too_close_sl = 2499.0; // cách 1 giá = 0.04%
        let clamped_close = RiskManager::clamp_sl_price(price, too_close_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        let expected_max_sl = price * (1.0 - min_initial_sl_pct); // 2495.0
        assert_eq!(clamped_close, expected_max_sl, "SL quá gần phải bị kẹp bởi sàn min_pct");

        // 3. Universal Floor Guard khi đòn bẩy cao (50x, ROE Cap 4% -> 0.08% giá = 2 giá)
        // Trần ROE 4% (0.08% giá) luôn được bảo toàn tuyệt đối, Sàn co lại còn 50% trần (0.04% giá = 1 giá)
        let nuke_leverage = 50.0;
        let small_roe_cap = 0.04; // 4% ROE / 50x = 0.08% giá (2.0 giá)
        let clamped_nuke = RiskManager::clamp_sl_price(price, too_far_sl, is_long, small_roe_cap, min_initial_sl_pct, nuke_leverage);
        let expected_nuke_sl = price * (1.0 - 0.0008); // 2500 - 2 = 2498.0
        assert_eq!(clamped_nuke, expected_nuke_sl, "Universal Floor Guard bảo vệ trần ROE 4% (2498.0) tuyệt đối không bị đè bẹp thành 2495.0");
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

    #[test]
    fn test_dynamic_structural_tp_bollinger_and_guards() {
        let entry_price: f64 = 2500.0;
        let current_atr: f64 = 10.0;
        let min_gain_pct: f64 = 0.002; // 0.2% = 5.0 giá
        let calculated_sl: f64 = 2480.0;

        // 1. Bình thường: BB Upper mở rộng 2530.0 -> TP lấy 2530.0
        let bb_upper_wide: f64 = 2530.0;
        let min_tp_floor: f64 = entry_price * (1.0 + min_gain_pct); // 2505.0
        let spread_floor: f64 = calculated_sl + (0.5 * current_atr); // 2485.0
        let guarded_tp: f64 = bb_upper_wide.max(min_tp_floor).max(spread_floor);
        assert_eq!(guarded_tp, 2530.0, "Dải BB mở rộng thì lấy đúng giá trị dải");

        // 2. Co thắt nguy hiểm: BB Upper sập xuống 2495.0 (< entry_price)
        // Guard 1 phải can thiệp ép TP lên tối thiểu 2505.0 (min profit floor)
        let bb_upper_collapsed: f64 = 2495.0;
        let guarded_tp_collapsed: f64 = bb_upper_collapsed.max(min_tp_floor).max(spread_floor);
        assert_eq!(guarded_tp_collapsed, 2505.0, "Khi dải BB co thắt, Min Profit Floor Guard phải giữ TP luôn có lãi tối thiểu 0.2%");

        // 3. Spread Guard: Khi SL được dời lên rất cao (2504.0), spread_floor = 2504 + 5 = 2509.0
        let high_sl: f64 = 2504.0;
        let high_spread_floor: f64 = high_sl + (0.5 * current_atr);
        let final_tp: f64 = min_tp_floor.max(high_spread_floor);
        assert_eq!(final_tp, 2509.0, "Spread Guard phải đảm bảo khoảng cách an toàn với SL tối thiểu 0.5 ATR");
    }

    #[test]
    fn test_dynamic_structural_tp_reversal_kama_short() {
        let entry_price: f64 = 2500.0;
        let current_atr: f64 = 10.0;
        let min_gain_pct: f64 = 0.002; // 0.2%
        let calculated_sl: f64 = 2520.0;

        // Vị thế Short: KAMA Mid là mục tiêu chốt lời hồi quy
        let max_tp_ceiling: f64 = entry_price * (1.0 - min_gain_pct); // 2495.0
        let spread_ceiling: f64 = calculated_sl - (0.5 * current_atr); // 2515.0

        // 1. KAMA Mid bình thường: 2470.0 -> TP lấy 2470.0
        let kama_mid: f64 = 2470.0;
        let final_tp: f64 = kama_mid.min(max_tp_ceiling).min(spread_ceiling);
        assert_eq!(final_tp, 2470.0);

        // 2. KAMA Mid bị kéo lên cao 2505.0 (> entry) -> Max Profit Ceiling Guard phải kẹp tại 2495.0
        let kama_mid_bad: f64 = 2505.0;
        let guarded_bad: f64 = kama_mid_bad.min(max_tp_ceiling).min(spread_ceiling);
        assert_eq!(guarded_bad, 2495.0, "Short TP không bao giờ được cao hơn giá vốn trừ min_gain");
    }

    #[test]
    fn test_peak_callback_trailing_tp_trigger_math() {
        let entry_price: f64 = 2500.0;
        let leverage: f64 = 20.0;
        let current_atr: f64 = 10.0;
        let ttp_activation_roe: f64 = 0.20; // 20% ROE = 1% giá = 25.0 giá (đỉnh >= 2525.0)
        let ttp_callback_atr: f64 = 0.80; // 0.8 ATR = 8.0 giá
        let min_gain_pct: f64 = 0.002; // 0.2% = 5.0 giá (giá sàn chốt lời = 2505.0)

        // Đỉnh đạt 2540.0 -> ROE = ((2540 - 2500)/2500) * 20 = 32% (>= 20% kích hoạt)
        let peak_val: f64 = 2540.0;
        let peak_roe: f64 = ((peak_val - entry_price) / entry_price) * leverage;
        assert!(peak_roe >= ttp_activation_roe, "TTP phải được kích hoạt vì ROE đạt 32% >= 20%");

        let trigger_price: f64 = peak_val - (ttp_callback_atr * current_atr); // 2540 - 8 = 2532.0
        let min_exit_price: f64 = entry_price * (1.0 + min_gain_pct); // 2505.0

        // Khi giá ở 2535.0 (> 2532.0): Chưa chạm callback -> Giữ lệnh
        let close_holding: f64 = 2535.0;
        let should_exit_holding = close_holding <= trigger_price && close_holding >= min_exit_price;
        assert!(!should_exit_holding, "Chưa chạm trigger price thì tiếp tục thả diều gồng lãi");

        // Khi giá quay đầu rơi về 2531.0 (<= 2532.0): Kích hoạt chốt lời đỉnh
        let close_callback: f64 = 2531.0;
        let should_exit_callback = close_callback <= trigger_price && close_callback >= min_exit_price;
        assert!(should_exit_callback, "Giá quay đầu chạm 0.8 ATR từ đỉnh -> Kích hoạt thoát lệnh chốt lời");
    }

    #[test]
    fn test_flaw_tsl5_high_watermark_entry_bar_bias_free() {
        // Mô phỏng chuỗi giá sau khi vào lệnh Long tại entry_bar = 2, entry_price = 2500.0
        let entry_price = 2500.0;
        let entry_bar = 2;
        let i = 5;
        // highs: [0, 1, 2490.0 (entry), 2480.0, 2520.0, 2510.0]
        let highs = vec![0.0, 0.0, 2490.0, 2480.0, 2520.0, 2510.0];

        // Thuật toán FLAW-TSL.5: Neo từ entry_price, duyệt từ (entry_bar + 1)..=i
        let mut max_high = entry_price;
        let mut max_bar = entry_bar;
        for j in (entry_bar + 1)..=i {
            if highs[j] > max_high {
                max_high = highs[j];
                max_bar = j;
            }
        }

        assert_eq!(max_high, 2520.0, "Đỉnh cao nhất phải là 2520.0 tại bar 4");
        assert_eq!(max_bar, 4, "Bar đỉnh phải là bar 4");
        let effective_stagnant_bars = i - max_bar; // 5 - 4 = 1 bar
        assert_eq!(effective_stagnant_bars, 1, "Số nến đi ngang kể từ đỉnh là 1");

        // Trường hợp thị trường sập ngay sau nến entry: nến sau đỉnh chỉ đạt 2470.0
        let highs_bear = vec![0.0, 0.0, 2490.0, 2470.0, 2460.0, 2450.0];
        let mut max_high_bear = entry_price;
        let mut max_bar_bear = entry_bar;
        for j in (entry_bar + 1)..=i {
            if highs_bear[j] > max_high_bear {
                max_high_bear = highs_bear[j];
                max_bar_bear = j;
            }
        }
        assert_eq!(max_high_bear, 2500.0, "Đỉnh không bao giờ bị ghi nhận âm dưới giá vốn entry_price");
        assert_eq!(max_bar_bear, entry_bar, "Nếu không có nến nào vượt entry, peak_bar là entry_bar");
    }

    #[test]
    fn test_clamp_sl_price_spot_1x() {
        let price = 100.0;
        let is_long = true;
        let max_sl_roe_cap = 0.20; // 20% ROE / 1x = 20% giá -> SL xa nhất là 80.0
        let min_initial_sl_pct = 0.005; // 0.5% giá -> SL gần nhất là 99.5
        let leverage = 1.0;

        let too_far_sl = 50.0;
        let clamped = RiskManager::clamp_sl_price(price, too_far_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        assert_eq!(clamped, 80.0, "Với đòn bẩy Spot 1x, ROE Cap 20% tương đương 20% khoảng cách giá");

        let normal_sl = 90.0; // cách 10% giá
        let clamped_normal = RiskManager::clamp_sl_price(price, normal_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        assert_eq!(clamped_normal, 90.0, "SL hợp lệ phải được giữ nguyên");
    }

    #[test]
    fn test_clamp_sl_price_short_20x() {
        let price = 2000.0;
        let is_long = false;
        let max_sl_roe_cap = 0.25; // 25% ROE / 20x = 1.25% giá = 25.0 giá -> SL xa nhất Short là 2025.0
        let min_initial_sl_pct = 0.002; // 0.2% giá = 4.0 giá -> SL gần nhất Short là 2004.0
        let leverage = 20.0;

        let too_far_short_sl = 2100.0; // cách 100 giá
        let clamped = RiskManager::clamp_sl_price(price, too_far_short_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        assert_eq!(clamped, 2025.0, "Short SL quá xa phải bị kẹp bởi trần 25% ROE trên 20x (2025.0)");

        let too_close_short_sl = 2001.0; // cách 1 giá < 4 giá sàn
        let clamped_close = RiskManager::clamp_sl_price(price, too_close_short_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage);
        assert_eq!(clamped_close, 2004.0, "Short SL quá gần phải bị đẩy ra sàn an toàn 2004.0");
    }

    #[test]
    fn test_clamp_sl_price_extreme_100x() {
        let price = 50000.0; // BTC
        let is_long = true;
        let small_roe_cap = 0.04; // 4% ROE / 100x = 0.04% giá (20 giá)
        let min_initial_sl_pct = 0.0015; // 0.15% giá sàn (75 giá)
        let leverage = 100.0;

        let too_far_sl = 40000.0;
        let clamped = RiskManager::clamp_sl_price(price, too_far_sl, is_long, small_roe_cap, min_initial_sl_pct, leverage);
        // Universal Adaptive Floor Guard:
        // cap_price_pct = 0.0004 (20 giá = 4% ROE)
        // effective_min_pct = min(0.0015, 0.0004 * 0.5) = 0.0002 (10 giá = 2% ROE)
        // effective_cap = 0.0004 (20 giá = 4% ROE)
        let expected_sl = price * (1.0 - 0.0004); // 50000 * 0.9996 = 49980.0
        assert_eq!(clamped, expected_sl, "Đòn bẩy 100x cực đoan: Trần ROE 4% phải được bảo toàn tuyệt đối không bị đè bẹp");
    }

    #[test]
    fn test_clamp_sl_price_floor_guard_never_exceeds_roe_cap() {
        // Tái hiện chính xác tình huống Log 34 (Lệnh #003 RANGE_OSC)
        let price = 2447.37;
        let is_long = true;
        let range_roe_cap = 0.20; // 20% ROE trần cho Range
        let min_initial_sl_pct = 0.0355; // GA Optimizer chọn sàn spread 3.55% giá
        let leverage = 20.0;

        let raw_sl = 2360.49; // SL cách tới 3.55% giá (71% ROE)
        let clamped = RiskManager::clamp_sl_price(price, raw_sl, is_long, range_roe_cap, min_initial_sl_pct, leverage);
        
        // cap_price_pct = 0.20 / 20.0 = 0.010 (1.0% giá)
        // effective_cap = 0.010 -> SL xa nhất cho phép: 2447.37 * (1 - 0.01) = 2422.8963
        let expected_sl = price * (1.0 - 0.010);
        assert!((clamped - expected_sl).abs() < 1e-4, "SL bắt buộc phải bị chặn tại trần 20% ROE (2422.90), không được dạt ra 2360.49 (71% ROE)");
        
        let roe_loss = ((price - clamped) / price) * leverage;
        assert!((roe_loss - 0.20).abs() < 1e-4, "Mức lỗ tối đa ROE phải đúng bằng 20%");
    }

    #[test]
    fn test_fallback_hierarchy_untagged_trade_roe_cap() {
        let mut config = HeliosConfigV3::default();
        config.risk_management.stop_management.default_sl_roe_cap = 0.15; // 15% ROE
        config.tactical_sniper.pullback.max_initial_sl_pct = 0.30; // 30% ROE

        // Giả lập trade có tag TREND_PB: Phải lấy 30% ROE của Pullback
        let tagged_roe = match Some("TREND_PB") {
            Some("TREND_PB") => config.tactical_sniper.pullback.max_initial_sl_pct,
            _ => config.risk_management.stop_management.default_sl_roe_cap,
        };
        assert_eq!(tagged_roe, 0.30, "Lệnh có tag TREND_PB phải nhận đúng ROE Cap của scanner đó");

        // Giả lập trade không có tag (Adopted / Manual): Phải fallback về default_sl_roe_cap (15%)
        let untagged_trade_tag: Option<&str> = None;
        let fallback_roe = match untagged_trade_tag {
            Some("TREND_PB") => config.tactical_sniper.pullback.max_initial_sl_pct,
            _ => config.risk_management.stop_management.default_sl_roe_cap,
        };
        assert_eq!(fallback_roe, 0.15, "Lệnh không có tag phải tự động fallback về default_sl_roe_cap");
    }
}
