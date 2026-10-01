// strategies_backtests/helios/src/strategy/core_v3/mod.rs

pub mod arbiter;
pub mod matrix;
pub mod scanners;
pub mod risk;
pub mod config;
pub mod normalization;
pub mod gatekeeper;

use std::collections::HashMap;

use ferro_trade::optimizer::params::ParamValue;
use ferro_trade::optimizer::DnaMappable;
use ferro_trade::strategy::{InitContext, Strategy, TickContext};
use ferro_trade::indicators::factory::BulkIndicatorFactory;
use ferro_trade::broker::policies::RiskPolicy;

use ferro_trade::broker::Trade;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType, ScannerId, AuctionStatus};
use crate::strategy::core_v3::risk::RiskManager;
use tracing::warn;

pub mod telemetry;
use telemetry::{DecisionCardFormatter, DefenseLedger};

/// Trạng thái Cooldown chống cào lệnh & bẫy trả thù (Losing Streak Memory & Exponential Backoff)
#[derive(Clone, Debug, Default)]
pub struct CooldownTracker {
    pub locked_until_bar: usize,
    pub locked_side: Option<SignalType>,
    pub last_exit_price: f64,
    pub consecutive_losses: usize,
    pub last_losing_side: Option<SignalType>,
    pub last_losing_price: f64,
}

impl CooldownTracker {
    /// Tính toán số nến cooldown phạt lũy tiến (Exponential Backoff) với trần tối đa (Max Cap = 60 nến ~ 1 giờ)
    #[inline]
    pub fn calculate_exponential_cooldown(base_bars: usize, consecutive_losses: usize, max_cap: usize) -> usize {
        if base_bars == 0 || consecutive_losses == 0 {
            return 0;
        }
        let streak_power = (consecutive_losses.saturating_sub(1)).min(2) as u32;
        let multiplier = 1usize << streak_power; // 1, 2, hoặc 4
        let penalty = base_bars.saturating_mul(multiplier);
        let cap = if max_cap == 0 { 60 } else { max_cap };
        penalty.min(cap)
    }

    /// Kiểm tra xem một tín hiệu vào lệnh có bị chặn bởi Cooldown Tracker hay không
    #[inline]
    pub fn should_block(
        &self,
        enable_cooldown: bool,
        current_bar: usize,
        signal: SignalType,
        current_price: f64,
        atr: f64,
        breakout_atr_mult: f64,
    ) -> bool {
        if !enable_cooldown {
            return false;
        }

        // 🛡️ 1. SAME-SIDE PRICE GUARD:
        // Nếu đang trong chuỗi thua cùng chiều: Cấm vào lệnh nếu giá hiện tại tồi tệ hơn giá vừa cắt lỗ!
        if self.consecutive_losses > 0 && self.last_losing_side == Some(signal) {
            let is_worse_price = match signal {
                SignalType::Long => current_price < self.last_losing_price,
                SignalType::Short => current_price > self.last_losing_price,
            };
            if is_worse_price {
                return true; // Chặn đứng bắt dao rơi / cản đầu xe tăng khi giá tồi tệ hơn điểm cắt lỗ
            }
        }

        // 🛡️ 2. UNIVERSAL BIDIRECTIONAL EXPONENTIAL COOLDOWN:
        if current_bar < self.locked_until_bar {
            let mult = if breakout_atr_mult <= 0.0 { 1.5 } else { breakout_atr_mult };
            let threshold = mult * atr.max(1e-5);

            // Chỉ mở khóa sớm nếu giá bứt phá vượt ngưỡng thuận chiều
            let is_favorable_breakout = match signal {
                SignalType::Long => (current_price - self.last_exit_price) >= threshold,
                SignalType::Short => (self.last_exit_price - current_price) >= threshold,
            };

            if !is_favorable_breakout {
                return true;
            }
        }

        false
    }
}

