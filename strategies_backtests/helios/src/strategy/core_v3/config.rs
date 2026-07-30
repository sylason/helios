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
pub struct HeliosConfigV3 {
    #[dna(nest)]
    pub indicators: IndicatorsConfig,
    #[dna(nest)]
    pub matrix: MatrixConfig,
    #[dna(nest)]
    pub matrix_weights: MatrixWeightsConfig,
    #[dna(nest)]
    pub arbiter: ArbiterConfig,
    #[dna(nest)]
    pub tactical_sniper: TacticalSniperConfig,
    #[dna(nest)]
    pub risk_management: RiskManagementConfig,
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub min_approval_confidence: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
#[dna(default(min = 8.0, max = 80.0, step = 3.0))] // Default 25 items
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
pub struct LinRegParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "3m, 5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MatrixConfig {
    /// Số kỳ nhìn lại (lookback) để tính toán sức mạnh xu hướng (Trend Strength & Direction)
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub trend_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán độ nén của biến động (Squeeze Intensity)
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub squeeze_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán xung lượng (Momentum Percentile)
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub momentum_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán sức ép khối lượng (Volume Force Percentile)
    #[dna(min = 100.0, max = 1000.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub volume_lookback: usize,
    
    /// Độ nhạy của hàm Tanh khi chuẩn hóa Z-Score của giá
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub z_score_sensitivity: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct ArbiterConfig {
    #[dna(min = 0.00, max = 0.50, step = 0.02)]
    pub min_multiplier_veto: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub scanner_weight: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MatrixWeightsConfig {
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub anomaly_z_weight: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub reversal_momentum_weight: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub absorption_squeeze_weight: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub vol_exp_chaos_weight: f64,
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
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Trend Pullback
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub rsi_buy_thresh: f64,
    #[dna(min = 0.005, max = 0.035, step = 0.001)]
    pub kama_tolerance_pct: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub stoch_pullback_thresh: f64,
    #[dna(min = 5.0, max = 65.0, step = 2.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.00, max = 0.60, step = 0.02)]
    pub trend_dir_threshold: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub stoch_vs_kama_weight: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Momentum Ignition
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub rsi_threshold: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub rsi_climax: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub volume_spike_min: f64,
    #[dna(min = 5.0, max = 65.0, step = 2.0)]
    pub macd_lookback: f64,
    #[dna(min = 1.1, max = 2.6, step = 0.05)]
    pub accel_factor: f64,
    #[dna(min = 2.0, max = 8.0, step = 0.2)]
    pub vol_max_multiplier: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub vol_vs_break_weight: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Range Oscillator
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.05, max = 0.45, step = 0.015)]
    pub stoch_oversold: f64,
    #[dna(min = 0.55, max = 0.95, step = 0.015)]
    pub stoch_overbought: f64,
    #[dna(min = 0.005, max = 0.05, step = 0.0015)]
    pub min_width_pct: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub volume_max_threshold: f64,
    #[dna(min = 0.02, max = 0.32, step = 0.01)]
    pub range_bb_tolerance_pct: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub bb_vs_stoch_weight: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Reversal
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// Bội số ATR độ lệch trung vị tối thiểu để nổ lệnh Reversal (tránh nổ lệnh quá gần kama_mid)
    #[dna(min = 0.5, max = 3.5, step = 0.1)]
    pub min_reversal_displacement: f64,

    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub rsi_extreme: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub exhaustion_vol_drop: f64,
    #[dna(min = 5.0, max = 65.0, step = 2.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub z_score_threshold: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub z_vs_rsi_weight: f64,
    #[dna(min = 0.10, max = 2.20, step = 0.07)]
    pub wick_sl_buffer_atr: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Anomaly Flash Sweeps
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub rsi_climax: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub min_rejection_pct: f64,
    #[dna(min = 5.0, max = 65.0, step = 2.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub z_score_threshold: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub z_vs_rsi_weight: f64,
    #[dna(min = 0.10, max = 2.20, step = 0.07)]
    pub wick_sl_buffer_atr: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub pattern_vs_wick_weight: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    /// Phần trăm cắt lỗ ban đầu tối đa (Hard Max Cap %) cho Absorption
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub vol_min_threshold: f64,
    #[dna(min = 5.0, max = 65.0, step = 2.0)]
    pub macd_lookback: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub vol_vs_squeeze_weight: f64,
    #[dna(min = 0.05, max = 0.95, step = 0.03)]
    pub min_activation_score: f64,
    #[dna(min = 0.50, max = 0.95, step = 0.025)]
    pub trend_filter_threshold: f64,
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
    /// Phần trăm tổng vốn cược cho mỗi lệnh (1.0 = 100%)
    #[dna(min = 0.01, max = 0.15, step = 0.01)]
    pub percent_per_trade: f64,
    /// Khoảng cách Stoploss tối thiểu (%) để tránh bị dính spread/trượt giá
    #[dna(min = 0.001, max = 0.01, step = 0.001)]
    pub sl_min_dist_pct: f64,
    /// Quy mô lô cơ bản (Base Lot Size) - Dùng khi Trade kích thước cố định thay vì %
    #[dna(min = 0.1, max = 1.5, step = 0.1)]
    pub base_lot_size: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    /// Sàn cắt lỗ ban đầu tối thiểu (%) để chống trượt giá lệnh micro
    #[dna(min = 0.002, max = 0.017, step = 0.0005)]
    pub min_initial_sl_pct: f64,

