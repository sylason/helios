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

use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType, ScannerId, AuctionStatus};
use crate::strategy::core_v3::risk::RiskManager;
use tracing::warn;

pub mod telemetry;
use telemetry::{DecisionCardFormatter, DefenseLedger};

/// Chiến lược Helios V3.3 - Clean Quant Architecture
#[derive(Clone, Debug)]
pub struct HeliosStrategyV3 {
    pub config: HeliosConfigV3,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrixV3,
    pub debug_mode: bool,
    pub arbiter: HeliosArbiter,
    pub telemetry_enabled: bool,
    pub last_decision_telemetry: Option<String>,
    /// Sổ Cái Quản Trị & Kiểm Toán Ma Trận Phòng Thủ Vốn (Zero-cost trong GA, Some trong Live/Paper)
    pub defense_ledger: Option<DefenseLedger>,
    /// Chữ ký và mốc thời gian của lần phát Thẻ Đỏ VETO gần nhất (hỗ trợ First-Strike & Suppression chống spam)
    pub last_veto_signature: Option<(String, i64)>,
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        let mut arbiter = HeliosArbiter::new();
        // 🎯 5 PLAYBOOK CHỦ LỰC V3.3: Khởi tạo sẵn 1 lần duy nhất (Zero-Allocation ở runtime)
        arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
        arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
        arbiter.add_scanner(Box::new(scanners::reversal::ReversalScanner));
        arbiter.add_scanner(Box::new(scanners::absorption::AbsorptionScanner));
        arbiter.add_scanner(Box::new(scanners::range_osc::RangeOscScanner));

        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
            arbiter,
            telemetry_enabled: false,
            last_decision_telemetry: None,
            defense_ledger: None,
            last_veto_signature: None,
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
        // 🛡️ TODO-AUDIT-NEW-60 (H-CFG-05): Khóa bất biến thứ bậc phân tầng Trailing Stop (tight <= normal <= loose)
        self.config.risk_management.stop_management.enforce_tsl_hierarchy();