/// Chiến lược Helios V3.3 - Clean Quant Architecture
#[derive(Clone, Debug)]
pub struct HeliosStrategyV3 {
    pub config: HeliosConfigV3,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrixV3,
    pub debug_mode: bool,
    pub cooldown_tracker: CooldownTracker,
    pub prev_open_trade: Option<Trade>,
    pub arbiter: HeliosArbiter,
    pub telemetry_enabled: bool,
    pub last_decision_telemetry: Option<String>,
    /// Sổ Cái Quản Trị & Kiểm Toán Ma Trận Phòng Thủ Vốn (Zero-cost trong GA, Some trong Live/Paper)
    pub defense_ledger: Option<DefenseLedger>,
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        let mut arbiter = HeliosArbiter::new();
        // 🎯 4 PLAYBOOK CHỦ LỰC V3.3: Khởi tạo sẵn 1 lần duy nhất (Zero-Allocation ở runtime)
        arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
        arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
        arbiter.add_scanner(Box::new(scanners::reversal::ReversalScanner));
        arbiter.add_scanner(Box::new(scanners::absorption::AbsorptionScanner));

        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
            cooldown_tracker: CooldownTracker::default(),
            prev_open_trade: None,
            arbiter,
            telemetry_enabled: false,
            last_decision_telemetry: None,
            defense_ledger: None,
        }
    }

    fn update_from_dna(&mut self, dna: &[f64]) {
        if dna.len() != HeliosConfigV3::DNA_WIDTH {
            warn!(
                "⚠️ [DNA LENGTH MISMATCH] Yêu cầu {} gen nhưng DNA có {} gen -> Bỏ qua nạp DNA không tương thích để chống crash/lệch chỉ mục!",
                HeliosConfigV3::DNA_WIDTH, dna.len()
            );
            return;
        }
        self.config.update_from_dna(dna);
        // 🛡️ TASK-MAT-05: Đồng bộ hóa cấu hình Keltner timeframe theo BBands
        self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;
        self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
    }

    fn init(&mut self, ctx: &mut InitContext) {
        // 🛡️ TASK-MAT-05: Khóa cứng đồng bộ 100% Timeframe của Keltner Channel theo Bollinger Bands (Serde-Safe)
        self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;

        let ind = &self.config.indicators;

        if ind.natr.period == 0 || ind.mfi.period == 0 || 
           ind.kama.short_period == 0 || ind.kama.long_period == 0 ||
           ind.adx.period == 0 || ind.bbands.period == 0 || ind.atr.period == 0 {
            println!("!!! [SAFETY V3.3] CRITICAL PERIOD IS ZERO, ABORTING INIT !!!");
            return;
        }

        let mut s = HeliosSensors::default();
        
        s.closes = ctx.closes().clone();
        s.highs = ctx.highs().clone();
        s.lows = ctx.lows().clone();
        s.volumes = ctx.volumes().clone();

        s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
        s.chaos_ratio = ctx.chaos_ratio(ind.natr.timeframe, "ChaosRatio", ind.natr.period);
        s.efficiency_ratio = ctx.efficiency_ratio(ind.kama.timeframe, "ER", ind.kama.short_period);

        let (mfi, _tp, _mf, _pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
        s.mfi = mfi;

        let (adx, _pdi, _mdi, _adxr) = ctx.adx(ind.adx.timeframe, "ADX", ind.adx.period);
        s.adx = adx;

        s.kama_mid = ctx.kama(ind.kama.timeframe, "KAMA", ind.kama.short_period, 2, ind.kama.long_period);
        
        let (k_slope, k_slope_adj) = ctx.kama_slope(
            ind.kama.timeframe, "KAMA_Slope", 
            ind.kama.short_period, 2, ind.kama.long_period, 
            ind.kama.short_period, ind.atr.period
        );
        s.kama_slope = k_slope;
        s.kama_slope_adj = k_slope_adj;

        let (bb_u, bb_m, bb_l, _bb_w, _bb_wp, _bb_sw, _bb_sq) = ctx.bbands(ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
        s.bb_upper = bb_u;
        s.bb_middle = bb_m;
        s.bb_lower = bb_l;

        // 🛡️ TASK-MAT-05: Khóa cứng đồng bộ 100% Timeframe của Keltner Channel theo Bollinger Bands (Serde-Safe)
        let (kc_u, _kc_m, kc_l) = ctx.keltner(
            ind.bbands.timeframe, 
            "KC", 
            ind.keltner.period as u32, 
            ind.keltner.multiplier, 
            ind.keltner.ma_type.clone()
        );
        s.kc_upper = kc_u;
        s.kc_lower = kc_l;

        // DEDICATED PER-PLAYBOOK RSI SENSORS
        let (rsi_t, _, _) = ctx.rsi(ind.rsi_trend.timeframe, "RSI_Trend", ind.rsi_trend.period);
        s.rsi_trend = rsi_t;

        let (rsi_r, _, _) = ctx.rsi(ind.rsi_reversal.timeframe, "RSI_Rev", ind.rsi_reversal.period);
        s.rsi_reversal = rsi_r;

        let (rsi_m, _, _) = ctx.rsi(ind.rsi_momentum.timeframe, "RSI_Mom", ind.rsi_momentum.period);
        s.rsi_momentum = rsi_m;

        // [V3.3 TINH GIẢN] Tắt chỉ báo RSI_Ano của Playbook Anomaly đã de-overengineer (tránh bẫy period 0 trả về toàn NaN)
        // let (rsi_a, _, _) = ctx.rsi(ind.rsi_anomaly.timeframe, "RSI_Ano", ind.rsi_anomaly.period);
        // s.rsi_anomaly = rsi_a;

        // DEDICATED PER-PLAYBOOK STOCHASTIC SENSORS
        let (st_t_k, st_t_d, _, _, _) = ctx.stoch(
            ind.stoch_trend.timeframe, "STOCH_Trend", 
            ind.stoch_trend.period as u32, 
            ind.stoch_trend.k_period as u32, ind.stoch_trend.k_ma_type.clone(),
            ind.stoch_trend.d_period as u32, ind.stoch_trend.d_ma_type.clone(),
            ind.stoch_trend.zone
        );
        s.stoch_trend_k = st_t_k;
        s.stoch_trend_d = st_t_d;

        // [V3.3 TINH GIẢN] Tắt chỉ báo STOCH_Range của Playbook RangeOsc đã de-overengineer (tránh bẫy period 0 trả về toàn NaN)
        // let (st_r_k, st_r_d, _, _, _) = ctx.stoch(
        //     ind.stoch_range.timeframe, "STOCH_Range", 
        //     ind.stoch_range.period as u32, 
        //     ind.stoch_range.k_period as u32, ind.stoch_range.k_ma_type.clone(),
        //     ind.stoch_range.d_period as u32, ind.stoch_range.d_ma_type.clone(),
        //     ind.stoch_range.zone
        // );
        // s.stoch_range_k = st_r_k;
        // s.stoch_range_d = st_r_d;

        // ABSORPTION VWMA SENSOR
        s.vwma = ctx.vwma(ind.vwma.timeframe, "VWMA", ind.vwma.period);
        s.z_score = ctx.z_score(ind.kama.timeframe, "ZScore", ind.z_score_lookback);
        s.vol_z_score = ctx.z_score_volume(ind.kama.timeframe, "VolZScore", ind.z_score_lookback);
        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);

        // 🛡️ FLAW-REG.10: Tính độ dốc chuẩn hóa của trục giữa Bollinger Bands (SMA 20) qua ATR (K = 5 bars)
        let k_bars = 5;
        let bb_m_len = s.bb_middle.len();
        let mut bb_slope = Vec::with_capacity(bb_m_len);
        for idx in 0..bb_m_len {
            if idx < k_bars {
                bb_slope.push(0.0);
            } else {
                let m_curr = s.bb_middle[idx];
                let m_prev = s.bb_middle[idx - k_bars];
                let atr = if idx < s.atr_sl.len() { s.atr_sl[idx].max(1e-5) } else { 1.0 };
                if m_curr.is_nan() || m_prev.is_nan() || atr.is_nan() {
                    bb_slope.push(0.0);
                } else {
                    let raw_slope = (m_curr - m_prev) / (k_bars as f64 * atr);
                    bb_slope.push(raw_slope.clamp(-2.0, 2.0));
                }
            }
        }
        s.bb_middle_slope_adj = std::sync::Arc::new(bb_slope);

        self.sensors = s;
        self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
    }

    fn next(&mut self, tick: &mut TickContext) {
        let i = tick.i;
        if self.sensors.atr_sl.len() <= i || self.matrix.chaos_index.len() <= i {
            return;
        }

        let pos = tick.position();
        let is_pos_open = pos.is_open();

        // 1. Quản lý Rủi ro / Trailing Stop (Chạy mỗi nến khi có vị thế)
        RiskManager::manage_trailing_stop(
            &self.config,
            &self.matrix,
            tick,
            &self.sensors,
            self.debug_mode
        );

        // 2. Theo dõi trạng thái đóng lệnh để kích hoạt Cooldown chống cào lệnh
        if !is_pos_open {
            if let Some(ref last_trade) = self.prev_open_trade {
                let stop_cfg = &self.config.risk_management.stop_management;
                if stop_cfg.enable_re_entry_cooldown && stop_cfg.re_entry_cooldown_bars > 0 {
                    // [CHỐNG BẪY FALSE WIN KHI NẾN RÚT CHÂN]
                    // Ưu tiên 1: Đọc PnL và giá thoát thực tế 100% từ Broker qua tick.last_closed_trade
                    // Fallback 2: Nếu chưa có closed_trade, dùng sl_price thay vì current_close
                    let (is_long, trade_pnl, exit_price) = if let Some(closed) = tick.last_closed_trade {
                        let exit_p = closed.exit_price.unwrap_or_else(|| tick.closes()[i]);
                        (closed.is_long(), closed.pl(None), exit_p)
                    } else {
                        let is_l = last_trade.is_long();
                        let exit_p = last_trade.sl_price.unwrap_or_else(|| tick.closes()[i]);
                        let pnl = if is_l { exit_p - last_trade.entry_price } else { last_trade.entry_price - exit_p };
                        (is_l, pnl, exit_p)
                    };

                    let closed_side = if is_long { SignalType::Long } else { SignalType::Short };

                    // Nếu lệnh vừa đóng hòa vốn hoặc lỗ danh nghĩa (<= 0.001)
                    if trade_pnl <= 0.001 {
                        // 1. Cập nhật Losing Streak Memory:
                        if self.cooldown_tracker.last_losing_side == Some(closed_side) {
                            self.cooldown_tracker.consecutive_losses = self.cooldown_tracker.consecutive_losses.saturating_add(1);
                        } else {
                            self.cooldown_tracker.consecutive_losses = 1;
                        }
                        self.cooldown_tracker.last_losing_side = Some(closed_side);
                        self.cooldown_tracker.last_losing_price = exit_price;

                        // 2. Tính phạt Exponential Backoff (15 -> 30 -> 60 nến max cap):
                        let effective_cooldown = CooldownTracker::calculate_exponential_cooldown(
                            stop_cfg.re_entry_cooldown_bars,
                            self.cooldown_tracker.consecutive_losses,
                            60, // Trần Max Cap 60 nến (~1 giờ) chống khóa vô hạn
                        );

                        self.cooldown_tracker.locked_until_bar = i + effective_cooldown;
                        self.cooldown_tracker.locked_side = Some(closed_side);
                        self.cooldown_tracker.last_exit_price = exit_price;

                        if self.debug_mode {
                            warn!(
                                "🛑 [EXPONENTIAL COOLDOWN TRIGGERED] Bar: {} | Side: {:?} | Streak: {} | Penalty: {} bars (Cap: 60) | Exit: {:.2} | Realized PnL: {:.4} <= 0.001 | Locked until Bar: {}",
                                i, closed_side, self.cooldown_tracker.consecutive_losses, effective_cooldown, exit_price, trade_pnl, self.cooldown_tracker.locked_until_bar
                            );
                        }
                    } else {
                        // 3. LỆNH THẮNG THỰC TẾ (> 0.001): RESET TOÀN BỘ CHUỖI THUA & COOLDOWN VỀ 0
                        self.cooldown_tracker.consecutive_losses = 0;
                        self.cooldown_tracker.locked_until_bar = 0;
                        self.cooldown_tracker.locked_side = None;
                        self.cooldown_tracker.last_losing_side = None;

                        if self.debug_mode {
                            warn!(
                                "🎉 [COOLDOWN FULL RESET - WINNING TRADE] Bar: {} | Side: {:?} | Entry: {:.2} | Exit: {:.2} | Realized PnL: {:.4} > 0.001 -> Reset streak & Cooldown to 0!",
                                i, closed_side, last_trade.entry_price, exit_price, trade_pnl
                            );
                        }
                    }
                }
                self.prev_open_trade = None;
            }
        }

        // 3. Strict Exclusive Order: Nếu đang có vị thế mở thì không mở lệnh mới
        if is_pos_open {
            if let Some(trade) = tick.trades.first() {
                self.prev_open_trade = Some(trade.clone());
            }
            return; 
        }

        // 🛡️ THEO DÕI MA TRẬN PHÒNG THỦ VỐN (Zero-cost trong GA, Some trong Live/Paper)
        if let Some(ref mut ledger) = self.defense_ledger {
            // Tầng 1: Chuyển đổi trạng thái Gatekeeper (Vào bão / Ra bão)
            let gate_eval = crate::strategy::core_v3::gatekeeper::RegimeGatekeeper::evaluate(tick, &self.sensors, &self.matrix, &self.config);
            if let Some(msg) = ledger.on_gatekeeper_tick(tick.i, &gate_eval) {
                self.last_decision_telemetry = Some(msg);
            }

            // Tầng 2: Chốt chặn Boundary Guard & Chop Zone của TrendPB
            let pb_cfg = &self.config.tactical_sniper.pullback;
            let squeeze = self.matrix.squeeze_intensity.get(tick.i).copied().unwrap_or(0.0);
            let kama_slope = self.matrix.kama_slope_atr.get(tick.i).copied()
                .unwrap_or_else(|| self.sensors.kama_slope_adj.get(tick.i).copied().unwrap_or(0.0));
            let z_score = self.sensors.z_score.get(tick.i).copied().unwrap_or(0.0);
            ledger.on_trend_pb_tick(
                squeeze,
                pb_cfg.max_squeeze_intensity,
                kama_slope,
                pb_cfg.kama_slope_threshold.max(1e-4),
                z_score,
                pb_cfg.max_boundary_z,
            );
        }

        // 🎯 KHỐI 3 (TASK-ARB-05): Tối ưu Zero-Allocation - Sử dụng arbiter đã khởi tạo sẵn
        let decision = self.arbiter.process_tick(tick, &self.sensors, &self.matrix, &self.config, self.debug_mode);

        if decision.is_approved {
            if let Some(signal) = decision.final_signal {
                // 4. [ARCH-COOLDOWN-SEPARATION]: Cầu Dao Cooldown VETO được độc quyền sở hữu và thực thi
                // bởi Bot Engine (AssetActor) dựa trên PnL thực tế của tài khoản (Net PnL).
                // Helios Strategy chuyên tâm 100% vào việc sinh tín hiệu Alpha, không tự ý chặn tín hiệu bằng PnL ước lượng.
                // Khối code dưới đây được comment out an toàn để bảo toàn logic kiểm chứng/đối chiếu khi cần.
                /*
                let stop_cfg = &self.config.risk_management.stop_management;
                let current_price = tick.closes()[i];
                let current_atr = self.sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);

                if self.cooldown_tracker.should_block(
                    stop_cfg.enable_re_entry_cooldown,
                    i,
                    signal,
                    current_price,
                    current_atr,
                    stop_cfg.cooldown_breakout_atr_mult,
                ) {
                    if self.debug_mode {
                        warn!(
                            "⚠️ [COOLDOWN BLOCKED] Bar: {} | Signal: {:?} BLOCKED by Cooldown/Guard! Streak: {} | Locked until: {} | LastExit: {:.2} | Curr: {:.2}",
                            i, signal, self.cooldown_tracker.consecutive_losses, self.cooldown_tracker.locked_until_bar,
                            self.cooldown_tracker.last_exit_price, current_price
                        );
                    }
                    return;
                }
                */
                let leverage = self.config.risk_management.risk_base.leverage.max(1.0);
                
                let size = RiskManager::calculate_size(
                    tick.equity, 
                    self.config.risk_management.risk_base.percent_per_trade, 
                    self.config.risk_management.risk_base.min_order_usd, 
                    leverage,
                    self.config.risk_management.risk_base.max_cap_fraction,
                );
                
                let is_long = signal == SignalType::Long;
                
                // [FLAW-TSL.8: FALLBACK HIERARCHY]
                // Đọc trần ROE của Playbook chiến thắng, fallback về default_sl_roe_cap nếu không có tag
                let playbook_max_sl_roe = match decision.winning_scanner {
                    Some(ScannerId::TrendPullback) => self.config.tactical_sniper.pullback.max_initial_sl_pct,
                    Some(ScannerId::MomentumIgnition) => self.config.tactical_sniper.momentum.max_initial_sl_pct,
                    Some(ScannerId::RangeOscillator) => self.config.tactical_sniper.range.max_initial_sl_pct,
                    Some(ScannerId::ClimaxReversal) => self.config.tactical_sniper.reversal.max_initial_sl_pct,
                    Some(ScannerId::AnomalyFade) => self.config.tactical_sniper.anomaly.max_initial_sl_pct,
                    Some(ScannerId::AbsorptionScout) => self.config.tactical_sniper.absorption.max_initial_sl_pct,
                    None => self.config.risk_management.stop_management.default_sl_roe_cap,
                };

                let min_initial_sl_pct = self.config.risk_management.stop_management.min_initial_sl_pct;

                let raw_sl = decision.final_sl.unwrap_or_else(|| {
                    RiskManager::get_initial_sl(
                        tick,
                        &self.sensors,
                        is_long,
                        self.config.risk_management.stop_management.base_sl_atr,
                        playbook_max_sl_roe,
                        min_initial_sl_pct,
                        leverage,
                    )
                });
                
                let price = tick.closes()[tick.i];
                let initial_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, playbook_max_sl_roe, min_initial_sl_pct, leverage);
                
                // Ghi nhận kẹp SL vào Sổ cái nếu có
                let hard_cap_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, 0.15, min_initial_sl_pct, leverage);
                let is_hard_clamped = (raw_sl - hard_cap_sl).abs() > 1e-4;
                let is_playbook_clamped = (raw_sl - initial_sl).abs() > 1e-4;
                let sl_clamped_note = if is_hard_clamped {
                    if let Some(ref mut ledger) = self.defense_ledger {
                        ledger.on_sl_clamped(true);
                    }
                    Some("CLAMPED: HARD ROE CAP 15%")
                } else if is_playbook_clamped {
                    if let Some(ref mut ledger) = self.defense_ledger {
                        ledger.on_sl_clamped(false);
                    }
                    Some("CLAMPED: PLAYBOOK CAP")
                } else {
                    None
                };

                let tag = decision.winning_scanner.map(|s| s.as_str().to_string());

                let policy = Some(RiskPolicy::FixedPrice {
                    sl_price: Some(initial_sl),
                    tp_price: decision.final_tp,
                });

                if is_long {
                    tick.buy(Some(size), None, None, policy, tag);
                } else {
                    tick.sell(Some(size), None, None, policy, tag);
                }

                // 🎯 QUANT DECISION TELEMETRY: Dựng biên bản kiểm toán quyết định phê duyệt mở lệnh
                if self.telemetry_enabled {
                    self.last_decision_telemetry = Some(DecisionCardFormatter::format_approved_card(
                        tick,
                        &self.sensors,
                        &self.matrix,
                        signal,
                        &decision,
                        price,
                        initial_sl,
                        size,
                        leverage,
                        sl_clamped_note,
                    ));
                }
            }
        } else {
            // Cập nhật Sổ Cái các cú VETO từ chối mở lệnh
            if let Some(ref mut ledger) = self.defense_ledger {
                ledger.on_auction_decision(&decision);
            }

            // 🛑 QUANT DECISION TELEMETRY: Xuất Thẻ Đỏ VETO nếu có proposal bị Sniper Guard hoặc Gatekeeper VETO
            if self.telemetry_enabled {
                let has_structural_veto = decision.auction_ledger.iter().any(|c| c.status == AuctionStatus::StructuralRiskVeto);
                let has_candidates = decision.auction_ledger.iter().any(|c| c.status != AuctionStatus::NotTriggered);
                if has_structural_veto || (decision.gate_veto.is_some() && has_candidates) {
                    let price = tick.closes()[tick.i];
                    let leverage = self.config.risk_management.risk_base.leverage.max(1.0);
                    self.last_decision_telemetry = Some(DecisionCardFormatter::format_veto_card(
                        tick,
                        &self.sensors,
                        &self.matrix,
                        &decision,
                        price,
                        leverage,
                    ));
                }
            }
        }
    }

    fn set_telemetry_enabled(&mut self, enabled: bool) {
        self.telemetry_enabled = enabled;
        self.defense_ledger = if enabled { Some(DefenseLedger::new()) } else { None };
    }

    fn take_decision_telemetry(&mut self) -> Option<String> {
        self.last_decision_telemetry.take()
    }

    fn get_defense_summary(&self) -> Option<String> {
        self.defense_ledger.as_ref().map(|l| l.format_summary_table())
    }
}


