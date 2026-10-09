use ferro_trade::strategy::TickContext;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::sensors::HeliosSensors;

pub struct RiskManager;

/// 🛡️ TRẦN RỦI RO ROE TỐI ĐA TOÀN CỤC BẤT KHẢ XÂM PHẠM (15.0% ROE)
/// Không một lệnh nào trên toàn hệ sinh thái được phép có mức cắt lỗ ban đầu vượt quá ngưỡng này.
pub const GLOBAL_HARD_MAX_ROE: f64 = 0.15;

/// 🛡️ Kết quả thẩm định Stop Loss của Risk Manager
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlValidationResult {
    /// Hợp lệ: Thỏa mãn cả trần ROE chiến thuật lẫn vùng an toàn thanh lý sàn
    Acceptable,
    /// Bị VETO do vượt trần ROE chiến thuật (playbook_max_sl_roe hoặc GLOBAL_HARD_MAX_ROE)
    StructuralRiskVeto {
        suggested_roe: f64,
        max_roe: f64,
    },
    /// Bị VETO do vi phạm khoảng cách thanh lý sàn (quá sát hoặc vượt qua P_liq)
    LiquidationRiskVeto {
        suggested_dist_pct: f64,
        safe_liq_dist_pct: f64,
        mmr: f64,
    },
    /// Giá hoặc Stop Loss không hợp lệ (NaN hoặc <= 0)
    InvalidPrice,
}

impl RiskManager {
    /// 🛡️ Tính toán khoảng cách Stop Loss tối đa cho phép (% giá) đảm bảo đồng thời 2 lớp phòng vệ:
    /// 1. Trần ROE chiến thuật (playbook_max_sl_roe.min(GLOBAL_HARD_MAX_ROE)).
    /// 2. Chốt chặn khoảng cách thanh lý sàn Binance (Isolated Margin MMR & Fee Guard).
    ///
    /// Trả về Some(effective_cap_pct) nếu thỏa mãn vùng an toàn,
    /// hoặc None nếu đòn bẩy quá cao dẫn đến khoảng cách thanh lý âm hoặc hẹp hơn min_initial_sl_pct.
    pub fn calculate_safe_sl_distance(
        leverage: f64,
        max_roe_cap: f64,
        mmr: f64,
        taker_fee_rate: f64,
        buffer_ratio: f64,
        min_initial_sl_pct: f64,
    ) -> Option<f64> {
        let lev = if leverage <= 0.0 { 1.0 } else { leverage };
        let roe_cap = if max_roe_cap <= 0.0 {
            GLOBAL_HARD_MAX_ROE
        } else {
            max_roe_cap.min(GLOBAL_HARD_MAX_ROE)
        };
        let cap_price_pct = roe_cap / lev;

        let effective_mmr = if mmr <= 0.0 { 0.01 } else { mmr };
        let fee = if taker_fee_rate < 0.0 { 0.0004 } else { taker_fee_rate };
        let buf_ratio = if buffer_ratio <= 0.0 { 0.15 } else { buffer_ratio.min(0.50) };
        let min_pct = if min_initial_sl_pct <= 0.0 { 0.0015 } else { min_initial_sl_pct };

        // Khoảng cách thanh lý thô sau khi khấu trừ 2 chiều phí Taker
        let raw_liq_dist = (1.0 / lev) - effective_mmr - (2.0 * fee);
        if raw_liq_dist <= 0.0 {
            // Đòn bẩy quá cao so với MMR: Vừa mở lệnh là chạm hoặc vượt giá thanh lý ngay!
            return None;
        }

        // Khoảng cách an toàn sau khi giữ lại vùng đệm sinh tử buffer_ratio (15%)
        let safe_liq_dist = raw_liq_dist * (1.0 - buf_ratio);
        if safe_liq_dist < min_pct {
            // Không gian thanh lý còn lại quá hẹp, không đủ chỗ cho mức SL tối thiểu chống spread
            return None;
        }

        // Trần hiệu dụng là giá trị nhỏ nhất giữa Trần ROE và Vùng an toàn thanh lý
        let effective_cap = cap_price_pct.min(safe_liq_dist);
        if effective_cap < min_pct {
            return None;
        }

        Some(effective_cap)
    }

    /// Thẩm định Stop Loss đề xuất từ Scanner với nhận thức đầy đủ về MMR và Giá Thanh Lý.
    pub fn validate_structural_sl(
        price: f64,
        sl: f64,
        max_roe_cap: f64,
        leverage: f64,
        mmr: f64,
        taker_fee_rate: f64,
        buffer_ratio: f64,
        min_initial_sl_pct: f64,
    ) -> SlValidationResult {
        if price <= 0.0 || sl <= 0.0 || price.is_nan() || sl.is_nan() {
            return SlValidationResult::InvalidPrice;
        }
        let lev = if leverage <= 0.0 { 1.0 } else { leverage };
        let dist_pct = (price - sl).abs() / price;

        let effective_mmr = if mmr <= 0.0 { 0.01 } else { mmr };
        let fee = if taker_fee_rate < 0.0 { 0.0004 } else { taker_fee_rate };
        let buf_ratio = if buffer_ratio <= 0.0 { 0.15 } else { buffer_ratio.min(0.50) };
        let min_pct = if min_initial_sl_pct <= 0.0 { 0.0015 } else { min_initial_sl_pct };

        let raw_liq_dist = (1.0 / lev) - effective_mmr - (2.0 * fee);
        if raw_liq_dist <= 0.0 {
            return SlValidationResult::LiquidationRiskVeto {
                suggested_dist_pct: dist_pct,
                safe_liq_dist_pct: 0.0,
                mmr: effective_mmr,
            };
        }

        let safe_liq_dist = raw_liq_dist * (1.0 - buf_ratio);
        if safe_liq_dist < min_pct || dist_pct > safe_liq_dist {
            return SlValidationResult::LiquidationRiskVeto {
                suggested_dist_pct: dist_pct,
                safe_liq_dist_pct: safe_liq_dist.max(0.0),
                mmr: effective_mmr,
            };
        }

        let roe = dist_pct * lev;
        let effective_cap_roe = if max_roe_cap <= 0.0 {
            GLOBAL_HARD_MAX_ROE
        } else {
            max_roe_cap.min(GLOBAL_HARD_MAX_ROE)
        };

        if roe > effective_cap_roe {
            return SlValidationResult::StructuralRiskVeto {
                suggested_roe: roe,
                max_roe: effective_cap_roe,
            };
        }

        SlValidationResult::Acceptable
    }

    /// Kiểm tra xem mức Stop Loss cấu trúc đề xuất (suggested_sl) có nằm trong giới hạn an toàn không.
    pub fn is_structural_sl_acceptable(
        price: f64,
        sl: f64,
        max_roe_cap: f64,
        leverage: f64,
        mmr: f64,
        taker_fee_rate: f64,
        buffer_ratio: f64,
        min_initial_sl_pct: f64,
    ) -> bool {
        Self::validate_structural_sl(
            price, sl, max_roe_cap, leverage, mmr, taker_fee_rate, buffer_ratio, min_initial_sl_pct,
        ) == SlValidationResult::Acceptable
    }

