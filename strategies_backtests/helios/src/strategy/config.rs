// strategies_backtests/helios/src/strategy/config.rs

use ferro_trade::data::Timeframe;
use ferro_trade::indicators::identifier::MaType;
use ferro_trade::optimizer::{DnaMappable, DnaMappableMacro};
use serde::Deserialize;

/// [TASK 1.1] BẢN ĐỒ GENE CỐ ĐỊNH (FIXED DNA ORDER)
/// Hệ thống hiện đã sử dụng Macro `DnaMappable` để tự động hóa việc ánh xạ.
/// Metadata (min/max/step) được khai báo trực tiếp tại từng trường.

/// CẤU TRÚC TỔNG THỂ (Root Config)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct HeliosConfig {
    #[dna(nest)]
    pub indicators: IndicatorsConfig,
    #[dna(nest)]
    pub matrix: MatrixConfig,
    #[dna(nest)]
    pub environment_fuse: EnvironmentFuseConfig,
    #[dna(nest)]
    pub market_regime: MarketRegimeConfig,
    #[dna(nest)]
    pub tactical_sniper: TacticalSniperConfig,
    #[dna(nest)]
    pub risk_management: RiskManagementConfig,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))] // Default chung cho Indicators
pub struct IndicatorsConfig {
    #[dna(nest)]
    pub kama: KamaParams,
    #[dna(nest)]
    pub adx: AdxParams,
    #[dna(nest)]
    pub rsi: RsiParams,
    #[dna(nest)]
    pub bbands: BbandsParams,
    #[dna(nest)]
    pub keltner: KeltnerParams,
    #[dna(nest)]
    pub macd: MacdParams,
    #[dna(nest)]
    pub stochastic: StochasticParams,
    #[dna(nest)]
    pub atr: AtrParams,
    #[dna(nest)]
    pub natr: NAtrParams,
    #[dna(nest)]
    pub mfi: MfiParams,
    #[dna(nest)]
    pub vol_ma: VolMaParams,
    #[dna(nest)]
    pub linreg: LinRegParams,

