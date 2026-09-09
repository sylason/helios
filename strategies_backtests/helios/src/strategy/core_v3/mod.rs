// strategies_backtests/helios/src/strategy/core_v3/mod.rs

pub mod arbiter;
pub mod matrix;
pub mod scanners;
pub mod risk;
pub mod config;

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
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType, ScannerId};
use crate::strategy::core_v3::risk::RiskManager;
use tracing::warn;

/// Trạng thái Cooldown chống cào lệnh sau khi cắt lỗ / hòa vốn
#[derive(Clone, Debug, Default)]
pub struct CooldownTracker {
    pub locked_until_bar: usize,
    pub locked_side: Option<SignalType>,
    pub last_exit_price: f64,
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
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
            cooldown_tracker: CooldownTracker::default(),
            prev_open_trade: None,
        }
    }

    fn update_from_dna(&mut self, dna: &[f64]) {
        self.config.update_from_dna(dna);
        self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
    }

    fn init(&mut self, ctx: &mut InitContext) {
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
        s.vol_sma = ctx.volume_sma(ind.vol_ma.timeframe, "VOL_SMA", ind.vol_ma.period);
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

        let (kc_u, _kc_m, kc_l) = ctx.keltner(ind.keltner.timeframe, "KC", ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone());
        s.kc_upper = kc_u;
        s.kc_lower = kc_l;

        // DEDICATED PER-PLAYBOOK RSI SENSORS
        let (rsi_t, _, _) = ctx.rsi(ind.rsi_trend.timeframe, "RSI_Trend", ind.rsi_trend.period);
        s.rsi_trend = rsi_t;

        let (rsi_r, _, _) = ctx.rsi(ind.rsi_reversal.timeframe, "RSI_Rev", ind.rsi_reversal.period);
        s.rsi_reversal = rsi_r;

        let (rsi_m, _, _) = ctx.rsi(ind.rsi_momentum.timeframe, "RSI_Mom", ind.rsi_momentum.period);
        s.rsi_momentum = rsi_m;

        let (rsi_a, _, _) = ctx.rsi(ind.rsi_anomaly.timeframe, "RSI_Ano", ind.rsi_anomaly.period);
        s.rsi_anomaly = rsi_a;

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

        let (st_r_k, st_r_d, _, _, _) = ctx.stoch(
            ind.stoch_range.timeframe, "STOCH_Range", 
            ind.stoch_range.period as u32, 
            ind.stoch_range.k_period as u32, ind.stoch_range.k_ma_type.clone(),
            ind.stoch_range.d_period as u32, ind.stoch_range.d_ma_type.clone(),
            ind.stoch_range.zone
        );
        s.stoch_range_k = st_r_k;
        s.stoch_range_d = st_r_d;

        // ABSORPTION VWMA SENSOR
        s.vwma = ctx.vwma(ind.vwma.timeframe, "VWMA", ind.vwma.period);
        s.z_score = ctx.z_score(ind.kama.timeframe, "ZScore", ind.z_score_lookback);
        s.vol_z_score = ctx.z_score_volume(ind.kama.timeframe, "VolZScore", ind.z_score_lookback);
        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);

        // MACD SENSOR (Vẫn được tính toán đầy đủ cho hệ thống Cảm biến)
        if ind.macd.fast > 0 && ind.macd.slow > 0 && ind.macd.signal > 0 {
            let (macd_m, macd_s, macd_h, _, _) = ctx.macd(ind.macd.timeframe, "MACD", ind.macd.fast, ind.macd.slow, ind.macd.signal);
            s.macd_main = macd_m;
            s.macd_signal = macd_s;
            s.macd_hist = macd_h;
        }

        self.sensors = s;
        self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
    }

    fn next(&mut self, tick: &mut TickContext) {
        let i = tick.i;
        if self.sensors.atr_sl.len() <= i || self.matrix.chaos_ratio.len() <= i {
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

                    // Nếu lệnh vừa đóng hòa vốn hoặc lỗ danh nghĩa (<= 0.001)
                    if trade_pnl <= 0.001 {
                        self.cooldown_tracker.locked_until_bar = i + stop_cfg.re_entry_cooldown_bars;
                        self.cooldown_tracker.locked_side = if is_long { Some(SignalType::Long) } else { Some(SignalType::Short) };
                        self.cooldown_tracker.last_exit_price = exit_price;

                        if self.debug_mode {
                            warn!(
                                "🛑 [COOLDOWN TRIGGERED] Bar: {} | Side: {:?} | Entry: {:.2} | Exit: {:.2} | Realized PnL: {:.4} <= 0.001 | Locked until Bar: {} ({} bars)",
                                i, self.cooldown_tracker.locked_side, last_trade.entry_price, exit_price, trade_pnl, self.cooldown_tracker.locked_until_bar, stop_cfg.re_entry_cooldown_bars
                            );
                        }
                    } else if self.debug_mode {
                        warn!(
                            "ℹ️ [COOLDOWN SKIPPED - REAL PROFIT] Bar: {} | Side: {:?} | Entry: {:.2} | Exit: {:.2} | Realized PnL: {:.4} > 0.001 -> Cooldown NOT triggered",
                            i, if is_long { SignalType::Long } else { SignalType::Short }, last_trade.entry_price, exit_price, trade_pnl
                        );
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

        let mut arbiter = HeliosArbiter::new(); 
        
        arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
        arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
        arbiter.add_scanner(Box::new(scanners::range_osc::RangeOscScanner));
        arbiter.add_scanner(Box::new(scanners::reversal::ReversalScanner));
        arbiter.add_scanner(Box::new(scanners::anomaly::AnomalyScanner));
        arbiter.add_scanner(Box::new(scanners::absorption::AbsorptionScanner));

        let decision = arbiter.process_tick(tick, &self.sensors, &self.matrix, &self.config, self.debug_mode);

        if decision.is_approved {
            if let Some(signal) = decision.final_signal {
                // 4. KIỂM TRA BỘ LỌC RE-ENTRY COOLDOWN
                let stop_cfg = &self.config.risk_management.stop_management;
                if stop_cfg.enable_re_entry_cooldown && i < self.cooldown_tracker.locked_until_bar {
                    if self.cooldown_tracker.locked_side == Some(signal) {
                        let current_price = tick.closes()[i];
                        let current_atr = self.sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
                        
                        // 🎯 SỬA FLAW-4.3: Kiểm tra bứt phá định hướng (Directional Breakout) thay vì dùng .abs()
                        // Với Long: Chỉ mở khóa nếu giá HỒI PHỤC TĂNG NGƯỢC LÊN (current_price - last_exit >= 1.5 * ATR)
                        // Với Short: Chỉ mở khóa nếu giá TIẾP TỤC GIẢM SÂU HƠN (last_exit - current_price >= 1.5 * ATR)
                        let is_favorable_breakout = if signal == SignalType::Long {
                            (current_price - self.cooldown_tracker.last_exit_price) >= 1.5 * current_atr
                        } else {
                            (self.cooldown_tracker.last_exit_price - current_price) >= 1.5 * current_atr
                        };

                        if !is_favorable_breakout {
                            if self.debug_mode {
                                let diff = if signal == SignalType::Long {
                                    current_price - self.cooldown_tracker.last_exit_price
                                } else {
                                    self.cooldown_tracker.last_exit_price - current_price
                                };
                                warn!(
                                    "⚠️ [COOLDOWN BLOCKED] Bar: {} | Signal: {:?} BLOCKED by Cooldown! Locked until bar: {} (Remaining: {} bars) | LastExit: {:.2} | Curr: {:.2} | FavorableDiff: {:+.2} < 1.5*ATR ({:.2})",
                                    i, signal, self.cooldown_tracker.locked_until_bar, self.cooldown_tracker.locked_until_bar.saturating_sub(i),
                                    self.cooldown_tracker.last_exit_price, current_price, diff, 1.5 * current_atr
                                );
                            }
                            // Chưa đủ bứt phá biến động có lợi -> CHẶN BẮT DAO RƠI & CÀO LỆNH RÁC!
                            return;
                        }
                    }
                }
                let leverage = self.config.risk_management.risk_base.leverage.max(1.0);
                
                let size = RiskManager::calculate_size(
                    tick.equity, 
                    self.config.risk_management.risk_base.percent_per_trade, 
                    self.config.risk_management.risk_base.min_order_usd, 
                    leverage,
                    self.config.risk_management.risk_base.max_cap_fraction,
                );
                
                let is_long = signal == SignalType::Long;
                
                let playbook_max_sl_pct = match decision.winning_scanner {
                    Some(ScannerId::TrendPullback) => self.config.tactical_sniper.pullback.max_initial_sl_pct,
                    Some(ScannerId::MomentumIgnition) => self.config.tactical_sniper.momentum.max_initial_sl_pct,
                    Some(ScannerId::RangeOscillator) => self.config.tactical_sniper.range.max_initial_sl_pct,
                    Some(ScannerId::ClimaxReversal) => self.config.tactical_sniper.reversal.max_initial_sl_pct,
                    Some(ScannerId::AnomalyFade) => self.config.tactical_sniper.anomaly.max_initial_sl_pct,
                    Some(ScannerId::AbsorptionScout) => self.config.tactical_sniper.absorption.max_initial_sl_pct,
                    None => 0.05,
                };

                let min_initial_sl_pct = self.config.risk_management.stop_management.min_initial_sl_pct;

                let raw_sl = decision.final_sl.unwrap_or_else(|| {
                    RiskManager::get_initial_sl(
                        tick,
                        &self.sensors,
                        is_long,
                        self.config.risk_management.stop_management.base_sl_atr,
                        playbook_max_sl_pct,
                        min_initial_sl_pct,
                        leverage,
                    )
                });
                
                let price = tick.closes()[tick.i];
                let initial_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, playbook_max_sl_pct, min_initial_sl_pct, leverage);
                
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
            }
        }
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

    fn should_block_by_cooldown(
        enable_cooldown: bool,
        current_bar: usize,
        locked_until_bar: usize,
        locked_side: Option<SignalType>,
        signal: SignalType,
        current_price: f64,
        last_exit_price: f64,
        atr: f64,
    ) -> bool {
        if !enable_cooldown {
            return false;
        }
        if current_bar < locked_until_bar {
            if locked_side == Some(signal) {
                // 🎯 SỬA FLAW-4.3: Kiểm tra bứt phá định hướng có lợi
                let is_favorable_breakout = if signal == SignalType::Long {
                    (current_price - last_exit_price) >= 1.5 * atr
                } else {
                    (last_exit_price - current_price) >= 1.5 * atr
                };
                if !is_favorable_breakout {
                    return true; // Bị chặn bởi Cooldown (chống bắt dao rơi / cào lệnh rác)
                }
            }
        }
        false
    }

    #[test]
    fn test_cooldown_toggle_disabled() {
        // Khi enable_re_entry_cooldown = false -> KHÔNG BAO GIỜ CHẶN
        let blocked = should_block_by_cooldown(
            false,
            10,
            20,
            Some(SignalType::Long),
            SignalType::Long,
            1760.0,
            1760.5,
            5.0,
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
            Some(SignalType::Long),
            SignalType::Long,
            1760.0,
            1760.5,
            5.0, // 1.5 * ATR = 7.5; diff = 0.5 < 7.5
        );
        assert!(blocked, "Cùng chiều trong vùng nén phải bị chặn");
    }

    #[test]
    fn test_cooldown_opposite_side_allowed() {
        // Khi BẬT, nhưng tín hiệu ngược chiều (Short) -> KHÔNG ĐƯỢC CHẶN
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            Some(SignalType::Long),
            SignalType::Short, // Tín hiệu đảo chiều
            1760.0,
            1760.5,
            5.0,
        );
        assert!(!blocked, "Ngược chiều không được chặn Cooldown");
    }

    #[test]
    fn test_cooldown_volatility_breakout_override() {
        // Khi BẬT, cùng chiều, nhưng giá bứt phá TĂNG NGƯỢC LÊN >= 1.5*ATR -> MỞ KHÓA SỚM
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            Some(SignalType::Long),
            SignalType::Long,
            1770.0, // diff = 1770.0 - 1760.0 = +10.0 >= 1.5*5.0 (7.5) -> Tăng có lợi
            1760.0,
            5.0,
        );
        assert!(!blocked, "Bứt phá biến động tăng mạnh phải mở khóa Cooldown sớm");
    }

    #[test]
    fn test_cooldown_falling_knife_blocked() {
        // 🎯 KIỂM CHỨNG FLAW-4.3: Giá tiếp tục sập hầm giảm sâu 10 USD (1750 < 1760)
        // Code cũ dùng .abs() sẽ tính |1750 - 1760| = 10 > 7.5 và mở khóa bậy bạ!
        // Code mới phát hiện giá đang rơi tự do -> TIẾP TỤC CHẶN COOLDOWN!
        let blocked = should_block_by_cooldown(
            true,
            10,
            25,
            Some(SignalType::Long),
            SignalType::Long,
            1750.0, // Giá sập hầm từ 1760 về 1750
            1760.0,
            5.0,
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
            Some(SignalType::Long),
            SignalType::Long,
            1760.0,
            1760.5,
            5.0,
        );
        assert!(!blocked, "Hết thời gian nghỉ phải cho phép vào lệnh");
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
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // 🎯 KIỂM CHỨNG FLAW-3.2:
        // Price Z-Score phải bằng 0.0 (giá phẳng)
        assert_eq!(matrix.price_z_score[len - 1], 0.0, "Price Z-Score phải phản ánh giá phẳng = 0.0");
        // Volume Force phải phản ứng với Vol Z-Score = 3.5 và MFI = 80 -> vf phải cao > 0.60
        assert!(matrix.volume_force[len - 1] > 0.60, "Volume Force phải phát hiện bùng nổ thanh khoản!");
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
    fn test_dna_width_and_parameter_defs_integrity() {
        let width = HeliosConfigV3::DNA_WIDTH;
        let defs = HeliosConfigV3::parameter_defs();
        assert_eq!(width, defs.len(), "DNA_WIDTH phải khớp chính xác 1:1 với số lượng parameter_defs!");

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
}