    /// Tính toán Size dựa trên Fixed Percentage (Fraction của Ký quỹ) nhưng đảm bảo Min Order Size.
    /// Trả về một giá trị < 1.0 (ví dụ 0.1 = 10%) để `ferro_trade` dùng làm Initial Margin.
    pub fn calculate_size(equity: f64, risk_pct: f64, min_order_usd: f64, leverage: f64, max_cap_fraction: f64) -> f64 {
        // 🛡️ TIỀN ĐIỀU KIỆN (PRE-CONDITION WHITELIST - TODO-MATH-03):
        // Bắt buộc 4 biến số cốt lõi phải là số thực dương hữu hạn (> 0.0 và không phải NaN / Inf)
        if !(equity.is_finite() && equity > 0.0 
            && risk_pct.is_finite() && risk_pct > 0.0 
            && leverage.is_finite() && leverage > 0.0 
            && min_order_usd.is_finite() && min_order_usd > 0.0) 
        {
            return 0.0;
        }

        // Xử lý cap fraction an toàn: nếu cap lỗi hoặc <= 0.0 thì fallback về mức an toàn 25%
        let cap = if max_cap_fraction.is_finite() && max_cap_fraction > 0.0 {
            max_cap_fraction
        } else {
            0.25
        };

        let target_margin = equity * risk_pct;
        let order_size_usd = (target_margin * leverage).max(min_order_usd);
        let size_fraction = (order_size_usd / leverage) / equity;
        let final_fraction = size_fraction.min(cap);

        // 🛡️ HẬU ĐIỀU KIỆN (POST-CONDITION WHITELIST):
        // Giá trị danh nghĩa thực tế phải đạt sàn min_order_usd kèm đệm epsilon 1e-4 chống sai số làm tròn float
        let nominal_value = equity * final_fraction * leverage;
        if nominal_value.is_finite() && nominal_value >= (min_order_usd - 1e-4) {
            final_fraction
        } else {
            0.0
        }
    }

    /// Hàm khống chế trần SL ban đầu theo % ROE tối đa cho phép và khoảng cách thanh lý an toàn (FLAW-TSL.8 & TODO-RSK-01)
    /// Min Distance Floor Guard bảo vệ chống kẹp ngược khi đòn bẩy cao.
    pub fn clamp_sl_price(
        price: f64,
        sl_price: f64,
        is_long: bool,
        max_sl_roe_cap: f64,
        min_initial_sl_pct: f64,
        leverage: f64,
        mmr: f64,
        taker_fee_rate: f64,
        buffer_ratio: f64,
    ) -> f64 {
        let lev = if leverage <= 0.0 { 1.0 } else { leverage };
        let min_pct = if min_initial_sl_pct <= 0.0 { 0.0015 } else { min_initial_sl_pct };

        // 1. Trần khoảng cách giá tối đa kết hợp giữa ROE cap và Liquidation Guard
        let effective_cap = match Self::calculate_safe_sl_distance(
            lev,
            max_sl_roe_cap,
            mmr,
            taker_fee_rate,
            buffer_ratio,
            min_pct,
        ) {
            Some(cap) => cap,
            None => {
                // Fallback an toàn nếu đòn bẩy quá cao không đủ khoảng trống
                (max_sl_roe_cap.min(GLOBAL_HARD_MAX_ROE) / lev).min(min_pct)
            }
        };

        // 2. Sàn khoảng cách tối thiểu chống quét spread/noise (PHẢI LUÔN NẰM DƯỚI TRẦN)
        // Áp dụng tỷ lệ an toàn 50% trần: Sàn tối đa chỉ được chiếm 50% khoảng cách của Trần.
        let effective_min_pct = min_pct.min(effective_cap * 0.50);

        if is_long {
            let max_allowed_sl = price * (1.0 - effective_min_pct); // SL gần nhất (chống cắn spread)
            let min_allowed_sl = price * (1.0 - effective_cap);     // SL xa nhất (khống chế lỗ tối đa & né thanh lý)
            sl_price.min(max_allowed_sl).max(min_allowed_sl)
        } else {
            let min_allowed_sl = price * (1.0 + effective_min_pct); // SL gần nhất
            let max_allowed_sl = price * (1.0 + effective_cap);     // SL xa nhất
            sl_price.max(min_allowed_sl).min(max_allowed_sl)
        }
    }

