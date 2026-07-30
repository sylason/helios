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
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType};
use crate::strategy::core_v3::risk::RiskManager;

/// Chiến lược Helios V3 - Signal-First Architecture
#[derive(Clone, Debug)]
pub struct HeliosStrategyV3 {
    pub config: HeliosConfigV3,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrixV3,
    pub debug_mode: bool,
    pub cooldown_bars_remaining: usize,
    pub was_position_open: bool,
    pub equity_at_entry: f64,
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
            cooldown_bars_remaining: 0,
            was_position_open: false,
            equity_at_entry: 0.0,
        }
    }

    fn update_from_dna(&mut self, dna: &[f64]) {
        self.config.update_from_dna(dna);
    }

    fn init(&mut self, ctx: &mut InitContext) {
        let raw_closes = ctx.closes();
        let raw_volumes = ctx.volumes();
        let ind = &self.config.indicators;

        if ind.natr.period == 0 || ind.mfi.period == 0 || ind.vol_ma.period == 0 || 
           ind.kama.short_period == 0 || ind.kama.long_period == 0 ||
           ind.adx.period == 0 || ind.rsi.period == 0 || ind.bbands.period == 0 ||
           ind.macd.fast == 0 || ind.macd.slow == 0 || ind.atr.period == 0 {
            println!("!!! [SAFETY V3] CRITICAL PERIOD IS ZERO, ABORTING INIT !!!");
            return;
        }

        let mut s = HeliosSensors::default();
        
        s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
        s.chaos_ratio = ctx.chaos_ratio(ind.natr.timeframe, "ChaosRatio", ind.natr.period);
        s.vol_ma_series = ctx.volume_sma(ind.vol_ma.timeframe, "VOL_MA", ind.vol_ma.period);
        s.efficiency_ratio = ctx.efficiency_ratio(ind.kama.timeframe, "ER", ind.kama.short_period);

        let (mfi, _tp, _mf, _pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
        s.mfi = mfi;

        let (adx, _pdi, _mdi, _adxr) = ctx.adx(ind.adx.timeframe, "ADX", ind.adx.period);
        s.adx = adx;

        s.linreg_slope = ctx.linreg_slope(ind.linreg.timeframe, "Slope", ind.linreg.period);
        s.linreg_angle = ctx.linreg_angle(ind.linreg.timeframe, "Angle", ind.linreg.period);
        s.kama_mid = ctx.kama(ind.kama.timeframe, "KAMA", ind.kama.short_period, 2, ind.kama.long_period);

        let (bb_u, _bb_m, bb_l, _bb_w, _bb_wp, _bb_sw, _bb_sq) = ctx.bbands(ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
        s.bb_upper = bb_u;
        s.bb_lower = bb_l;

        let (kc_u, _kc_m, kc_l) = ctx.keltner(ind.keltner.timeframe, "KC", ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone());
        s.kc_upper = kc_u;
        s.kc_lower = kc_l;

        let (rsi, _rsi_gain, _rsi_loss) = ctx.rsi(ind.rsi.timeframe, "RSI", ind.rsi.period);
        s.rsi = rsi;

        let (macd_m, macd_s, macd_h, _macd_ma, _macd_sa) = ctx.macd(ind.macd.timeframe, "MACD", ind.macd.fast, ind.macd.slow, ind.macd.signal);
        s.macd_main = macd_m;
        s.macd_signal = macd_s;
        s.macd_hist = macd_h;

        let (st_k, st_d, st_cr, st_kz, st_dz) = ctx.stoch(
            ind.stochastic.timeframe, "STOCH", 
            ind.stochastic.period as u32, 
            ind.stochastic.k_period as u32, ind.stochastic.k_ma_type.clone(),
            ind.stochastic.d_period as u32, ind.stochastic.d_ma_type.clone(),
            ind.stochastic.zone
        );
        s.stoch_k = st_k;
        s.stoch_d = st_d;
        s.stoch_cross = st_cr;
        s.stoch_k_zone = st_kz;
        s.stoch_d_zone = st_dz;

        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);

        let vol_ratio = match ind.vol_ma.timeframe {
            ferro_trade::data::Timeframe::T5m => 5.0, 
            ferro_trade::data::Timeframe::T15m => 15.0, 
            ferro_trade::data::Timeframe::T1h => 60.0, 
            ferro_trade::data::Timeframe::T4h => 240.0, 
            _ => 1.0,
        };

        self.sensors = s;
        self.sensors.closes = raw_closes.clone();

        self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors, &raw_volumes, vol_ratio);
    }

    fn next(&mut self, tick: &TickContext) {
        let i = tick.i;
        if self.sensors.atr_sl.len() <= i || self.matrix.chaos_ratio.len() <= i {
            return;
        }

        let pos = tick.position();
        let is_pos_open = pos.is_open();

        // Đánh giá khi vừa đóng vị thế: Nếu thua lỗ thì bật Cooldown Recovery Period
        if self.was_position_open && !is_pos_open {
            if tick.equity < self.equity_at_entry {
                self.cooldown_bars_remaining = self.config.risk_management.stop_management.cooldown_duration_bars;
            }
        }
        self.was_position_open = is_pos_open;

        // 1. Quản lý Rủi ro / Trailing Stop (Chạy mỗi nến)
        RiskManager::manage_trailing_stop(
            &self.config,
            &self.matrix,
            tick,
            &self.sensors,
            self.debug_mode
        );

        // 2. Chặn đánh nhồi lệnh: Nếu đã có lệnh mở thì không dò tín hiệu mới
        if is_pos_open {
            return; 
        }

        // 3. Thời gian tạm dừng đếm ngược hồi phục sau lệnh thua (Cooldown Recovery Period)
        if self.cooldown_bars_remaining > 0 {
            self.cooldown_bars_remaining -= 1;
            return;
        }

        let mut arbiter = HeliosArbiter::new(self.config.min_approval_confidence); 
        
        arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
        arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
        arbiter.add_scanner(Box::new(scanners::range_osc::RangeOscScanner));
        arbiter.add_scanner(Box::new(scanners::reversal::ReversalScanner));
        arbiter.add_scanner(Box::new(scanners::anomaly::AnomalyScanner));
        arbiter.add_scanner(Box::new(scanners::absorption::AbsorptionScanner));

        let decision = arbiter.process_tick(tick, &self.sensors, &self.matrix, &self.config, self.debug_mode);

        if decision.is_approved {
            if let Some(signal) = decision.final_signal {
                // Tính Size (% Initial Margin)
                let size = RiskManager::calculate_size(
                    tick.equity, 
                    self.config.risk_management.risk_base.percent_per_trade, 
                    21.0, 
                    20.0
                );
                
                let is_long = signal == SignalType::Long;
                
                let tag_name = decision.winning_scanner.unwrap_or("");
                let playbook_max_sl_pct = match tag_name {
                    "TREND_PB" => self.config.tactical_sniper.pullback.max_initial_sl_pct,
                    "MOMENTUM_IGN" => self.config.tactical_sniper.momentum.max_initial_sl_pct,
                    "RANGE_OSC" => self.config.tactical_sniper.range.max_initial_sl_pct,
                    "REVERSAL" => self.config.tactical_sniper.reversal.max_initial_sl_pct,
                    "ANOMALY" | "ANOMALY_BREAKOUT" => self.config.tactical_sniper.anomaly.max_initial_sl_pct,
                    "ABSORPTION" => self.config.tactical_sniper.absorption.max_initial_sl_pct,
                    _ => 0.05,
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
                    )
                });
                
                // Bắt buộc áp dụng Hard Cap SL & Sàn SL tối thiểu của Playbook cho MỌI lệnh mới mở
                let price = tick.closes()[tick.i];
                let initial_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, playbook_max_sl_pct, min_initial_sl_pct);
                
                let tag = decision.winning_scanner.map(|s| s.to_string());

                let policy = Some(RiskPolicy::FixedPrice { 
                    sl_price: Some(initial_sl), 
                    tp_price: decision.final_tp 
                });

                self.equity_at_entry = tick.equity;

                if is_long {
                    tick.buy(Some(size), None, None, policy, tag);
                } else {
                    tick.sell(Some(size), None, None, policy, tag);
                }
            }
        }
    }
}
