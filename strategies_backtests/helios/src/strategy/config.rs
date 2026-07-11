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

    #[dna(min = 10.0, max = 150.0, step = 10.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub z_score_lookback: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
// #[dna(default(min = 2, max = 44, step = 3))]
pub struct KamaParams {
    #[dna(min = 2.0, max = 44.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub short_period: usize, 
    #[dna(min = 20.0, max = 90.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub long_period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct AdxParams {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct RsiParams {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct BbandsParams {
    #[dna(min = 5.0, max = 75.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.2, step = 0.3)]
    pub nbdev: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KeltnerParams {
    #[dna(min = 5.0, max = 75.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.2, step = 0.3)]
    pub multiplier: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub ma_type: MaType,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct MacdParams {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub fast: usize, 
    #[dna(min = 50.0, max = 120.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub slow: usize,
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub signal: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct StochasticParams {
    #[dna(min = 5.0, max = 47.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 2.0, max = 30.0, step = 2.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub k_period: usize,
    #[dna(min = 2.0, max = 30.0, step = 2.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub d_period: usize,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub k_ma_type: MaType,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub d_ma_type: MaType,
    #[dna(min = 0.05, max = 0.45, step = 0.03)]
    pub zone: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct AtrParams {
    #[dna(min = 5.0, max = 75.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct NAtrParams {
    #[dna(min = 5.0, max = 75.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 2.0, max = 44.0, step = 3.0))]
pub struct MfiParams {
    #[dna(min = 5.0, max = 75.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 10.0, max = 150.0, step = 10.0))]
pub struct VolMaParams {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 5.0, max = 145.0, step = 10.0))]
pub struct LinRegParams {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 20.0, max = 500.0, step = 20.0))]
pub struct MatrixConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub trend_lookback: usize,
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub squeeze_lookback: usize,
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub momentum_lookback: usize,
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub volume_lookback: usize,
    #[dna(min = 0.5, max = 5.0, step = 0.5)]
    pub z_score_sensitivity: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct EnvironmentFuseConfig {
    #[dna(min = 0.1, max = 0.9, step = 0.05)]
    pub er_threshold: f64,
    #[dna(min = 0.005, max = 0.065, step = 0.003)]
    pub chaos_z_threshold: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.05)]
    pub min_liquidity_ratio: f64,
    #[dna(min = 5.0, max = 65.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub min_orderly_bars: usize,
    #[dna(min = 20.0, max = 300.0, step = 15.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub percentile_lookback: usize,
    #[dna(min = 50.0, max = 500.0, step = 25.0)]
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
    #[dna(min = 0.1, max = 0.95, step = 0.05)]
    pub threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeTrendParams {
    #[dna(min = 0.2, max = 0.65, step = 0.03)]
    pub adx_strong: f64,
    #[dna(min = 0.05, max = 0.35, step = 0.02)]
    pub adx_moderate: f64,
    #[dna(min = 15.0, max = 55.0, step = 2.5)]
    pub angle_healthy: f64,
    #[dna(min = 60.0, max = 85.0, step = 1.5)]
    pub angle_parabolic: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimePhaseParams {
    #[dna(min = 0.1, max = 0.9, step = 0.05)]
    pub accel_min: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.05)]
    pub accel_max: f64,
    #[dna(min = 0.85, max = 0.99, step = 0.01)]
    pub climax: f64,
    #[dna(min = 1.5, max = 9.0, step = 0.5)]
    pub spike_mult: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeAbsorptionParams {
    #[dna(min = 0.5, max = 0.99, step = 0.03)]
    pub volume_threshold: f64,
    #[dna(min = 0.1, max = 0.4, step = 0.05)]
    pub range_width_max: f64,
    #[dna(min = 0.55, max = 0.95, step = 0.02)]
    pub bias_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeAnomalyParams {
    #[dna(min = 0.5, max = 0.95, step = 0.03)]
    pub z_score_threshold: f64,
    #[dna(min = 1.25, max = 5.0, step = 0.25)]
    pub natr_spike_mult: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeParabolicParams {
    #[dna(min = 50.0, max = 85.0, step = 2.0)]
    pub angle_threshold: f64,
    #[dna(min = 0.8, max = 1.0, step = 0.02)]
    pub accel_angle_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimeConfidenceParams {
    #[dna(min = 0.1, max = 0.9, step = 0.05)]
    pub min_threshold: f64,
    #[dna(min = 0.5, max = 4.5, step = 0.25)]
    pub weight_angle: f64,
    #[dna(min = 0.5, max = 4.5, step = 0.25)]
    pub weight_adx: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RegimePriorityParams {
    #[dna(min = 0.1, max = 0.95, step = 0.05)]
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
    #[dna(min = 0.1, max = 2.5, step = 0.15)]
    pub break_confirm_mult: f64,
    #[dna(min = 1.005, max = 1.05, step = 0.005)]
    pub vol_expansion_thresh: f64,
    #[dna(min = 0.5, max = 0.99, step = 0.03)]
    pub volume_confirm_thresh: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    #[dna(min = 0.1, max = 0.5, step = 0.03)]
    pub rsi_buy_thresh: f64,
    #[dna(min = 0.001, max = 0.02, step = 0.0015)]
    pub kama_tolerance_pct: f64,
    #[dna(min = 0.05, max = 0.45, step = 0.03)]
    pub stoch_pullback_thresh: f64,
    #[dna(min = 1.0, max = 15.0, step = 1.0)]
    pub macd_lookback: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    #[dna(min = 0.6, max = 0.85, step = 0.02)]
    pub rsi_threshold: f64,
    #[dna(min = 0.86, max = 0.99, step = 0.01)]
    pub rsi_climax: f64,
    #[dna(min = 0.6, max = 0.99, step = 0.03)]
    pub volume_spike_min: f64,
    #[dna(min = 1.0, max = 15.0, step = 1.0)]
    pub macd_lookback: f64,
    #[dna(min = 1.05, max = 2.25, step = 0.1)]
    pub accel_factor: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    #[dna(min = 0.05, max = 0.35, step = 0.02)]
    pub stoch_oversold: f64,
    #[dna(min = 0.65, max = 0.95, step = 0.02)]
    pub stoch_overbought: f64,
    #[dna(min = 0.005, max = 0.05, step = 0.003)]
    pub min_width_pct: f64,
    #[dna(min = 0.1, max = 0.65, step = 0.04)]
    pub volume_max_threshold: f64,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub macd_lookback: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    #[dna(min = 0.86, max = 0.99, step = 0.01)]
    pub rsi_extreme: f64,
    #[dna(min = 0.1, max = 0.9, step = 0.05)]
    pub exhaustion_vol_drop: f64,
    #[dna(min = 1.0, max = 15.0, step = 1.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.6, max = 0.99, step = 0.03)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    #[dna(min = 0.86, max = 0.99, step = 0.01)]
    pub rsi_climax: f64,
    #[dna(min = 0.1, max = 0.7, step = 0.04)]
    pub min_rejection_pct: f64,
    #[dna(min = 1.0, max = 15.0, step = 1.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.6, max = 0.99, step = 0.03)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    #[dna(min = 0.6, max = 0.99, step = 0.03)]
    pub vol_min_threshold: f64,
    #[dna(min = 1.0, max = 15.0, step = 1.0)]
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
    #[dna(min = 0.01, max = 0.15, step = 0.01)]
    pub percent_per_trade: f64,
    #[dna(min = 0.001, max = 0.015, step = 0.001)]
    pub sl_min_dist_pct: f64,
    #[dna(min = 0.0, max = 1.0, step = 1.0)]
    pub use_breakeven: f64,
    #[dna(min = 0.5, max = 7.5, step = 0.5)]
    pub be_profit_target_rr: f64,
    #[dna(min = 0.1, max = 1.5, step = 0.1)]
    pub base_lot_size: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    #[dna(min = 0.5, max = 7.5, step = 0.5)]
    pub base_sl_atr: f64,
    #[dna(min = 1.0, max = 11.5, step = 0.75)]
    pub base_tp_rr: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub squeeze_sl_factor: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub trend_sl_factor: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub momentum_sl_factor: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub range_sl_factor: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub reversal_sl_factor: f64,
    #[dna(min = 0.2, max = 3.0, step = 0.2)]
    pub anomaly_sl_factor: f64,
    #[dna(min = 0.2, max = 1.6, step = 0.1)]
    pub tsl_atr_tight: f64,
    #[dna(min = 1.0, max = 4.5, step = 0.25)]
    pub tsl_atr_normal: f64,
    #[dna(min = 3.0, max = 10.0, step = 0.5)]
    pub tsl_atr_loose: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
