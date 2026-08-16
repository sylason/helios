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

use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType, ScannerId};
use crate::strategy::core_v3::risk::RiskManager;

/// Chiến lược Helios V3.3 - Clean Quant Architecture
#[derive(Clone, Debug)]
pub struct HeliosStrategyV3 {
    pub config: HeliosConfigV3,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrixV3,
    pub debug_mode: bool,
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
        }
    }

    fn update_from_dna(&mut self, dna: &[f64]) {
        self.config.update_from_dna(dna);
    }

    fn init(&mut self, ctx: &mut InitContext) {
        let ind = &self.config.indicators;

        if ind.natr.period == 0 || ind.mfi.period == 0 || ind.vol_ma.period == 0 || 
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

    fn next(&mut self, tick: &TickContext) {
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

        // 2. Strict Exclusive Order: Nếu đang có vị thế mở thì không mở lệnh mới
        if is_pos_open {
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
                let leverage = self.config.risk_management.risk_base.leverage.max(1.0);
                
                let size = RiskManager::calculate_size(
                    tick.equity, 
                    self.config.risk_management.risk_base.percent_per_trade, 
                    21.0, 
                    leverage
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
                        self.config.risk_management.stop_management.tsl_atr_normal,
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
