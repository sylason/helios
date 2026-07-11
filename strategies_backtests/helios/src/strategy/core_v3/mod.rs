// strategies_backtests/helios/src/strategy/core_v3/mod.rs

pub mod arbiter;
pub mod matrix;
pub mod scanners;
pub mod risk;

use std::collections::HashMap;

use ferro_trade::optimizer::params::ParamValue;
use ferro_trade::optimizer::DnaMappable;
use ferro_trade::strategy::{InitContext, Strategy, TickContext};
use ferro_trade::indicators::factory::BulkIndicatorFactory;

use crate::strategy::config::HeliosConfig;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{HeliosArbiter, SignalType};
use crate::strategy::core_v3::risk::RiskManager;

/// Chiến lược Helios V3 - Signal-First Architecture
#[derive(Clone, Debug)]
pub struct HeliosStrategyV3 {
    pub config: HeliosConfig,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrixV3,
    pub debug_mode: bool,
}

impl Strategy for HeliosStrategyV3 {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfig::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrixV3::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
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

        // 1. Quản lý Rủi ro / Trailing Stop (Chạy mỗi nến)
        RiskManager::manage_trailing_stop(
            &self.config,
            &self.matrix,
            tick,
            &self.sensors,
            self.debug_mode
        );

        let pos = tick.position();
        
        // 2. Chặn đánh nhồi lệnh: Nếu đã có lệnh mở thì không dò tín hiệu mới
        if pos.is_open() {
            return; 
        }

        let mut arbiter = HeliosArbiter::new(self.config.market_regime.priority.min_score_threshold); 
        
        arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
        arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
        arbiter.add_scanner(Box::new(scanners::range_osc::RangeOscScanner));

        let decision = arbiter.process_tick(tick, &self.sensors, &self.matrix, self.debug_mode);

        if decision.is_approved {
            if let Some(signal) = decision.final_signal {
                let _price = tick.closes()[i];
                
                // Tính Size (% Initial Margin)
                // Đảm bảo thỏa mãn Min Order Size (Binance: ~21 USDT với đòn bẩy 20x)
                let _size = RiskManager::calculate_size(tick.equity, 0.10, 21.0, 20.0);
                
                let is_long = signal == SignalType::Long;
                let initial_sl = RiskManager::get_initial_sl(
                    tick,
                    &self.sensors,
                    is_long,
                    self.config.risk_management.stop_management.tsl_atr_normal
                );

                if is_long {
                    // tick.buy(Some(size), None, Some(initial_sl), None, None);
                    tick.buy(Some(0.2), None, Some(initial_sl), None, None);
                } else {
                    // tick.sell(Some(size), None, Some(initial_sl), None, None);
                    tick.sell(Some(0.2), None, Some(initial_sl), None, None);
                }
            }
        }
    }
}