        // 🛡️ [TODO-AUDIT-NEW-53 / H-MOD-01]: Chỉ tái tính toán Ma trận khi sensors đã được nạp dữ liệu từ init()
        // Nếu sensors rỗng (ví dụ trước khi init trong BacktestBuilder::run_with_dna), giữ matrix ở trạng thái default để tránh cấp phát thừa.
        if !self.sensors.closes.is_empty() {
            self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
        }
    }

    fn init(&mut self, ctx: &mut InitContext) {
        // 🛡️ TASK-MAT-05: Khóa cứng đồng bộ 100% Timeframe của Keltner Channel theo Bollinger Bands (Serde-Safe)
        self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;
        // 🛡️ TODO-AUDIT-NEW-60 (H-CFG-05): Khóa bất biến thứ bậc phân tầng Trailing Stop khi khởi tạo
        self.config.risk_management.stop_management.enforce_tsl_hierarchy();

        let ind = &self.config.indicators;

        // 🛡️ [TODO-AUDIT-NEW-57 / H-MOD-07]: Khử bỏ Silent Abort, áp dụng Fail-Fast dứt khoát
        // Kiểm tra toàn bộ chu kỳ chỉ báo cốt tử thực tế được đăng ký với ctx để ngăn ngừa chia cho 0 và mảng rỗng
        let is_zero_period = ind.natr.period == 0 || ind.micro_natr.period == 0 ||
            ind.mfi.period == 0 || ind.kama.short_period == 0 || ind.kama.long_period == 0 ||
            ind.adx.period == 0 || ind.bbands.period == 0 || ind.atr.period == 0 ||
            ind.vol_ma.period == 0 || ind.vwma.period == 0 ||
            ind.price_z_score.lookback == 0 || ind.vol_z_score.lookback == 0 ||
            ind.rsi_trend.period == 0 || ind.rsi_reversal.period == 0 || ind.rsi_momentum.period == 0 ||
            ind.stoch_trend.period == 0 || ind.stoch_trend.k_period == 0 || ind.stoch_trend.d_period == 0 ||
            ind.stoch_range.period == 0 || ind.stoch_range.k_period == 0 || ind.stoch_range.d_period == 0;

        if is_zero_period {
            tracing::error!(
                "🚨 [HELIOS V3 FATAL] Critical indicator period/lookback is 0 in init()! \
                NATR: {}, MicroNATR: {}, MFI: {}, KAMA: ({},{}), ADX: {}, BB: {}, ATR: {}, VolMA: {}, VWMA: {}, \
                PriceZ: {}, VolZ: {}, RSI: (T:{},R:{},M:{}), StochTrend: ({},{},{}), StochRange: ({},{},{})",
                ind.natr.period, ind.micro_natr.period, ind.mfi.period, ind.kama.short_period, ind.kama.long_period,
                ind.adx.period, ind.bbands.period, ind.atr.period, ind.vol_ma.period, ind.vwma.period,
                ind.price_z_score.lookback, ind.vol_z_score.lookback, ind.rsi_trend.period, ind.rsi_reversal.period,
                ind.rsi_momentum.period, ind.stoch_trend.period, ind.stoch_trend.k_period, ind.stoch_trend.d_period,
                ind.stoch_range.period, ind.stoch_range.k_period, ind.stoch_range.d_period
            );
            panic!("🚨 [HELIOS V3 FATAL] Critical indicator period is 0 in init(). Aborting to prevent silent zero-trade failure!");
        }

        let mut s = HeliosSensors::default();
        
        s.closes = ctx.closes().clone();
        s.highs = ctx.highs().clone();
        s.lows = ctx.lows().clone();
        s.volumes = ctx.volumes().clone();

        s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
        s.micro_natr = ctx.natr(ind.micro_natr.timeframe, "NATR_Micro", ind.micro_natr.period);
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

        // 🎯 [TODO-AUDIT-H-03] Kích hoạt cảm biến Stochastic cho RangeOscScanner
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
        s.vol_sma = ctx.volume_sma(ind.vol_ma.timeframe, "VolSMA", ind.vol_ma.period);
        s.z_score = ctx.z_score(ind.price_z_score.timeframe, "PriceZScore", ind.price_z_score.lookback);
        s.vol_z_score = ctx.z_score_volume(ind.vol_z_score.timeframe, "VolZScore", ind.vol_z_score.lookback);
        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);

        // 🛡️ FLAW-REG.10 & TODO-AUDIT-NEW-37: Tính độ dốc chuẩn hóa của trục giữa Bollinger Bands (SMA 20) qua ATR
        // Bước nhảy k_bars tỷ lệ thuận theo timeframe của BB (Single Source of Truth qua Timeframe::to_minutes())
        // triệt tiêu hoàn toàn dị tật bậc thang phẳng (trừ ra 0.0 giả tạo).
        let k_bars = ind.bbands.timeframe.to_minutes().max(1) as usize;
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

        // 2. Strict Exclusive Order: Nếu đang có vị thế mở thì không mở lệnh mới
        if is_pos_open {
            return; 
        }

        // 🎯 KHỐI 3 (TASK-ARB-05 / TODO-AUDIT-NEW-54): Đánh giá Gatekeeper & Đấu thầu tập trung 1 lần duy nhất tại Arbiter
        let decision = self.arbiter.process_tick(tick, &self.sensors, &self.matrix, &self.config, self.debug_mode);

        // 🛡️ THEO DÕI MA TRẬN PHÒNG THỦ VỐN (Zero-cost trong GA, Some trong Live/Paper)
        // Tái sử dụng trực tiếp decision.gate_eval, triệt tiêu hoàn toàn lệnh gọi Gatekeeper::evaluate lần 2!
        if let Some(ref mut ledger) = self.defense_ledger {
            // Tầng 1: Chuyển đổi trạng thái Gatekeeper (Vào bão / Ra bão)
            let candle_ts = tick.data_view.raw_cache().datetime.get(tick.i).copied().unwrap_or(0) / 1_000_000;
            let squeeze = self.matrix.squeeze_intensity.get(tick.i).copied().unwrap_or(0.0);
            let kama_slope = self.matrix.kama_slope_atr.get(tick.i).copied()
                .unwrap_or_else(|| self.sensors.kama_slope_adj.get(tick.i).copied().unwrap_or(0.0));

            if let Some(msg) = ledger.on_gatekeeper_tick(tick.i, candle_ts, &decision.gate_eval) {
                self.last_decision_telemetry = Some(msg);
            } else if self.last_decision_telemetry.is_none() {
                let price = tick.closes().get(tick.i).copied().unwrap_or(0.0);
                if let Some(hb_msg) = ledger.check_hourly_heartbeat(tick.i, candle_ts, price, &decision.gate_eval, kama_slope, squeeze) {
                    self.last_decision_telemetry = Some(hb_msg);
                }
            }

            // Tầng 2: Chốt chặn Boundary Guard & Chop Zone của TrendPB
            let pb_cfg = &self.config.tactical_sniper.pullback;
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

        if decision.is_approved {
            if let Some(signal) = decision.final_signal {
                // 4. [ARCH-COOLDOWN-SEPARATION]: Cầu Dao Cooldown VETO được độc quyền sở hữu và thực thi
                // bởi Bot Engine (AssetActor) dựa trên PnL thực tế của tài khoản (Net PnL).
                // Helios Strategy chuyên tâm 100% vào việc sinh tín hiệu Alpha, không tự ý can thiệp Cooldown.
                let leverage = self.config.risk_management.risk_base.leverage.max(1.0);
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
                let risk_base = &self.config.risk_management.risk_base;

                let raw_sl = decision.final_sl.unwrap_or_else(|| {
                    RiskManager::get_initial_sl(
                        tick,
                        &self.sensors,
                        is_long,
                        self.config.risk_management.stop_management.base_sl_atr,
                        playbook_max_sl_roe,
                        min_initial_sl_pct,
                        leverage,
                        risk_base.maintenance_margin_rate,
                        risk_base.taker_fee_rate,
                        risk_base.liquidation_buffer_ratio,
                    )
                });
                
                let price = tick.closes()[tick.i];
                let initial_sl = RiskManager::clamp_sl_price(
                    price,
                    raw_sl,
                    is_long,
                    playbook_max_sl_roe,
                    min_initial_sl_pct,
                    leverage,
                    risk_base.maintenance_margin_rate,
                    risk_base.taker_fee_rate,
                    risk_base.liquidation_buffer_ratio,
                );

                // 🛡️ [TODO-AUDIT-NEW-37]: Định cỡ vị thế động theo $Risk Parity (Dynamic De-leverage)
                // Tính toán size dựa trên khoảng cách SL thực tế đã được kẹp an toàn.
                let size = RiskManager::calculate_dynamic_size(
                    tick.equity, 
                    risk_base.percent_per_trade, 
                    price,
                    initial_sl,
                    leverage,
                    risk_base.min_order_usd, 
                    risk_base.max_cap_fraction,
                );

                // Chốt chặn Capital Starvation Guard (TODO-MATH-03):
                // Nếu tài khoản không đủ vốn để đạt min_order_usd hoặc size không hợp lệ,
                // hủy bỏ mở lệnh ngay lập tức để không gửi lệnh ảo tick.buy/sell(Some(0.0))
                // và triệt tiêu nguy cơ downstream nhầm tưởng size None rồi All-in 100%.
                if size <= 0.0 || !size.is_finite() {
                    if self.debug_mode {
                        warn!(
                            "🛑 [CAPITAL STARVATION VETO] Bar: {} | Equity: {:.2} | Lev: {:.1}x | Signal: {:?} -> Size <= 0.0 (Order Aborted)",
                            tick.i, tick.equity, leverage, signal
                        );
                    }
                    return;
                }
                
                // Ghi nhận kẹp SL vào Sổ cái nếu có
                let hard_cap_sl = RiskManager::clamp_sl_price(
                    price,
                    raw_sl,
                    is_long,
                    0.15,
                    min_initial_sl_pct,
                    leverage,
                    risk_base.maintenance_margin_rate,
                    risk_base.taker_fee_rate,
                    risk_base.liquidation_buffer_ratio,
                );
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

            // 🛑 QUANT DECISION TELEMETRY: Xuất Thẻ Đỏ VETO nếu có proposal bị Sniper Guard, Liquidation Guard, Phân luồng Thẩm quyền hoặc Gatekeeper VETO
            if self.telemetry_enabled {
                let has_structural_veto = decision.auction_ledger.iter().any(|c| {
                    c.status == AuctionStatus::StructuralRiskVeto
                        || c.status == AuctionStatus::LiquidationRiskVeto
                        || matches!(c.status, AuctionStatus::RegimeIneligible(_))
                        || c.status == AuctionStatus::FrictionalHurdleVeto
                });
                let has_candidates = decision.auction_ledger.iter().any(|c| c.status != AuctionStatus::NotTriggered);
                if has_structural_veto || (decision.gate_veto.is_some() && has_candidates) {
                    let candle_ts = tick.data_view.raw_cache().datetime.get(tick.i).copied().unwrap_or(0) / 1_000_000;

                    // Tạo chữ ký VETO đại diện: Scanner đầu tiên bị veto + Trạng thái
                    let primary_veto = decision.auction_ledger.iter()
                        .find(|c| c.status != AuctionStatus::NotTriggered && c.status != AuctionStatus::LowConfidence)
                        .map(|c| format!("{}:{:?}", c.scanner_id.as_str(), c.status))
                        .unwrap_or_else(|| format!("GATE:{:?}", decision.gate_veto));

                    // Giao thức First-Strike & Suppression: Chỉ xuất thẻ lần đầu; nén nếu lặp lại cùng lý do trong vòng 30 phút
                    let should_emit = match &self.last_veto_signature {
                        Some((last_sig, last_ts)) => {
                            if last_sig == &primary_veto {
                                if candle_ts > 0 && *last_ts > 0 {
                                    candle_ts.saturating_sub(*last_ts) >= 30 * 60 * 1000
                                } else {
                                    tick.i.saturating_sub(*last_ts as usize) >= 30
                                }
                            } else {
                                true
                            }
                        }
                        None => true,
                    };

                    if should_emit {
                        let record_ts = if candle_ts > 0 { candle_ts } else { tick.i as i64 };
                        self.last_veto_signature = Some((primary_veto, record_ts));
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

    fn get_gatekeeper_zones_json(&self) -> Option<String> {
        self.defense_ledger.as_ref().map(|l| l.export_zones_json())
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

        let config = HeliosConfigV3::default();
        let _matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // 🎯 KIỂM CHỨNG FLAW-3.2:
        // Price Z-Score phải bằng 0.0 (giá phẳng)
        assert_eq!(sensors.z_score[len - 1], 0.0, "Price Z-Score phải phản ánh giá phẳng = 0.0");
        // Volume Z-Score phải phản ứng độc lập với Vol Z-Score = 3.5 -> phát hiện bùng nổ thanh khoản
        assert!(sensors.vol_z_score[len - 1] > 3.0, "Vol Z-Score phải độc lập phản ánh bùng nổ thanh khoản!");
    }

    #[test]
    fn test_single_gatekeeper_eval_propagated_through_arbiter_decision() {
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use ferro_trade::strategy::Strategy;
        use smallvec::SmallVec;
        use std::collections::HashMap;
        use crate::strategy::core_v3::gatekeeper::GateVetoReason;
        use crate::strategy::core_v3::telemetry::defense_ledger::DefenseLedger;

        let len = 50;
        let opens = vec![2000.0; len];
        let highs = vec![2010.0; len];
        let lows = vec![1990.0; len];
        let closes = vec![2000.0; len];
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

        // 1. Khởi tạo Strategy với Telemetry Enabled và Defense Ledger
        let params = HashMap::new();
        let mut strategy = HeliosStrategyV3::new(&params);
        strategy.set_telemetry_enabled(true);
        strategy.defense_ledger = Some(DefenseLedger::new());

        // Khởi tạo Sensors giả lập
        let mut sensors = HeliosSensors::default();
        sensors.closes = std::sync::Arc::new(vec![2000.0; len]);
        sensors.highs = std::sync::Arc::new(vec![2010.0; len]);
        sensors.lows = std::sync::Arc::new(vec![1990.0; len]);
        sensors.volumes = std::sync::Arc::new(vec![1000.0; len]);
        sensors.natr = std::sync::Arc::new(vec![1.5; len]);
        sensors.micro_natr = std::sync::Arc::new(vec![1.0; len]);
        sensors.z_score = std::sync::Arc::new(vec![0.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![0.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![50.0; len]);
        sensors.efficiency_ratio = std::sync::Arc::new(vec![0.5; len]);
        sensors.adx = std::sync::Arc::new(vec![25.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2000.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2010.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1990.0; len]);
        sensors.kc_upper = std::sync::Arc::new(vec![2012.0; len]);
        sensors.kc_lower = std::sync::Arc::new(vec![1988.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);

        // ==========================================
        // NẾN 1: BÃO DỮ DỘI (chaos_ratio = 3.5, efficiency_ratio = 0.0 -> chaos_index = (tanh(2.5) * 1.0) ~ 0.98 > t_enter ~ 0.70)
        // ==========================================
        let mut chaos_vec_storm = vec![0.5; len];
        chaos_vec_storm[len - 1] = 3.5;
        sensors.chaos_ratio = std::sync::Arc::new(chaos_vec_storm);

        let mut er_vec_storm = vec![0.5; len];
        er_vec_storm[len - 1] = 0.0; // Thị trường hoàn toàn nhiễu loạn giật cục
        sensors.efficiency_ratio = std::sync::Arc::new(er_vec_storm);

        strategy.sensors = sensors.clone();
        strategy.matrix = MarketMatrixV3::from_sensors(&strategy.config, &strategy.sensors);

        let mut tick_1 = ferro_trade::strategy::TickContext {
            i: len - 1,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        strategy.next(&mut tick_1);

        // Kiểm tra Decision trực tiếp từ Arbiter và Gatekeeper Evaluation
        let dec_1 = strategy.arbiter.process_tick(&tick_1, &strategy.sensors, &strategy.matrix, &strategy.config, false);
        assert!(!dec_1.is_approved, "Bão thị trường phải VETO không phê duyệt lệnh!");
        assert!(!dec_1.gate_eval.passed, "gate_eval.passed phải là false!");
        assert_eq!(dec_1.gate_eval.veto_reason, Some(GateVetoReason::ChaosWhipsaw), "Lý do Veto phải là ChaosWhipsaw!");

        // Kiểm tra Defense Ledger đã nhận gate_eval thông qua arbiter.process_tick() trong next() và ghi nhận bão
        let ledger = strategy.defense_ledger.as_ref().expect("Phải có ledger");
        assert_eq!(ledger.prev_gate_veto, Some(GateVetoReason::ChaosWhipsaw), "Ledger phải ghi nhận veto ChaosWhipsaw!");
        assert_eq!(ledger.chaos_whipsaw_bars, 1, "Ledger phải tăng đếm chaos_whipsaw_bars lên 1!");

        // ==========================================
        // NẾN 2: TAN BÃO (chaos_ratio = 0.20 < ngưỡng 0.70)
        // ==========================================
        let mut chaos_vec_calm = vec![0.5; len];
        chaos_vec_calm[len - 1] = 0.20;
        sensors.chaos_ratio = std::sync::Arc::new(chaos_vec_calm);
        strategy.sensors = sensors;
        strategy.matrix = MarketMatrixV3::from_sensors(&strategy.config, &strategy.sensors);

        let mut tick_2 = ferro_trade::strategy::TickContext {
            i: len - 1,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        strategy.next(&mut tick_2);

        // Kiểm tra Decision từ Arbiter nến 2
        let dec_2 = strategy.arbiter.process_tick(&tick_2, &strategy.sensors, &strategy.matrix, &strategy.config, false);
        assert!(dec_2.gate_eval.passed, "Nến 2 tan bão thì gate_eval.passed phải là true!");
        assert_eq!(dec_2.gate_eval.veto_reason, None, "Không được có Veto!");

        // Kiểm tra Defense Ledger cập nhật trạng thái tan bão (prev_gate_veto quay về None và lưu zone)
        let ledger_2 = strategy.defense_ledger.as_ref().expect("Phải có ledger");
        assert_eq!(ledger_2.prev_gate_veto, None, "Ledger phải giải phóng veto khi tan bão!");
        assert_eq!(ledger_2.zones.len(), 1, "Ledger phải đóng lại 1 zone bão đã qua!");
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
        sensors.kama_mid = std::sync::Arc::new(vec![2070.0; len]); // KAMA ở trên (2070, đảm bảo R:R >= 1.20)
        sensors.kama_slope_adj = std::sync::Arc::new(vec![-0.05; len]);
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
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
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

        // CLV = (2*1995 - 2000 - 1950) / 50 = +0.80 >= 0.20, Lower Wick = (1980 - 1950)/50 = 60% >= 20% -> Chấp thuận vào lệnh Long!
        let prop_hammer = ReversalScanner.scan(&tick_hammer, &sensors, &matrix, &config);
        assert!(prop_hammer.is_some(), "ReversalScanner PHẢI kích hoạt khi nến có râu rút chân Hammer!");
        assert_eq!(prop_hammer.unwrap().signal_type, SignalType::Long);
    }

    #[test]
    fn test_reversal_rejects_poor_low_without_excess_wick() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // KỊCH BẢN: Nến xanh tăng đóng cao nhưng đáy phẳng lì Poor Low (Open = Low = 1990, Close = 1996, High = 2000)
        // Range = 10, CLV = (2*1996 - 2000 - 1990)/10 = +0.20 (thỏa mãn min_clv = 0.20)
        // Nhưng Lower Wick = min(1990, 1996) - 1990 = 0.0 -> Wick Ratio = 0.0% < 20% -> PHẢI VETO!
        let opens = vec![1990.0; len];
        let highs = vec![2000.0; len];
        let lows = vec![1990.0; len];
        let closes_poor_low = vec![1996.0; len];
        let volumes = vec![1000.0; len];
        let datetimes = vec![0i64; len];

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes_poor_low),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick_poor_low = ferro_trade::strategy::TickContext {
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
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.z_score_threshold = 2.5;
        config.tactical_sniper.reversal.min_activation_score = 0.50;
        config.tactical_sniper.reversal.min_reversal_clv = 0.20;
        config.tactical_sniper.reversal.min_excess_wick_ratio = 0.20;
        config.tactical_sniper.reversal.mfi_reversal_thresh = 30.0;

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        let prop = ReversalScanner.scan(&tick_poor_low, &sensors, &matrix, &config);
        assert!(prop.is_none(), "ReversalScanner PHẢI từ chối nến Poor Low đáy phẳng lì không có râu nến từ chối!");
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
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
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
    fn test_reversal_volume_exhaustion_behavior() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.reversal.z_score_threshold = 2.0;
        config.tactical_sniper.reversal.min_activation_score = 0.70;
        config.tactical_sniper.reversal.min_reversal_clv = 0.20;
        config.tactical_sniper.reversal.mfi_reversal_thresh = 30.0;
        config.tactical_sniper.reversal.climax_vol_threshold = 1.50;
        config.tactical_sniper.reversal.w_z_score = 0.35;
        config.tactical_sniper.reversal.w_displacement = 0.35;

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        let make_tick = |vol: f64| {
            let raw_cache = RawDataCache {
                datetime: std::sync::Arc::new(vec![0i64; len]),
                open: std::sync::Arc::new(vec![2000.0; len]),
                high: std::sync::Arc::new(vec![2000.0; len]),
                low: std::sync::Arc::new(vec![1950.0; len]),
                close: std::sync::Arc::new(vec![1995.0; len]),
                volume: std::sync::Arc::new(vec![vol; len]),
                custom_columns: HashMap::new(),
            };
            DataView::new(raw_cache)
        };

        let indicator_manager = IndicatorManager::new();
        let mut actions_climax = SmallVec::new();
        let mut actions_weak = SmallVec::new();
        let trades = Vec::new();

        // 1. KỊCH BẢN 1 (TODO-AUDIT-H-01): Volume xả lũ cao trào Selling Climax (vol = 2500 > 1500 = 1.5 * SMA)
        // Nến búa rút chân (CLV = +0.80) kèm volume bão thanh lý -> Phê duyệt mở Long bắt đáy cao trào!
        let dv_climax = make_tick(2500.0);
        let tick_climax = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &dv_climax, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions_climax, position: Position::default(),
        };
        let prop_climax = ReversalScanner.scan(&tick_climax, &sensors, &matrix, &config);
        assert!(prop_climax.is_some(), "ReversalScanner PHẢI phê duyệt khi có nến rút chân kèm volume cao trào (Selling Climax)!");

        // 2. KỊCH BẢN 2 (TODO-AUDIT-H-01): Volume teo tóp thiếu lực (vol = 100 << 1000) -> f_vol tiệm cận 0 -> Bị loại bỏ!
        let dv_weak = make_tick(100.0);
        let tick_weak = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &dv_weak, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions_weak, position: Position::default(),
        };
        let prop_weak = ReversalScanner.scan(&tick_weak, &sensors, &matrix, &config);
        assert!(prop_weak.is_none(), "ReversalScanner PHẢI từ chối khi volume teo tóp thiếu lực (không có cao trào xả lũ)!");

        // 3. KỊCH BẢN 3: Warmup NaN trong vol_sma -> Fallback an toàn, không sinh panic
        let mut sensors_nan = sensors.clone();
        sensors_nan.vol_sma = std::sync::Arc::new(vec![f64::NAN; len]);
        let prop_nan = ReversalScanner.scan(&tick_climax, &sensors_nan, &matrix, &config);
        assert!(prop_nan.is_some());
        assert!(!prop_nan.unwrap().raw_score.is_nan(), "raw_score không được là NaN khi vol_sma warmup");
    }

    #[test]
    fn test_reversal_kama_slope_adverse_rejection() {
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
            volume: std::sync::Arc::new(vec![2500.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let config = HeliosConfigV3::default();

        // 1. Kịch bản A: KAMA dốc xuống xả lũ (-0.50 < -0.40) -> Bắt buộc VETO!
        let mut matrix_adverse = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_adverse.kama_slope_atr = std::sync::Arc::new(vec![-0.50; len]);
        let prop_adverse = ReversalScanner.scan(&tick, &sensors, &matrix_adverse, &config);
        assert!(prop_adverse.is_none(), "ReversalScanner PHẢI từ chối Long khi KAMA dốc xả lũ (-0.50 < -0.40)!");

        // 2. Kịch bản B: KAMA dốc nhẹ an toàn (-0.15 >= -0.40) -> Phê duyệt!
        let mut matrix_safe = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_safe.kama_slope_atr = std::sync::Arc::new(vec![-0.15; len]);
        let prop_safe = ReversalScanner.scan(&tick, &sensors, &matrix_safe, &config);
        assert!(prop_safe.is_some(), "ReversalScanner PHẢI phê duyệt khi KAMA dốc nhẹ an toàn (-0.15 >= -0.40)!");
    }

    #[test]
    fn test_reversal_chaos_index_storm_rejection() {
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
            volume: std::sync::Arc::new(vec![2500.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let config = HeliosConfigV3::default();

        // 1. Kịch bản A: Bão Whipsaw hỗn loạn (chaos_index = 0.85 > 0.65) -> Bắt buộc VETO!
        let mut matrix_storm = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_storm.chaos_index = std::sync::Arc::new(vec![0.85; len]);
        let prop_storm = ReversalScanner.scan(&tick, &sensors, &matrix_storm, &config);
        assert!(prop_storm.is_none(), "ReversalScanner PHẢI từ chối khi thị trường bão Whipsaw (0.85 > 0.65)!");

        // 2. Kịch bản B: Môi trường ổn định (chaos_index = 0.30 <= 0.65) -> Phê duyệt!
        let mut matrix_calm = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_calm.chaos_index = std::sync::Arc::new(vec![0.30; len]);
        let prop_calm = ReversalScanner.scan(&tick, &sensors, &matrix_calm, &config);
        assert!(prop_calm.is_some(), "ReversalScanner PHẢI phê duyệt khi môi trường ổn định (0.30 <= 0.65)!");
    }

    #[test]
    fn test_reversal_adx_hurricane_selling_rejection() {
        // 🌪️ TODO-AUDIT-NEW-191 (REV-MACRO-GATE): Kiểm tra VETO Long khi KAMA âm nhẹ nhưng ADX vọt lên bão xả lũ
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
            volume: std::sync::Arc::new(vec![2500.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);
        // Bão ADX = 48.0 (vượt trần mặc định 35.0)
        sensors.adx = std::sync::Arc::new(vec![48.0; len]);

        let config = HeliosConfigV3::default();

        // KAMA chỉ hơi dốc nhẹ âm (-0.08 >= -0.20 thỏa mãn dốc), nhưng ADX = 48.0 >= 35.0 -> Bắt buộc VETO Long!
        let mut matrix_storm = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_storm.kama_slope_atr = std::sync::Arc::new(vec![-0.08; len]);
        let prop_storm = ReversalScanner.scan(&tick, &sensors, &matrix_storm, &config);
        assert!(prop_storm.is_none(), "ReversalScanner PHẢI từ chối Long khi xu hướng giảm vĩ mô có bão ADX (48.0 >= 35.0)!");

        // Khi bão ADX hạ nhiệt (22.0 < 35.0) với cùng độ dốc KAMA an toàn (-0.08) -> Phê duyệt!
        let mut sensors_calm = sensors.clone();
        sensors_calm.adx = std::sync::Arc::new(vec![22.0; len]);
        let matrix_calm = MarketMatrixV3::from_sensors(&config, &sensors_calm);
        let prop_calm = ReversalScanner.scan(&tick, &sensors_calm, &matrix_calm, &config);
        assert!(prop_calm.is_some(), "ReversalScanner PHẢI phê duyệt khi bão ADX đã suy yếu (22.0 < 35.0)!");
    }

    #[test]
    fn test_reversal_adx_hurricane_buying_rejection() {
        // 🌪️ TODO-AUDIT-NEW-191 (REV-MACRO-GATE): Kiểm tra VETO Short khi KAMA dương nhẹ nhưng ADX vọt lên bão tăng vĩ mô
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Dựng nến Shooting Star đè đầu tạo Short Reversal: Open=2000, Low=2000, High=2050, Close=2005
        // Range = 50, CLV = (2*2005 - 2050 - 2000)/50 = -40/50 = -0.80 <= -0.20
        // Upper wick = (2050 - 2005)/50 = 45/50 = 0.90 >= 0.20
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2000.0; len];
        let mut lows = vec![2000.0; len];
        let mut closes = vec![2000.0; len];
        opens[i - 1] = 1950.0; highs[i - 1] = 2020.0; lows[i - 1] = 1950.0; closes[i - 1] = 2015.0;
        opens[i] = 2010.0; highs[i] = 2050.0; lows[i] = 2000.0; closes[i] = 2005.0;

        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(vec![2500.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![3.0; len]); // Quá mua
        sensors.kama_mid = std::sync::Arc::new(vec![1900.0; len]); // KAMA ở dưới (1900 < 2005)
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![85.0; len]); // MFI >= 70.0 (quá tải)
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);
        sensors.adx = std::sync::Arc::new(vec![55.0; len]); // Bão tăng giá ADX = 55.0

        let config = HeliosConfigV3::default();

        // KAMA hơi dốc lên dương (+0.08 <= 0.20 thỏa mãn dốc), nhưng ADX = 55.0 >= 35.0 -> Bắt buộc VETO Short cản tàu!
        let mut matrix_storm = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix_storm.kama_slope_atr = std::sync::Arc::new(vec![0.08; len]);
        let prop_storm = ReversalScanner.scan(&tick, &sensors, &matrix_storm, &config);
        assert!(prop_storm.is_none(), "ReversalScanner PHẢI từ chối Short cản tàu khi xu hướng tăng vĩ mô có bão ADX (55.0 >= 35.0)!");

        // Khi bão ADX hạ nhiệt (20.0 < 35.0) với cùng độ dốc KAMA an toàn (+0.08) -> Phê duyệt Short!
        let mut sensors_calm = sensors.clone();
        sensors_calm.adx = std::sync::Arc::new(vec![20.0; len]);
        let matrix_calm = MarketMatrixV3::from_sensors(&config, &sensors_calm);
        let prop_calm = ReversalScanner.scan(&tick, &sensors_calm, &matrix_calm, &config);
        assert!(prop_calm.is_some(), "ReversalScanner PHẢI phê duyệt Short khi bão ADX đã suy yếu (20.0 < 35.0)!");
    }

    #[test]
    fn test_reversal_rr_hurdle_gate() {
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
            close: std::sync::Arc::new(vec![1995.0; len]), // SL = 1950 - 15 = 1935, SL_dist = 60
            volume: std::sync::Arc::new(vec![2500.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let config = HeliosConfigV3::default();

        // 1. Kịch bản A: KAMA Midline quá gần (2040.0), TP_dist = 45.0, R:R = 45/60 = 0.75 < 1.20 -> VETO!
        sensors.kama_mid = std::sync::Arc::new(vec![2040.0; len]);
        let matrix_low_rr = MarketMatrixV3::from_sensors(&config, &sensors);
        let prop_low_rr = ReversalScanner.scan(&tick, &sensors, &matrix_low_rr, &config);
        assert!(prop_low_rr.is_none(), "ReversalScanner PHẢI từ chối khi tỷ lệ R:R < 1.20 (0.75)!");

        // 2. Kịch bản B: KAMA Midline đủ xa (2070.0), TP_dist = 75.0, R:R = 75/60 = 1.25 >= 1.20 -> Phê duyệt!
        sensors.kama_mid = std::sync::Arc::new(vec![2070.0; len]);
        let matrix_good_rr = MarketMatrixV3::from_sensors(&config, &sensors);
        let prop_good_rr = ReversalScanner.scan(&tick, &sensors, &matrix_good_rr, &config);
        assert!(prop_good_rr.is_some(), "ReversalScanner PHẢI phê duyệt khi tỷ lệ R:R >= 1.20 (1.25)!");
    }

    #[test]
    fn test_reversal_dual_bar_confirmation() {
        use crate::strategy::core_v3::scanners::reversal::ReversalScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        let mut opens = vec![2000.0; len];
        let mut highs = vec![2000.0; len];
        let mut lows = vec![1950.0; len];
        let mut closes = vec![1995.0; len];
        let volumes = vec![2500.0; len];
        let datetimes = vec![0i64; len];

        // 1. Kịch bản A: Nến trước đó i-1 là nến xả lũ (High=2050, Low=1960).
        // Nến hiện tại i tiếp tục cắm đầu rơi tự do (Open=1960, High=1960, Low=1920, Close=1925)
        // Không hề đóng trên điểm giữa nến trước và phá vỡ đáy cũ -> Bắt buộc VETO!
        opens[i - 1] = 2050.0; highs[i - 1] = 2050.0; lows[i - 1] = 1960.0; closes[i - 1] = 1965.0;
        opens[i] = 1960.0; highs[i] = 1960.0; lows[i] = 1920.0; closes[i] = 1925.0;

        let raw_cache_knife = RawDataCache {
            datetime: std::sync::Arc::new(datetimes.clone()),
            open: std::sync::Arc::new(opens.clone()),
            high: std::sync::Arc::new(highs.clone()),
            low: std::sync::Arc::new(lows.clone()),
            close: std::sync::Arc::new(closes.clone()),
            volume: std::sync::Arc::new(volumes.clone()),
            custom_columns: HashMap::new(),
        };
        let data_view_knife = DataView::new(raw_cache_knife);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();

        let tick_knife = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view_knife, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let mut sensors = HeliosSensors::default();
        sensors.z_score = std::sync::Arc::new(vec![-3.0; len]);
        sensors.kama_mid = std::sync::Arc::new(vec![2100.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![15.0; len]);
        sensors.mfi = std::sync::Arc::new(vec![25.0; len]);
        sensors.vol_sma = std::sync::Arc::new(vec![1000.0; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        let prop_knife = ReversalScanner.scan(&tick_knife, &sensors, &matrix, &config);
        assert!(prop_knife.is_none(), "ReversalScanner PHẢI từ chối khi nến sau tiếp tục cắm đầu rơi tự do (Falling knife)!");

        // 2. Kịch bản B: Nến hiện tại i là nến Wyckoff Spring xác nhận:
        // Đáy không bị rơi thêm (Low=1960 >= Low[i-1]=1960), nến xanh rút chân đóng vượt điểm giữa nến trước
        // Prev Mid = (2050 + 1960)/2 = 2005. Close = 2010 > 2005, Open = 1965, High = 2015.
        // Range = 55, CLV = (2*2010 - 2015 - 1960)/55 = 45/55 = 0.818 >= 0.20
        // Lower wick = (1965 - 1960)/55 = 5/55 = 0.091 < 0.20 => Để thỏa min_excess_wick_ratio = 0.20:
        // Cần lower_wick = min(1980, 2010) - 1960 = 20 / 55 = 0.364 >= 0.20!
        // Cho Open = 1980.0, High = 2015.0, Low = 1960.0, Close = 2010.0!
        opens[i] = 1980.0; highs[i] = 2015.0; lows[i] = 1960.0; closes[i] = 2010.0;
        let raw_cache_spring = RawDataCache {
            datetime: std::sync::Arc::new(datetimes),
            open: std::sync::Arc::new(opens),
            high: std::sync::Arc::new(highs),
            low: std::sync::Arc::new(lows),
            close: std::sync::Arc::new(closes),
            volume: std::sync::Arc::new(volumes),
            custom_columns: HashMap::new(),
        };
        let data_view_spring = DataView::new(raw_cache_spring);
        let tick_spring = ferro_trade::strategy::TickContext {
            i, equity: 1000.0, data_view: &data_view_spring, indicator_manager: &indicator_manager,
            trades: &trades, last_closed_trade: None, actions: &mut actions, position: Position::default(),
        };

        let prop_spring = ReversalScanner.scan(&tick_spring, &sensors, &matrix, &config);
        assert!(prop_spring.is_some(), "ReversalScanner PHẢI phê duyệt khi nến Wyckoff Spring giữ đáy và đóng vượt điểm giữa nến trước!");
    }

    #[test]
    fn test_dna_width_and_parameter_defs_integrity() {
        let width = HeliosConfigV3::DNA_WIDTH;
        assert!(width > 0, "DNA_WIDTH phải lớn hơn 0!");
        let defs = HeliosConfigV3::parameter_defs();
        assert!(!defs.iter().any(|(name, _)| name.contains("trend_lookback")), "defs không được chứa trend_lookback đã xóa bỏ!");
        assert!(!defs.iter().any(|(name, _)| name.contains("chaos_lookback")), "defs không được chứa chaos_lookback đã xóa bỏ!");
        let has_be_trigger_abs = defs.iter().any(|(name, _)| name.contains("be_atr_trigger_absorption"));
        let has_be_buffer_abs = defs.iter().any(|(name, _)| name.contains("be_buffer_atr_absorption"));
        assert!(has_be_trigger_abs, "defs phải chứa be_atr_trigger_absorption theo TODO-STAB-01!");
        assert!(has_be_buffer_abs, "defs phải chứa be_buffer_atr_absorption theo TODO-STAB-01!");
        let has_stoch_climax = defs.iter().any(|(name, _)| name.contains("stoch_climax_thresh"));
        assert!(has_stoch_climax, "defs phải chứa tactical_sniper.pullback.stoch_climax_thresh!");
        let has_micro_natr_tf = defs.iter().any(|(name, _)| name.contains("indicators.micro_natr.timeframe"));
        let has_micro_natr_period = defs.iter().any(|(name, _)| name.contains("indicators.micro_natr.period"));
        assert!(has_micro_natr_tf, "defs phải chứa indicators.micro_natr.timeframe!");
        assert!(has_micro_natr_period, "defs phải chứa indicators.micro_natr.period!");
        let has_pb_min_clv = defs.iter().any(|(name, _)| name.contains("pullback.min_clv"));
        assert!(has_pb_min_clv, "defs phải chứa tactical_sniper.pullback.min_clv theo TODO-PB-02!");
        let has_base_sl = defs.iter().any(|(name, _)| name.contains("base_sl_atr"));
        assert!(!has_base_sl, "defs KHÔNG được chứa base_sl_atr (đã đánh dấu #[dna(skip)] theo TODO-RSK-02)!");
        let has_price_z = defs.iter().any(|(name, _)| name.contains("indicators.price_z_score.lookback"));
        let has_vol_z = defs.iter().any(|(name, _)| name.contains("indicators.vol_z_score.lookback"));
        assert!(has_price_z, "defs phải chứa indicators.price_z_score.lookback!");
        assert!(has_vol_z, "defs phải chứa indicators.vol_z_score.lookback!");
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
        // 🛡️ TODO-AUDIT-NEW-61 (H-CFG-06): Khóa dải DNA phủ Forex -> Memecoin, 150 nấc đều không lệch biên
        let min_natr_def = defs.iter().find(|(name, _)| name.contains("min_natr_threshold")).map(|(_, d)| d);
        match min_natr_def {
            Some(ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step }) => {
                assert!((min - 0.0002).abs() < 1e-12, "min_natr_threshold.min phải = 0.0002, nhận {}", min);
                assert!((max - 0.0300).abs() < 1e-12, "min_natr_threshold.max phải = 0.0300, nhận {}", max);
                let s = step.expect("min_natr_threshold phải có step");
                assert!((s - 0.0002).abs() < 1e-12, "min_natr_threshold.step phải = 0.0002, nhận {}", s);
                let n_steps = (max - min) / s;
                assert!((n_steps - n_steps.round()).abs() < 1e-9, "Dải phải chia hết cho step (chống Discrete Binning Bias)");
            }
            other => panic!("min_natr_threshold phải là ParamDefinition::Range, nhận {:?}", other),
        }
        // 🛡️ TODO-AUDIT-NEW-60 (H-CFG-05): Khóa dải DNA Trailing Stop phân tầng phủ rộng [0.40, 6.00], step 0.10 (56 bước đều)
        for tsl_param in &["tsl_atr_tight", "tsl_atr_normal", "tsl_atr_loose"] {
            let tsl_def = defs.iter().find(|(name, _)| name.contains(tsl_param)).map(|(_, d)| d);
            match tsl_def {
                Some(ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step }) => {
                    assert!((min - 0.40).abs() < 1e-12, "{}.min phải = 0.40, nhận {}", tsl_param, min);
                    assert!((max - 6.00).abs() < 1e-12, "{}.max phải = 6.00, nhận {}", tsl_param, max);
                    let s = step.expect("phải có step");
                    assert!((s - 0.10).abs() < 1e-12, "{}.step phải = 0.10, nhận {}", tsl_param, s);
                    let n_steps = (max - min) / s;
                    assert!((n_steps - n_steps.round()).abs() < 1e-9, "{}: Dải phải chia hết cho step (chống Discrete Binning Bias)", tsl_param);
                }
                other => panic!("{} phải là ParamDefinition::Range, nhận {:?}", tsl_param, other),
            }
        }
        assert!(has_pb_boundary_z, "defs phải chứa tactical_sniper.pullback.max_boundary_z!");
        assert!(has_pb_squeeze, "defs phải chứa tactical_sniper.pullback.max_squeeze_intensity!");
        assert!(!has_max_chaos, "defs KHÔNG được chứa max_chaos_penalty (đã loại bỏ gen theo Pure Confidence FLAW-RISK.11)!");
        assert!(!has_act_score, "defs KHÔNG được chứa min_activation_score ở các Tactical Scanners!");
        let has_matrix_adx = defs.iter().any(|(name, _)| name.contains("adx_trend_threshold"));
        let has_matrix_vol_lb = defs.iter().any(|(name, _)| name.contains("volume_lookback"));
        let has_matrix_mom_lb = defs.iter().any(|(name, _)| name.contains("momentum_lookback"));
        let has_matrix_z_sens = defs.iter().any(|(name, _)| name.contains("z_score_sensitivity"));
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
        let has_rev_climax_vol = defs.iter().any(|(name, _)| name.contains("tactical_sniper.reversal.climax_vol_threshold"));
        let has_rev_exhaust_vol = defs.iter().any(|(name, _)| name.contains("exhaustion_vol_drop"));
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
        assert!(has_rev_climax_vol, "defs phải chứa tactical_sniper.reversal.climax_vol_threshold theo TODO-AUDIT-H-01!");
        assert!(!has_rev_exhaust_vol, "defs KHÔNG được chứa exhaustion_vol_drop theo TODO-AUDIT-H-01!");
        assert!(has_mom_adverse, "defs phải chứa tactical_sniper.momentum.max_adverse_slope!");
        assert!(has_mom_w_break, "defs phải chứa tactical_sniper.momentum.w_breakout!");
        assert!(!has_matrix_adx, "defs KHÔNG được chứa matrix.adx_trend_threshold (đã loại bỏ theo TASK-MAT-07)!");
        assert!(!has_matrix_vol_lb, "defs KHÔNG được chứa matrix.volume_lookback (đã loại bỏ theo TASK-MAT-07)!");
        assert!(!has_matrix_mom_lb, "defs KHÔNG được chứa matrix.momentum_lookback (đã loại bỏ theo TASK-MAT-07)!");
        assert!(!has_matrix_z_sens, "defs KHÔNG được chứa matrix.z_score_sensitivity (đã loại bỏ theo TODO-CFG-05)!");
        assert!(has_matrix_kama_scale, "defs phải chứa matrix.kama_slope_scale_atr!");
        assert!(!has_matrix_vol_z, "defs KHÔNG được chứa matrix.vol_z_sensitivity (đã loại bỏ theo TASK-MAT-01)!");
        assert!(!has_phantom_comp, "defs KHÔNG được chứa gen ma min_confidence_compression!");
        assert!(!has_phantom_veto, "defs KHÔNG được chứa gen ma min_multiplier_veto!");
        assert!(!has_breakout, "defs KHÔNG được chứa cooldown_breakout_atr_mult (Cooldown thuộc quyền sở hữu của Bot Engine)!");
        assert!(has_abs_adverse, "defs phải chứa tactical_sniper.absorption.max_adverse_slope!");
        assert!(has_pb_max_dist, "defs phải chứa tactical_sniper.pullback.max_pullback_dist_atr!");
        assert!(has_pb_inval, "defs phải chứa tactical_sniper.pullback.invalidation_buffer_atr!");

        // ⚡ TODO-AUDIT-NEW-161 (GATE-FLASH-CRASH-BREAKER): 4 gen của Cầu Dao Khẩn Cấp Flash Crash & Liquidity Vacuum
        let has_cb_atr_shock = defs.iter().any(|(name, _)| name.contains("circuit_breaker.atr_shock_threshold"));
        let has_cb_vol_z = defs.iter().any(|(name, _)| name.contains("circuit_breaker.vol_z_shock_threshold"));
        let has_cb_vacuum = defs.iter().any(|(name, _)| name.contains("circuit_breaker.vacuum_atr_threshold"));
        let has_cb_freeze = defs.iter().any(|(name, _)| name.contains("circuit_breaker.freeze_bars"));
        assert!(has_cb_atr_shock, "defs phải chứa circuit_breaker.atr_shock_threshold!");
        assert!(has_cb_vol_z, "defs phải chứa circuit_breaker.vol_z_shock_threshold!");
        assert!(has_cb_vacuum, "defs phải chứa circuit_breaker.vacuum_atr_threshold!");
        assert!(has_cb_freeze, "defs phải chứa circuit_breaker.freeze_bars!");

        // 🌪️ TODO-AUDIT-NEW-191 (REV-MACRO-GATE): 2 gen Cổng Chặn Sức Mạnh Xu Hướng Vĩ Mô Cho Reversal
        let rev_slope_def = defs.iter().find(|(name, _)| name.contains("tactical_sniper.reversal.max_adverse_kama_slope"));
        assert!(rev_slope_def.is_some(), "defs phải chứa tactical_sniper.reversal.max_adverse_kama_slope!");
        let (_, rev_slope_attr) = rev_slope_def.unwrap();
        match rev_slope_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert!((min - 0.02).abs() < 1e-12, "max_adverse_kama_slope min phải là 0.02");
                assert!((max - 0.80).abs() < 1e-12, "max_adverse_kama_slope max phải là 0.80");
                assert_eq!(*step, Some(0.02), "max_adverse_kama_slope step phải là Some(0.02)");
            }
            _ => panic!("max_adverse_kama_slope phải là ParamDefinition::Range"),
        }

        let rev_adx_def = defs.iter().find(|(name, _)| name.contains("tactical_sniper.reversal.max_adverse_adx"));
        assert!(rev_adx_def.is_some(), "defs phải chứa tactical_sniper.reversal.max_adverse_adx!");
        let (_, rev_adx_attr) = rev_adx_def.unwrap();
        match rev_adx_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert!((min - 15.0).abs() < 1e-12, "max_adverse_adx min phải là 15.0");
                assert!((max - 75.0).abs() < 1e-12, "max_adverse_adx max phải là 75.0");
                assert_eq!(*step, Some(5.0), "max_adverse_adx step phải là Some(5.0)");
            }
            _ => panic!("max_adverse_adx phải là ParamDefinition::Range"),
        }

        assert_eq!(width, 115, "DNA_WIDTH phải là 115 chiều sau khi thêm gen mfi_absorption_thresh cho Absorption (TODO-AUDIT-NEW-49)!");

        let abs_mfi_def = defs.iter().find(|(name, _)| name.contains("tactical_sniper.absorption.mfi_absorption_thresh"));
        assert!(abs_mfi_def.is_some(), "defs phải chứa tactical_sniper.absorption.mfi_absorption_thresh!");
        let (_, abs_mfi_attr) = abs_mfi_def.unwrap();
        match abs_mfi_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert!((min - 10.0).abs() < 1e-12, "mfi_absorption_thresh min phải là 10.0");
                assert!((max - 50.0).abs() < 1e-12, "mfi_absorption_thresh max phải là 50.0");
                assert_eq!(*step, Some(5.0), "mfi_absorption_thresh step phải là Some(5.0)");
            }
            _ => panic!("mfi_absorption_thresh phải là ParamDefinition::Range"),
        }

        let diss_span_def = defs.iter().find(|(name, _)| name.contains("circuit_breaker.dissipation_span_threshold"));
        assert!(diss_span_def.is_some(), "defs phải chứa circuit_breaker.dissipation_span_threshold!");
        let (_, diss_span_attr) = diss_span_def.unwrap();
        match diss_span_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert!((min - 1.0).abs() < 1e-12, "dissipation_span_threshold min phải là 1.0");
                assert!((max - 2.5).abs() < 1e-12, "dissipation_span_threshold max phải là 2.5");
                assert_eq!(*step, Some(0.1), "dissipation_span_threshold step phải là Some(0.1)");
            }
            _ => panic!("dissipation_span_threshold phải là ParamDefinition::Range"),
        }

        let diss_body_def = defs.iter().find(|(name, _)| name.contains("circuit_breaker.dissipation_max_body_ratio"));
        assert!(diss_body_def.is_some(), "defs phải chứa circuit_breaker.dissipation_max_body_ratio!");
        let (_, diss_body_attr) = diss_body_def.unwrap();
        match diss_body_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert!((min - 0.15).abs() < 1e-12, "dissipation_max_body_ratio min phải là 0.15");
                assert!((max - 0.45).abs() < 1e-12, "dissipation_max_body_ratio max phải là 0.45");
                assert_eq!(*step, Some(0.05), "dissipation_max_body_ratio step phải là Some(0.05)");
            }
            _ => panic!("dissipation_max_body_ratio phải là ParamDefinition::Range"),
        }

        let count_struct_sl = defs.iter().filter(|(name, _)| name.contains("struct_sl_buffer_atr")).count();
        assert_eq!(count_struct_sl, 3, "defs phải chứa chính xác 3 gene struct_sl_buffer_atr cho 3 Scanners (pullback, momentum, absorption)!");

        let rsi_mom_def = defs.iter().find(|(name, _)| name.contains("tactical_sniper.momentum.rsi_threshold"));
        assert!(rsi_mom_def.is_some(), "defs phải chứa tactical_sniper.momentum.rsi_threshold!");
        let (_, rsi_attr) = rsi_mom_def.unwrap();
        match rsi_attr {
            ferro_trade::optimizer::params::ParamDefinition::Range { min, max, step } => {
                assert_eq!(*min, 52.0, "rsi_threshold min phải là 52.0!");
                assert_eq!(*max, 88.0, "rsi_threshold max phải là 88.0!");
                assert_eq!(*step, Some(1.0), "rsi_threshold step phải là Some(1.0)!");
            }
            _ => panic!("rsi_threshold phải là ParamDefinition::Range"),
        }

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
    fn test_tsl_hierarchy_enforcement_prevents_inversion() {
        use crate::strategy::core_v3::config::RiskStopParams;

        // 1. Kiểm tra trực tiếp trên struct RiskStopParams khi có giá trị nghịch lý
        let mut stop_params = RiskStopParams {
            tsl_atr_tight: 4.8,
            tsl_atr_normal: 2.5,
            tsl_atr_loose: 0.8,
            ..Default::default()
        };
        stop_params.enforce_tsl_hierarchy();
        assert!((stop_params.tsl_atr_tight - 0.8).abs() < 1e-12, "tsl_atr_tight phải là giá trị nhỏ nhất (0.8)");
        assert!((stop_params.tsl_atr_normal - 2.5).abs() < 1e-12, "tsl_atr_normal phải là giá trị trung gian (2.5)");
        assert!((stop_params.tsl_atr_loose - 4.8).abs() < 1e-12, "tsl_atr_loose phải là giá trị lớn nhất (4.8)");

        // 2. Kiểm tra trường hợp GA bốc 3 số bằng nhau (cho phép chấp nhận)
        let mut equal_params = RiskStopParams {
            tsl_atr_tight: 2.0,
            tsl_atr_normal: 2.0,
            tsl_atr_loose: 2.0,
            ..Default::default()
        };
        equal_params.enforce_tsl_hierarchy();
        assert!((equal_params.tsl_atr_tight - 2.0).abs() < 1e-12);
        assert!((equal_params.tsl_atr_normal - 2.0).abs() < 1e-12);
        assert!((equal_params.tsl_atr_loose - 2.0).abs() < 1e-12);

        // 3. Kiểm tra thông qua HeliosStrategyV3::update_from_dna
        let mut strategy = HeliosStrategyV3::default();
        let mut dna = vec![0.5; HeliosConfigV3::DNA_WIDTH];
        
        let defs = HeliosConfigV3::parameter_defs();
        let tight_idx = defs.iter().position(|(name, _)| name.contains("tsl_atr_tight")).unwrap();
        let normal_idx = defs.iter().position(|(name, _)| name.contains("tsl_atr_normal")).unwrap();
        let loose_idx = defs.iter().position(|(name, _)| name.contains("tsl_atr_loose")).unwrap();

        // Nạp giá trị nghịch lý thô: tight=5.0, normal=3.0, loose=1.0
        dna[tight_idx] = 5.0;
        dna[normal_idx] = 3.0;
        dna[loose_idx] = 1.0;

        strategy.update_from_dna(&dna);
        let stop_cfg = &strategy.config.risk_management.stop_management;
        assert!(stop_cfg.tsl_atr_tight <= stop_cfg.tsl_atr_normal, "Tight ({}) phải <= Normal ({})", stop_cfg.tsl_atr_tight, stop_cfg.tsl_atr_normal);
        assert!(stop_cfg.tsl_atr_normal <= stop_cfg.tsl_atr_loose, "Normal ({}) phải <= Loose ({})", stop_cfg.tsl_atr_normal, stop_cfg.tsl_atr_loose);
        assert!((stop_cfg.tsl_atr_tight - 1.0).abs() < 0.15, "Tight phải là 1.0 sau khi sắp xếp, nhận {}", stop_cfg.tsl_atr_tight);
        assert!((stop_cfg.tsl_atr_loose - 5.0).abs() < 0.15, "Loose phải là 5.0 sau khi sắp xếp, nhận {}", stop_cfg.tsl_atr_loose);
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

        let mut opens_range = vec![2000.0; len];
        let mut highs_range = vec![2050.0; len];
        let mut lows_range = vec![1970.0; len];
        let mut closes_range = vec![2000.0; len];
        opens_range[i] = 1972.0;
        highs_range[i] = 1985.0;
        lows_range[i] = 1970.0;
        closes_range[i] = 1982.0;

        let raw_cache_range = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(opens_range),
            high: std::sync::Arc::new(highs_range),
            low: std::sync::Arc::new(lows_range),
            close: std::sync::Arc::new(closes_range),
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
        let mut mfi_abs = vec![30.0; len];
        mfi_abs[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_abs);
        
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
        let mut slope_abs = vec![-0.05; len];
        slope_abs[i] = 0.02; // a = +0.07 > 0
        matrix_abs.kama_slope_atr = std::sync::Arc::new(slope_abs);
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
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![-0.05; len];
        slope_vec[i] = 0.02; // a = +0.07 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "AbsorptionScanner PHẢI sinh Proposal Long khi thị trường tích lũy chuẩn trong vùng phẳng!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.raw_score >= 0.1);
        assert!(p.suggested_sl.is_some());
    }

    #[test]
    fn test_absorption_spring_geometry_wick_deep_close_controlled() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến Spring vi mô 1m: low 1985 (nhúng sâu đáy %B_low = 5/40 = 0.125 <= 0.40), rút chân mạnh lên 1995 (CLV = +0.818 >= 0.20)
        // Giá đóng cửa 1995 <= bb_middle (2000) và %B_close = 15/40 = 0.375 <= 0.40 + 0.10
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1990.0; len]),
            high: std::sync::Arc::new(vec![1996.0; len]),
            low: std::sync::Arc::new(vec![1985.0; len]),
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

        let mut config = HeliosConfigV3::default();
        config.tactical_sniper.absorption.squeeze_threshold = 0.5;
        config.tactical_sniper.absorption.vol_z_threshold = 0.5;
        config.tactical_sniper.absorption.min_activation_score = 0.1;
        config.tactical_sniper.absorption.struct_sl_buffer_atr = 0.5;
        config.tactical_sniper.absorption.max_adverse_slope = 0.15;
        config.tactical_sniper.absorption.spring_tolerance = 0.10;
        config.tactical_sniper.absorption.min_clv = 0.20;
        config.tactical_sniper.absorption.max_boundary_pct_b = 0.40;

        let mut sensors = HeliosSensors::default();
        // Giả lập VWMA = 2005.0 > bb_middle = 2000.0 (Ngược với logic VWMA cũ!)
        // Chứng minh: Việc bỏ VWMA giúp mở khóa cây nến Spring hoàn hảo này mà không bị VETO vô lý!
        sensors.vwma = std::sync::Arc::new(vec![2005.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![-0.05; len];
        slope_vec[i] = 0.02; // a = +0.07 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "AbsorptionScanner PHẢI phê duyệt nến Spring khi râu quét sâu và rút chân mạnh, bất chấp VWMA!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);
        assert!(p.suggested_sl.unwrap() < 1995.0, "SL phải nhỏ hơn giá đóng cửa!");
    }

    #[test]
    fn test_absorption_blocks_chasing_when_close_crosses_middle() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến rút chân tăng thốc quá đà, giá đóng cửa vượt quá nửa trên dải nén (close 2005 > bb_middle 2000)
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1985.0; len]),
            high: std::sync::Arc::new(vec![2006.0; len]),
            low: std::sync::Arc::new(vec![1982.0; len]),
            close: std::sync::Arc::new(vec![2005.0; len]), // Đu đỉnh nến vượt quá BB middle
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

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![-0.05; len];
        slope_vec[i] = 0.02; // a = +0.07 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "AbsorptionScanner PHẢI CHẶN ĐỨNG khi giá đóng cửa vượt quá BB Middle (chống mua đu đỉnh)!");
    }

    #[test]
    fn test_absorption_blocks_marubozu_knife_at_lower_band() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến đỏ đặc ruột cắm đầu rơi tự do (Marubozu), low chạm 1982 (%B_low <= 0.40) nhưng close = 1982 (CLV = -1.0)
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1995.0; len]),
            high: std::sync::Arc::new(vec![1995.0; len]),
            low: std::sync::Arc::new(vec![1982.0; len]),
            close: std::sync::Arc::new(vec![1982.0; len]), // CLV = -1.0, không có rút chân!
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
        config.tactical_sniper.absorption.min_clv = 0.20;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![-0.05; len];
        slope_vec[i] = 0.02; // a = +0.07 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "AbsorptionScanner PHẢI CHẶN ĐỨNG nến đỏ Marubozu xả đặc ruột (chống bắt dao rơi)!");
    }

    #[test]
    fn test_absorption_upthrust_short_symmetric() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Nến Upthrust vi mô 1m: high 2015 (nhô cao đỉnh %B_high = 35/40 = 0.875 >= 0.60), đè đầu giảm mạnh xuống close 2003 (CLV = -0.846 <= -0.20)
        // Giá đóng cửa 2003 >= bb_middle (2000) và %B_close = 23/40 = 0.575 >= 1.0 - 0.40 - 0.10 = 0.50
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2010.0; len]),
            high: std::sync::Arc::new(vec![2015.0; len]),
            low: std::sync::Arc::new(vec![2002.0; len]),
            close: std::sync::Arc::new(vec![2003.0; len]),
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
        config.tactical_sniper.absorption.max_boundary_pct_b = 0.40;

        let mut sensors = HeliosSensors::default();
        // Giả lập VWMA = 1995.0 < bb_middle = 2000.0 (Ngược với logic VWMA cũ cho Short)
        sensors.vwma = std::sync::Arc::new(vec![1995.0; len]);
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![70.0; len];
        mfi_vec[i] = 68.0; // curr_mfi >= 65.0 && curr_mfi < prev_mfi (70 -> 68)
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![0.05; len];
        slope_vec[i] = -0.02; // kama_deceleration = -0.02 - 0.05 = -0.07 < 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "AbsorptionScanner PHẢI phê duyệt Short Upthrust đối xứng!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Short);
        assert!(p.suggested_sl.unwrap() > 2003.0, "SL Short phải lớn hơn giá đóng cửa!");
    }

    #[test]
    fn test_absorption_blocks_deep_underground_breakdown() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Cây nến xả lũ cực mạnh (Waterfall): bb_lower = 1980.0, bb_upper = 2020.0 (width = 40.0)
        // spring_tolerance = 0.10 -> sàn dưới tối đa là bb_lower - 0.10 * 40 = 1976.0 (pct_b = -0.10)
        // Nến này rơi xuống low = 1950 (pct_b_low = -0.75 <= 0.40), rút nhẹ lên close = 1965 (pct_b_close = -0.375 < -0.10)
        // Mặc dù rút chân có CLV = (2*1965 - 1975 - 1950) / 25 = 5/25 = +0.20 >= min_clv, nhưng đóng cửa thủng sâu dưới sàn!
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1975.0; len]),
            high: std::sync::Arc::new(vec![1975.0; len]),
            low: std::sync::Arc::new(vec![1950.0; len]),
            close: std::sync::Arc::new(vec![1965.0; len]),
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
        config.tactical_sniper.absorption.max_boundary_pct_b = 0.40;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![30.0; len];
        mfi_vec[i] = 32.0;
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![-0.05; len];
        slope_vec[i] = 0.02; // a = +0.07 > 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "AbsorptionScanner PHẢI CHẶN ĐỨNG khi nến đóng sâu dưới sàn hỗ trợ (True Breakdown pct_b_close < -spring_tol)!");
    }

    #[test]
    fn test_absorption_blocks_runaway_breakout() {
        use crate::strategy::core_v3::scanners::absorption::AbsorptionScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;
        use ferro_trade::data::{RawDataCache, DataView};
        use ferro_trade::backtest::manager::IndicatorManager;
        use ferro_trade::broker::position::Position;
        use smallvec::SmallVec;
        use std::collections::HashMap;

        let len = 50;
        let i = len - 1;

        // Cây nến bứt phá điên cuồng (Runaway Pump): bb_lower = 1980.0, bb_upper = 2020.0 (width = 40.0)
        // spring_tolerance = 0.10 -> trần trên tối đa là bb_upper + 0.10 * 40 = 2024.0 (pct_b = 1.10)
        // Nến này phóng lên high = 2050 (pct_b_high = 1.75 >= 0.60), thụt nhẹ đỉnh về close = 2035 (pct_b_close = 1.375 > 1.10)
        // Mặc dù thụt đỉnh có CLV = (2*2035 - 2050 - 2025) / 25 = -5/25 = -0.20 <= -min_clv, nhưng đóng cửa vượt trần trên trời!
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2025.0; len]),
            high: std::sync::Arc::new(vec![2050.0; len]),
            low: std::sync::Arc::new(vec![2025.0; len]),
            close: std::sync::Arc::new(vec![2035.0; len]),
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
        config.tactical_sniper.absorption.max_boundary_pct_b = 0.40;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        let mut mfi_vec = vec![70.0; len];
        mfi_vec[i] = 68.0; // curr_mfi >= 65.0 && curr_mfi < prev_mfi (70 -> 68)
        sensors.mfi = std::sync::Arc::new(mfi_vec);

        let mut matrix = MarketMatrixV3::from_sensors(&config, &sensors);
        matrix.squeeze_intensity = std::sync::Arc::new(vec![1.2; len]);
        let mut slope_vec = vec![0.05; len];
        slope_vec[i] = -0.02; // kama_deceleration = -0.02 - 0.05 = -0.07 < 0
        matrix.kama_slope_atr = std::sync::Arc::new(slope_vec);

        let prop = AbsorptionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "AbsorptionScanner PHẢI CHẶN ĐỨNG khi nến bứt phá đóng vượt trần kháng cự (Runaway Breakout pct_b_close > 1.0 + spring_tol)!");
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

        let mut opens = vec![2000.0; len];
        let mut highs = vec![2050.0; len];
        let mut lows = vec![1970.0; len];
        let mut closes = vec![2000.0; len];
        opens[i] = 1972.0;
        highs[i] = 1985.0;
        lows[i] = 1970.0;
        closes[i] = 1982.0; // Nến xanh rút chân tại VAL

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
        assert!((tp - 2050.0).abs() < 1.0, "TP ({}) phải hướng sang biên đối diện VAH 2050.0", tp);
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
        let mut opens = vec![1996.0; len];
        let mut highs = vec![1997.0; len];
        let mut lows = vec![1988.0; len];
        let mut closes = vec![1990.0; len];
        // CLV = (2*1990 - 1997 - 1980)/(1997-1980) = (3980 - 3977)/17 = 3/17 = 0.176 >= min_clv(0.05)
        // Giá đóng cửa 1990.0 < KAMA (2000.0) - 0.5 * 17.32 = 1991.34 -> delta_kama = (1990 - 2000)/17.32 = -0.577 < -0.50!
        opens[i] = 1985.0;
        highs[i] = 1997.0;
        lows[i] = 1980.0;
        closes[i] = 1990.0;
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
        let mut opens = vec![2020.0; len];
        let mut highs = vec![2030.0; len];
        let mut lows = vec![2019.0; len];
        let mut closes = vec![2025.0; len];
        // CLV = (2*2028 - 2030 - 2019)/(2030-2019) = (4056 - 4049)/11 = 7/11 = 0.636 >= min_clv(0.05)
        // Giá đóng cửa 2028.0 > KAMA (2000.0) + 2.0 * 10.0 = 2020.0 -> Bị chặn ĐÍCH DANH bởi max_pullback_dist_atr!
        opens[i] = 2020.0;
        highs[i] = 2030.0;
        lows[i] = 2019.0;
        closes[i] = 2028.0;
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
        config.tactical_sniper.momentum.max_adverse_slope = 0.02;

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.rsi_momentum = std::sync::Arc::new(vec![75.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.5; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![-0.25; len]); // Ngược trend mạnh

        let matrix = MarketMatrixV3::default();

        let prop = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_none(), "Bùng nổ Long khi macro trend dốc xuống quá ngưỡng adverse_slope (0.25 > 0.02) PHẢI bị VETO!");
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
    fn test_momentum_rsi_sub_50_blocks_long() {
        let len = 50;
        let i = 49;

        // Nến bứt phá nhẹ qua BB Upper: Close=2021 > bb_upper 2020.0, Low=2018, High=2022, Open=2019
        // CLV = (2*2021 - 2022 - 2018) / 4 = 2/4 = 0.50 >= 0.20
        // Breakout dist = 1.0, Target breakout = 10 * 0.25 = 2.5 => f_break = 1.0 / 2.5 = 0.40
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![2019.0; len]),
            high: std::sync::Arc::new(vec![2022.0; len]),
            low: std::sync::Arc::new(vec![2018.0; len]),
            close: std::sync::Arc::new(vec![2021.0; len]),
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
        // Cố tình truyền cấu hình cũ dị rsi_threshold = 25.0 để kiểm tra chốt chặn clamp(50.0, 90.0)
        config.tactical_sniper.momentum.rsi_threshold = 25.0;
        config.tactical_sniper.momentum.vol_z_threshold = 1.5;
        config.tactical_sniper.momentum.w_breakout = 0.35;
        config.tactical_sniper.momentum.w_volume = 0.35;
        // Điểm Breakout (0.35 * 0.40) + Vol (0.35 * tanh(1.0) = 0.266) = 0.406.
        // Cần thêm f_rsi > 0 để vượt ngưỡng kích hoạt 0.50.
        config.tactical_sniper.momentum.min_activation_score = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![1.5; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.10; len]);

        let matrix = MarketMatrixV3::default();

        // 1. Kịch bản RSI = 45.0 (< 50.0, xu hướng yếu/giảm):
        // Nếu không có clamp >= 50.0, rsi_threshold=25.0 sẽ cho rsi_excess = 45 - 25 = 20, f_rsi = tanh(20/15) = 0.87,
        // khiến raw_score = 0.406 + 0.30 * 0.87 = 0.667 > 0.50 (kích hoạt bùng nổ sai lầm trong bear market)!
        // Nhờ clamp(50.0, 90.0), rsi_thresh = 50.0 => rsi_excess = 0 => f_rsi = 0 => raw_score = 0.406 < 0.50 => CHẶN ĐỨNG!
        sensors.rsi_momentum = std::sync::Arc::new(vec![45.0; len]);
        let prop_sub_50 = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_sub_50.is_none(), "Khi RSI < 50.0, điểm RSI Long phải triệt tiêu về 0, không được kích hoạt bùng nổ Long!");

        // 2. Kịch bản RSI = 70.0 (xung lực bùng nổ thực sự):
        // rsi_thresh = 50.0 => rsi_excess = 70 - 50 = 20 => f_rsi = 0.87 => raw_score = 0.667 > 0.50 => DUYỆT!
        sensors.rsi_momentum = std::sync::Arc::new(vec![70.0; len]);
        let prop_strong = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_strong.is_some(), "Khi RSI = 70.0 vượt ngưỡng xung lực, scanner phải kích hoạt bùng nổ Long hợp lệ!");
    }

    #[test]
    fn test_mtf_phase_coherence_blocks_counter_trend_micro_noise() {
        let len = 50;
        let i = 49;

        // Nến 1m bùng nổ vượt đỉnh BB Upper (Long candidate):
        // Open=2015, High=2026, Low=2014, Close=2025 > bb_upper 2020.0 (CLV = 0.833, close > open)
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
        config.tactical_sniper.momentum.max_adverse_slope = 0.02; // Ngưỡng dung sai vi sai 0.02 (0.4% ATR)

        let mut sensors = HeliosSensors::default();
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.rsi_momentum = std::sync::Arc::new(vec![75.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![2.5; len]);

        let mut matrix = MarketMatrixV3::default();

        // 1. Kịch bản Ca 1: Vĩ mô 15m/1h dốc âm mạnh (Downtrend thác lũ -0.10 < -0.02)
        // Dù nến 1m bùng nổ rất đẹp, MTF Phase Coherence Gate PHẢI VETO cấm bắt dao rơi!
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.10; len]);
        let prop_veto = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_veto.is_none(), "Bùng nổ Long khi macro KAMA dốc âm mạnh (-0.10 < -0.02) PHẢI bị VETO!");

        // 2. Kịch bản Ca 2: Vĩ mô 15m/1h đi ngang phẳng trong vùng nén tích lũy (-0.01 nằm trong [-0.02, 0.02])
        // Chốt chặn cho phép nổ bứt phá chân sóng Altcoin, KHÔNG được bóp nghẹt!
        matrix.kama_slope_atr = std::sync::Arc::new(vec![-0.01; len]);
        let prop_flat = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_flat.is_some(), "Bùng nổ Long khi macro KAMA đi ngang tích lũy (-0.01 >= -0.02) PHẢI được DUYỆT!");

        // 3. Kịch bản Ca 3: Vĩ mô 15m/1h đồng thuận pha dốc lên (+0.15)
        // Thuận sóng vĩ mô -> DUYỆT!
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.15; len]);
        let prop_aligned = MomentumIgnitionScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop_aligned.is_some(), "Bùng nổ Long khi macro KAMA đồng thuận pha dốc lên (+0.15) PHẢI được DUYỆT!");

        // 4. Kịch bản Ca 4: Nến 1m Short (close < bb_lower) nhưng vĩ mô dốc dương mạnh (+0.10 > 0.02)
        // Cản đầu xe tăng -> VETO!
        let raw_cache_short = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![1985.0; len]),
            high: std::sync::Arc::new(vec![1986.0; len]),
            low: std::sync::Arc::new(vec![1974.0; len]),
            close: std::sync::Arc::new(vec![1975.0; len]), // < bb_lower 1980.0
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view_short = DataView::new(raw_cache_short);
        let mut actions_short = SmallVec::new();
        let trades_short = Vec::new();
        let tick_short = ferro_trade::strategy::TickContext {
            i,
            equity: 1000.0,
            data_view: &data_view_short,
            indicator_manager: &indicator_manager,
            trades: &trades_short,
            last_closed_trade: None,
            actions: &mut actions_short,
            position: Position::default(),
        };
        sensors.rsi_momentum = std::sync::Arc::new(vec![25.0; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.10; len]);
        let prop_short_veto = MomentumIgnitionScanner.scan(&tick_short, &sensors, &matrix, &config);
        assert!(prop_short_veto.is_none(), "Bùng nổ Short khi macro KAMA dốc dương mạnh (+0.10 > 0.02) PHẢI bị VETO!");
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

    #[test]
    fn test_update_from_dna_safe_on_empty_sensors() {
        use ferro_trade::strategy::Strategy;

        let mut strategy = HeliosStrategyV3::new(&std::collections::HashMap::new());
        assert!(strategy.sensors.closes.is_empty(), "Strategy mới tạo phải có sensors rỗng");
        assert!(strategy.matrix.kama_slope_atr.is_empty(), "Strategy mới tạo phải có matrix rỗng");

        let dummy_dna = vec![0.5; HeliosConfigV3::DNA_WIDTH];
        strategy.update_from_dna(&dummy_dna);

        // Đảm bảo không gọi from_sensors sinh ma trận rác và matrix vẫn giữ nguyên trạng thái an toàn
        assert!(strategy.matrix.kama_slope_atr.is_empty(), "Matrix phải giữ nguyên rỗng khi sensors rỗng");
        assert_eq!(
            strategy.config.indicators.keltner.timeframe,
            strategy.config.indicators.bbands.timeframe,
            "Keltner timeframe vẫn phải được đồng bộ"
        );
    }

    #[test]
    fn test_update_from_dna_updates_matrix_when_sensors_present() {
        use ferro_trade::strategy::Strategy;

        let mut strategy = HeliosStrategyV3::new(&std::collections::HashMap::new());
        let len = 50;
        
        // Giả lập sensors đã có dữ liệu sau khi chạy init()
        let mut sensors = HeliosSensors::default();
        sensors.closes = std::sync::Arc::new(vec![2000.0; len]);
        sensors.highs = std::sync::Arc::new(vec![2010.0; len]);
        sensors.lows = std::sync::Arc::new(vec![1990.0; len]);
        sensors.volumes = std::sync::Arc::new(vec![100.0; len]);
        sensors.kama_slope_adj = std::sync::Arc::new(vec![0.40; len]); // raw slope
        strategy.sensors = sensors;

        // Ban đầu chưa chạy from_sensors
        assert!(strategy.matrix.kama_slope_atr.is_empty());

        let mut dummy_dna = vec![0.5; HeliosConfigV3::DNA_WIDTH];
        let defs = HeliosConfigV3::parameter_defs();
        let scale_idx = defs.iter().position(|(name, _)| name.contains("kama_slope_scale_atr")).unwrap();

        // Nạp kama_slope_scale_atr = 0.40
        // Khi slope = 0.40 và scale_atr = 0.40 => tanh(0.40 / 0.40) = tanh(1.0) ~ 0.76159
        dummy_dna[scale_idx] = 0.40;
        strategy.update_from_dna(&dummy_dna);

        assert!(!strategy.matrix.kama_slope_atr.is_empty(), "Matrix phải được recompute khi sensors có dữ liệu");
        let actual_val = strategy.matrix.kama_slope_atr[0];
        let expected_val = (0.40 / 0.40_f64).tanh();
        assert!((actual_val - expected_val).abs() < 1e-4, "kama_slope_atr phải được re-scale chính xác: expected {}, actual {}", expected_val, actual_val);
    }

    #[test]
    fn test_veto_card_formatting_and_suppression_logic() {
        use ferro_trade::strategy::Strategy;
        use crate::strategy::core_v3::arbiter::{ArbiterDecision, AuctionCandidate, AuctionStatus, ScannerId};
        use crate::strategy::core_v3::gatekeeper::MarketRegime;

        let mut strategy = HeliosStrategyV3::new(&std::collections::HashMap::new());
        strategy.set_telemetry_enabled(true);
        assert!(strategy.telemetry_enabled);
        assert!(strategy.last_veto_signature.is_none());

        // Giả lập candidate bị RegimeIneligible
        let mut decision = ArbiterDecision::default();
        decision.gate_eval.regime = MarketRegime::RangeBound;
        decision.auction_ledger.push(AuctionCandidate {
            scanner_id: ScannerId::TrendPullback,
            signal_type: Some(crate::strategy::core_v3::arbiter::SignalType::Long),
            raw_score: 0.85,
            confidence: 0.80,
            status: AuctionStatus::RegimeIneligible(MarketRegime::RangeBound),
            reason: "Disabled in RangeBound".to_string(),
        });

        // 1. Kiểm tra format_veto_card có chứa đầy đủ thông tin TIER 1.5 và Macro Regime
        let len = 50;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![1_700_000_000_000_000_000i64; len]),
            open: std::sync::Arc::new(vec![2000.0; len]),
            high: std::sync::Arc::new(vec![2010.0; len]),
            low: std::sync::Arc::new(vec![1990.0; len]),
            close: std::sync::Arc::new(vec![2005.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        let trades = Vec::new();
        let tick = ferro_trade::strategy::TickContext {
            i: len - 1,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: Position::default(),
        };

        let card = DecisionCardFormatter::format_veto_card(
            &tick,
            &strategy.sensors,
            &strategy.matrix,
            &decision,
            2005.0,
            20.0,
        );
        assert!(card.contains("ORDER DEFENSE VETO"));
        assert!(card.contains("TIER 1.5 (STRUCTURAL ELIGIBILITY ROUTING)"));
        assert!(card.contains("Macro Regime: RANGE_BOUND"));
        assert!(card.contains("Eligible: [RangeOsc, ClimaxRev, AbsScout, AnomalyFade]"));
        assert!(card.contains("TREND_PB"));
        assert!(card.contains("[REGIME-VETO]"));

        // 2. Kiểm tra cơ chế First-Strike & Suppression
        let sig = format!("{}:{:?}", ScannerId::TrendPullback.as_str(), AuctionStatus::RegimeIneligible(MarketRegime::RangeBound));
        let ts_init = 1_700_000_000_000i64;

        // Lần đầu ghi nhận signature
        strategy.last_veto_signature = Some((sig.clone(), ts_init));

        // Nến tiếp theo sau 5 phút (cùng signature): Phải bị nén (should_emit = false)
        let ts_5m = ts_init + 5 * 60 * 1000;
        let should_emit_5m = match &strategy.last_veto_signature {
            Some((last_sig, last_ts)) if last_sig == &sig => ts_5m.saturating_sub(*last_ts) >= 30 * 60 * 1000,
            _ => true,
        };
        assert!(!should_emit_5m, "Cùng lý do VETO trong 30 phút bắt buộc phải bị NÉN (chống spam)!");

        // Nến sau 31 phút (cùng signature): Đã qua 30 phút -> Phải cho phép phát lại (should_emit = true)
        let ts_31m = ts_init + 31 * 60 * 1000;
        let should_emit_31m = match &strategy.last_veto_signature {
            Some((last_sig, last_ts)) if last_sig == &sig => ts_31m.saturating_sub(*last_ts) >= 30 * 60 * 1000,
            _ => true,
        };
        assert!(should_emit_31m, "Sau 30 phút, thẻ VETO phải được phép phát lại để cập nhật trạng thái!");
    }

    /// 🛡️ TODO-AUDIT-NEW-17 (TODO-SEN-08): Khử bom nổ chậm Panic Crash trong RangeOsc khi cảm biến rỗng
    #[test]
    fn test_range_osc_empty_sensors_no_panic() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;

        let len = 50;
        let i = 25;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new((0..len).map(|idx| (idx * 60_000) as i64).collect()),
            open: std::sync::Arc::new(vec![100.0; len]),
            high: std::sync::Arc::new(vec![105.0; len]),
            low: std::sync::Arc::new(vec![95.0; len]),
            close: std::sync::Arc::new(vec![100.0; len]),
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

        // 1. Cảm biến hoàn toàn rỗng mặc định (như khi stoch_range bị tắt trong init())
        let empty_sensors = HeliosSensors::default();
        let empty_matrix = MarketMatrixV3::default();
        let config = HeliosConfigV3::default();

        let scanner = RangeOscScanner;
        // Bắt buộc không được panic và trả về None an toàn
        let result = scanner.scan(&tick, &empty_sensors, &empty_matrix, &config);
        assert!(result.is_none(), "RangeOscScanner phải trả về None mà không panic khi cảm biến rỗng!");

        // 2. Cảm biến có BB nhưng stoch_range_k/d vẫn rỗng
        let mut partial_sensors = HeliosSensors::default();
        partial_sensors.bb_upper = std::sync::Arc::new(vec![105.0; len]);
        partial_sensors.bb_middle = std::sync::Arc::new(vec![100.0; len]);
        partial_sensors.bb_lower = std::sync::Arc::new(vec![95.0; len]);
        let mut partial_matrix = MarketMatrixV3::default();
        partial_matrix.bb_width_norm = std::sync::Arc::new(vec![0.10; len]);
        partial_matrix.trend_strength = std::sync::Arc::new(vec![0.02; len]);

        let result2 = scanner.scan(&tick, &partial_sensors, &partial_matrix, &config);
        assert!(result2.is_none(), "RangeOscScanner phải trả về None mà không panic khi stoch_range_k/d rỗng!");
    }

    #[test]
    #[should_panic(expected = "Critical indicator period is 0 in init()")]
    fn test_mod_init_panics_on_zero_indicator_period() {
        let len = 50;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![100.0; len]),
            high: std::sync::Arc::new(vec![105.0; len]),
            low: std::sync::Arc::new(vec![95.0; len]),
            close: std::sync::Arc::new(vec![100.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache.clone());
        let mut indicator_manager = IndicatorManager::new();
        let cache = ferro_trade::data::global_cache::GlobalCache::new(Default::default());
        let mut init_ctx = InitContext {
            raw_data: &raw_cache,
            data_view: &data_view,
            cache: &cache,
            indicator_manager: &mut indicator_manager,
        };

        let mut strategy = HeliosStrategyV3::new(&HashMap::new());
        strategy.config.indicators.natr.period = 0; // Kích hoạt lỗi period = 0
        strategy.init(&mut init_ctx);
    }

    #[test]
    fn test_mod_clean_exclusive_order_without_prev_open_trade() {
        let len = 20;
        let raw_cache = RawDataCache {
            datetime: std::sync::Arc::new(vec![0i64; len]),
            open: std::sync::Arc::new(vec![100.0; len]),
            high: std::sync::Arc::new(vec![105.0; len]),
            low: std::sync::Arc::new(vec![95.0; len]),
            close: std::sync::Arc::new(vec![100.0; len]),
            volume: std::sync::Arc::new(vec![1000.0; len]),
            custom_columns: HashMap::new(),
        };
        let data_view = DataView::new(raw_cache);
        let indicator_manager = IndicatorManager::new();
        let mut actions = SmallVec::new();
        use ferro_trade::broker::Trade;
        let trade = Trade::new(1, 1.0, 100.0, 0, None, Some("TREND_PB".to_string()));
        let trades = vec![trade];
        let mut pos = Position::default();
        pos.size = 1.0; // Vị thế đang mở
        let mut tick = ferro_trade::strategy::TickContext {
            i: 5,
            equity: 1000.0,
            data_view: &data_view,
            indicator_manager: &indicator_manager,
            trades: &trades,
            last_closed_trade: None,
            actions: &mut actions,
            position: pos,
        };

        let mut strategy = HeliosStrategyV3::new(&HashMap::new());
        strategy.sensors.atr_sl = std::sync::Arc::new(vec![1.0; len]);
        strategy.matrix.chaos_index = std::sync::Arc::new(vec![0.5; len]);

        strategy.next(&mut tick);
        // Khi đang có vị thế mở, next() lập tức return sau RiskManager, không đi vào Arbiter mở lệnh Buy/Sell mới
        let has_entry_order = actions.iter().any(|a| matches!(a, ferro_trade::broker::actions::Action::Buy { .. } | ferro_trade::broker::actions::Action::Sell { .. }));
        assert!(!has_entry_order, "Strict Exclusive Order phải chặn không cho mở thêm lệnh Buy/Sell khi vị thế đang mở!");
    }

    #[test]
    fn test_range_osc_revived_with_opposite_boundary_tp_rr() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::arbiter::{TacticalScanner, SignalType};

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2050.0; len];
        let mut lows = vec![1970.0; len];
        let mut closes = vec![2000.0; len];
        opens[i] = 1972.0;
        highs[i] = 1985.0;
        lows[i] = 1970.0;
        closes[i] = 1982.0;
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
        config.tactical_sniper.range.max_bb_slope = 0.10;
        config.tactical_sniper.range.max_adverse_slope = 0.15;
        config.tactical_sniper.range.min_activation_score = 0.10;
        config.tactical_sniper.range.min_width_pct = 0.005;
        config.tactical_sniper.range.bb_proximity_pct = 0.015;
        config.tactical_sniper.range.stoch_oversold = 25.0;
        config.tactical_sniper.range.stoch_overbought = 75.0;
        config.tactical_sniper.range.struct_sl_buffer_atr = 1.0;
        config.tactical_sniper.range.tp_target_ratio = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_k = std::sync::Arc::new(vec![15.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![12.0; len]); // Hook up
        sensors.vol_z_score = std::sync::Arc::new(vec![0.5; len]);
        sensors.bb_middle_slope_adj = std::sync::Arc::new(vec![0.01; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.bb_width_norm = std::sync::Arc::new(vec![0.02; len]);
        matrix.trend_strength = std::sync::Arc::new(vec![0.15; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);
        matrix.bb_middle_slope_adj = sensors.bb_middle_slope_adj.clone();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeOscScanner phải phê duyệt điểm vào Long!");
        let p = prop.unwrap();
        assert_eq!(p.signal_type, SignalType::Long);

        let tp = p.suggested_tp.expect("Phải có TP");
        let sl = p.suggested_sl.expect("Phải có SL");
        let entry = 1982.0;

        // Take Profit phải hướng sang biên trên VAH (2050.0), vượt xa trục giữa POC (2010.0)
        assert!(tp > 2010.0, "Take profit ({}) phải hướng sang biên trên, vượt xa trục giữa 2010.0!", tp);

        let risk = entry - sl;
        let reward = tp - entry;
        let rr_ratio = reward / risk;
        assert!(rr_ratio >= 1.50, "Tỷ lệ R:R ({:.2}) phải >= 1.50 để bảo đảm EV > 0 sau phí!", rr_ratio);
    }

    #[test]
    fn test_range_osc_spring_liquidity_sweep_approved() {
        use crate::strategy::core_v3::scanners::range_osc::RangeOscScanner;
        use crate::strategy::core_v3::arbiter::TacticalScanner;

        let len = 50;
        let i = len - 1;
        let mut opens = vec![2000.0; len];
        let mut highs = vec![2050.0; len];
        let mut lows = vec![1970.0; len];
        let mut closes = vec![2000.0; len];
        // Nến Spring quét râu ra ngoài biên dưới VAL (1970.0) nhưng đóng nến rút chân vào trong
        opens[i] = 1975.0;
        highs[i] = 1985.0;
        lows[i] = 1965.0;
        closes[i] = 1980.0;
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
        config.tactical_sniper.range.max_bb_slope = 0.10;
        config.tactical_sniper.range.max_adverse_slope = 0.15;
        config.tactical_sniper.range.min_activation_score = 0.10;
        config.tactical_sniper.range.min_width_pct = 0.005;
        config.tactical_sniper.range.bb_proximity_pct = 0.015;
        config.tactical_sniper.range.stoch_oversold = 25.0;
        config.tactical_sniper.range.stoch_overbought = 75.0;
        config.tactical_sniper.range.struct_sl_buffer_atr = 1.0;
        config.tactical_sniper.range.tp_target_ratio = 0.50;

        let mut sensors = HeliosSensors::default();
        sensors.bb_middle = std::sync::Arc::new(vec![2000.0; len]);
        sensors.bb_lower = std::sync::Arc::new(vec![1980.0; len]);
        sensors.bb_upper = std::sync::Arc::new(vec![2020.0; len]);
        sensors.atr_sl = std::sync::Arc::new(vec![10.0; len]);
        sensors.stoch_range_k = std::sync::Arc::new(vec![15.0; len]);
        sensors.stoch_range_d = std::sync::Arc::new(vec![12.0; len]);
        sensors.vol_z_score = std::sync::Arc::new(vec![0.5; len]);
        sensors.bb_middle_slope_adj = std::sync::Arc::new(vec![0.01; len]);

        let mut matrix = MarketMatrixV3::default();
        matrix.bb_width_norm = std::sync::Arc::new(vec![0.02; len]);
        matrix.trend_strength = std::sync::Arc::new(vec![0.15; len]);
        matrix.kama_slope_atr = std::sync::Arc::new(vec![0.0; len]);
        matrix.bb_middle_slope_adj = sensors.bb_middle_slope_adj.clone();

        let prop = RangeOscScanner.scan(&tick, &sensors, &matrix, &config);
        assert!(prop.is_some(), "RangeOscScanner phải phê duyệt cú rút chân Spring quét râu ngoài biên dưới!");
    }
}


