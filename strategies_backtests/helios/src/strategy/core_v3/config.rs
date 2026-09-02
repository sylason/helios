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
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct AdxParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RsiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct BbandsParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub nbdev: f64,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct KeltnerParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub multiplier: f64,
    #[dna(subset = "5m, 15m, 1h, 4h")]
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
    #[dna(subset = "5m, 15m, 1h, 4h")]
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
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct AtrParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct NAtrParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct MfiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct VolMaParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct VwmaParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
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

/// ============================================================================
/// PHẦN 3: BỘ TRỌNG TÀI & ĐẤU GIÁ TÍN HIỆU (ARBITER CONFIDENCE AUCTION CONFIG)
/// ============================================================================
/// Điều phối, kiểm soát quyền phủ quyết và phê duyệt tín hiệu từ 6 Tactical Scanners.
/// Áp dụng dải phổ quát Universal 1% -> 99% với bước nhảy 3% (~33 nấc nhảy ý nghĩa).
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct ArbiterConfig {
    /// 1. Ngưỡng Phủ Quyết Bối Cảnh Nghịch (Veto Threshold: 1% -> 50%, Step 3% -> 17 nấc)
    /// Khi hệ số bối cảnh (Multiplier) thấp hơn ngưỡng này, lệnh bị hủy ngay lập tức.
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_multiplier_veto: f64,
    
    /// 2. Tỷ Trọng Điểm Số Scanner vs Trọng Số Bối Cảnh (1% -> 99%, Step 3% -> 33 nấc)
    /// Trọng số quyết định mức độ ưu tiên giữa độ tin cậy của Scanner và ma trận thị trường.
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub scanner_weight: f64,

    /// 3. Ngưỡng Phê Duyệt Tự Tin Khi Thị Trường Có Xu Hướng (Trending: 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_confidence_trending: f64,
    
    /// 4. Ngưỡng Phê Duyệt Tự Tin Khi Thị Trường Đi Ngang (Ranging: 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_confidence_ranging: f64,
    
    /// 5. Ngưỡng Phê Duyệt Tự Tin Khi Thị Trường Biến Động Mạnh (Volatile: 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_confidence_volatile: f64,
}