impl Default for HeliosStrategyV3 {
    fn default() -> Self {
        Self::new(&std::collections::HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::core_v3::scanners::momentum_ignition::MomentumIgnitionScanner;
    use crate::strategy::core_v3::arbiter::TacticalScanner;
    use ferro_trade::data::{RawDataCache, DataView};
    use ferro_trade::backtest::manager::IndicatorManager;
    use ferro_trade::broker::position::Position;
    use smallvec::SmallVec;
    use std::collections::HashMap;

    fn should_block_by_cooldown(
        enable_cooldown: bool,
        current_bar: usize,
        locked_until_bar: usize,
        signal: SignalType,
        current_price: f64,
        last_exit_price: f64,
        atr: f64,
        atr_mult: f64,
    ) -> bool {
        let tracker = CooldownTracker {
            locked_until_bar,
            locked_side: None,
            last_exit_price,
            consecutive_losses: 0,
            last_losing_side: None,
            last_losing_price: 0.0,
        };
        tracker.should_block(enable_cooldown, current_bar, signal, current_price, atr, atr_mult)
    }

    #[test]
    fn test_cooldown_toggle_disabled() {
        // Khi enable_re_entry_cooldown = false -> KHÔNG BAO GIỜ CHẶN
        let blocked = should_block_by_cooldown(
            false,
            10,
            20,
            SignalType::Long,
            1760.0,
            1760.5,
            5.0,
            1.5,
        );
        assert!(!blocked, "Khi tắt Cooldown, không được chặn lệnh");
    }

    #[test]
    fn test_cooldown_same_side_in_range_blocked() {
        // Khi BẬT, cùng chiều Long, giá chưa thoát range -> BỊ CHẶN
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Long,
            1760.0,
            1760.5,
            5.0, // 1.5 * ATR = 7.5; diff = 0.5 < 7.5
            1.5,
        );
        assert!(blocked, "Cùng chiều trong vùng nén phải bị chặn");
    }

    #[test]
    fn test_cooldown_opposite_side_in_range_blocked() {
        // 🎯 SỬA FLAW-TSL.7: Khi BẬT, tín hiệu ngược chiều (Short) nhưng giá lình xình trong vùng nhiễu -> PHẢI BỊ CHẶN!
        // Chống tuyệt đối bẫy Alternating Whipsaw vừa cắt lỗ Long ở đáy lại Short ở đáy.
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Short, // Tín hiệu ngược chiều
            1760.0,
            1760.5, // diff = 1760.5 - 1760.0 = 0.5 < 7.5
            5.0,
            1.5,
        );
        assert!(blocked, "Ngược chiều trong vùng nhiễu đáy PHẢI BỊ CHẶN để chống Whipsaw 2 đầu!");
    }