    /// Thời gian (số nến) ngâm tối đa cho phép trước khi đánh giá lợi nhuận
    #[dna(min = 60.0, max = 1220.0, step = 40.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub max_stagnant_hold_bars: usize,

    /// Ngưỡng PnL tối thiểu (%) bắt buộc phải đạt được nếu ngâm quá max_stagnant_hold_bars
    #[dna(min = 0.005, max = 0.155, step = 0.005)]
    pub stagnant_min_profit_pct: f64,

    /// Hệ số ATR cho Stoploss cơ bản
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub base_sl_atr: f64,
    
    /// Thời gian (số nến) đếm ngược để đánh giá lệnh có phá đỉnh hay không (High-watermark)
    #[dna(min = 30.0, max = 1230.0, step = 40.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_countdown_bars: usize,
    
    /// Hệ số siết chặt SL sau mỗi chu kỳ đếm ngược
    #[dna(min = 0.50, max = 0.95, step = 0.025)]
    pub tighten_step_multiplier: f64,
    
    /// Hệ số Trailing Stop thắt chặt (Dùng khi Sideway hoặc lệnh ngâm quá lâu)
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_tight: f64,
    /// Hệ số Trailing Stop tiêu chuẩn
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_normal: f64,
    /// Hệ số Trailing Stop nới lỏng (Dùng khi Bão tố - Volatility bùng nổ)
    #[dna(min = 1.0, max = 10.0, step = 0.3)]
    pub tsl_atr_loose: f64,
    /// Mức PnL tối thiểu để kích hoạt chế độ gồng Siêu Trend
    #[dna(min = 0.02, max = 0.15, step = 0.01)]
    pub mega_trend_pnl_trigger: f64,
    /// Ngưỡng Squeeze intensity để đóng băng đếm ngược Time Decay
    #[dna(min = 0.50, max = 0.95, step = 0.025)]
    pub squeeze_pause_threshold: f64,
    /// Thời gian (số nến) tạm dừng scan tín hiệu mới sau khi vừa thua lệnh
    #[dna(min = 0.0, max = 500.0, step = 20.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub cooldown_duration_bars: usize,
    /// Số chu kỳ decay tối đa duy trì chế độ gồng Siêu Trend
    #[dna(min = 5.0, max = 50.0, step = 2.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub mega_trend_max_cycles: usize,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