/// ============================================================================
/// PHẦN 4: BỘ 6 CHIẾN ĐỘI TÁC CHIẾN BẮN TỈA (TACTICAL SNIPER CONFIG)
/// ============================================================================
/// Cấu hình các cảm biến kích hoạt mềm và ngưỡng kích hoạt (min_activation_score)
/// cho đúng 6 Tactical Scanners độc lập theo chuẩn Phổ Quát (Universal 1% -> 99%, Step 3%).
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalSniperConfig {
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

/// 🏹 1. CHIẾN ĐỘI TREND PULLBACK (Săn nhịp hồi trong xu hướng mạnh)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// RSI Vùng Hồi Quy (Universal 1.0 -> 99.0, Step 3.0 -> 33 nấc)
    #[dna(min = 1.0, max = 99.0, step = 3.0)]
    pub rsi_buy_thresh: f64,

    /// Stochastic Vùng Hồi Quy (Universal 1.0 -> 99.0, Step 3.0 -> 33 nấc)
    #[dna(min = 1.0, max = 99.0, step = 3.0)]
    pub stoch_pullback_thresh: f64,

    /// Độ dốc tối thiểu của KAMA (0.01 -> 0.50, Step 0.02 -> 25 nấc)
    #[dna(min = 0.01, max = 0.50, step = 0.02)]
    pub kama_slope_threshold: f64,

    /// Ngưỡng kích hoạt mềm Trend Pullback (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// ⚡ 2. CHIẾN ĐỘI MOMENTUM IGNITION (Săn các cú bùng nổ xung lực Volume & Momentum)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// RSI Xung Lực Bùng Nổ (Universal 1.0 -> 99.0, Step 3.0 -> 33 nấc)
    #[dna(min = 1.0, max = 99.0, step = 3.0)]
    pub rsi_threshold: f64,

    /// Độ lệch chuẩn Z-Score Volume bùng nổ (0.2 -> 5.0 sigma, Step 0.2 -> 25 nấc)
    #[dna(min = 0.20, max = 5.00, step = 0.20)]
    pub vol_z_threshold: f64,

    /// Ngưỡng kích hoạt mềm Momentum Ignition (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// 🌊 3. CHIẾN ĐỘI RANGE OSCILLATOR (Bắt đỉnh/đáy biên độ thị trường đi ngang)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// Khoảng cách tiệm cận biên Bollinger (0.1% -> 5.0% giá, Step 0.2% -> 25 nấc)
    #[dna(min = 0.001, max = 0.050, step = 0.002)]
    pub bb_proximity_pct: f64,

    /// Ngưỡng quá bán Stochastic (Universal 1.0 -> 50.0, Step 1.5 -> 33 nấc)
    #[dna(min = 1.0, max = 50.0, step = 1.5)]
    pub stoch_oversold: f64,

    /// Ngưỡng quá mua Stochastic (Universal 50.0 -> 99.0, Step 1.5 -> 33 nấc)
    #[dna(min = 50.0, max = 99.0, step = 1.5)]
    pub stoch_overbought: f64,

    /// Độ rộng tối thiểu dải Range (0.2% -> 6.0% giá, Step 0.2% -> 30 nấc)
    #[dna(min = 0.002, max = 0.060, step = 0.002)]
    pub min_width_pct: f64,

    /// Trần Volume né tin tức (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub volume_max_threshold: f64,

    /// Ngưỡng kích hoạt mềm Range Oscillator (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// 🔄 4. CHIẾN ĐỘI CLIMAX REVERSAL (Bắt đỉnh/đáy kiệt sức Parabol)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// Độ lệch chuẩn giá cực đại Z-Score (0.5 -> 5.0 sigma, Step 0.2 -> 23 nấc)
    #[dna(min = 0.5, max = 5.0, step = 0.2)]
    pub z_score_threshold: f64,

    /// Biên độ nến đảo chiều kiệt sức (0.5 -> 6.0 ATR, Step 0.25 -> 23 nấc)
    #[dna(min = 0.5, max = 6.0, step = 0.25)]
    pub min_reversal_displacement: f64,

    /// Tỷ lệ sụt giảm volume kiệt sức (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub exhaustion_vol_drop: f64,

    /// Đệm ATR bảo vệ qua râu nến (0.2 -> 6.0 ATR, Step 0.2 -> 30 nấc)
    #[dna(min = 0.20, max = 6.00, step = 0.20)]
    pub wick_sl_buffer_atr: f64,

    /// Ngưỡng kích hoạt mềm Climax Reversal (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// 💥 5. CHIẾN ĐỘI ANOMALY FADE (Săn các râu nến quét thanh khoản dị thường)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// Độ dài râu nến quét thanh khoản (0.5 -> 6.0 ATR, Step 0.2 -> 28 nấc)
    #[dna(min = 0.5, max = 6.0, step = 0.2)]
    pub wick_atr_threshold: f64,

    /// Z-Score độ lệch giá dị thường (0.5 -> 5.0 sigma, Step 0.2 -> 23 nấc)
    #[dna(min = 0.5, max = 5.0, step = 0.2)]
    pub z_score_threshold: f64,

    /// Đệm ATR qua râu nến (0.2 -> 6.0 ATR, Step 0.2 -> 30 nấc)
    #[dna(min = 0.20, max = 6.00, step = 0.20)]
    pub wick_sl_buffer_atr: f64,

    /// Ngưỡng kích hoạt mềm Anomaly Fade (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// 🧲 6. CHIẾN ĐỘI ABSORPTION SCOUT (Phát hiện dòng tiền cá mập gom/xả hàng trong vùng nén)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    /// Kẹp trần SL ban đầu (0.5% -> 10.0% giá, Step 0.3% -> 32 nấc)
    #[dna(min = 0.005, max = 0.100, step = 0.003)]
    pub max_initial_sl_pct: f64,

    /// Độ nén dải Bollinger/Keltner (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub squeeze_threshold: f64,

    /// Z-Score Volume hấp thụ dòng tiền (0.2 -> 5.0 sigma, Step 0.2 -> 25 nấc)
    #[dna(min = 0.20, max = 5.00, step = 0.20)]
    pub vol_z_threshold: f64,

    /// Ngưỡng kích hoạt mềm Absorption Scout (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,
}

/// ============================================================================
/// PHẦN 5: QUẢN TRỊ RỦI RO & TRAILING STOPLOSS (RISK MANAGEMENT CONFIG)
/// ============================================================================
/// Điều phối quy mô vị thế, đòn bẩy an toàn và cơ chế Trailing Stop 3 phân tầng ATR
/// kết hợp phân rã thời gian (Time Decay) và bảo vệ siêu sóng (Mega Trend Shield).
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskManagementConfig {
    #[dna(nest)]
    pub risk_base: RiskBaseParams,
    #[dna(nest)]
    pub stop_management: RiskStopParams,
}

/// 💰 QUẢN TRỊ VỐN CƠ SỞ & ĐÒN BẨY
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskBaseParams {
    /// 1. Tỷ Lệ Vốn Ký Quỹ Trên Mỗi Lệnh (2% -> 50% vốn, Step 2% -> 25 nấc)
    #[dna(min = 0.02, max = 0.50, step = 0.02)]
    pub percent_per_trade: f64,

    /// 2. Khoảng Cách Đặt SL Tối Thiểu Tránh Quét Spread (0.1% -> 3.0% giá, Step 0.1% -> 30 nấc)
    #[dna(min = 0.001, max = 0.030, step = 0.001)]
    pub sl_min_dist_pct: f64,

    /// 3. Kích Thước Lot Cơ Sở Tối Thiểu (0.1 -> 2.0 Lot, Step 0.1 -> 20 nấc)
    #[dna(min = 0.1, max = 2.0, step = 0.1)]
    pub base_lot_size: f64,

    /// 4. Đòn Bẩy Tài Khoản Margin/Futures (1x -> 50x, Step 1.0 -> 50 nấc)
    #[dna(min = 1.0, max = 50.0, step = 1.0)]
    pub leverage: f64,
}