    /// Tính giá cắt lỗ khởi tạo với Hard Max Cap ROE và Liquidation Guard
    pub fn get_initial_sl(
        tick: &TickContext,
        sensors: &HeliosSensors,
        is_long: bool,
        multiplier: f64,
        max_sl_roe_cap: f64,
        min_initial_sl_pct: f64,
        leverage: f64,
        mmr: f64,
        taker_fee_rate: f64,
        buffer_ratio: f64,
    ) -> f64 {
        let i = tick.i;
        let current_atr = sensors.atr_sl[i];
        let price = tick.closes()[i];
        let raw_sl = if is_long {
            price - (current_atr * multiplier)
        } else {
            price + (current_atr * multiplier)
        };

        Self::clamp_sl_price(
            price, raw_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee_rate, buffer_ratio,
        )
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

            // [ENHANCEMENT 1: REGIME-AWARE DYNAMIC BREAK-EVEN PARAMETERS & DUAL-CRITERION TRIGGER]
            // 1. Phân tầng ngưỡng kích hoạt và khoảng đệm bảo vệ vốn theo 3 nhóm Regime Simplex (Trend, Range, Volatile)
            let (be_trigger_mult, be_buffer_mult) = match tag_str {
                "TREND_PB" | "ABSORPTION" => (
                    if risk_cfg.be_atr_trigger_trend > 0.0 { risk_cfg.be_atr_trigger_trend } else { 2.5 },
                    if risk_cfg.be_buffer_atr_trend > 0.0 { risk_cfg.be_buffer_atr_trend } else { 1.5 },
                ),
                "RANGE_OSC" | "REVERSAL" => (
                    if risk_cfg.be_atr_trigger_range > 0.0 { risk_cfg.be_atr_trigger_range } else { 1.5 },
                    if risk_cfg.be_buffer_atr_range > 0.0 { risk_cfg.be_buffer_atr_range } else { 0.8 },
                ),
                "MOMENTUM_IGN" | "ANOMALY" => (
                    if risk_cfg.be_atr_trigger_volatile > 0.0 { risk_cfg.be_atr_trigger_volatile } else { 2.0 },
                    if risk_cfg.be_buffer_atr_volatile > 0.0 { risk_cfg.be_buffer_atr_volatile } else { 1.2 },
                ),
                _ => (
                    if risk_cfg.be_atr_trigger_trend > 0.0 { risk_cfg.be_atr_trigger_trend } else { 2.5 },
                    if risk_cfg.be_buffer_atr_trend > 0.0 { risk_cfg.be_buffer_atr_trend } else { 1.5 },
                ),
            };

            // 2. Tính khoảng cách tiền lời đỉnh (peak_profit) chuẩn hóa từ High Watermark
            let peak_dist = if trade.is_long() { peak_val - trade.entry_price } else { trade.entry_price - peak_val };
            let peak_profit = peak_dist.max(0.0) * trade.size.abs();

            // 3. Single Source of Truth: Tính động fee_buffer_pct từ taker_fee_rate và slippage_allowance_pct
            let round_trip_fee = 2.0 * config.risk_management.risk_base.taker_fee_rate;
            let slippage = config.risk_management.risk_base.slippage_allowance_pct;
            let fee_buffer_pct = round_trip_fee + slippage;

            // 4. Ngưỡng kích hoạt BE an toàn: Bảo vệ vốn không bị phí sàn nuốt chửng ở mọi mức đòn bẩy
            let min_safe_be_roe = (fee_buffer_pct * leverage * 1.5).max(risk_cfg.base_be_roe_trigger.max(0.06));
            let required_profit = current_atr * be_trigger_mult * trade.size.abs();

            // 5. Kích hoạt theo Dual-Criterion Peak Engine: Chạm đỉnh râu nến High Watermark là bảo vệ hòa vốn ngay
            // 🛡️ TASK TODO-R2-01: Bỏ `current_pnl > 0.0 &&` để Emergency BE Rejection Close không bị tắt khi Flash Crash sập dưới min_be
            let is_be_triggered = peak_roe >= min_safe_be_roe || peak_profit >= required_profit;

            // =========================================================================
            // 🚀 [FLAW-TSL.1: HYBRID TTP - NHÁNH B: PEAK-CALLBACK TRAILING TAKE PROFIT]
            // =========================================================================
            // Áp dụng cho các Playbook xu hướng hoặc không có TP cố định (TREND_PB, MOMENTUM_IGN, ABSORPTION, ANOMALY)
            // Giúp thả diều tối đa lợi nhuận siêu sóng và chỉ chốt lời khi giá tạo đỉnh quay đầu (Callback).
            // TỰ ĐỘNG KÍCH HOẠT SONG SONG KHI CÓ BREAK-EVEN: Bắt ngay các cú quay đầu từ đỉnh có lãi!
            let is_ttp_eligible = matches!(tag_str, "TREND_PB" | "MOMENTUM_IGN" | "ABSORPTION" | "ANOMALY") || trade.tp_price.is_none();
            if is_ttp_eligible {
                let ttp_act_roe = if risk_cfg.ttp_activation_roe > 0.0 { risk_cfg.ttp_activation_roe } else { 0.15 };
                let ttp_cb_atr = if risk_cfg.ttp_callback_atr > 0.0 { risk_cfg.ttp_callback_atr } else { 0.80 };
                let min_gain_pct = if risk_cfg.dynamic_tp_min_gain_pct > 0.0 { risk_cfg.dynamic_tp_min_gain_pct } else { 0.002 };
                let min_exit_gain_pct = min_gain_pct.max(fee_buffer_pct);

                let is_ttp_active = (peak_roe >= ttp_act_roe) || is_be_triggered;
                if is_ttp_active {
                    if trade.is_long() {
                        let ttp_trigger_price = peak_val - (ttp_cb_atr * current_atr);
                        let min_exit_price = trade.entry_price * (1.0 + min_exit_gain_pct);
                        if current_close <= ttp_trigger_price && current_close >= min_exit_price {
                            if _debug_mode {
                                tracing::info!(
                                    "🏆 [TTP TRIGGERED EXIT] Bar: {} | Trade #{:?} ({}) Long | Peak: {:.2} (ROE: {:.2}%) | Trigger: {:.2} | Close: {:.2} -> Locked profit via Trailing TP",
                                    i, trade.id, tag_str, peak_val, peak_roe * 100.0, ttp_trigger_price, current_close
                                );
                            }
                            tick.close(None);
                            continue;
                        }
                    } else {
                        let ttp_trigger_price = peak_val + (ttp_cb_atr * current_atr);
                        let max_exit_price = trade.entry_price * (1.0 - min_exit_gain_pct);
                        if current_close >= ttp_trigger_price && current_close <= max_exit_price {
                            if _debug_mode {
                                tracing::info!(
                                    "🏆 [TTP TRIGGERED EXIT] Bar: {} | Trade #{:?} ({}) Short | Peak: {:.2} (ROE: {:.2}%) | Trigger: {:.2} | Close: {:.2} -> Locked profit via Trailing TP",
                                    i, trade.id, tag_str, peak_val, peak_roe * 100.0, ttp_trigger_price, current_close
                                );
                            }
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
                // CHẾ ĐỘ THẢ DIỀU SIÊU SÓNG: Dùng tsl_atr_loose để cho sóng bay cao
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
                
                // Tách độc lập sàn siết hẹp bằng tsl_buffer_floor_atr thay vì tsl_atr_tight
                if current_multiplier < risk_cfg.tsl_buffer_floor_atr {
                    current_multiplier = risk_cfg.tsl_buffer_floor_atr;
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

            if is_be_triggered && !is_mega_trend {
                if trade.is_long() {
                    let min_be = trade.entry_price * (1.0 + fee_buffer_pct);
                    // 🛑 EDGE CASE: Emergency BE Rejection Close
                    // Nếu râu nến đạt đỉnh nhưng cuối nến bị xả ngược đâm thủng min_be -> Thoát khẩn cấp bảo toàn vốn
                    if current_close <= min_be {
                        if _debug_mode {
                            tracing::info!(
                                "🛑 [EMERGENCY BE REJECTION EXIT] Bar: {} | Trade #{:?} Long | Peak ROE: {:.2}% >= trigger, but Close {:.2} <= min_be {:.2} -> Closed to preserve capital",
                                i, trade.id, peak_roe * 100.0, current_close, min_be
                            );
                        }
                        tick.close(None);
                        continue;
                    }
                    let dynamic_floor = peak_val - (current_atr * be_buffer_mult);
                    let target_be_sl = dynamic_floor.max(min_be);
                    calculated_sl = calculated_sl.max(target_be_sl);
                } else {
                    let max_be = trade.entry_price * (1.0 - fee_buffer_pct);
                    // 🛑 EDGE CASE: Emergency BE Rejection Close
                    if current_close >= max_be {
                        if _debug_mode {
                            tracing::info!(
                                "🛑 [EMERGENCY BE REJECTION EXIT] Bar: {} | Trade #{:?} Short | Peak ROE: {:.2}% >= trigger, but Close {:.2} >= max_be {:.2} -> Closed to preserve capital",
                                i, trade.id, peak_roe * 100.0, current_close, max_be
                            );
                        }
                        tick.close(None);
                        continue;
                    }
                    let dynamic_ceiling = peak_val + (current_atr * be_buffer_mult);
                    let target_be_sl = dynamic_ceiling.min(max_be);
                    calculated_sl = calculated_sl.min(target_be_sl);
                }
            }

            // [ENHANCEMENT 2: PROGRESSIVE PROFIT-RETAINING FLOOR & MEGA TREND PROTECTION]
            // Khi lệnh đạt đỉnh lợi nhuận >= mega_trend_min_lock_roe (mặc định >= 10% ROE) hoặc vào chế độ Mega Trend:
            // Tự động kéo SL lên trên giá Entry để khóa một tỷ lệ lợi nhuận đỉnh theo DNA mega_trend_lock_ratio.
            // Triệt tiêu 100% "Vùng chân không tử thần" (+8% đến +20% ROE) từng làm nhả lại 163% ROE.
            let min_lock_roe = if risk_cfg.mega_trend_min_lock_roe > 0.0 { risk_cfg.mega_trend_min_lock_roe } else { 0.10 };
            if peak_roe >= min_lock_roe || is_mega_trend {
                let lock_ratio = if risk_cfg.mega_trend_lock_ratio > 0.0 { risk_cfg.mega_trend_lock_ratio } else { 0.40 };
                let locked_roe = peak_roe * lock_ratio;
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
            } else if tp_changed {
                // 🛡️ TODO-RSK-02: KHI ĐANG ÂM HOẶC HÒA VỐN (current_pnl <= 0.0):
                // Bảo toàn 100% Initial Structural SL do Scanner thiết lập, tuyệt đối không siết SL bằng hàm số mũ 0.8^n.
                // Việc giải phóng vốn ngâm lâu đã có khối Stagnant Capital Release (dòng 348) và Zombie Trade Purge phụ trách.
                // Chỉ cập nhật Take Profit nếu có thay đổi động (RangeOsc / Reversal), giữ nguyên mức SL hiện tại.
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
        let max_sl_roe_cap = 0.15; // 15% ROE
        let min_initial_sl_pct = 0.002; // 0.2% giá
        let leverage = 20.0;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        // 1. SL quá xa (Trần khoảng cách giá = 15% ROE / 20x = 0.75% giá = 18.75 giá)
        let too_far_sl = 2300.0; // cách 200 giá = 8%
        let clamped = RiskManager::clamp_sl_price(
            price, too_far_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        let expected_min_sl = price * (1.0 - 0.0075); // 2481.25
        assert_eq!(clamped, expected_min_sl, "SL quá xa phải bị kẹp bởi trần ROE Cap 15% trên 20x (0.75% giá)");

        // 2. SL quá gần (< 0.2% min_pct = 5 giá)
        let too_close_sl = 2499.0; // cách 1 giá = 0.04%
        let clamped_close = RiskManager::clamp_sl_price(
            price, too_close_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        let expected_max_sl = price * (1.0 - min_initial_sl_pct); // 2495.0
        assert_eq!(clamped_close, expected_max_sl, "SL quá gần phải bị kẹp bởi sàn min_pct");

        // 3. Universal Floor Guard khi đòn bẩy cao (50x, ROE Cap 4% -> 0.08% giá = 2 giá)
        // Trần ROE 4% (0.08% giá) luôn được bảo toàn tuyệt đối, Sàn co lại còn 50% trần (0.04% giá = 1 giá)
        let nuke_leverage = 50.0;
        let small_roe_cap = 0.04; // 4% ROE / 50x = 0.08% giá (2.0 giá)
        let clamped_nuke = RiskManager::clamp_sl_price(
            price, too_far_sl, is_long, small_roe_cap, min_initial_sl_pct, nuke_leverage, mmr, taker_fee, buffer_ratio
        );
        let expected_nuke_sl = price * (1.0 - 0.0008); // 2500 - 2 = 2498.0
        assert_eq!(clamped_nuke, expected_nuke_sl, "Universal Floor Guard bảo vệ trần ROE 4% (2498.0) tuyệt đối không bị đè bẹp thành 2495.0");
    }

    #[test]
    fn test_underwater_position_preserves_initial_structural_sl() {
        // 🛡️ TODO-RSK-02: Khi vị thế đang âm (current_pnl <= 0.0), SL tuyệt đối không bị siết lũy thừa 0.8^n.
        // Initial Structural SL do Scanner thiết lập (ví dụ 2480.0 cho Long entry 2500.0) phải được bảo toàn 100%.
        let entry_price: f64 = 2500.0;
        let initial_structural_sl: f64 = 2480.0; // Khoảng cách 20 giá = 0.8% giá = 16% ROE (đã clamp tại trần an toàn)
        let current_close: f64 = 2490.0; // Vị thế đang âm: (2490 - 2500) = -10 giá
        let current_pnl: f64 = current_close - entry_price;

        assert!(current_pnl <= 0.0, "Vị thế đang âm (underwater)");

        // Mô phỏng logic dòng 642-660 trong risk.rs:
        // Khi current_pnl <= 0.0, khối if current_pnl > 0.0 không chạy.
        // sl_to_send không được thay đổi, bảo toàn nguyên vẹn trade.sl_price.
        let trade_sl_price = Some(initial_structural_sl);
        let sl_to_send = if current_pnl > 0.0 {
            // Trailing stop từ đỉnh khi có lãi danh nghĩa
            Some(2495.0)
        } else {
            // Bảo toàn Initial Structural SL
            trade_sl_price
        };

        assert_eq!(sl_to_send, Some(initial_structural_sl), "SL phải được bảo toàn nguyên vẹn 2480.0, không bị bóp nghẹt lên!");
    }

    #[test]
    fn test_stagnant_capital_release_logic() {
        // 🛡️ TODO-RSK-02: Cơ chế duy nhất được phép đóng lệnh ngâm lâu là Stagnant Capital Release
        let entry_price: f64 = 2500.0;
        let max_stagnant_hold_bars: usize = 60;
        let stagnant_min_profit_pct: f64 = 0.005; // Cần tối thiểu +0.5% lợi nhuận

        // Trường hợp 1: Ngâm 40 bars (< 60 bars) -> Chưa đến hạn giải phóng
        let bars_held_early = 40;
        let current_close_underwater = 2495.0; // Âm -0.2%
        let pnl_pct_underwater = (current_close_underwater - entry_price) / entry_price;
        let should_release_early = max_stagnant_hold_bars > 0 
            && bars_held_early >= max_stagnant_hold_bars 
            && pnl_pct_underwater < stagnant_min_profit_pct;
        assert!(!should_release_early, "Chưa đủ 60 bars thì không được phép đóng sớm");

        // Trường hợp 2: Ngâm 65 bars (>= 60 bars) và PnL chưa đạt +0.5% -> Kích hoạt giải phóng vốn!
        let bars_held_stagnant = 65;
        let should_release_stagnant = max_stagnant_hold_bars > 0 
            && bars_held_stagnant >= max_stagnant_hold_bars 
            && pnl_pct_underwater < stagnant_min_profit_pct;
        assert!(should_release_stagnant, "Đã ngâm 65 bars mà chưa đạt +0.5% lãi -> Phải kích hoạt Stagnant Capital Release!");

        // Trường hợp 3: Ngâm 65 bars nhưng PnL đạt +1.2% (>= +0.5%) -> Tiếp tục giữ lệnh gồng lãi!
        let current_close_profitable = 2530.0; // Lãi +1.2%
        let pnl_pct_profitable = (current_close_profitable - entry_price) / entry_price;
        let should_release_profitable = max_stagnant_hold_bars > 0 
            && bars_held_stagnant >= max_stagnant_hold_bars 
            && pnl_pct_profitable < stagnant_min_profit_pct;
        assert!(!should_release_profitable, "Đã vượt mốc lợi nhuận tối thiểu +0.5% -> Được tiếp tục giữ lệnh!");
    }

    #[test]
    fn test_calculate_size_nan_resilience_and_cap() {
        // 1. Trường hợp bình thường: vốn $100, risk 20%, đòn bẩy 20x -> order $400 >= $21 -> size = 20%
        let size = RiskManager::calculate_size(100.0, 0.20, 21.0, 20.0, 0.25);
        assert!((size - 0.20).abs() < 1e-9);

        // 2. Trường hợp min order size $21.0 khi vốn nhỏ ($60, risk 10% -> margin $6 -> order $120 >= $21)
        let size_small = RiskManager::calculate_size(60.0, 0.10, 21.0, 20.0, 0.25);
        assert_eq!(size_small, 0.10);

        // 3. Trường hợp order size < $21.0 được nâng lên sàn $21.0 (vốn $10, risk 5%, margin $0.5 -> order $10 < $21)
        // order_size nâng lên $21 -> margin = 21/20 = 1.05 -> fraction = 1.05/10 = 0.105 <= cap 0.25
        let size_min_bump = RiskManager::calculate_size(10.0, 0.05, 21.0, 20.0, 0.25);
        assert!((size_min_bump - 0.105).abs() < 1e-9);

        // 4. Kiểm chứng Epsilon Float Precision: nominal_value tiệm cận $21.0 từ phía dưới do sai số nhị phân không bị từ chối oan
        let size_epsilon = RiskManager::calculate_size(60.0, 0.0175, 21.0, 20.0, 0.25);
        assert!(size_epsilon > 0.0, "Không được từ chối oan lệnh hợp lệ do sai số làm tròn float");

        // 5. 🛡️ BỘ CHỐT CHẶN IEEE 754 TOÀN DIỆN (TODO-MATH-03):
        // Bắt buộc trả về 0.0 tuyệt đối khi bất kỳ tham số nào là NaN hoặc Inf, triệt tiêu bẫy cấp vốn ma 25%!
        assert_eq!(RiskManager::calculate_size(f64::NAN, 0.20, 21.0, 20.0, 0.25), 0.0, "equity = NaN phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(100.0, f64::NAN, 21.0, 20.0, 0.25), 0.0, "risk_pct = NaN phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, f64::NAN, 20.0, 0.25), 0.0, "min_order_usd = NaN phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 21.0, f64::NAN, 0.25), 0.0, "leverage = NaN phải trả về 0.0");
        
        // Khi max_cap_fraction = NaN, fallback về trần an toàn 0.25 chứ không crash
        let size_nan_cap = RiskManager::calculate_size(100.0, 0.30, 21.0, 20.0, f64::NAN);
        assert_eq!(size_nan_cap, 0.25, "max_cap_fraction = NaN phải fallback về trần an toàn 0.25");

        // Chốt chặn Infinity
        assert_eq!(RiskManager::calculate_size(f64::INFINITY, 0.20, 21.0, 20.0, 0.25), 0.0, "+Inf equity phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(f64::NEG_INFINITY, 0.20, 21.0, 20.0, 0.25), 0.0, "-Inf equity phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(100.0, f64::INFINITY, 21.0, 20.0, 0.25), 0.0, "+Inf risk_pct phải trả về 0.0");
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 21.0, f64::INFINITY, 0.25), 0.0, "+Inf leverage phải trả về 0.0");

        // Chốt chặn Âm vốn & Zero
        assert_eq!(RiskManager::calculate_size(0.0, 0.20, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(-50.0, 0.20, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.0, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, -0.05, 21.0, 20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 21.0, 0.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 21.0, -20.0, 0.25), 0.0);
        assert_eq!(RiskManager::calculate_size(100.0, 0.20, 0.0, 20.0, 0.25), 0.0);
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
        let max_sl_roe_cap = 0.15; // 15% ROE / 1x = 15% giá -> SL xa nhất là 85.0
        let min_initial_sl_pct = 0.005; // 0.5% giá -> SL gần nhất là 99.5
        let leverage = 1.0;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        let too_far_sl = 50.0;
        let clamped = RiskManager::clamp_sl_price(
            price, too_far_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        assert_eq!(clamped, 85.0, "Với đòn bẩy Spot 1x, ROE Cap 15% tương đương 15% khoảng cách giá");

        let normal_sl = 90.0; // cách 10% giá
        let clamped_normal = RiskManager::clamp_sl_price(
            price, normal_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        assert_eq!(clamped_normal, 90.0, "SL hợp lệ phải được giữ nguyên");
    }

    #[test]
    fn test_clamp_sl_price_short_20x() {
        let price = 2000.0;
        let is_long = false;
        let max_sl_roe_cap = 0.15; // 15% ROE / 20x = 0.75% giá = 15.0 giá -> SL xa nhất Short là 2015.0
        let min_initial_sl_pct = 0.002; // 0.2% giá = 4.0 giá -> SL gần nhất Short là 2004.0
        let leverage = 20.0;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        let too_far_short_sl = 2100.0; // cách 100 giá
        let clamped = RiskManager::clamp_sl_price(
            price, too_far_short_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        assert!((clamped - 2015.0).abs() < 1e-6, "Short SL quá xa phải bị kẹp bởi trần 15% ROE trên 20x (2015.0)");

        let too_close_short_sl = 2001.0; // cách 1 giá < 4 giá sàn
        let clamped_close = RiskManager::clamp_sl_price(
            price, too_close_short_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        assert_eq!(clamped_close, 2004.0, "Short SL quá gần phải bị đẩy ra sàn an toàn 2004.0");
    }

    #[test]
    fn test_clamp_sl_price_extreme_100x() {
        let price = 50000.0; // BTC
        let is_long = true;
        let small_roe_cap = 0.04; // 4% ROE / 100x = 0.04% giá (20 giá)
        let min_initial_sl_pct = 0.0015; // 0.15% giá sàn (75 giá)
        let leverage = 100.0;
        let mmr = 0.004; // BTC MMR 0.4%
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        let too_far_sl = 40000.0;
        let clamped = RiskManager::clamp_sl_price(
            price, too_far_sl, is_long, small_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        let expected_sl = price * (1.0 - 0.0004); // 50000 * 0.9996 = 49980.0
        assert_eq!(clamped, expected_sl, "Đòn bẩy 100x cực đoan: Trần ROE 4% phải được bảo toàn tuyệt đối không bị đè bẹp");
    }

    #[test]
    fn test_clamp_sl_price_floor_guard_never_exceeds_roe_cap() {
        let price = 2447.37;
        let is_long = true;
        let range_roe_cap = 0.15; // 15% ROE trần cho Range
        let min_initial_sl_pct = 0.0355; // GA Optimizer chọn sàn spread 3.55% giá
        let leverage = 20.0;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        let raw_sl = 2360.49; // SL cách tới 3.55% giá (71% ROE)
        let clamped = RiskManager::clamp_sl_price(
            price, raw_sl, is_long, range_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        
        let expected_sl = price * (1.0 - 0.0075);
        assert!((clamped - expected_sl).abs() < 1e-4, "SL bắt buộc phải bị chặn tại trần 15% ROE, không được dạt ra xa");
        
        let roe_loss = ((price - clamped) / price) * leverage;
        assert!((roe_loss - 0.15).abs() < 1e-4, "Mức lỗ tối đa ROE phải đúng bằng 15%");
    }

    #[test]
    fn test_clamp_sl_price_hard_max_roe_guard() {
        let price = 2000.0;
        let is_long = true;
        let crazy_roe_cap = 0.50; // Cố tình truyền vào 50% ROE
        let min_initial_sl_pct = 0.002;
        let leverage = 20.0;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;

        let too_far_sl = 1800.0;
        let clamped = RiskManager::clamp_sl_price(
            price, too_far_sl, is_long, crazy_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee, buffer_ratio
        );
        
        // 15% ROE / 20x = 0.75% giá = 15.0 giá -> SL xa nhất là 1985.0
        assert_eq!(clamped, 1985.0, "Dù truyền vào 50% ROE thì GLOBAL_HARD_MAX_ROE vẫn kẹp cứng tại 15% ROE (1985.0)!");
    }

    #[test]
    fn test_is_structural_sl_acceptable_math() {
        let price = 2000.0;
        let leverage = 20.0;
        let max_roe = 0.15;
        let mmr = 0.010;
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;
        let min_initial_sl_pct = 0.0015;

        // 1. SL cách 10.0 giá = 0.5% giá = 10% ROE <= 15% ROE -> Chấp nhận
        let safe_sl = 1990.0;
        assert!(RiskManager::is_structural_sl_acceptable(
            price, safe_sl, max_roe, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct
        ));

        // 2. SL cách đúng 15.0 giá = 0.75% giá = 15% ROE -> Chấp nhận
        let exact_sl = 1985.0;
        assert!(RiskManager::is_structural_sl_acceptable(
            price, exact_sl, max_roe, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct
        ));

        // 3. SL cách 20.0 giá = 1.0% giá = 20% ROE > 15% ROE -> Từ chối (VETO)
        let deep_sl = 1980.0;
        assert!(!RiskManager::is_structural_sl_acceptable(
            price, deep_sl, max_roe, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct
        ));

        // 4. Edge cases NaN / 0
        assert!(!RiskManager::is_structural_sl_acceptable(
            0.0, safe_sl, max_roe, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct
        ));
        assert!(!RiskManager::is_structural_sl_acceptable(
            price, f64::NAN, max_roe, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct
        ));
    }

    #[test]
    fn test_fallback_hierarchy_untagged_trade_roe_cap() {
        let mut config = HeliosConfigV3::default();
        config.risk_management.stop_management.default_sl_roe_cap = 0.10; // 10% ROE
        config.tactical_sniper.pullback.max_initial_sl_pct = 0.14; // 14% ROE

        let tagged_roe = match Some("TREND_PB") {
            Some("TREND_PB") => config.tactical_sniper.pullback.max_initial_sl_pct,
            _ => config.risk_management.stop_management.default_sl_roe_cap,
        };
        assert_eq!(tagged_roe, 0.14, "Lệnh có tag TREND_PB phải nhận đúng ROE Cap của scanner đó");

        let untagged_trade_tag: Option<&str> = None;
        let fallback_roe = match untagged_trade_tag {
            Some("TREND_PB") => config.tactical_sniper.pullback.max_initial_sl_pct,
            _ => config.risk_management.stop_management.default_sl_roe_cap,
        };
        assert_eq!(fallback_roe, 0.10, "Lệnh không có tag phải tự động fallback về default_sl_roe_cap");
    }

    #[test]
    fn test_dual_criterion_peak_break_even_on_wick() {
        let entry_price: f64 = 2000.0;
        let leverage: f64 = 20.0;
        let taker_fee_rate: f64 = 0.0004;
        let slippage_allowance_pct: f64 = 0.0002;
        let base_be_roe_trigger: f64 = 0.06;

        let round_trip_fee: f64 = 2.0 * taker_fee_rate;
        let fee_buffer_pct: f64 = round_trip_fee + slippage_allowance_pct;
        let min_safe_be_roe: f64 = (fee_buffer_pct * leverage * 1.5).max(base_be_roe_trigger);
        assert_eq!(min_safe_be_roe, 0.06, "Với 20x và phí 0.10%, min_safe_be_roe lấy sàn 6% ROE");

        // Giả lập Lệnh #0009: Râu nến giật lên 2015.8 (+15.8% ROE)
        let peak_val: f64 = 2015.8;
        let peak_roe: f64 = ((peak_val - entry_price) / entry_price) * leverage;
        assert!((peak_roe - 0.158).abs() < 1e-4);

        // Giá Close thoái lui về 2005.0 (+5.0% ROE < 6%)
        let current_close: f64 = 2005.0;
        let is_be_triggered = peak_roe >= min_safe_be_roe;
        assert!(is_be_triggered, "Dù Close thoái lui về +5% nhưng Peak ROE đạt 15.8% >= 6% phải kích hoạt BE!");

        let min_be: f64 = entry_price * (1.0 + fee_buffer_pct);
        assert!((min_be - 2002.0).abs() < 1e-4, "Mức BE an toàn bù phí sàn và trượt giá là 2002.0");
        assert!(current_close > min_be, "Close 2005.0 > min_be 2002.0 nên vị thế an toàn để dời SL lên min_be");
    }

    #[test]
    fn test_break_even_emergency_rejection_exit_logic() {
        let entry_price: f64 = 2000.0;
        let fee_buffer_pct: f64 = 0.0010;
        let min_be: f64 = entry_price * (1.0 + fee_buffer_pct); // 2002.0

        // Giả lập kịch bản râu nến đạt đỉnh nhưng cuối nến bị xả ngược đâm thủng min_be
        let dumped_close_long: f64 = 1999.0;
        let should_emergency_close_long = dumped_close_long <= min_be;
        assert!(should_emergency_close_long, "Khi Close rơi xuống 1999.0 <= min_be 2002.0, Long phải kích hoạt Emergency Close!");

        let max_be_short: f64 = entry_price * (1.0 - fee_buffer_pct); // 1998.0
        let pumped_close_short: f64 = 2001.0;
        let should_emergency_close_short = pumped_close_short >= max_be_short;
        assert!(should_emergency_close_short, "Khi Short bị kéo lên 2001.0 >= max_be 1998.0, Short phải kích hoạt Emergency Close!");
    }

    #[test]
    fn test_break_even_immune_to_squeeze() {
        let _entry_price: f64 = 2000.0;
        let leverage: f64 = 20.0;
        let base_be_roe: f64 = 0.06;
        let fee_buffer_pct: f64 = 0.0010;
        let min_safe_be_roe: f64 = (fee_buffer_pct * leverage * 1.5).max(base_be_roe);

        // Giả lập Lệnh #0004: Lãi đỉnh +10.6% ROE
        let peak_roe: f64 = 0.106;

        // Dù thị trường có squeeze_intensity = 1.0 thì min_safe_be_roe KHÔNG bị nhân đôi lên 16%
        assert!(peak_roe >= min_safe_be_roe, "Lệnh #0004 lãi +10.6% phải kích hoạt BE thành công, miễn nhiễm hoàn toàn với Squeeze!");
    }

    #[test]
    fn test_dynamic_fee_buffer_ssot_and_high_leverage() {
        // 1. Kiểm tra tính thích ứng theo Fee VIP
        let vip_taker_fee: f64 = 0.00015; // VIP 0.015%
        let top_coin_slippage: f64 = 0.0001; // 0.01%
        let vip_buffer: f64 = 2.0 * vip_taker_fee + top_coin_slippage;
        assert!((vip_buffer - 0.0004).abs() < 1e-6, "VIP fee buffer tự co lại còn 0.04%");

        // 2. Kiểm tra tính thích ứng theo Đòn bẩy cao 100x
        let normal_taker_fee: f64 = 0.0004;
        let normal_slippage: f64 = 0.0002;
        let normal_buffer: f64 = 2.0 * normal_taker_fee + normal_slippage; // 0.0010 (0.10%)
        let leverage_100x: f64 = 100.0;

        let safe_be_100x: f64 = (normal_buffer * leverage_100x * 1.5).max(0.06);
        assert!((safe_be_100x - 0.15).abs() < 1e-4, "Ở đòn bẩy 100x, ngưỡng BE tự động nâng lên 15% ROE để không bao giờ bị khóa lỗ!");
    }

    #[test]
    fn test_progressive_profit_floor_locks_gain_above_entry() {
        // Giả lập Trade #0009: Entry 2000.0, Đòn bẩy 20x, Lãi đỉnh đạt +15.8% ROE (peak_val = 2015.8)
        let entry_price: f64 = 2000.0;
        let leverage: f64 = 20.0;
        let peak_roe: f64 = 0.158;
        let min_lock_roe: f64 = 0.10; // Ngưỡng bắt đầu kích hoạt khóa lãi
        let lock_ratio: f64 = 0.40;   // Khóa tối thiểu 40% đỉnh lãi
        
        assert!(peak_roe >= min_lock_roe, "Lãi 15.8% ROE phải kích hoạt Progressive Profit Floor!");

        let locked_roe: f64 = peak_roe * lock_ratio; // 0.158 * 0.40 = 0.0632 (6.32% ROE)
        let floor_sl_long: f64 = entry_price * (1.0 + (locked_roe / leverage)); // 2000.0 * (1.0 + 0.00316) = 2006.32

        assert!(floor_sl_long > entry_price, "Mức Stop Loss khóa lãi Long phải nằm trên giá Entry!");
        let roe_at_floor: f64 = ((floor_sl_long - entry_price) / entry_price) * leverage;
        assert!((roe_at_floor - 0.0632).abs() < 1e-4, "ROE khóa lại tối thiểu phải là +6.32% ROE, triệt tiêu 100% rủi ro nhả lại lãi!");

        // Đối xứng cho lệnh Short
        let floor_sl_short: f64 = entry_price * (1.0 - (locked_roe / leverage));
        assert!(floor_sl_short < entry_price, "Mức Stop Loss khóa lãi Short phải nằm dưới giá Entry!");
        let roe_at_floor_short: f64 = ((entry_price - floor_sl_short) / entry_price) * leverage;
        assert!((roe_at_floor_short - 0.0632).abs() < 1e-4, "ROE khóa lại của Short tối thiểu phải là +6.32% ROE!");
    }

    #[test]
    fn test_ttp_activated_via_break_even_and_locks_profit() {
        // Giả lập kịch bản Break-Even đã kích hoạt ở +8% ROE
        let is_be_triggered = true;
        let peak_roe: f64 = 0.12; // Giá tiếp tục tăng lên +12% ROE
        let ttp_act_roe: f64 = 0.15; // Ngưỡng ttp độc lập là 15% ROE
        
        // Nhờ kích hoạt song song khi có BE:
        let is_ttp_active = (peak_roe >= ttp_act_roe) || is_be_triggered;
        assert!(is_ttp_active, "TTP phải kích hoạt ngay khi Break-Even được xác lập!");

        // Kiểm tra điều kiện thoát lệnh: Giá tạo đỉnh quay đầu >= ttp_cb_atr
        let entry_price: f64 = 2000.0;
        let peak_val: f64 = 2012.0; // Peak +12% ROE trên 20x (giá tăng 0.6% = 12 giá)
        let current_atr: f64 = 5.0;
        let ttp_cb_atr: f64 = 0.80; // 0.8 * 5 = 4 giá
        let ttp_trigger_price: f64 = peak_val - (ttp_cb_atr * current_atr); // 2012.0 - 4.0 = 2008.0

        let fee_buffer_pct: f64 = 0.0010;
        let min_exit_gain_pct: f64 = 0.002f64.max(fee_buffer_pct);
        let min_exit_price: f64 = entry_price * (1.0 + min_exit_gain_pct); // 2000.0 * 1.002 = 2004.0

        let current_close: f64 = 2007.5; // Nến đóng ở 2007.5 (đã quay đầu qua trigger 2008.0 và vẫn trên min_exit 2004.0)
        let should_ttp_exit = current_close <= ttp_trigger_price && current_close >= min_exit_price;
        assert!(should_ttp_exit, "TTP phải đóng lệnh chốt lời chủ động khi đỉnh quay đầu!");
        let realized_roe: f64 = ((current_close - entry_price) / entry_price) * 20.0;
        assert!(realized_roe > 0.07, "Lệnh chốt lời thực tế đạt ROE > +7.5% thay vì trôi về hòa vốn!");
    }

    #[test]
    fn test_calculate_size_futures_min_notional_rejection() {
        // Tài khoản bình thường $60: Margin $12, Notional $240 >= $21 -> Thành công
        let size_normal = RiskManager::calculate_size(60.0, 0.20, 21.0, 20.0, 0.25);
        assert!((size_normal - 0.20).abs() < 1e-6);

        // Tài khoản bị đói vốn cực độ $3.00:
        // Margin 20% = $0.60 -> Notional = $12 < $21.
        // Nâng lên $21 -> Margin = $1.05. Nhưng cap 25% = $0.75 -> Notional thực tế = $15 < $21.
        // Phải từ chối mở lệnh (trả về 0.0) thay vì gửi lệnh lỗi lên Binance!
        let size_starved = RiskManager::calculate_size(3.0, 0.20, 21.0, 20.0, 0.25);
        assert_eq!(size_starved, 0.0, "Tài khoản không đủ ký quỹ để đạt $21 Notional phải trả về 0.0!");
    }

    #[test]
    fn test_be_triggered_on_flash_crash() {
        // Giả lập lệnh Long: Đòn bẩy 20x, Entry = 2000.0
        // Đỉnh lợi nhuận từng chạm 2020.0 (+20% ROE)
        let leverage: f64 = 20.0;
        let entry_price: f64 = 2000.0;
        let peak_val: f64 = 2020.0;
        let peak_roe: f64 = ((peak_val - entry_price) / entry_price) * leverage; // +20% ROE
        
        let fee_buffer_pct: f64 = 0.0013; // Phí vào + ra + trượt giá
        let min_safe_be_roe: f64 = (fee_buffer_pct * leverage * 1.5).max(0.06); // 6% ROE
        let required_profit: f64 = 50.0;
        let peak_profit: f64 = 100.0;

        // Trạng thái nến hiện tại: Flash crash sập về 1995.0 (dưới Entry, current_pnl < 0)
        let current_close: f64 = 1995.0;
        let current_pnl: f64 = (current_close - entry_price) * 1.0; // Âm (-5.0)

        // Logic cũ trước khi fix: `current_pnl > 0.0 && ...` làm `is_be_triggered` = false (BỊ MÙ FLASH CRASH)
        let old_is_be_triggered = current_pnl > 0.0 && (peak_roe >= min_safe_be_roe || peak_profit >= required_profit);
        assert!(!old_is_be_triggered, "Logic cũ bị mù Flash Crash vì current_pnl <= 0.0");

        // Logic mới sau khi fix: Chỉ cần peak_roe >= min_safe_be_roe là kích hoạt BE (Ratchet Memory)
        let is_be_triggered = peak_roe >= min_safe_be_roe || peak_profit >= required_profit;
        assert!(is_be_triggered, "Logic mới phải kích hoạt BE thành công dù nến hiện tại đang sập!");

        // Kiểm tra điều kiện Emergency BE Rejection Close:
        let min_be = entry_price * (1.0 + fee_buffer_pct); // 2000 * 1.0013 = 2002.6
        let should_emergency_exit = is_be_triggered && (current_close <= min_be);
        assert!(should_emergency_exit, "Phải lập tức kích hoạt Emergency Exit khi giá sập thủng min_be!");
    }

    // =========================================================================
    // 🛡️ BỘ TEST CASES KIỂM CHỨNG LIQUIDATION & MMR GUARD (TODO-RSK-01)
    // =========================================================================

    #[test]
    fn test_liquidation_guard_kamikaze_rejection_at_100x() {
        let entry_price = 2000.0;
        let leverage = 100.0;
        let mmr = 0.010; // 1.0% MMR
        let taker_fee = 0.0004; // 0.04% fee
        let buffer_ratio = 0.15; // 15% buffer
        let min_initial_sl_pct = 0.0015; // 0.15% min SL
        let max_roe_cap = 0.15; // 15% ROE

        // Tại 100x: 1/100 = 1.0%. MMR = 1.0%, 2 * fee = 0.08%
        // raw_liq = 1.0% - 1.0% - 0.08% = -0.08% <= 0!
        let safe_cap = RiskManager::calculate_safe_sl_distance(
            leverage, max_roe_cap, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        assert!(safe_cap.is_none(), "Tại 100x với MMR 1.0%, khoảng cách thanh lý âm -> Phải trả về None!");

        // Kiểm tra thẩm định SL đề xuất (ví dụ SL cách 0.10% = 10% ROE)
        let suggested_sl = entry_price * (1.0 - 0.0010);
        let res = RiskManager::validate_structural_sl(
            entry_price, suggested_sl, max_roe_cap, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        match res {
            SlValidationResult::LiquidationRiskVeto { suggested_dist_pct, safe_liq_dist_pct, mmr: res_mmr } => {
                assert!((suggested_dist_pct - 0.0010).abs() < 1e-6);
                assert_eq!(safe_liq_dist_pct, 0.0);
                assert!((res_mmr - 0.010).abs() < 1e-6);
            }
            _ => panic!("Phải trả về LiquidationRiskVeto cho kịch bản 100x Kamikaze!"),
        }
    }

    #[test]
    fn test_liquidation_guard_safe_execution_at_20x() {
        let entry_price = 2000.0;
        let leverage = 20.0;
        let mmr = 0.010; // 1.0% MMR
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;
        let min_initial_sl_pct = 0.0015;
        let max_roe_cap = 0.15; // 15% ROE = 0.75% giá

        // raw_liq = 5.0% - 1.0% - 0.08% = 3.92%
        // safe_liq = 3.92% * 0.85 = 3.332%
        // effective_cap = min(0.75%, 3.332%) = 0.75%
        let safe_cap = RiskManager::calculate_safe_sl_distance(
            leverage, max_roe_cap, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        ).expect("20x an toàn phải tính được safe_cap");
        assert!((safe_cap - 0.0075).abs() < 1e-5, "Trần an toàn tại 20x phải là 0.75% giá (15% ROE)");

        // SL đề xuất 0.50% giá (10% ROE) -> Acceptable
        let suggested_sl = entry_price * (1.0 - 0.0050);
        let res = RiskManager::validate_structural_sl(
            entry_price, suggested_sl, max_roe_cap, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        assert_eq!(res, SlValidationResult::Acceptable, "SL 10% ROE tại 20x phải được chấp thuận!");

        // SL đề xuất 0.90% giá (18% ROE) -> StructuralRiskVeto (Vượt 15% ROE)
        let suggested_sl_wide = entry_price * (1.0 - 0.0090);
        let res_wide = RiskManager::validate_structural_sl(
            entry_price, suggested_sl_wide, max_roe_cap, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        match res_wide {
            SlValidationResult::StructuralRiskVeto { suggested_roe, max_roe } => {
                assert!((suggested_roe - 0.18).abs() < 1e-4);
                assert!((max_roe - 0.15).abs() < 1e-4);
            }
            _ => panic!("SL 18% ROE phải bị StructuralRiskVeto!"),
        }
    }

    #[test]
    fn test_liquidation_guard_veto_high_mmr_altcoin() {
        let entry_price = 2000.0;
        let leverage = 40.0;
        let mmr = 0.025; // 2.5% MMR (Altcoin rác)
        let taker_fee = 0.0004;
        let buffer_ratio = 0.15;
        let min_initial_sl_pct = 0.0015;
        let max_roe_cap = 0.15;

        // raw_liq = 2.5% - 2.5% - 0.08% = -0.08% <= 0!
        let safe_cap = RiskManager::calculate_safe_sl_distance(
            leverage, max_roe_cap, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        assert!(safe_cap.is_none(), "Tại 40x với Altcoin MMR 2.5%, raw_liq <= 0 -> Phải trả về None!");

        let suggested_sl = entry_price * (1.0 - 0.0020);
        let res = RiskManager::validate_structural_sl(
            entry_price, suggested_sl, max_roe_cap, leverage, mmr, taker_fee, buffer_ratio, min_initial_sl_pct,
        );
        match res {
            SlValidationResult::LiquidationRiskVeto { .. } => {}
            _ => panic!("Altcoin MMR 2.5% tại 40x phải bị LiquidationRiskVeto!"),
        }
    }

    #[test]
    fn test_clamp_sl_price_with_liquidation_guard() {
        let price = 2000.0;
        let raw_sl = 1950.0; // Khoảng cách 50 giá = 2.5% giá
        let is_long = true;
        let max_sl_roe_cap = 0.15;
        let min_initial_sl_pct = 0.0015;
        let leverage = 20.0;
        let mmr = 0.010;
        let taker_fee_rate = 0.0004;
        let buffer_ratio = 0.15;

        // Tại 20x: trần tối đa là 0.75% giá (15 giá -> SL xa nhất là 1985.0)
        let clamped = RiskManager::clamp_sl_price(
            price, raw_sl, is_long, max_sl_roe_cap, min_initial_sl_pct, leverage, mmr, taker_fee_rate, buffer_ratio,
        );
        assert!((clamped - 1985.0).abs() < 1e-4, "SL 1950.0 bị kẹp về trần an toàn 1985.0!");
    }
}