    #[dna(min = 20.0, max = 500.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub z_score_lookback: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KamaParams {
    #[dna(min = 2.0, max = 18.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub short_period: usize, 
    #[dna(min = 20.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub long_period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct AdxParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RsiParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct BbandsParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub nbdev: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KeltnerParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub multiplier: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub ma_type: MaType,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MacdParams {
    #[dna(min = 2.0, max = 18.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub fast: usize, 
    #[dna(min = 20.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub slow: usize,
    #[dna(min = 2.0, max = 20.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub signal: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct StochasticParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 2.0, max = 20.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub k_period: usize,
    #[dna(min = 2.0, max = 20.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub d_period: usize,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub k_ma_type: MaType,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub d_ma_type: MaType,
    #[dna(min = 0.05, max = 0.45, step = 0.1)]
    pub zone: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct AtrParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct NAtrParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MfiParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct VolMaParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct LinRegParams {
    #[dna(min = 10.0, max = 80.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MatrixConfig {
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub trend_lookback: usize,
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub squeeze_lookback: usize,
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub momentum_lookback: usize,
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub volume_lookback: usize,
    #[dna(min = 1.0, max = 10.0, step = 1.0)]
    pub z_score_sensitivity: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct EnvironmentFuseConfig {
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub er_threshold: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub chaos_z_threshold: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub min_liquidity_ratio: f64,
    #[dna(min = 20.0, max = 200.0, step = 20.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub min_orderly_bars: usize,
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub percentile_lookback: usize,
    #[dna(min = 100.0, max = 1500.0, step = 200.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub min_data_points: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MarketRegimeConfig {
    #[dna(nest)]
    pub squeeze: RegimeSqueezeParams,
    #[dna(nest)]
    pub trend: RegimeTrendParams,
    #[dna(nest)]
    pub phase: RegimePhaseParams,
    #[dna(nest)]
    pub absorption: RegimeAbsorptionParams,
    #[dna(nest)]
    pub anomaly: RegimeAnomalyParams,
    #[dna(nest)]
    pub parabolic: RegimeParabolicParams,
    #[dna(nest)]
    pub confidence: RegimeConfidenceParams,
    #[dna(nest)]
    pub priority: RegimePriorityParams,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeSqueezeParams {
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeTrendParams {
    #[dna(min = 0.2, max = 0.6, step = 0.1)]
    pub adx_strong: f64,
    #[dna(min = 0.3, max = 0.7, step = 0.1)]
    pub adx_moderate: f64,
    #[dna(min = 15.0, max = 55.0, step = 10.0)]
    pub angle_healthy: f64,
    #[dna(min = 60.0, max = 80.0, step = 10.0)]
    pub angle_parabolic: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimePhaseParams {
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub accel_min: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub accel_max: f64,
    #[dna(min = 0.85, max = 0.95, step = 0.05)]
    pub climax: f64,
    #[dna(min = 1.5, max = 9.5, step = 1.0)]
    pub spike_mult: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeAbsorptionParams {
    #[dna(min = 0.5, max = 0.9, step = 0.1)]
    pub volume_threshold: f64,
    #[dna(min = 0.1, max = 0.4, step = 0.1)]
    pub range_width_max: f64,
    #[dna(min = 0.55, max = 0.95, step = 0.1)]
    pub bias_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeAnomalyParams {
    #[dna(min = 0.5, max = 0.9, step = 0.1)]
    pub z_score_threshold: f64,
    #[dna(min = 1.5, max = 5.5, step = 1.0)]
    pub natr_spike_mult: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeParabolicParams {
    #[dna(min = 50.0, max = 80.0, step = 10.0)]
    pub angle_threshold: f64,
    #[dna(min = 0.8, max = 1.0, step = 0.1)]
    pub accel_angle_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeConfidenceParams {
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub min_threshold: f64,
    #[dna(min = 0.5, max = 4.5, step = 1.0)]
    pub weight_angle: f64,
    #[dna(min = 0.5, max = 4.5, step = 1.0)]
    pub weight_adx: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimePriorityParams {
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub min_score_threshold: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalSniperConfig {
    #[dna(nest)]
    pub squeeze: TacticalSqueezeParams,
    #[dna(nest)]
    pub pullback: TacticalPullbackParams,
    #[dna(nest)]
    pub momentum: TacticalMomentumParams,
    #[dna(nest)]
    pub range: TacticalRangeParams,
    #[dna(nest)]
    pub reversal: TacticalReversalParams,
    #[dna(nest)]
    pub anomaly: TacticalAnomalyParams,
    #[dna(nest)]
    pub absorption: TacticalAbsorptionParams,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalSqueezeParams {
    #[dna(min = 0.5, max = 2.5, step = 0.5)]
    pub break_confirm_mult: f64,
    #[dna(min = 1.01, max = 1.05, step = 0.02)]
    pub vol_expansion_thresh: f64,
    #[dna(min = 0.5, max = 0.9, step = 0.1)]
    pub volume_confirm_thresh: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    #[dna(min = 0.1, max = 0.5, step = 0.1)]
    pub rsi_buy_thresh: f64,
    #[dna(min = 0.005, max = 0.02, step = 0.005)]
    pub kama_tolerance_pct: f64,
    #[dna(min = 0.05, max = 0.45, step = 0.1)]
    pub stoch_pullback_thresh: f64,
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    #[dna(min = 0.6, max = 0.8, step = 0.1)]
    pub rsi_threshold: f64,
    #[dna(min = 0.86, max = 0.96, step = 0.05)]
    pub rsi_climax: f64,
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub volume_spike_min: f64,
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    #[dna(min = 1.1, max = 2.1, step = 0.2)]
    pub accel_factor: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    #[dna(min = 0.05, max = 0.35, step = 0.1)]
    pub stoch_oversold: f64,
    #[dna(min = 0.65, max = 0.95, step = 0.1)]
    pub stoch_overbought: f64,
    #[dna(min = 0.01, max = 0.05, step = 0.01)]
    pub min_width_pct: f64,
    #[dna(min = 0.1, max = 0.6, step = 0.1)]
    pub volume_max_threshold: f64,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub macd_lookback: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    #[dna(min = 0.86, max = 0.96, step = 0.05)]
    pub rsi_extreme: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub exhaustion_vol_drop: f64,
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    #[dna(min = 0.86, max = 0.96, step = 0.05)]
    pub rsi_climax: f64,
    #[dna(min = 0.1, max = 0.7, step = 0.2)]
    pub min_rejection_pct: f64,
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub vol_min_threshold: f64,
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskManagementConfig {
    #[dna(nest)]
    pub risk_base: RiskBaseParams,
    #[dna(nest)]
    pub stop_management: RiskStopParams,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskBaseParams {
    #[dna(min = 0.01, max = 0.15, step = 0.02)]
    pub percent_per_trade: f64,
    #[dna(min = 0.001, max = 0.01, step = 0.002)]
    pub sl_min_dist_pct: f64,
    #[dna(min = 0.0, max = 1.0, step = 1.0)]
    pub use_breakeven: f64,
    #[dna(min = 0.5, max = 2.5, step = 0.5)]
    pub be_profit_target_rr: f64,
    #[dna(min = 0.1, max = 1.5, step = 0.3)]
    pub base_lot_size: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    #[dna(min = 1.0, max = 15.0, step = 2.0)]
    pub base_sl_atr: f64,
    #[dna(min = 2.0, max = 20.0, step = 2.0)]
    pub base_tp_rr: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub squeeze_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub trend_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub momentum_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub range_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub reversal_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub anomaly_sl_factor: f64,
    #[dna(min = 0.5, max = 3.0, step = 0.5)]
    pub absorption_sl_factor: f64,
    #[dna(min = 200.0, max = 5000.0, step = 200.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_decay_bars: usize,
    #[dna(min = 0.1, max = 2.0, step = 0.2)]
    pub tsl_atr_tight: f64,
    #[dna(min = 1.0, max = 4.5, step = 0.5)]
    pub tsl_atr_normal: f64,
    #[dna(min = 2.0, max = 15.0, step = 1.0)]
    pub tsl_atr_loose: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