/// 🛡️ CƠ CHẾ CẮT LỖ & TRAILING STOP THÍCH ỨNG
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    /// 1. Sàn Cắt Lỗ Khởi Tạo Tối Thiểu (0.1% -> 3.0% giá, tương đương 2% -> 60% ROE trên 20x, Step 0.1% -> 30 nấc)
    #[dna(min = 0.001, max = 0.030, step = 0.001)]
    pub min_initial_sl_pct: f64,
    
    /// 2. Chu Kỳ Đếm Nến Đi Ngang Kể Từ Đỉnh Lợi Nhuận (15 nến -> 720 nến / 12 tiếng, Step 30 nến -> 24 nấc)
    #[dna(min = 15.0, max = 360.0, step = 15.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_countdown_bars: usize,

    /// 3. Hệ Số Lũy Thừa Siết SL Mỗi Chu Kỳ Đi Ngang (50% -> 98%, Step 2% -> 25 nấc)
    #[dna(min = 0.50, max = 0.98, step = 0.02)]
    pub tighten_step_multiplier: f64,

    // =========================================================================
    // 4. BỘ 3 PHÂN TẦNG ATR TRAILING STOP PHỔ QUÁT (0.5 -> 15.0 ATR KHÔNG CHỒNG CHÉO)
    // =========================================================================
    /// - Phân Tầng Siết Chặt: Dành cho thị trường Range / Reversal / Đỉnh sóng (0.5 -> 3.0 ATR, Step 0.10 -> 26 nấc)
    #[dna(min = 0.5, max = 3.0, step = 0.10)]
    pub tsl_atr_tight: f64,

    /// - Phân Tầng Chuẩn: Dành cho xu hướng bậc thang Trend Pullback & Absorption (3.0 -> 7.5 ATR, Step 0.15 -> 31 nấc)
    #[dna(min = 3.0, max = 7.5, step = 0.15)]
    pub tsl_atr_normal: f64,

    /// - Phân Tầng Nới Lỏng: Dành cho siêu sóng bùng nổ Momentum & Anomaly (7.5 -> 15.0 ATR, Step 0.25 -> 31 nấc)
    #[dna(min = 7.5, max = 15.0, step = 0.25)]
    pub tsl_atr_loose: f64,

    // =========================================================================
    // 5. CƠ CHẾ BẢO VỆ SIÊU SÓNG (MEGA TREND PROTECTION)
    // =========================================================================
    /// Ngưỡng Lợi Nhuận Kích Hoạt Chế Độ Bảo Vệ Siêu Sóng (1.0% -> 20.0% giá, tương đương +20% -> +400% ROE trên 20x, Step 0.5% -> 39 nấc)
    #[dna(min = 0.01, max = 0.20, step = 0.005)]
    pub mega_trend_pnl_trigger: f64,

    /// Số Chu Kỳ Tối Đa Được Hưởng Chế Độ Bảo Vệ Thả Diều (2 -> 30 chu kỳ, Step 1.0 -> 29 nấc)
    #[dna(min = 2.0, max = 30.0, step = 1.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub mega_trend_max_cycles: usize,

    // =========================================================================
    // 6. THAM SỐ PHỤ TRỢ & TÍCH LŨY
    // =========================================================================
    /// Thời Gian Ngâm Tối Đa Trước Khi Giải Phóng Vốn (60 nến -> 1440 nến / 24 tiếng, Step 60 nến -> 24 nấc)
    #[dna(min = 60.0, max = 1440.0, step = 60.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub max_stagnant_hold_bars: usize,

    /// Lợi Nhuận Tối Thiểu Để Giữ Lệnh Ngâm Không Bị Đóng Sớm (0.2% -> 5.0% giá, Step 0.2% -> 25 nấc)
    #[dna(min = 0.002, max = 0.050, step = 0.002)]
    pub stagnant_min_profit_pct: f64,

    /// Khoảng Cách Cắt Lỗ Ban Đầu Mặc Định Theo ATR (0.5 -> 6.0 ATR, Step 0.2 -> 28 nấc)
    #[dna(min = 0.5, max = 6.0, step = 0.2)]
    pub base_sl_atr: f64,

    /// Ngưỡng Tạm Dừng Siết SL Khi Thị Trường Tích Lũy Squeeze (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub squeeze_pause_threshold: f64,
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}
