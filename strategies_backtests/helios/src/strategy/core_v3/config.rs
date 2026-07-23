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
    pub tactical_sniper: TacticalSniperConfig,
    #[dna(nest)]
    pub risk_management: RiskManagementConfig,
    #[dna(min = 0.3, max = 0.9, step = 0.1)]
    pub min_approval_confidence: f64,
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
    /// Số kỳ nhìn lại (lookback) để tính toán sức mạnh xu hướng (Trend Strength & Direction)
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub trend_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán độ nén của biến động (Squeeze Intensity)
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub squeeze_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán xung lượng (Momentum Percentile)
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub momentum_lookback: usize,
    
    /// Số kỳ nhìn lại để tính toán sức ép khối lượng (Volume Force Percentile)
    #[dna(min = 100.0, max = 1000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub volume_lookback: usize,
    
    /// Độ nhạy của hàm Tanh khi chuẩn hóa Z-Score của giá
    #[dna(min = 1.0, max = 10.0, step = 1.0)]
    pub z_score_sensitivity: f64,
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
    /// Bội số volume tối thiểu so với trung bình để xác nhận breakout khỏi vùng nén
    #[dna(min = 0.5, max = 2.5, step = 0.5)]
    pub break_confirm_mult: f64,
    /// Ngưỡng giãn nở biến động (Bollinger Band chui ra khỏi Keltner Channel)
    #[dna(min = 1.01, max = 1.05, step = 0.02)]
    pub vol_expansion_thresh: f64,
    /// Ngưỡng xác nhận sức ép khối lượng (Volume Force Percentile)
    #[dna(min = 0.5, max = 0.9, step = 0.1)]
    pub volume_confirm_thresh: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    /// Mức RSI tối đa/tối thiểu cho phép khi hồi giá (Pullback)
    #[dna(min = 0.1, max = 0.5, step = 0.1)]
    pub rsi_buy_thresh: f64,
    /// Biên độ dung sai (tolerance %) cho phép giá dao động quanh đường KAMA
    #[dna(min = 0.005, max = 0.02, step = 0.005)]
    pub kama_tolerance_pct: f64,
    /// Ngưỡng Stochastic (K) bắt buộc phải đạt để kích hoạt lệnh Pullback
    #[dna(min = 0.05, max = 0.45, step = 0.1)]
    pub stoch_pullback_thresh: f64,
    /// Số kỳ nhìn lại của MACD để đánh giá phân kỳ/động lượng
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    /// Mức RSI tối thiểu để xác nhận xu hướng đà tăng/giảm mạnh
    #[dna(min = 0.6, max = 0.8, step = 0.1)]
    pub rsi_threshold: f64,
    /// Mức RSI Climax (Cực đoan) để chặn việc vào lệnh quá trễ (FOMO)
    #[dna(min = 0.70, max = 0.90, step = 0.05)]
    pub rsi_climax: f64,
    /// Đỉnh Volume Force tối thiểu phải đạt được để kích hoạt Momentum Ignition
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub volume_spike_min: f64,
    /// Số kỳ nhìn lại của MACD
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    /// Hệ số gia tốc (Gia tăng lực mua/bán đột ngột)
    #[dna(min = 1.1, max = 2.1, step = 0.2)]
    pub accel_factor: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    /// Ngưỡng Quá bán của Stochastic trong thị trường đi ngang
    #[dna(min = 0.05, max = 0.35, step = 0.1)]
    pub stoch_oversold: f64,
    /// Ngưỡng Quá mua của Stochastic trong thị trường đi ngang
    #[dna(min = 0.65, max = 0.95, step = 0.1)]
    pub stoch_overbought: f64,
    /// Chiều rộng tối thiểu của Bollinger Bands (tính theo %) để giao dịch Range
    #[dna(min = 0.01, max = 0.05, step = 0.01)]
    pub min_width_pct: f64,
    /// Giới hạn Khối lượng tối đa (Để tránh giao dịch khi Breakout sắp xảy ra)
    #[dna(min = 0.1, max = 0.6, step = 0.1)]
    pub volume_max_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    /// Mức RSI cực đoan (Quá mua/Quá bán sâu) cần thiết để bắt Đảo chiều
    #[dna(min = 0.70, max = 0.90, step = 0.05)]
    pub rsi_extreme: f64,
    /// Tỷ lệ Volume sụt giảm (Exhaustion) - Dấu hiệu phe đẩy giá đã kiệt sức
    #[dna(min = 0.1, max = 0.9, step = 0.2)]
    pub exhaustion_vol_drop: f64,
    /// Số kỳ nhìn lại của MACD (Tìm phân kỳ)
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    /// Giá trị Z-Score cực đoan của giá để xác định đã lệch khỏi giá trị trung bình quá xa
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    /// RSI Climax để xác nhận cú quét (Flash Pump / Flash Crash)
    #[dna(min = 0.70, max = 0.90, step = 0.05)]
    pub rsi_climax: f64,
    /// Tỷ lệ từ chối (Wick Rejection) của bóng nến so với toàn bộ cây nến
    #[dna(min = 0.1, max = 0.7, step = 0.2)]
    pub min_rejection_pct: f64,
    /// MACD Lookback
    #[dna(min = 5.0, max = 50.0, step = 10.0)]
    pub macd_lookback: f64,
    /// Z-Score cực đại để bắt dao rơi/cản tàu
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub z_score_threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    /// Áp lực Khối lượng (Volume Force Percentile) tối thiểu (Gom hàng/Xả hàng ngầm)
    #[dna(min = 0.6, max = 0.9, step = 0.1)]
    pub vol_min_threshold: f64,
    /// MACD Lookback
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
    /// Phần trăm tổng vốn cược cho mỗi lệnh (1.0 = 100%)
    #[dna(min = 0.01, max = 0.15, step = 0.02)]
    pub percent_per_trade: f64,
    /// Khoảng cách Stoploss tối thiểu (%) để tránh bị dính spread/trượt giá
    #[dna(min = 0.001, max = 0.01, step = 0.002)]
    pub sl_min_dist_pct: f64,
    // [DEAD CODE] Break-Even params temporarily disabled to prevent whipsaws
    // #[dna(min = 0.0, max = 1.0, step = 1.0)]
    // pub use_breakeven: f64,
    // #[dna(min = 0.5, max = 2.5, step = 0.5)]
    // pub be_profit_target_rr: f64,
    /// Quy mô lô cơ bản (Base Lot Size) - Dùng khi Trade kích thước cố định thay vì %
    #[dna(min = 0.1, max = 1.5, step = 0.3)]
    pub base_lot_size: f64,
}
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    /// Hệ số ATR cho Stoploss cơ bản
    #[dna(min = 1.0, max = 15.0, step = 0.5)]
    pub base_sl_atr: f64,
    
    // [DEAD CODE] Tạm thời loại bỏ các SL factor dư thừa để giảm bớt không gian tìm kiếm vô ích
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub squeeze_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub trend_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub momentum_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub range_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub reversal_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub anomaly_sl_factor: f64,
    // #[dna(min = 0.5, max = 3.0, step = 0.5)]
    // pub absorption_sl_factor: f64,
    
    /// Thời gian (số nến) đếm ngược để đánh giá lệnh có phá đỉnh hay không (High-watermark)
    #[dna(min = 100.0, max = 5000.0, step = 100.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_countdown_bars: usize,
    
    /// Hệ số siết chặt SL sau mỗi chu kỳ đếm ngược (Ví dụ: 0.8 nghĩa là siết SL lại 20%)
    #[dna(min = 0.5, max = 0.9, step = 0.1)]
    pub tighten_step_multiplier: f64,
    
    /// Hệ số Trailing Stop thắt chặt (Dùng khi Sideway hoặc lệnh ngâm quá lâu)
    #[dna(min = 0.1, max = 2.0, step = 0.2)]
    pub tsl_atr_tight: f64,
    /// Hệ số Trailing Stop tiêu chuẩn
    #[dna(min = 1.0, max = 4.5, step = 0.3)]
    pub tsl_atr_normal: f64,
    /// Hệ số Trailing Stop nới lỏng (Dùng khi Bão tố - Volatility bùng nổ)
    #[dna(min = 2.0, max = 15.0, step = 0.5)]
    pub tsl_atr_loose: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
