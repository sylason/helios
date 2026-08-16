// strategies_backtests/helios/src/strategy/core_v3/config.rs

use ferro_trade::data::Timeframe;
use ferro_trade::indicators::identifier::MaType;
use ferro_trade::optimizer::{DnaMappable, DnaMappableMacro};
use serde::Deserialize;

/// CẤU TRÚC TỔNG THỂ (Root Config V3.3)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct HeliosConfigV3 {
    #[dna(nest)]
    pub indicators: IndicatorsConfig,
    #[dna(nest)]
    pub matrix: MatrixConfig,
    #[dna(nest)]
    pub arbiter: ArbiterConfig,
    #[dna(nest)]
    pub tactical_sniper: TacticalSniperConfig,
    #[dna(nest)]
    pub risk_management: RiskManagementConfig,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 8.0, max = 80.0, step = 3.0))]
pub struct IndicatorsConfig {
    #[dna(nest)]
    pub kama: KamaParams,
    #[dna(nest)]
    pub adx: AdxParams,

    // CẢM BIẾN RSI ĐỘC LẬP PER-PLAYBOOK
    #[dna(nest)]
    pub rsi_trend: RsiParams,
    #[dna(nest)]
    pub rsi_reversal: RsiParams,
    #[dna(nest)]
    pub rsi_momentum: RsiParams,
    #[dna(nest)]
    pub rsi_anomaly: RsiParams,

    #[dna(nest)]
    pub bbands: BbandsParams,
    #[dna(nest)]
    pub keltner: KeltnerParams,
    #[dna(nest)]
    pub macd: MacdParams,

    // CẢM BIẾN STOCHASTIC ĐỘC LẬP PER-PLAYBOOK
    #[dna(nest)]
    pub stoch_trend: StochasticParams,
    #[dna(nest)]
    pub stoch_range: StochasticParams,

    #[dna(nest)]
    pub atr: AtrParams,
    #[dna(nest)]
    pub natr: NAtrParams,
    #[dna(nest)]
    pub mfi: MfiParams,
    #[dna(nest)]
    pub vol_ma: VolMaParams,
    #[dna(nest)]
    pub vwma: VwmaParams,

    #[dna(min = 20.0, max = 600.0, step = 20.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub z_score_lookback: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KamaParams {
    #[dna(min = 2.0, max = 18.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub short_period: usize, 
    #[dna(min = 20.0, max = 80.0, step = 4.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub long_period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct AdxParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RsiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct BbandsParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub nbdev: f64,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KeltnerParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
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
    #[dna(min = 20.0, max = 80.0, step = 4.0)]
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
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
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
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct NAtrParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MfiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct VolMaParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct VwmaParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MatrixConfig {
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub trend_lookback: usize,
    
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub squeeze_lookback: usize,
    
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub momentum_lookback: usize,
    
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub volume_lookback: usize,
    
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub z_score_sensitivity: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct ArbiterConfig {
    #[dna(min = 0.00, max = 0.50, step = 0.02)]
    pub min_multiplier_veto: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub scanner_weight: f64,

    // 3 DẢI NGƯỠNG DUYỆT ĐỘNG THEO BỐI CẢNH REGIME
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub min_confidence_trending: f64,
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub min_confidence_ranging: f64,
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub min_confidence_volatile: f64,
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
    #[dna(min = 0.5, max = 3.6, step = 0.1)]
    pub break_confirm_mult: f64,
    #[dna(min = 1.00, max = 1.15, step = 0.005)]
    pub vol_expansion_thresh: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub volume_confirm_thresh: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 30.0, max = 70.0, step = 2.0)]
    pub rsi_buy_thresh: f64,
    #[dna(min = 20.0, max = 60.0, step = 2.0)]
    pub stoch_pullback_thresh: f64,
    #[dna(min = 0.02, max = 0.30, step = 0.02)]
    pub kama_slope_threshold: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 55.0, max = 80.0, step = 2.0)]
    pub rsi_threshold: f64,
    #[dna(min = 0.20, max = 0.80, step = 0.05)]
    pub vol_z_threshold: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.001, max = 0.015, step = 0.001)]
    pub bb_proximity_pct: f64,
    #[dna(min = 10.0, max = 40.0, step = 2.0)]
    pub stoch_oversold: f64,
    #[dna(min = 60.0, max = 90.0, step = 2.0)]
    pub stoch_overbought: f64,
    #[dna(min = 0.005, max = 0.050, step = 0.002)]
    pub min_width_pct: f64,
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub volume_max_threshold: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 1.2, max = 3.5, step = 0.1)]
    pub z_score_threshold: f64,
    #[dna(min = 0.8, max = 3.0, step = 0.1)]
    pub min_reversal_displacement: f64,
    #[dna(min = 0.10, max = 0.60, step = 0.05)]
    pub exhaustion_vol_drop: f64,
    #[dna(min = 0.10, max = 2.00, step = 0.10)]
    pub wick_sl_buffer_atr: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.5, max = 2.5, step = 0.1)]
    pub wick_atr_threshold: f64,
    #[dna(min = 1.0, max = 3.0, step = 0.1)]
    pub z_score_threshold: f64,
    #[dna(min = 0.10, max = 1.50, step = 0.10)]
    pub wick_sl_buffer_atr: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.20, max = 0.70, step = 0.05)]
    pub squeeze_threshold: f64,
    #[dna(min = 0.20, max = 0.80, step = 0.05)]
    pub vol_z_threshold: f64,
    #[dna(min = 0.25, max = 0.75, step = 0.05)]
    pub min_activation_score: f64,
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
    #[dna(min = 0.001, max = 0.01, step = 0.001)]
    pub sl_min_dist_pct: f64,
    #[dna(min = 0.1, max = 1.5, step = 0.1)]
    pub base_lot_size: f64,
    #[dna(min = 1.0, max = 50.0, step = 1.0)]
    pub leverage: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    #[dna(min = 0.002, max = 0.017, step = 0.0005)]
    pub min_initial_sl_pct: f64,

    #[dna(min = 60.0, max = 1220.0, step = 40.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub max_stagnant_hold_bars: usize,

    #[dna(min = 0.005, max = 0.155, step = 0.005)]
    pub stagnant_min_profit_pct: f64,

    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub base_sl_atr: f64,
    
    #[dna(min = 30.0, max = 1230.0, step = 40.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_countdown_bars: usize,
    
    #[dna(min = 0.50, max = 0.95, step = 0.025)]
    pub tighten_step_multiplier: f64,
    
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_tight: f64,
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_normal: f64,
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_loose: f64,
    #[dna(min = 0.02, max = 0.15, step = 0.01)]
    pub mega_trend_pnl_trigger: f64,
    #[dna(min = 0.50, max = 0.95, step = 0.025)]
    pub squeeze_pause_threshold: f64,
    #[dna(min = 5.0, max = 50.0, step = 2.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub mega_trend_max_cycles: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