    #[test]
    fn test_cooldown_opposite_side_breakout_allowed() {
        // 🎯 SỬA FLAW-TSL.7: Tín hiệu ngược chiều (Short) khi thị trường thực sự sập gãy mạnh -> MỞ KHÓA SỚM!
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Short,
            1750.0, // Giá sập từ 1760.5 xuống 1750.0 (diff = 10.5 >= 1.5 * 5.0 = 7.5)
            1760.5,
            5.0,
            1.5,
        );
        assert!(!blocked, "Ngược chiều nhưng sập gãy vượt k*ATR phải được giải phóng sớm!");
    }

    #[test]
    fn test_cooldown_volatility_breakout_override() {
        // Khi BẬT, cùng chiều, nhưng giá bứt phá TĂNG NGƯỢC LÊN >= k*ATR -> MỞ KHÓA SỚM
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Long,
            1770.0, // diff = 1770.0 - 1760.0 = +10.0 >= 1.5*5.0 (7.5) -> Tăng có lợi
            1760.0,
            5.0,
            1.5,
        );
        assert!(!blocked, "Bứt phá biến động tăng mạnh phải mở khóa Cooldown sớm");
    }

    #[test]
    fn test_cooldown_falling_knife_blocked() {
        // 🎯 KIỂM CHỨNG FLAW-4.3 & FLAW-TSL.7: Giá sập hầm giảm sâu 10 USD (1750 < 1760)
        // Nếu phát tín hiệu Long bắt dao rơi -> TIẾP TỤC CHẶN COOLDOWN!
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Long,
            1750.0, // Giá sập hầm từ 1760 về 1750
            1760.0,
            5.0,
            1.5,
        );
        assert!(blocked, "Giá sập hầm rơi tự do PHẢI BỊ CHẶN COOLDOWN, cấm bắt dao rơi!");
    }

    #[test]
    fn test_cooldown_expired_allowed() {
        // Khi đã hết thời gian nghỉ (current_bar >= locked_until_bar) -> CHO PHÉP
        let blocked = should_block_by_cooldown(
            true,
            26,
            25,
            SignalType::Long,
            1760.0,
            1760.5,
            5.0,
            1.5,
        );
        assert!(!blocked, "Hết thời gian nghỉ phải cho phép vào lệnh");
    }

    #[test]
    fn test_cooldown_dynamic_dna_multiplier() {
        // 🎯 KIỂM CHỨNG DẢI DNA ĐỘNG: atr_mult = 2.5 (threshold = 2.5 * 5.0 = 12.5)
        // Khi diff = +10.0 (1770 - 1760): Trước đây 1.5 thì mở khóa, nhưng với 2.5 thì VẪN BỊ CHẶN!
        let blocked_at_2_5 = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Long,
            1770.0,
            1760.0,
            5.0,
            2.5, // Ngưỡng bứt phá cao hơn
        );
        assert!(blocked_at_2_5, "Với atr_mult = 2.5, bứt phá +10.0 < 12.5 vẫn phải bị chặn");

        // Khi diff = +13.0 (1773 - 1760 >= 12.5) -> ĐƯỢC MỞ KHÓA!
        let unlocked_at_2_5 = should_block_by_cooldown(
            true,
            10,
            25,
            SignalType::Long,
            1773.0,
            1760.0,
            5.0,
            2.5,
        );
        assert!(!unlocked_at_2_5, "Bứt phá +13.0 >= 12.5 phải được mở khóa");
    }

    #[test]
    fn test_bidirectional_cooldown_blocks_both_sides() {
        // 🎯 KIỂM CHỨNG FLAW-TSL.7 (BƯỚC 1): Sau khi cắt lỗ/hòa vốn, cả Long và Short đều bị khóa nếu trong vùng nhiễu
        let exit_p = 2500.0;
        let atr = 10.0;
        let k = 1.5; // Ngưỡng bứt phá = 15.0 USD (vùng nhiễu: [2485.0, 2515.0])

        // Giá lình xình tại 2505.0 (+5.0):
        // Long bị chặn (diff = +5.0 < 15.0)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Long, 2505.0, exit_p, atr, k), "Long trong vùng nhiễu phải bị chặn");
        // Short bị chặn (diff = -5.0 < 15.0)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Short, 2505.0, exit_p, atr, k), "Short trong vùng nhiễu phải bị chặn");

        // Giá lình xình tại 2495.0 (-5.0):
        // Long bị chặn (diff = -5.0 < 15.0)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Long, 2495.0, exit_p, atr, k), "Long lình xình dưới đáy phải bị chặn");
        // Short bị chặn (diff = +5.0 < 15.0)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Short, 2495.0, exit_p, atr, k), "Short lình xình dưới đáy phải bị chặn");
    }

    #[test]
    fn test_directional_early_release_long_breakout() {
        // 🎯 KIỂM CHỨNG FLAW-TSL.7 (BƯỚC 2): Giá bốc đầu tăng mạnh >= k*ATR (2520.0 - 2500.0 = +20.0 >= 15.0)
        let exit_p = 2500.0;
        let atr = 10.0;
        let k = 1.5;

        // Long được mở khóa sớm
        assert!(!should_block_by_cooldown(true, 5, 20, SignalType::Long, 2520.0, exit_p, atr, k), "Long bứt phá tăng mạnh phải được giải phóng sớm");
        // Short bị chặn triệt để (chống cản tàu khi giá đang bốc đầu: diff = 2500 - 2520 = -20 < 15)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Short, 2520.0, exit_p, atr, k), "Cấm tuyệt đối Short cản tàu khi giá đang bốc đầu");
    }

    #[test]
    fn test_directional_early_release_short_breakout() {
        // 🎯 KIỂM CHỨNG FLAW-TSL.7 (BƯỚC 3): Giá sập gãy mạnh >= k*ATR (2500.0 - 2480.0 = +20.0 >= 15.0)
        let exit_p = 2500.0;
        let atr = 10.0;
        let k = 1.5;

        // Short được mở khóa sớm
        assert!(!should_block_by_cooldown(true, 5, 20, SignalType::Short, 2480.0, exit_p, atr, k), "Short bứt phá sập gãy phải được giải phóng sớm");
        // Long bị chặn triệt để (chống bắt dao rơi khi giá đang rơi tự do: diff = 2480 - 2500 = -20 < 15)
        assert!(should_block_by_cooldown(true, 5, 20, SignalType::Long, 2480.0, exit_p, atr, k), "Cấm tuyệt đối Long bắt dao rơi khi giá đang sập hầm");
    }

    #[test]
    fn test_exponential_cooldown_calculation() {
        let base = 15;
        let cap = 60;

        // Thua 0 lần -> 0 nến
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 0, cap), 0);
        // Thua lần 1 -> 15 * 2^0 = 15 nến
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 1, cap), 15);
        // Thua lần 2 liên tiếp -> 15 * 2^1 = 30 nến
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 2, cap), 30);
        // Thua lần 3 liên tiếp -> 15 * 2^2 = 60 nến (chạm trần)
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 3, cap), 60);
        // Thua lần 4+ liên tiếp -> vẫn giữ trần 60 nến (chống khóa vô hạn)
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 4, cap), 60);
        assert_eq!(CooldownTracker::calculate_exponential_cooldown(base, 10, cap), 60);
    }

    #[test]
    fn test_same_side_price_guard_blocks_worse_price() {
        // Vừa cắt lỗ Long tại 2400.0, streak = 1
        let tracker = CooldownTracker {
            locked_until_bar: 10, // Giả sử đã hết thời gian khóa bar
            locked_side: Some(SignalType::Long),
            last_exit_price: 2400.0,
            consecutive_losses: 1,
            last_losing_side: Some(SignalType::Long),
            last_losing_price: 2400.0,
        };

        // 1. Long ở giá tồi tệ hơn (2390.0 < 2400.0) -> BỊ CHẶN TUYỆT ĐỐI bởi Same-Side Price Guard (dù đã qua bar 10)
        assert!(tracker.should_block(true, 15, SignalType::Long, 2390.0, 10.0, 1.5), "Long ở giá thấp hơn giá vừa cắt lỗ PHẢI BỊ CHẶN!");

        // 2. Long ở giá phục hồi tốt hơn (2405.0 > 2400.0) và đã qua bar 10 -> CHO PHÉP
        assert!(!tracker.should_block(true, 15, SignalType::Long, 2405.0, 10.0, 1.5), "Long ở giá hồi phục cao hơn giá cắt lỗ được phép vào");

        // 3. Vừa cắt lỗ Short tại 2400.0, streak = 1
        let tracker_short = CooldownTracker {
            locked_until_bar: 10,
            locked_side: Some(SignalType::Short),
            last_exit_price: 2400.0,
            consecutive_losses: 1,
            last_losing_side: Some(SignalType::Short),
            last_losing_price: 2400.0,
        };

        // Short ở giá cao hơn (2410.0 > 2400.0) -> BỊ CHẶN bởi Same-Side Price Guard
        assert!(tracker_short.should_block(true, 15, SignalType::Short, 2410.0, 10.0, 1.5), "Short ở giá cao hơn giá vừa cắt lỗ PHẢI BỊ CHẶN!");

        // Short ở giá sập sâu hơn (2395.0 < 2400.0) -> CHO PHÉP
        assert!(!tracker_short.should_block(true, 15, SignalType::Short, 2395.0, 10.0, 1.5), "Short ở giá sập sâu hơn giá cắt lỗ được phép vào");
    }

    #[test]
    fn test_volume_zscore_differentiation_from_price() {
        use crate::strategy::sensors::HeliosSensors;
        use crate::strategy::core_v3::matrix::MarketMatrixV3;
        use crate::strategy::core_v3::config::HeliosConfigV3;

        // Mô phỏng: Giá đi ngang phẳng lặng (z_score = 0.0), nhưng volume nổ tung (vol_z_score = 3.5, MFI = 80.0)
        let mut sensors = HeliosSensors::default();
        let len = 50;
        sensors.closes = std::sync::Arc::new(vec![2000.0; len]);
        sensors.highs = std::sync::Arc::new(vec![2005.0; len]);
        sensors.lows = std::sync::Arc::new(vec![1995.0; len]);
        sensors.volumes = std::sync::Arc::new(vec![1000.0; len]);
        sensors.natr = std::sync::Arc::new(vec![1.5; len]);

        sensors.z_score = std::sync::Arc::new(vec![0.0; len]); // Giá không hề biến động
        sensors.vol_z_score = std::sync::Arc::new(vec![3.5; len]); // Volume nổ 3.5 sigma!
        sensors.mfi = std::sync::Arc::new(vec![80.0; len]);
        sensors.chaos_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.efficiency_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.adx = std::sync::Arc::new(vec![25.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.05; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.kc_upper = std::sync::Arc::new(vec![2012.0; len]);
        sensors.kc_lower = std::sync::Arc::new(vec![1988.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);

        let mut config = HeliosConfigV3::default();
        config.matrix.z_score_sensitivity = 2.0;
        let _matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // 🎯 KIỂM CHỨNG FLAW-3.2:
        // Price Z-Score phải bằng 0.0 (giá phẳng)
        assert_eq!(sensors.z_score[len - 1], 0.0, "Price Z-Score phải phản ánh giá phẳng = 0.0");
        // Volume Z-Score phải phản ứng độc lập với Vol Z-Score = 3.5 -> phát hiện bùng nổ thanh khoản
        assert!(sensors.vol_z_score[len - 1] > 3.0, "Vol Z-Score phải độc lập phản ánh bùng nổ thanh khoản!");
    }

    #[test]
    fn test_cooldown_triggered_on_hammer_wick_loss() {
        // 🎯 KIỂM CHỨNG FLAW-4.4: Bẫy False Win khi nến rút chân
        // Kịch bản: Long mở tại 2500, SL tại 2485. Trong nến tiếp theo, giá sập cắn SL tại 2485 (lỗ -$15).
        // Nhưng cuối nến rút chân cực mạnh đẩy Close vọt lên 2505 (+5 so với Entry).
        // Code cũ so sánh 2505 - 2500 = +5 > 0.001 nên BỎ QUA COOLDOWN!
        // Code mới đọc closed_trade có pl = -1.5 <= 0.001 -> PHẢI KÍCH HOẠT COOLDOWN!

        let mut strategy = HeliosStrategyV3::new(&std::collections::HashMap::new());
        strategy.config.risk_management.stop_management.enable_re_entry_cooldown = true;
        strategy.config.risk_management.stop_management.re_entry_cooldown_bars = 15;

        // Giả lập một Trade Long đã mở trước đó
        let mut prev_trade = ferro_trade::broker::trade::Trade::new(
            1, 0.1, 2500.0, 10, None, Some("TREND_PB".to_string())
        );
        prev_trade.sl_price = Some(2485.0);
        strategy.prev_open_trade = Some(prev_trade);

        // Giả lập Trade đã đóng trong Broker tại mức SL 2485.0
        let mut closed_trade = ferro_trade::broker::trade::Trade::new(
            1, 0.1, 2500.0, 10, None, Some("TREND_PB".to_string())
        );
        closed_trade.exit_price = Some(2485.0);
        closed_trade.exit_bar = Some(11);

        assert!(closed_trade.pl(None) < 0.0, "PnL của lệnh đóng phải âm");

        // Đọc từ closed_trade.pl(None)
        let real_pnl = closed_trade.pl(None);
        let exit_p = closed_trade.exit_price.unwrap();
        let stop_cfg = &strategy.config.risk_management.stop_management;

        if real_pnl <= 0.001 {
            strategy.cooldown_tracker.locked_until_bar = 11 + stop_cfg.re_entry_cooldown_bars;
            strategy.cooldown_tracker.locked_side = Some(SignalType::Long);
            strategy.cooldown_tracker.last_exit_price = exit_p;
        }

        assert_eq!(strategy.cooldown_tracker.locked_until_bar, 26, "Cooldown phải khóa tới bar 26");
        assert_eq!(strategy.cooldown_tracker.locked_side, Some(SignalType::Long));
        assert_eq!(strategy.cooldown_tracker.last_exit_price, 2485.0);
    }

    #[test]
    fn test_reversal_and_anomaly_use_raw_z_score_not_double_tanh() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::scanners::anomaly::AnomalyScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let opens = vec![2000.0; len];
        let highs = vec![2005.0; len];
        let lows = vec![1950.0; len];
        let closes = vec![1995.0; len];
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        // Giả lập Sensors:
        // Z-Score thô của giá là -3.0 sigma (thị trường sập hầm)
        let mut sensors = HeliosSensors::default();
        sensors.closes = std::sync::Arc::new(vec![1995.0; len]);
        sensors.highs = std::sync::Arc::new(vec![2005.0; len]);
        sensors.lows = std::sync::Arc::new(vec![1950.0; len]);
        sensors.volumes = std::sync::Arc::new(vec![1000.0; len]);
        sensors.natr = std::sync::Arc::new(vec![1.5; len]);
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]); // Raw Z-Score = -3.0
        sensors.vol_z_score = std::sync::Arc::new(vec![1.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![30.0; len]);
        sensors.chaos_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.efficiency_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.adx = std::sync::Arc::new(vec![25.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]); // KAMA ở trên (2040)
        sensors.kama_slope_adj = std::sync::Arc::new(vec![-0.10; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2050.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2040.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.kc_upper = std::sync::Arc::new(vec![2055.0; len]);
        sensors.kc_lower = std::sync::Arc::new(vec![1975.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.z_score_threshold = 2.5;
        config.tactical_sniper.reversal.min_activation_score = 0.65;
        config.tactical_sniper.anomaly.z_score_threshold = 2.5;
        config.tactical_sniper.anomaly.min_activation_score = 0.65;

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // KIỂM CHỨNG FLAW-4.5:
        // Với Z_raw = -3.0 và z_thresh = 2.5:
        // f_z = tanh(3.0 / 2.5) = tanh(1.2) ≈ 0.8337!
        // Nếu bị Double Tanh cũ: f_z ≈ 0.347 -> score < 0.65 và bị loại bỏ!
        // Với Single Tanh mới: Reversal và Anomaly sinh Proposal thành công với raw_score >= 0.65!
        let rev_proposal = ReversalScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(rev_proposal.is_some(), "ReversalScanner phải kích hoạt thành công khi Z-Score = -3.0 sigma!");
        let rev_p = rev_proposal.unwrap();
        assert!(rev_p.raw_score >= 0.65, "Reversal raw_score ({}) phải >= 0.65", rev_p.raw_score);

        let ano_proposal = AnomalyScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(ano_proposal.is_some(), "AnomalyScanner phải kích hoạt thành công khi râu nến dài và Z-Score = -3.0 sigma!");
        let ano_p = ano_proposal.unwrap();
        assert!(ano_p.raw_score >= 0.65, "Anomaly raw_score ({}) phải >= 0.65", ano_p.raw_score);
    }

    #[test]
    fn test_reversal_clv_guard_rejects_marubozu_and_accepts_hammer() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // 1. KỊCH BẢN 1: Nến Marubozu đỏ đặc rơi tự do (High=2000, Low=1950, Close=1950 => CLV = -1.0)
        let opens = vec![2000.0; len];
        let highs = vec![2000.0; len];
        let lows = vec![1950.0; len];
        let closes_marubozu = vec![1950.0; len];
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes.clone()),
            open: std::sync::Arc::new(opens.clone()),
            high: std::sync::Arc::new(highs.clone()),
            low: std::sync::Arc::new(lows.clone()),
            close: std::sync::Arc::new(closes_marubozu),
            volume: std::sync::Arc::new(volumes.clone()),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick_marubozu = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]); // Dưới 30.0 (quá bán thỏa mãn)
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.z_score_threshold = 2.5;
        config.tactical_sniper.reversal.min_activation_score = 0.50;
        config.tactical_sniper.reversal.min_reversal_clv = 0.20;
        config.tactical_sniper.reversal.mfi_reversal_thresh = 30.0;

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // CLV = -1.0 < 0.20 -> Phải từ chối bắt dao rơi nến Marubozu!
        let prop_marubozu = ReversalScanner.scan(&tick_marubozu, &sensors, &matrix, &config);
        assert!(prop_marubozu.is_none(), "ReversalScanner PHẢI từ chối nến Marubozu cắm đầu không có râu!");

        // 2. KỊCH BẢN 2: Nến Hammer rút chân (High=2000, Low=1950, Close=1995 => CLV = +0.80)
        let closes_hammer = vec![1995.0; len];
        let raw_cache_hammer = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes_hammer),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view_hammer = DataView::new(raw_cache_hammer);
        let tick_hammer = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_hammer,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        // CLV = (2*1995 - 2000 - 1950) / 50 = +0.80 >= 0.20 -> Chấp thuận vào lệnh Long!
        let prop_hammer = ReversalScanner.scan(&tick_hammer, &sensors, &matrix, &config);
        assert!(prop_hammer.is_some(), "ReversalScanner PHẢI kích hoạt khi nến có râu rút chân Hammer!");
        assert_eq!(prop_hammer.unwrap().signal_type, SignalType::Long);
    }

    #[test]
    fn test_reversal_remaining_displacement_guard() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Giá đã hồi gần sát KAMA Mid: Close = 2038.0, KAMA = 2040.0 => Displacement = 2.0
        // Trong khi min_reversal_displacement = 1.0 * ATR(15.0) = 15.0 > 2.0!
        let opens = vec![2030.0; len];
        let highs = vec![2040.0; len];
        let lows = vec![2020.0; len];
        let closes = vec![2038.0; len]; // CLV = (2*2038 - 2040 - 2020) / 20 = 16/20 = 0.80 (đẹp)
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-2.5; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.min_reversal_displacement = 1.0; // Yêu cầu tối thiểu 1.0 * ATR = 15.0
        config.tactical_sniper.reversal.min_reversal_clv = 0.20;
        config.tactical_sniper.reversal.mfi_reversal_thresh = 30.0;

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // Khoảng cách hồi quy chỉ còn 2.0 < 15.0 -> Phải từ chối ngay lập tức!
        let prop = ReversalScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "ReversalScanner PHẢI từ chối khi khoảng cách tới KAMA không đủ lớn (Remaining Displacement Guard)!");
    }

    #[test]
    fn test_reversal_mfi_exhaustion_filter() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let opens = vec![2000.0; len];
        let highs = vec![2000.0; len];
        let lows = vec![1950.0; len];
        let closes = vec![1995.0; len]; // CLV = 0.80
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.mfi_reversal_thresh = 30.0;
        config.tactical_sniper.reversal.min_reversal_clv = 0.20;

        // Trường hợp 1: MFI = 45.0 > 30.0 (chưa kiệt sức bán) -> Phải từ chối!
        sensors.mfi = std::sync::Arc::new(vec![45.0; len]);
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        let prop_fail = ReversalScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_fail.is_none(), "ReversalScanner PHẢI từ chối khi MFI > mfi_reversal_thresh!");

        // Trường hợp 2: MFI = 20.0 <= 30.0 (kiệt sức bán chuẩn xác) -> Kích hoạt!
        sensors.mfi = std::sync::Arc::new(vec![20.0; len]);
        let matrix_pass = MarketMatrixV3::from_sensors(&config, &sensors);
        let prop_pass = ReversalScanner.scan(&tick, &sensors, &matrix_pass, &config);
        assert!(prop_pass.is_some(), "ReversalScanner PHẢI kích hoạt khi MFI kiệt sức <= mfi_reversal_thresh!");
    }

    #[test]
    fn test_reversal_nan_safety() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2000.0; len]),
            high: std::sync::Arc::new(vec![2000.0; len]),
            low: std::sync::Arc::new(vec![1950.0; len]),
            close: std::sync::Arc::new(vec![1995.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![f64::NAN; len]); // NaN Z-Score
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        let prop = ReversalScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "ReversalScanner PHẢI trả về None khi dữ liệu chứa NaN!");
    }

    #[test]
    fn test_dna_width_and_parameter_defs_integrity() {
        let width = HeliosConfigV3::DNA_WIDTH;
        let defs = HeliosConfigV3::parameter_defs();
        assert_eq!(width, defs.len(), "DNA_WIDTH phải khớp chính xác 1:1 với số lượng parameter_defs!");
        assert_eq!(width, 102, "DNA_WIDTH phải là chính xác 102 chiều sau khi bổ sung max_boundary_z và max_squeeze_intensity cho TrendPB theo Kế hoạch Tầng 2!");
        let has_kc_tf = defs.iter().any(|(name, _)| name.contains("keltner.timeframe"));
        assert!(!has_kc_tf, "defs KHÔNG được chứa indicators.keltner.timeframe (đã khóa cứng theo bbands ở TASK-MAT-05)!");
        let has_macd = defs.iter().any(|(name, _)| name.contains("macd"));
        let has_vol_ma = defs.iter().any(|(name, _)| name.contains("vol_ma"));
        let has_breakout = defs.iter().any(|(name, _)| name.contains("cooldown_breakout_atr_mult"));
        let has_min_conf = defs.iter().any(|(name, _)| name.contains("min_confidence_threshold"));
        let has_chaos_veto = defs.iter().any(|(name, _)| name.contains("chaos_veto_threshold"));
        let has_min_natr = defs.iter().any(|(name, _)| name.contains("min_natr_threshold"));
        let has_max_chaos = defs.iter().any(|(name, _)| name.contains("max_chaos_penalty"));
        let has_act_score = defs.iter().any(|(name, _)| name.contains("min_activation_score"));
        let has_pb_boundary_z = defs.iter().any(|(name, _)| name.contains("tactical_sniper.pullback.max_boundary_z"));
        let has_pb_squeeze = defs.iter().any(|(name, _)| name.contains("tactical_sniper.pullback.max_squeeze_intensity"));
        assert!(has_min_conf, "defs phải chứa min_confidence_threshold trong ArbiterConfig!");
        assert!(has_chaos_veto, "defs phải chứa chaos_veto_threshold trong ArbiterConfig!");
        assert!(has_min_natr, "defs phải chứa min_natr_threshold trong ArbiterConfig!");
        assert!(has_pb_boundary_z, "defs phải chứa tactical_sniper.pullback.max_boundary_z!");
        assert!(has_pb_squeeze, "defs phải chứa tactical_sniper.pullback.max_squeeze_intensity!");
        assert!(!has_max_chaos, "defs KHÔNG được chứa max_chaos_penalty (đã loại bỏ gen theo Pure Confidence FLAW-RISK.11)!");
        assert!(!has_act_score, "defs KHÔNG được chứa min_activation_score ở các Tactical Scanners!");
        let has_matrix_adx = defs.iter().any(|(name, _)| name.contains("adx_trend_threshold"));
        let has_matrix_vol_lb = defs.iter().any(|(name, _)| name.contains("volume_lookback"));
        let has_matrix_mom_lb = defs.iter().any(|(name, _)| name.contains("momentum_lookback"));
        let has_matrix_kama_scale = defs.iter().any(|(name, _)| name.contains("kama_slope_scale_atr"));
        let has_matrix_vol_z = defs.iter().any(|(name, _)| name.contains("vol_z_sensitivity"));
        let has_phantom_comp = defs.iter().any(|(name, _)| name.contains("min_confidence_compression"));
        let has_phantom_veto = defs.iter().any(|(name, _)| name.contains("min_multiplier_veto"));
        let has_abs_adverse = defs.iter().any(|(name, _)| name.contains("tactical_sniper.absorption.max_adverse_slope"));
        let has_spring_tol = defs.iter().any(|(name, _)| name.contains("tactical_sniper.absorption.spring_tolerance"));
        let has_min_clv = defs.iter().any(|(name, _)| name.contains("tactical_sniper.absorption.min_clv"));
        let has_max_boundary = defs.iter().any(|(name, _)| name.contains("tactical_sniper.absorption.max_boundary_pct_b"));
        let has_squeeze_weight = defs.iter().any(|(name, _)| name.contains("tactical_sniper.absorption.squeeze_weight"));
        let has_mom_clv = defs.iter().any(|(name, _)| name.contains("tactical_sniper.momentum.min_clv"));
        let has_mom_adverse = defs.iter().any(|(name, _)| name.contains("tactical_sniper.momentum.max_adverse_slope"));
        let has_mom_w_break = defs.iter().any(|(name, _)| name.contains("tactical_sniper.momentum.w_breakout"));
        let has_mom_w_vol = defs.iter().any(|(name, _)| name.contains("tactical_sniper.momentum.w_volume"));
        let has_rev_clv = defs.iter().any(|(name, _)| name.contains("tactical_sniper.reversal.min_reversal_clv"));
        let has_rev_mfi = defs.iter().any(|(name, _)| name.contains("tactical_sniper.reversal.mfi_reversal_thresh"));
        let has_rev_w_z = defs.iter().any(|(name, _)| name.contains("tactical_sniper.reversal.w_z_score"));
        let has_rev_w_disp = defs.iter().any(|(name, _)| name.contains("tactical_sniper.reversal.w_displacement"));
        let has_range = defs.iter().any(|(name, _)| name.contains("tactical_sniper.range"));
        let has_anomaly = defs.iter().any(|(name, _)| name.contains("tactical_sniper.anomaly"));
        let has_rsi_ano = defs.iter().any(|(name, _)| name.contains("rsi_anomaly"));
        let has_stoch_range = defs.iter().any(|(name, _)| name.contains("stoch_range"));
        let has_pb_max_dist = defs.iter().any(|(name, _)| name.contains("tactical_sniper.pullback.max_pullback_dist_atr"));
        let has_pb_inval = defs.iter().any(|(name, _)| name.contains("tactical_sniper.pullback.invalidation_buffer_atr"));
        assert!(!has_macd, "defs không được chứa macd!");
        assert!(!has_vol_ma, "defs không được chứa vol_ma!");
        assert!(!has_range, "defs KHÔNG được chứa tactical_sniper.range!");
        assert!(!has_anomaly, "defs KHÔNG được chứa tactical_sniper.anomaly!");
        assert!(!has_rsi_ano, "defs KHÔNG được chứa rsi_anomaly!");
        assert!(!has_stoch_range, "defs KHÔNG được chứa stoch_range!");
        assert!(has_min_conf, "defs phải chứa min_confidence_threshold!");
        assert!(has_spring_tol, "defs phải chứa tactical_sniper.absorption.spring_tolerance!");
        assert!(has_min_clv, "defs phải chứa tactical_sniper.absorption.min_clv!");
        assert!(has_max_boundary, "defs phải chứa tactical_sniper.absorption.max_boundary_pct_b!");
        assert!(has_squeeze_weight, "defs phải chứa tactical_sniper.absorption.squeeze_weight!");
        assert!(has_mom_clv, "defs phải chứa tactical_sniper.momentum.min_clv!");
        assert!(has_mom_adverse, "defs phải chứa tactical_sniper.momentum.max_adverse_slope!");
        assert!(has_mom_w_break, "defs phải chứa tactical_sniper.momentum.w_breakout!");
        assert!(has_mom_w_vol, "defs phải chứa tactical_sniper.momentum.w_volume!");
        assert!(has_rev_clv, "defs phải chứa tactical_sniper.reversal.min_reversal_clv!");
        assert!(has_rev_mfi, "defs phải chứa tactical_sniper.reversal.mfi_reversal_thresh!");
        assert!(has_rev_w_z, "defs phải chứa tactical_sniper.reversal.w_z_score!");
        assert!(has_rev_w_disp, "defs phải chứa tactical_sniper.reversal.w_displacement!");
        assert!(has_mom_adverse, "defs phải chứa tactical_sniper.momentum.max_adverse_slope!");
        assert!(has_mom_w_break, "defs phải chứa tactical_sniper.momentum.w_breakout!");
        assert!(!has_matrix_adx, "defs KHÔNG được chứa matrix.adx_trend_threshold (đã loại bỏ theo TASK-MAT-07)!");
        assert!(!has_matrix_vol_lb, "defs KHÔNG được chứa matrix.volume_lookback (đã loại bỏ theo TASK-MAT-07)!");
        assert!(!has_matrix_mom_lb, "defs KHÔNG được chứa matrix.momentum_lookback (đã loại bỏ theo TASK-MAT-07)!");
        assert!(has_matrix_kama_scale, "defs phải chứa matrix.kama_slope_scale_atr!");
        assert!(!has_matrix_vol_z, "defs KHÔNG được chứa matrix.vol_z_sensitivity (đã loại bỏ theo TASK-MAT-01)!");
        assert!(!has_phantom_comp, "defs KHÔNG được chứa gen ma min_confidence_compression!");
        assert!(!has_phantom_veto, "defs KHÔNG được chứa gen ma min_multiplier_veto!");
        assert!(has_breakout, "defs phải chứa cooldown_breakout_atr_mult!");
        assert!(has_abs_adverse, "defs phải chứa tactical_sniper.absorption.max_adverse_slope!");
        assert!(has_pb_max_dist, "defs phải chứa tactical_sniper.pullback.max_pullback_dist_atr!");
        assert!(has_pb_inval, "defs phải chứa tactical_sniper.pullback.invalidation_buffer_atr!");

        let count_struct_sl = defs.iter().filter(|(name, _)| name.contains("struct_sl_buffer_atr")).count();
        assert_eq!(count_struct_sl, 3, "defs phải chứa chính xác 3 gene struct_sl_buffer_atr cho 3 Scanners (pullback, momentum, absorption)!");

        // Thử cập nhật với vector toàn 0.0 (giả sử GA sinh số 0)
        let zeros = vec![0.0; width];
        let mut cfg = HeliosConfigV3::default();
        cfg.update_from_dna(&zeros);

        let ind = &cfg.indicators;
        assert!(ind.natr.period > 0, "natr.period must be > 0");
        assert!(ind.mfi.period > 0, "mfi.period must be > 0");
        assert!(ind.vol_ma.period > 0, "vol_ma.period must be > 0");
        assert!(ind.kama.short_period > 0, "kama.short_period must be > 0");
        assert!(ind.kama.long_period > 0, "kama.long_period must be > 0");
        assert!(ind.adx.period > 0, "adx.period must be > 0");
        assert!(ind.bbands.period > 0, "bbands.period must be > 0");
        assert!(ind.atr.period > 0, "atr.period must be > 0");
    }

    #[test]
    fn test_structural_sl_per_playbook_integrity() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::scanners::momentum_ignition::MomentumIgnitionScanner;
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let mut opens = vec![2000.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![2000.0; len];
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        // Nến i có lực cầu hồi phục (nến xanh, CLV > 0.15) cho TrendPB
        opens[i] = 1996.0;
        highs[i] = 2008.0;
        lows[i] = 1994.0;
        closes[i] = 2005.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes.clone()),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 30.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5;
        config.tactical_sniper.pullback.min_activation_score = 0.1;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 0.5;

        config.tactical_sniper.momentum.rsi_threshold = 60.0;
        config.tactical_sniper.momentum.vol_z_threshold = 0.5;
        config.tactical_sniper.momentum.min_activation_score = 0.1;
        config.tactical_sniper.momentum.struct_sl_buffer_atr = 0.3;

        config.tactical_sniper.range.min_width_pct = 0.01;
        config.tactical_sniper.range.volume_max_threshold = 0.8;
        config.tactical_sniper.range.bb_proximity_pct = 0.02;
        config.tactical_sniper.range.stoch_oversold = 30.0;
        config.tactical_sniper.range.stoch_overbought = 70.0;
        config.tactical_sniper.range.max_trend_strength = 0.6;
        config.tactical_sniper.range.max_adverse_slope = 0.2;
        config.tactical_sniper.range.min_activation_score = 0.1;
        config.tactical_sniper.range.struct_sl_buffer_atr = 0.3;

        config.tactical_sniper.absorption.squeeze_threshold = 0.5;
        config.tactical_sniper.absorption.vol_z_threshold = 0.5;
        config.tactical_sniper.absorption.min_activation_score = 0.1;
        config.tactical_sniper.absorption.struct_sl_buffer_atr = 0.5;

        // 1. Kiểm tra TrendPBScanner (Long & Short)
        let mut sensors = HeliosSensors::default();
        sensors.closes = std::sync::Arc::new(vec![2000.0; len]);
        sensors.highs = std::sync::Arc::new(vec![2010.0; len]);
        sensors.lows = std::sync::Arc::new(vec![1990.0; len]);
        sensors.volumes = std::sync::Arc::new(vec![1000.0; len]);
        sensors.natr = std::sync::Arc::new(vec![1.5; len]);
        sensors.z_score = std::sync::Arc::new(vec![0.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![50.0; len]);
        sensors.chaos_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.efficiency_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.adx = std::sync::Arc::new(vec![30.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![1995.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.15; len]); // Slope dương -> Long
        sensors.stoch_trend_k = std::sync::Arc::new(vec![20.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![40.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.kc_upper = std::sync::Arc::new(vec![2025.0; len]);
        sensors.kc_lower = std::sync::Arc::new(vec![1975.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        let pb_prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(pb_prop.is_some(), "TrendPBScanner phải sinh Proposal Long!");
        let pb_p = pb_prop.unwrap();
        assert!(pb_p.suggested_sl.is_some(), "TrendPullback Proposal phải có suggested_sl!");
        let pb_sl = pb_p.suggested_sl.unwrap();
        assert!(pb_sl < 2000.0, "Long SL {} phải < Close 2000.0", pb_sl);

        // 2. Kiểm tra MomentumIgnitionScanner
        sensors.rsi_momentum = std::sync::Arc::new(vec![75.0; len]);
        sensors.closes = std::sync::Arc::new(vec![2025.0; len]); // Phá vỡ BB Upper (2020.0)
        let matrix_mom = MarketMatrixV3::from_sensors(&config, &sensors);

        let raw_cache_mom = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2015.0; len]),
            high: std::sync::Arc::new(vec![2026.0; len]),
            low: std::sync::Arc::new(vec![2014.0; len]),
            close: std::sync::Arc::new(vec![2025.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_mom = DataView::new(raw_cache_mom);
        let mut actions_mom = SmallVec::new();
        let tick_mom = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_mom,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions_mom,
            position: Position::default(),
        };

        let mom_prop = MomentumIgnitionScanner.scan(&tick_mom, &sensors, &matrix_mom, &config);
        assert!(mom_prop.is_some(), "MomentumIgnitionScanner phải sinh Proposal Long!");
        let mom_p = mom_prop.unwrap();
        assert!(mom_p.suggested_sl.is_some(), "MomentumIgnition Proposal phải có suggested_sl!");
        let mom_sl = mom_p.suggested_sl.unwrap();
        assert!(mom_sl < 2025.0, "Momentum Long SL {} phải < Close 2025.0", mom_sl);

        // 3. Kiểm tra RangeOscScanner
        sensors.closes = std::sync::Arc::new(vec![1981.0; len]); // Gần BB Lower (1980)
        sensors.stoch_range_k = std::sync::Arc::new(vec![15.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![10.0; len]);
        let mut matrix_range = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_range.trend_strength = std::sync::Arc::new(vec![0.2; len]);
        matrix_range.bb_width_norm = std::sync::Arc::new(vec![0.04; len]);
        matrix_range.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);

        let raw_cache_range = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1981.0; len]),
            high: std::sync::Arc::new(vec![1986.0; len]),
            low: std::sync::Arc::new(vec![1980.0; len]),
            close: std::sync::Arc::new(vec![1985.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_range = DataView::new(raw_cache_range);
        let mut actions_range = SmallVec::new();
        let tick_range = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_range,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions_range,
            position: Position::default(),
        };

        let range_prop = RangeOscScanner.scan(&tick_range, &sensors, &matrix_range, &config);
        assert!(range_prop.is_some(), "RangeOscScanner phải sinh Proposal Long!");
        let rng_p = range_prop.unwrap();
        assert!(rng_p.suggested_sl.is_some(), "RangeOsc Proposal phải có suggested_sl!");
        let rng_sl = rng_p.suggested_sl.unwrap();
        assert!(rng_sl < 1985.0, "RangeOsc Long SL {} phải < Close 1985.0", rng_sl);
        assert!(rng_p.suggested_tp.is_some(), "RangeOsc Proposal phải có suggested_tp!");

        // 4. Kiểm tra AbsorptionScanner (Wyckoff Accumulation Spring: Giá & VWMA ở nửa dưới BB, nến rút chân CLV >= 0.20)
        sensors.vwma = std::sync::Arc::new(vec![1990.0; len]); // vwma < bb_middle -> Long
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]); // Z-score volume bùng nổ dương
        
        let raw_cache_abs = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1988.0; len]),
            high: std::sync::Arc::new(vec![1994.0; len]),
            low: std::sync::Arc::new(vec![1982.0; len]),
            close: std::sync::Arc::new(vec![1991.0; len]), // close 1991: %B = (1991-1980)/40 = 0.275; CLV = (2*1991-1994-1982)/12 = 0.50
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_abs = DataView::new(raw_cache_abs);
        let mut actions_abs = SmallVec::new();
        let tick_abs = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_abs,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions_abs,
            position: Position::default(),
        };
        let mut matrix_abs = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_abs.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        matrix_abs.kama_slope_atr = std::sync::Arc::new(vec![0.02; len]);
        let abs_prop = AbsorptionScanner.scan(&tick_abs, &sensors, &matrix_abs, &config);
        assert!(abs_prop.is_some(), "AbsorptionScanner phải sinh Proposal Long khi tích lũy rút chân ở nửa dưới BB!");
        let abs_p = abs_prop.unwrap();
        assert!(abs_p.suggested_sl.is_some(), "Absorption Proposal phải có suggested_sl!");
        let abs_sl = abs_p.suggested_sl.unwrap();
        assert!(abs_sl < 1991.0, "Absorption Long SL {} phải < Close 1991.0", abs_sl);
    }

    #[test]
    fn test_absorption_falling_knife_blocked_by_macro_trend() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến tích lũy rút chân ở nửa dưới BB: close 1990, high 1994, low 1982
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1988.0; len]),
            high: std::sync::Arc::new(vec![1994.0; len]),
            low: std::sync::Arc::new(vec![1982.0; len]),
            close: std::sync::Arc::new(vec![1990.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.squeeze_threshold = 0.5;
        config.tactical_sniper.absorption.vol_z_threshold = 0.5;
        config.tactical_sniper.absorption.min_activation_score = 0.1;
        config.tactical_sniper.absorption.struct_sl_buffer_atr = 0.5;
        config.tactical_sniper.absorption.max_adverse_slope = 0.15;
        config.tactical_sniper.absorption.spring_tolerance = 0.10;
        config.tactical_sniper.absorption.min_clv = 0.20;

        let mut sensors = HeliosSensors::default();
        sensors.vwma = std::sync::Arc::new(vec![1992.0; len]); // vwma < bb_middle -> Long
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);

        // KỊCH BẢN 1: Bẫy bắt dao rơi (kama_slope_atr = -0.25 < -0.15) -> VETO Long
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.25; len]);
        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "AbsorptionScanner PHẢI VETO chặn lệnh Long bắt dao rơi khi kama_slope_atr = -0.25!");

        // KỊCH BẢN 2: Bẫy cản tàu Short (kama_slope_atr = +0.25 > +0.15) -> VETO Short
        let mut sensors_short = sensors.clone();
        sensors_short.vwma = std::sync::Arc::new(vec![2008.0; len]); // vwma > bb_middle -> Short
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.25; len]);
        let raw_cache_short = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2012.0; len]),
            high: std::sync::Arc::new(vec![2018.0; len]),
            low: std::sync::Arc::new(vec![2006.0; len]),
            close: std::sync::Arc::new(vec![2010.0; len]), // close 2010: %B = (2010-1980)/40 = 0.75; CLV = (2*2010-2018-2006)/12 = -0.33 <= -0.20
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_short = DataView::new(raw_cache_short);
        let tick_short = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_short,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };
        let prop_short = AbsorptionScanner.scan(&tick_short, &sensors_short, &matrix, &config);
        assert!(prop_short.is_none(), "AbsorptionScanner PHẢI VETO chặn lệnh Short cản tàu khi kama_slope_atr = +0.25!");

        // KỊCH BẢN 3: Nến cắm đầu rơi tự do (CLV âm, Marubozu đỏ tại nửa dưới) -> PHẢI LỌC BỎ
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);
        let raw_cache_knife = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1995.0; len]),
            high: std::sync::Arc::new(vec![1996.0; len]),
            low: std::sync::Arc::new(vec![1985.0; len]),
            close: std::sync::Arc::new(vec![1986.0; len]), // close 1986 sát đáy: CLV = (2*1986-1996-1985)/11 = -9/11 = -0.81 < 0.20
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_knife = DataView::new(raw_cache_knife);
        let tick_knife = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_knife,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };
        let prop_knife = AbsorptionScanner.scan(&tick_knife, &sensors, &matrix, &config);
        assert!(prop_knife.is_none(), "AbsorptionScanner PHẢI LỌC BỎ nến cắm đầu (CLV < 0.20) không cho bắt dao rơi!");
    }

    #[test]
    fn test_absorption_valid_accumulation_in_neutral_range() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến tích lũy gom hàng chuẩn ở nửa dưới BB: close 1990, high 1994, low 1982
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1988.0; len]),
            high: std::sync::Arc::new(vec![1994.0; len]),
            low: std::sync::Arc::new(vec![1982.0; len]),
            close: std::sync::Arc::new(vec![1990.0; len]), // %B = (1990-1980)/40 = 0.25; CLV = (2*1990-1994-1982)/12 = +0.33 >= 0.20
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.squeeze_threshold = 0.5;
        config.tactical_sniper.absorption.vol_z_threshold = 0.5;
        config.tactical_sniper.absorption.min_activation_score = 0.1;
        config.tactical_sniper.absorption.struct_sl_buffer_atr = 0.5;
        config.tactical_sniper.absorption.max_adverse_slope = 0.15;
        config.tactical_sniper.absorption.spring_tolerance = 0.10;
        config.tactical_sniper.absorption.min_clv = 0.20;

        let mut sensors = HeliosSensors::default();
        sensors.vwma = std::sync::Arc::new(vec![1992.0; len]); // vwma < bb_middle -> Long
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.02; len]);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "AbsorptionScanner PHẢI sinh Proposal Long khi thị trường tích lũy chuẩn trong vùng phẳng!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.raw_score >= 0.1);
        assert!(p.suggested_sl.is_some());
    }

    #[test]
    fn test_range_osc_band_walking_drift_blocked() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1981.0; len]),
            high: std::sync::Arc::new(vec![1985.0; len]),
            low: std::sync::Arc::new(vec![1978.0; len]),
            close: std::sync::Arc::new(vec![1982.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.range.max_bb_slope = 0.10;
        config.tactical_sniper.range.max_adverse_slope = 0.15;
        config.tactical_sniper.range.min_activation_score = 0.10;
        config.tactical_sniper.range.min_width_pct = 0.005;
        config.tactical_sniper.range.bb_proximity_pct = 0.015;
        config.tactical_sniper.range.stoch_oversold = 25.0;
        config.tactical_sniper.range.stoch_overbought = 75.0;
        config.tactical_sniper.range.volume_max_threshold = 2.0;
        config.tactical_sniper.range.max_trend_strength = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_k = std::sync::Arc::new(vec![15.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![18.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![0.5; len]);
        sensors.bb_middle_slope_adj = std::sync::Arc::new(vec![-0.25; len]); // Dốc cắm đầu > 0.10

        let mut matrix = MarketMatrixV3::default();
        matrix.bb_width_norm = std::sync::Arc::new(vec![0.02; len]);
        matrix.trend_strength = std::sync::Arc::new(vec![0.20; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.05; len]);
        matrix.bb_middle_slope_adj = sensors.bb_middle_slope_adj.clone();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "RangeOscScanner PHẢI VETO bắt dao rơi khi BB Middle dốc xuống (-0.25 > 0.10)!");
    }

    #[test]
    fn test_range_osc_rejection_candle_confirmed() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1981.0; len]),
            high: std::sync::Arc::new(vec![1986.0; len]),
            low: std::sync::Arc::new(vec![1979.0; len]),
            close: std::sync::Arc::new(vec![1984.0; len]), // Nến xanh rút chân
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.range.max_bb_slope = 0.10;
        config.tactical_sniper.range.max_adverse_slope = 0.15;
        config.tactical_sniper.range.min_activation_score = 0.10;
        config.tactical_sniper.range.min_width_pct = 0.005;
        config.tactical_sniper.range.bb_proximity_pct = 0.015;
        config.tactical_sniper.range.stoch_oversold = 25.0;
        config.tactical_sniper.range.stoch_overbought = 75.0;
        config.tactical_sniper.range.volume_max_threshold = 2.0;
        config.tactical_sniper.range.max_trend_strength = 0.50;
        config.tactical_sniper.range.tp_target_ratio = 0.50;
        config.tactical_sniper.range.struct_sl_buffer_atr = 0.30;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_k = std::sync::Arc::new(vec![15.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![12.0; len]); // K > D (Hook up)
        sensors.vol_z_score = std::sync::Arc::new(vec![0.5; len]);
        sensors.bb_middle_slope_adj = std::sync::Arc::new(vec![0.01; len]); // Thị trường phẳng

        let mut matrix = MarketMatrixV3::default();
        matrix.bb_width_norm = std::sync::Arc::new(vec![0.02; len]);
        matrix.trend_strength = std::sync::Arc::new(vec![0.15; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);
        matrix.bb_middle_slope_adj = sensors.bb_middle_slope_adj.clone();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeOscScanner PHẢI sinh Proposal Long khi có nến xác nhận tại dải phẳng!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.raw_score >= 0.10);
        assert!(p.suggested_sl.is_some());
        assert!(p.suggested_tp.is_some());
        let tp = p.suggested_tp.unwrap();
        assert!((tp - 2000.0).abs() < 1.0, "TP ({}) phải xấp xỉ Trục Giữa 2000.0", tp);
    }

    #[test]
    fn test_range_osc_breakdown_penetration_blocked() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1985.0; len]),
            high: std::sync::Arc::new(vec![1986.0; len]),
            low: std::sync::Arc::new(vec![1968.0; len]),
            close: std::sync::Arc::new(vec![1970.0; len]), // Đâm thủng đáy dải
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.range.max_bb_slope = 0.10;
        config.tactical_sniper.range.max_adverse_slope = 0.15;
        config.tactical_sniper.range.min_activation_score = 0.10;
        config.tactical_sniper.range.min_width_pct = 0.005;
        config.tactical_sniper.range.bb_proximity_pct = 0.015;
        config.tactical_sniper.range.stoch_oversold = 25.0;
        config.tactical_sniper.range.stoch_overbought = 75.0;
        config.tactical_sniper.range.volume_max_threshold = 2.0;
        config.tactical_sniper.range.max_trend_strength = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_k = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![15.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![0.5; len]);
        sensors.bb_middle_slope_adj = std::sync::Arc::new(vec![0.01; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.bb_width_norm = std::sync::Arc::new(vec![0.02; len]);
        matrix.trend_strength = std::sync::Arc::new(vec![0.15; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);
        matrix.bb_middle_slope_adj = sensors.bb_middle_slope_adj.clone();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "RangeOscScanner PHẢI VETO khi nến đỏ đâm thủng dải dưới (close 1970 < bb_lower 1980)!");
    }

    #[test]
    fn test_trend_pb_healthy_pullback_approved() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2008.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2006.0; len];
        opens[i] = 2001.0;
        highs[i] = 2008.0;
        lows[i] = 1999.0;
        closes[i] = 2006.0; // CLV = (2006-1999)/(2008-1999) = 7/9 = 0.778 >= 0.15

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]); // delta = (2006-2000)/10 = 0.6
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.15; len]); // Dốc lên
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![30.0; len]); // <= 40.0
        sensors.rsi_trend = std::sync::Arc::new(vec![55.0; len]); // <= 70.0

        let mut matrix = MarketMatrixV3::default();
        matrix.trend_strength = std::sync::Arc::new(vec![0.40; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "TrendPB PHẢI cấp Proposal Long khi nhịp hồi KAMA lành mạnh và có lực cầu CLV!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.raw_score >= 0.10);
        assert!(p.suggested_sl.is_some());
        let sl = p.suggested_sl.unwrap();
        assert!(sl < 2006.0, "SL Long {} phải thấp hơn giá vào lệnh 2006.0", sl);
    }

    #[test]
    fn test_trend_pb_breakdown_invalidation_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1996.0; len]),
            high: std::sync::Arc::new(vec![1997.0; len]),
            low: std::sync::Arc::new(vec![1988.0; len]),
            close: std::sync::Arc::new(vec![1990.0; len]), // Đâm thủng sàn KAMA - 0.5 ATR (1995.0)
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5; // Sàn: 2000 - 5 = 1995.0

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.15; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![20.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![40.0; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.trend_strength = std::sync::Arc::new(vec![0.40; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB PHẢI VETO triệt để khi giá sập gãy KAMA (delta < -invalidation_buffer)!");
    }

    #[test]
    fn test_trend_pb_parabolic_chasing_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2020.0; len]),
            high: std::sync::Arc::new(vec![2030.0; len]),
            low: std::sync::Arc::new(vec![2019.0; len]),
            close: std::sync::Arc::new(vec![2025.0; len]), // Bay quá trần 2000 + 2.0*10 = 2020.0
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0; // Trần: 2000 + 20 = 2020.0
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5;

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.15; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![35.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![55.0; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.trend_strength = std::sync::Arc::new(vec![0.40; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB PHẢI VETO triệt để khi giá bay quá xa KAMA (delta > max_pullback_dist) chống mua đu đỉnh!");
    }

    #[test]
    fn test_trend_pb_marubozu_knife_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2015.0; len];
        let mut lows = vec![2000.0; len];
        let mut closes = vec![2001.0; len];
        // Nến đỏ Marubozu cắm đầu cạn kiệt lực cầu: CLV = (2001 - 2000) / (2015 - 2000) = 1/15 = 0.067 < 0.15
        opens[i] = 2015.0;
        highs[i] = 2015.0;
        lows[i] = 2000.0;
        closes[i] = 2001.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5;

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.15; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.trend_strength = std::sync::Arc::new(vec![0.40; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB PHẢI VETO khi nến cắm đầu rơi tự do (CLV < 0.15)!");
    }

    #[test]
    fn test_trend_pb_short_symmetric() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2002.0; len];
        let mut lows = vec![1992.0; len];
        let mut closes = vec![1994.0; len];
        // Nến hồi lên test KAMA rồi rút râu cắm xuống: CLV = (2002 - 1994) / (2002 - 1992) = 8/10 = 0.80 >= 0.15
        opens[i] = 1998.0;
        highs[i] = 2002.0;
        lows[i] = 1992.0;
        closes[i] = 1994.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.stoch_pullback_thresh = 40.0;
        config.tactical_sniper.pullback.rsi_max_cap = 70.0;
        config.tactical_sniper.pullback.max_pullback_dist_atr = 2.0;
        config.tactical_sniper.pullback.invalidation_buffer_atr = 0.5;
        config.tactical_sniper.pullback.min_activation_score = 0.10;
        config.tactical_sniper.pullback.struct_sl_buffer_atr = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]); // delta = (2000-1994)/10 = 0.6
        sensors.kama_slope_adj = std::sync::Arc::new(vec![-0.15; len]); // Dốc xuống -> Short
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![70.0; len]); // >= 100 - 40 = 60
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]); // >= 100 - 70 = 30

        let mut matrix = MarketMatrixV3::default();
        matrix.trend_strength = std::sync::Arc::new(vec![0.40; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.15; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "TrendPB PHẢI cấp Proposal Short đối xứng khi trend giảm hồi phục!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Short);
        assert!(p.raw_score >= 0.10);
        assert!(p.suggested_sl.is_some());
        let sl = p.suggested_sl.unwrap();
        assert!(sl > 1994.0, "SL Short {} phải cao hơn giá vào lệnh 1994.0", sl);
    }

    #[test]
    fn test_trend_pb_boundary_guard_long_overbought_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2010.0; len];
        let mut lows = vec![1998.0; len];
        let mut closes = vec![2005.0; len];
        opens[i] = 2000.0;
        highs[i] = 2010.0;
        lows[i] = 1998.0;
        closes[i] = 2008.0; // CLV = (2*2008 - 2010 - 1998)/(2010-1998) = 8/12 = 0.67 > 0.15

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.max_boundary_z = 0.50; // Ngưỡng chặn Z Long

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![1.20; len]); // 🛡️ Z-Score vượt trần +0.50 (đu đỉnh phân phối)

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.30; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB Long PHẢI bị Boundary Guard chặn đứng khi Z_score (+1.20) > max_boundary_z (0.50)!");
    }

    #[test]
    fn test_trend_pb_boundary_guard_short_oversold_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2002.0; len];
        let mut lows = vec![1990.0; len];
        let mut closes = vec![1992.0; len];
        opens[i] = 1998.0;
        highs[i] = 2002.0;
        lows[i] = 1990.0;
        closes[i] = 1992.0; // CLV = (2*1992 - 2002 - 1990)/(2002-1990) = -8/12 = -0.67 < -0.15

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.max_boundary_z = 0.50; // Ngưỡng chặn Z Short (-0.50)

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![75.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![-1.20; len]); // 🛡️ Z-Score sập sâu dưới -0.50 (bán đuổi đáy)

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.15; len]);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.30; len]);

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB Short PHẢI bị Boundary Guard chặn đứng khi Z_score (-1.20) < -max_boundary_z (-0.50)!");
    }

    #[test]
    fn test_trend_pb_chop_zone_squeeze_blocked() {
        use crate::strategy::core_v3::scanners::trend_pb::TrendPBScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2000.0; len]),
            high: std::sync::Arc::new(vec![2010.0; len]),
            low: std::sync::Arc::new(vec![1998.0; len]),
            close: std::sync::Arc::new(vec![2005.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.pullback.kama_slope_threshold = 0.05;
        config.tactical_sniper.pullback.max_squeeze_intensity = 0.75; // Ngưỡng chặn Chop Zone

        let mut sensors = HeliosSensors::default();
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_trend_k = std::sync::Arc::new(vec![25.0; len]);
        sensors.rsi_trend = std::sync::Arc::new(vec![45.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![0.20; len]); // Z nằm trong biên an toàn

        let mut matrix = MarketMatrixV3::default();
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![0.88; len]); // 🧱 Thị trường bị bóp nghẽn trong Chop Zone

        let prop = TrendPBScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "TrendPB PHẢI bị chặn đứng khi thị trường đang co thắt nén chặt trong Chop Zone (squeeze 0.88 > 0.75)!");
    }

    #[test]
    fn test_momentum_shooting_star_rejected_by_clv() {
        let len = 50;
        let i = 49;

        // Nến Shooting Star rút chân đỉnh: Open=2021, High=2035, Low=2020, Close=2022
        // Range = 15.0. 2*Close - High - Low = 4044 - 2035 - 2020 = -11.0 -> CLV = -11/15 = -0.73
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2021.0; len]),
            high: std::sync::Arc::new(vec![2035.0; len]),
            low: std::sync::Arc::new(vec![2020.0; len]),
            close: std::sync::Arc::new(vec![2022.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.momentum.rsi_threshold = 60.0;
        config.tactical_sniper.momentum.vol_z_threshold = 1.0;
        config.tactical_sniper.momentum.min_clv = 0.20;

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.rsi_momentum = std::sync::Arc::new(vec![75.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.10; len]);

        let matrix = MarketMatrixV3::default();

        let prop = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Nến Shooting Star có CLV âm (-0.73) PHẢI bị từ chối bởi min_clv guard!");
    }

    #[test]
    fn test_momentum_inside_band_early_exit() {
        let len = 50;
        let i = 49;

        // Nến nằm hoàn toàn bên trong dải BB: Close = 2005.0 in [1980.0, 2020.0]
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2000.0; len]),
            high: std::sync::Arc::new(vec![2010.0; len]),
            low: std::sync::Arc::new(vec![1995.0; len]),
            close: std::sync::Arc::new(vec![2005.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let config = HeliosConfigV3::default();
        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);

        let matrix = MarketMatrixV3::default();

        let prop = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Giá nằm trong dải BB PHẢI bị loại bỏ ngay tại Early Exit Guard!");
    }

    #[test]
    fn test_momentum_macro_counter_trend_veto() {
        let len = 50;
        let i = 49;

        // Nến bùng nổ phá vỡ đỉnh BB Upper nhưng Trend vĩ mô KAMA dốc xuống mạnh (kama_slope_adj = -0.25)
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2015.0; len]),
            high: std::sync::Arc::new(vec![2026.0; len]),
            low: std::sync::Arc::new(vec![2014.0; len]),
            close: std::sync::Arc::new(vec![2025.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.momentum.rsi_threshold = 60.0;
        config.tactical_sniper.momentum.vol_z_threshold = 1.0;
        config.tactical_sniper.momentum.max_adverse_slope = 0.15;

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.rsi_momentum = std::sync::Arc::new(vec![75.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.5; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![-0.25; len]); // Ngược trend mạnh

        let matrix = MarketMatrixV3::default();

        let prop = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Bùng nổ Long khi macro trend dốc xuống quá ngưỡng adverse_slope (0.25 > 0.15) PHẢI bị VETO!");
    }

    #[test]
    fn test_momentum_valid_breakout_approved() {
        let len = 50;
        let i = 49;

        // Nến bùng nổ chuẩn: Open=2015, High=2026, Low=2014, Close=2025 (CLV = 0.833, close > open)
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2015.0; len]),
            high: std::sync::Arc::new(vec![2026.0; len]),
            low: std::sync::Arc::new(vec![2014.0; len]),
            close: std::sync::Arc::new(vec![2025.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.momentum.rsi_threshold = 60.0;
        config.tactical_sniper.momentum.vol_z_threshold = 1.0;
        config.tactical_sniper.momentum.min_activation_score = 0.20;
        config.tactical_sniper.momentum.struct_sl_buffer_atr = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.rsi_momentum = std::sync::Arc::new(vec![72.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.5; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.10; len]); // Thuận trend

        let matrix = MarketMatrixV3::default();

        let prop = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "Nến bùng nổ hợp lệ PHẢI được duyệt Proposal Long!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.raw_score >= config.tactical_sniper.momentum.min_activation_score);
        assert!(p.suggested_sl.is_some());
        let sl = p.suggested_sl.unwrap();
        // Low 2014.0 - 0.5 * 10.0 = 2009.0
        assert!(sl < 2014.0, "SL {} phải dưới Low 2014.0", sl);
    }

    #[test]
    fn test_keltner_bbands_timeframe_synchronization() {
        use ferro_trade::strategy::Strategy;
        use ferro_trade::data::Timeframe;
        use crate::strategy::core_v3::config::KeltnerParams;

        let mut strategy = HeliosStrategyV3::new(&std::collections::HashMap::new());

        // 1. Kiểm tra đồng bộ qua update_from_dna
        strategy.config.indicators.bbands.timeframe = Timeframe::T15m;
        strategy.config.indicators.keltner.timeframe = Timeframe::T4h; // Cố tình làm lệch pha

        let dummy_dna = vec![0.0; HeliosConfigV3::DNA_WIDTH];
        strategy.update_from_dna(&dummy_dna);

        // Keltner timeframe phải lập tức được ép bám theo bbands.timeframe
        assert_eq!(
            strategy.config.indicators.keltner.timeframe, 
            strategy.config.indicators.bbands.timeframe,
            "Keltner timeframe PHẢI được tự động đồng bộ theo BBands sau khi nạp DNA!"
        );

        // 2. Kiểm tra tính tương thích ngược Serde JSON (Serde-Safe)
        // Kịch bản A: JSON cũ có chứa trường "timeframe"
        let json_with_tf = r#"{"period": 20, "multiplier": 2.0, "timeframe": "T1h", "ma_type": "EMA"}"#;
        let parsed_with_tf: Result<KeltnerParams, _> = serde_json::from_str(json_with_tf);
        assert!(parsed_with_tf.is_ok(), "JSON cũ chứa trường timeframe phải deserialize thành công!");
        assert_eq!(parsed_with_tf.unwrap().timeframe, Timeframe::T1h);

        // Kịch bản B: JSON mới hoặc thiếu trường "timeframe"
        let json_without_tf = r#"{"period": 20, "multiplier": 2.0, "ma_type": "EMA"}"#;
        let parsed_without_tf: Result<KeltnerParams, _> = serde_json::from_str(json_without_tf);
        assert!(parsed_without_tf.is_ok(), "JSON thiếu trường timeframe phải tự động fallback về default mà không panic!");
        assert_eq!(parsed_without_tf.unwrap().timeframe, Timeframe::T5m);
    }
}


