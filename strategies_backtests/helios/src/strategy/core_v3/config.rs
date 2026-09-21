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
    // [V3.3 TINH GIẢN DNA] Tắt gen cảm biến RSI riêng của Anomaly (giảm 2 gen)
    // #[dna(nest)]
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
    // [V3.3 TINH GIẢN DNA] Tắt gen cảm biến Stochastic của Range Oscillator (giảm 7 gen)
    // #[dna(nest)]
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
    /// 🧭 NEO KHUNG VĨ MÔ (Macro Anchor): Khóa cận dưới Timeframe KAMA tối thiểu >= 15m (chỉ chọn 15m, 1h, 4h)
    #[dna(subset = "15m, 1h, 4h")]
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

#[derive(Debug, Clone, Deserialize)]
pub struct MacdParams {
    #[serde(default = "default_macd_fast", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub fast: usize, 
    #[serde(default = "default_macd_slow", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub slow: usize,
    #[serde(default = "default_macd_signal", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub signal: usize, 
    #[serde(default = "default_timeframe")]
    pub timeframe: Timeframe,
}

fn default_macd_fast() -> usize { 12 }
fn default_macd_slow() -> usize { 26 }
fn default_macd_signal() -> usize { 9 }
fn default_timeframe() -> Timeframe { Timeframe::T5m }

impl Default for MacdParams {
    fn default() -> Self {
        Self {
            fast: default_macd_fast(),
            slow: default_macd_slow(),
            signal: default_macd_signal(),
            timeframe: default_timeframe(),
        }
    }
}

impl DnaMappable for MacdParams {
    const DNA_WIDTH: usize = 0;
    fn update_from_dna(&mut self, _dna: &[f64]) {}
    fn parameter_defs() -> Vec<(String, ferro_trade::optimizer::ParamDefinition)> {
        Vec::new()
    }
    fn dna_signature() -> u64 {
        0
    }
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

#[derive(Debug, Clone, Deserialize)]
pub struct VolMaParams {
    #[serde(default = "default_vol_ma_period", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[serde(default = "default_timeframe")]
    pub timeframe: Timeframe,
}

fn default_vol_ma_period() -> usize { 20 }

impl Default for VolMaParams {
    fn default() -> Self {
        Self {
            period: default_vol_ma_period(),
            timeframe: default_timeframe(),
        }
    }
}

impl DnaMappable for VolMaParams {
    const DNA_WIDTH: usize = 0;
    fn update_from_dna(&mut self, _dna: &[f64]) {}
    fn parameter_defs() -> Vec<(String, ferro_trade::optimizer::ParamDefinition)> {
        Vec::new()
    }
    fn dna_signature() -> u64 {
        0
    }
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

    /// 🚀 3 GEN TRỰC GIAO MỚI CHO DE-OVERENGINEERED MATRIX:
    /// 1. Ngưỡng ADX xác nhận xu hướng rõ rệt (15.0 -> 40.0, Step 1.0 -> 26 nấc)
    #[dna(min = 15.0, max = 40.0, step = 1.0)]
    #[serde(default = "default_adx_trend_threshold")]
    pub adx_trend_threshold: f64,

    /// 2. Hệ số co giãn độ dốc KAMA chuẩn hóa theo ATR (0.05 -> 0.50, Step 0.01 -> 46 nấc)
    #[dna(min = 0.05, max = 0.50, step = 0.01)]
    #[serde(default = "default_kama_slope_scale_atr")]
    pub kama_slope_scale_atr: f64,

    /// 3. Độ nhạy bùng nổ khối lượng qua Z-score (1.0 -> 3.0, Step 0.1 -> 21 nấc)
    #[dna(min = 1.0, max = 3.0, step = 0.1)]
    #[serde(default = "default_vol_z_sensitivity")]
    pub vol_z_sensitivity: f64,
}

fn default_adx_trend_threshold() -> f64 { 25.0 }
fn default_kama_slope_scale_atr() -> f64 { 0.20 }
fn default_vol_z_sensitivity() -> f64 { 1.50 }

/// ============================================================================
/// PHẦN 3: BỘ TRỌNG TÀI & ĐẤU GIÁ TÍN HIỆU (ARBITER CONFIDENCE AUCTION CONFIG)
/// ============================================================================
/// Đã được De-overengineer sang mô hình chấm điểm tuyến tính O(1).
/// Loại bỏ 5 gen ma và 7 Multipliers phi tuyến.
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct ArbiterConfig {
    /// 🎯 NGƯỠNG PHÊ DUYỆT TỰ TIN TUYẾN TÍNH DUY NHẤT (0.05 -> 0.50, Step 0.01 -> 46 nấc)
    #[dna(min = 0.05, max = 0.50, step = 0.01)]
    #[serde(default = "default_min_confidence_threshold")]
    pub min_confidence_threshold: f64,

    // =========================================================================
    // 🧹 CÁC TRƯỜNG CŨ ĐÃ BỎ GEN DNA (COMMENT OUT #[dna(...)]) ĐỂ DE-OVERENGINEER
    // Tạm thời giữ lại để tương thích mã nguồn cho đến khi Bước 4 (Arbiter) hoàn tất.
    // =========================================================================
    #[serde(default = "default_min_multiplier_veto")]
    pub min_multiplier_veto: f64,
    
    #[serde(default = "default_scanner_weight")]
    pub scanner_weight: f64,

    #[serde(default = "default_min_confidence_trending")]
    pub min_confidence_trending: f64,
    
    #[serde(default = "default_min_confidence_ranging")]
    pub min_confidence_ranging: f64,
    
    #[serde(default = "default_min_confidence_compression")]
    pub min_confidence_compression: f64,

    #[serde(default = "default_min_confidence_chaos", alias = "min_confidence_volatile")]
    pub min_confidence_chaos: f64,
}

fn default_min_confidence_threshold() -> f64 { 0.20 }
fn default_min_multiplier_veto() -> f64 { 0.01 }
fn default_scanner_weight() -> f64 { 0.50 }
fn default_min_confidence_trending() -> f64 { 0.20 }
fn default_min_confidence_ranging() -> f64 { 0.20 }
fn default_min_confidence_compression() -> f64 { 0.20 }
fn default_min_confidence_chaos() -> f64 { 0.20 }

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
    // [V3.3 TINH GIẢN DNA] Tắt gen của Range Oscillator (giảm 12 gen)
    // #[dna(nest)]
    pub range: TacticalRangeParams,
    #[dna(nest)]
    pub reversal: TacticalReversalParams,
    // [V3.3 TINH GIẢN DNA] Tắt gen của Anomaly Fade (giảm 5 gen)
    // #[dna(nest)]
    pub anomaly: TacticalAnomalyParams,
    #[dna(nest)]
    pub absorption: TacticalAbsorptionParams,
}

/// 🏹 1. CHIẾN ĐỘI TREND PULLBACK (Săn nhịp hồi trong xu hướng mạnh)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
    pub max_initial_sl_pct: f64,

    /// Trần RSI an toàn tránh hưng phấn đỉnh sóng (50.0 -> 80.0, Step 2.0 -> 16 nấc)
    #[dna(min = 50.0, max = 80.0, step = 2.0)]
    #[serde(default = "default_pullback_rsi_max_cap")]
    pub rsi_max_cap: f64,

    /// Stochastic Vùng Hồi Quy (10.0 -> 50.0, Step 2.0 -> 21 nấc)
    #[dna(min = 10.0, max = 50.0, step = 2.0)]
    pub stoch_pullback_thresh: f64,

    /// Độ dốc tối thiểu của KAMA (0.01 -> 0.40, Step 0.02 -> 20 nấc)
    #[dna(min = 0.01, max = 0.40, step = 0.02)]
    pub kama_slope_threshold: f64,

    /// Khoảng cách tối đa cho phép từ giá đến KAMA qua ATR (0.50 -> 3.50 ATR, Step 0.20 -> 16 nấc)
    #[dna(min = 0.50, max = 3.50, step = 0.20)]
    #[serde(default = "default_pullback_max_dist_atr")]
    pub max_pullback_dist_atr: f64,

    /// Ngưỡng cắt đứt hỗ trợ gãy trend KAMA (0.10 -> 1.50 ATR, Step 0.10 -> 15 nấc)
    #[dna(min = 0.10, max = 1.50, step = 0.10)]
    #[serde(default = "default_pullback_invalidation_buffer_atr")]
    pub invalidation_buffer_atr: f64,

    /// Ngưỡng kích hoạt mềm Trend Pullback (0.10 -> 0.90, Step 0.05 -> 17 nấc)
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    pub min_activation_score: f64,

    /// Đệm ATR bảo vệ qua KAMA / đáy nhịp hồi (0.10 -> 3.00 ATR, Step 0.15 -> 20 nấc)
    #[dna(min = 0.10, max = 3.00, step = 0.15)]
    #[serde(default = "default_pullback_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,
}

/// ⚡ 2. CHIẾN ĐỘI MOMENTUM IGNITION (Săn các cú bùng nổ xung lực Volume & Momentum)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
    pub max_initial_sl_pct: f64,

    /// RSI Xung Lực Bùng Nổ (50.0 -> 80.0, Step 1.0 -> 31 nấc)
    #[dna(min = 50.0, max = 80.0, step = 1.0)]
    pub rsi_threshold: f64,

    /// Ngưỡng Z-Score bùng nổ xung lực khối lượng (1.00 -> 3.50, Step 0.10 -> 26 nấc)
    #[dna(min = 1.00, max = 3.50, step = 0.10)]
    pub vol_z_threshold: f64,

    /// Ngưỡng kích hoạt mềm Momentum Ignition (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,

    /// Đệm ATR dưới đáy nến bùng nổ xung lực (0.20 -> 2.00 ATR, Step 0.10 -> 19 nấc)
    #[dna(min = 0.20, max = 2.00, step = 0.10)]
    #[serde(default = "default_momentum_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🎯 Ngưỡng lực nến rút chân/đè đầu CLV tối thiểu (0.10 -> 0.60, Step 0.05 -> 11 nấc)
    #[dna(min = 0.10, max = 0.60, step = 0.05)]
    #[serde(default = "default_momentum_min_clv")]
    pub min_clv: f64,

    /// 🛡️ Ngưỡng dốc xu hướng KAMA tối đa cấm bùng nổ ngược sóng (0.05 -> 0.35, Step 0.02 -> 16 nấc)
    #[dna(min = 0.05, max = 0.35, step = 0.02)]
    #[serde(default = "default_momentum_max_adverse_slope")]
    pub max_adverse_slope: f64,

    /// 🚀 Trọng số xung lực giá bứt phá vượt biên (0.20 -> 0.60, Step 0.05 -> 9 nấc)
    #[dna(min = 0.20, max = 0.60, step = 0.05)]
    #[serde(default = "default_momentum_w_breakout")]
    pub w_breakout: f64,

    /// 📊 Trọng số xung lực khối lượng bùng nổ (0.20 -> 0.60, Step 0.05 -> 9 nấc)
    #[dna(min = 0.20, max = 0.60, step = 0.05)]
    #[serde(default = "default_momentum_w_volume")]
    pub w_volume: f64,
}

/// 🌊 3. CHIẾN ĐỘI RANGE OSCILLATOR (Bắt đỉnh/đáy biên độ thị trường đi ngang)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalRangeParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
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

    /// Ngưỡng sức mạnh Trend tối đa mà Range được phép hoạt động (0.10 -> 0.80, Step 0.05)
    /// Đọc từ matrix.trend_strength (vốn được tính động từ ADX theo Timeframe mà DNA chọn!)
    #[dna(min = 0.10, max = 0.80, step = 0.05)]
    pub max_trend_strength: f64,

    /// Ngưỡng dốc xu hướng KAMA tối đa cấm bắt dao rơi (0.05 -> 0.50, Step 0.05)
    /// Ngăn chặn bắt dao rơi: Nếu Trend dốc xuống vượt ngưỡng này -> Cấm Long!
    #[dna(min = 0.05, max = 0.50, step = 0.05)]
    pub max_adverse_slope: f64,

    /// Đệm ATR ngoài biên dải Bollinger Bands (0.10 -> 3.00 ATR, Step 0.15 -> 20 nấc)
    #[dna(min = 0.10, max = 3.00, step = 0.15)]
    #[serde(default = "default_range_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🛡️ FLAW-REG.10: Ngưỡng dốc tối đa của trục giữa BB Middle (0.02 -> 0.50 ATR, Step 0.02 -> 25 nấc)
    #[dna(min = 0.02, max = 0.50, step = 0.02)]
    #[serde(default = "default_range_max_bb_slope")]
    pub max_bb_slope: f64,

    /// 🛡️ FLAW-REG.10: Tỷ lệ mục tiêu Take Profit từ Entry đến dải đối diện (0.10 -> 0.90, Step 0.05 -> 17 nấc)
    /// 0.50 = Đúng trục giữa BB Middle (SMA 20); < 0.50 = Chốt non an toàn; > 0.50 = Hướng về dải đối diện
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    #[serde(default = "default_range_tp_target_ratio")]
    pub tp_target_ratio: f64,
}

/// 🔄 4. CHIẾN ĐỘI CLIMAX REVERSAL (Bắt đỉnh/đáy kiệt sức Parabol)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalReversalParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
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

    /// 🎯 Ngưỡng lực nến rút chân/đè đầu CLV tối thiểu (0.05 -> 0.60, Step 0.05 -> 12 nấc)
    #[dna(min = 0.05, max = 0.60, step = 0.05)]
    #[serde(default = "default_reversal_min_clv")]
    pub min_reversal_clv: f64,

    /// 💧 Ngưỡng kiệt sức dòng tiền MFI (15.0 -> 45.0, Step 5.0 -> 7 nấc)
    /// Long yêu cầu MFI <= thresh, Short yêu cầu MFI >= 100 - thresh
    #[dna(min = 15.0, max = 45.0, step = 5.0)]
    #[serde(default = "default_reversal_mfi_thresh")]
    pub mfi_reversal_thresh: f64,

    /// ⚖️ Trọng số thành phần Z-Score trong chấm điểm Reversal (0.10 -> 0.80, Step 0.05 -> 15 nấc)
    #[dna(min = 0.10, max = 0.80, step = 0.05)]
    #[serde(default = "default_reversal_w_z_score")]
    pub w_z_score: f64,

    /// ⚖️ Trọng số thành phần Displacement trong chấm điểm Reversal (0.10 -> 0.80, Step 0.05 -> 15 nấc)
    #[dna(min = 0.10, max = 0.80, step = 0.05)]
    #[serde(default = "default_reversal_w_displacement")]
    pub w_displacement: f64,
}

/// 💥 5. CHIẾN ĐỘI ANOMALY FADE (Săn các râu nến quét thanh khoản dị thường)
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
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
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
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

    /// Đệm ATR dưới vùng nén hấp thụ thanh khoản (0.10 -> 2.50 ATR, Step 0.10 -> 25 nấc)
    #[dna(min = 0.10, max = 2.50, step = 0.10)]
    #[serde(default = "default_absorption_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🛡️ FLAW-REG.9: Ngưỡng dốc xu hướng KAMA tối đa cấm bắt dao rơi (0.02 -> 0.25, Step 0.01 -> 24 nấc)
    /// Ngăn chặn bắt dao rơi: Nếu Trend dốc xuống vượt ngưỡng này -> Cấm Long!
    #[dna(min = 0.02, max = 0.25, step = 0.01)]
    #[serde(default = "default_absorption_max_adverse_slope")]
    pub max_adverse_slope: f64,

    /// 🌊 Dung sai quét râu nến ngoài biên dải nén (0.00 -> 0.25, Step 0.05 -> 6 nấc)
    #[dna(min = 0.00, max = 0.25, step = 0.05)]
    #[serde(default = "default_absorption_spring_tolerance")]
    pub spring_tolerance: f64,

    /// 🎯 Ngưỡng lực nến rút chân/đè đầu CLV tối thiểu (0.00 -> 0.60, Step 0.05 -> 13 nấc)
    #[dna(min = 0.00, max = 0.60, step = 0.05)]
    #[serde(default = "default_absorption_min_clv")]
    pub min_clv: f64,

    /// 📏 Vị trí đóng nến tối đa bên trong dải nén (%B từ 0.15 -> 0.48, Step 0.03 -> 12 nấc)
    #[dna(min = 0.15, max = 0.48, step = 0.03)]
    #[serde(default = "default_absorption_max_boundary_pct_b")]
    pub max_boundary_pct_b: f64,

    /// ⚖️ Trọng số kết hợp giữa độ nén và xung lực dòng tiền (0.20 -> 0.80, Step 0.10 -> 7 nấc)
    #[dna(min = 0.20, max = 0.80, step = 0.10)]
    #[serde(default = "default_absorption_squeeze_weight")]
    pub squeeze_weight: f64,
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
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct RiskBaseParams {
    /// 1. Tỷ Lệ Vốn Ký Quỹ Trên Mỗi Lệnh (User Setting: Mặc định 20% vốn = 0.20)
    #[serde(default = "default_percent_per_trade")]
    pub percent_per_trade: f64,

    /// 2. Khoảng Cách Đặt SL Tối Thiểu Tránh Quét Spread
    #[serde(default = "default_sl_min_dist_pct")]
    pub sl_min_dist_pct: f64,

    /// 3. Kích Thước Lot Cơ Sở Tối Thiểu
    #[serde(default = "default_base_lot_size")]
    pub base_lot_size: f64,

    /// 4. Đòn Bẩy Tài Khoản Margin/Futures Chuẩn (User Setting: Mặc định 20x)
    #[serde(default = "default_leverage")]
    pub leverage: f64,

    /// 5. Đòn Bẩy Chiến Lược Nuke Độc Lập (User Setting: Mặc định 50x)
    #[serde(default = "default_nuke_leverage")]
    pub nuke_leverage: f64,

    /// 6. Kích Thước Lệnh Danh Nghĩa Tối Thiểu USD ($21.0 chống sàn từ chối min notional)
    #[serde(default = "default_min_order_usd")]
    pub min_order_usd: f64,

    /// 7. Tỷ Lệ Ký Quỹ Tối Đa Trên Mỗi Vị Thế (Trần bảo vệ rủi ro: Mặc định 25% vốn)
    #[serde(default = "default_max_cap_fraction")]
    pub max_cap_fraction: f64,
}

fn default_percent_per_trade() -> f64 { 0.20 }
fn default_sl_min_dist_pct() -> f64 { 0.005 }
fn default_base_lot_size() -> f64 { 0.01 }
fn default_leverage() -> f64 { 20.0 }
fn default_nuke_leverage() -> f64 { 50.0 }
fn default_min_order_usd() -> f64 { 21.0 }
fn default_max_cap_fraction() -> f64 { 0.25 }

impl Default for RiskBaseParams {
    fn default() -> Self {
        Self {
            percent_per_trade: default_percent_per_trade(),
            sl_min_dist_pct: default_sl_min_dist_pct(),
            base_lot_size: default_base_lot_size(),
            leverage: default_leverage(),
            nuke_leverage: default_nuke_leverage(),
            min_order_usd: default_min_order_usd(),
            max_cap_fraction: default_max_cap_fraction(),
        }
    }
}

/// 🛡️ CƠ CHẾ CẮT LỖ & TRAILING STOP THÍCH ỨNG
#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct RiskStopParams {
    /// 0. Khoảng Đệm ATR Tối Thiểu Khi Dời TSL Chống Râu Nến Liếm (0.4 -> 2.5 ATR, Step 0.10 -> 22 nấc)
    #[dna(min = 0.4, max = 2.5, step = 0.10)]
    #[serde(default = "default_tsl_buffer_floor_atr")]
    pub tsl_buffer_floor_atr: f64,

    /// 1. Sàn Cắt Lỗ Khởi Tạo Tối Thiểu (0.15% -> 4.0% giá, Step 0.05% -> 78 nấc)
    #[dna(min = 0.0015, max = 0.040, step = 0.0005)]
    pub min_initial_sl_pct: f64,

    /// 1.1 Trần Cắt Lỗ ROE Mặc Định Cho Lệnh Untagged / Adopted (4.0% -> 40.0% ROE, Step 1.5% -> 25 nấc)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
    #[serde(default = "default_sl_roe_cap")]
    pub default_sl_roe_cap: f64,
    
    /// 2. Chu Kỳ Đếm Nến Đi Ngang Kể Từ Đỉnh Lợi Nhuận (15 nến -> 720 nến / 12 tiếng, Step 30 nến -> 24 nấc)
    #[dna(min = 15.0, max = 360.0, step = 15.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub time_countdown_bars: usize,

    /// 3. Hệ Số Lũy Thừa Siết SL Mỗi Chu Kỳ Đi Ngang (50% -> 98%, Step 2% -> 25 nấc)
    #[dna(min = 0.50, max = 0.98, step = 0.02)]
    pub tighten_step_multiplier: f64,

    // =========================================================================
    // 4. KÍCH HOẠT HÒA VỐN ĐỘNG THEO 3 NHÓM REGIME SIMPLEX (DYNAMIC BREAK-EVEN)
    // =========================================================================
    /// Ngưỡng ROE Cơ Sở Kích Hoạt Hòa Vốn Break-Even Fallback (3% -> 25% ROE, Step 1% -> 23 nấc)
    #[dna(min = 0.03, max = 0.25, step = 0.01)]
    pub base_be_roe_trigger: f64,

    // --- 4.1 NHÓM XU HƯỚNG (TREND REGIME: TREND_PB, ABSORPTION) ---
    /// Ngưỡng kích hoạt BE cho Trend (Lãi đạt 1.0 -> 4.5 ATR mới kéo SL, Step 0.10)
    #[dna(min = 1.0, max = 4.5, step = 0.10)]
    pub be_atr_trigger_trend: f64,
    /// Khoảng đệm bảo vệ vốn dưới đỉnh cho Trend (Giữ khoảng cách 0.8 -> 3.0 ATR cho giá thở, Step 0.10)
    #[dna(min = 0.8, max = 3.0, step = 0.10)]
    pub be_buffer_atr_trend: f64,

    // --- 4.2 NHÓM ĐI NGANG (RANGE REGIME: RANGE_OSC, REVERSAL) ---
    /// Ngưỡng kích hoạt BE cho Range (Sóng ngắn: 0.8 -> 3.0 ATR, Step 0.10)
    #[dna(min = 0.8, max = 3.0, step = 0.10)]
    pub be_atr_trigger_range: f64,
    /// Khoảng đệm bảo vệ vốn cho Range (0.4 -> 2.0 ATR, Step 0.10)
    #[dna(min = 0.4, max = 2.0, step = 0.10)]
    pub be_buffer_atr_range: f64,

    // --- 4.3 NHÓM BIẾN ĐỘNG / XUNG LỰC (MOMENTUM_IGN, ANOMALY) ---
    /// Ngưỡng kích hoạt BE cho Xung lực (0.8 -> 3.5 ATR, Step 0.10)
    #[dna(min = 0.8, max = 3.5, step = 0.10)]
    pub be_atr_trigger_volatile: f64,
    /// Khoảng đệm bảo vệ vốn cho Xung lực (0.5 -> 2.5 ATR, Step 0.10)
    #[dna(min = 0.5, max = 2.5, step = 0.10)]
    pub be_buffer_atr_volatile: f64,

    // =========================================================================
    // 5. BỘ 3 PHÂN TẦNG ATR TRAILING STOP PHỔ QUÁT (0.5 -> 15.0 ATR KHÔNG CHỒNG CHÉO)
    // =========================================================================
    /// - Phân Tầng Siết Chặt: Dành cho thị trường Range / Reversal / Đỉnh sóng (0.5 -> 3.0 ATR, Step 0.10 -> 26 nấc)
    #[dna(min = 0.5, max = 3.0, step = 0.10)]
    pub tsl_atr_tight: f64,

    /// - Phân Tầng Chuẩn: Dành cho xu hướng bậc thang Trend Pullback & Absorption (3.0 -> 7.5 ATR, Step 0.10 -> 46 nấc)
    #[dna(min = 3.0, max = 7.5, step = 0.10)]
    pub tsl_atr_normal: f64,

    /// - Phân Tầng Nới Lỏng: Dành cho siêu sóng bùng nổ Momentum & Anomaly (7.5 -> 15.0 ATR, Step 0.15 -> 51 nấc)
    #[dna(min = 7.5, max = 15.0, step = 0.15)]
    pub tsl_atr_loose: f64,

    // =========================================================================
    // 6. CƠ CHẾ BẢO VỆ SIÊU SÓNG (MEGA TREND PROTECTION)
    // =========================================================================
    /// Ngưỡng Lợi Nhuận Kích Hoạt Chế Độ Bảo Vệ Siêu Sóng (1.0% -> 20.0% giá, tương đương +20% -> +400% ROE trên 20x, Step 0.5% -> 39 nấc)
    #[dna(min = 0.01, max = 0.20, step = 0.005)]
    pub mega_trend_pnl_trigger: f64,

    /// Số Chu Kỳ Tối Đa Được Hưởng Chế Độ Bảo Vệ Thả Diều (Tham số dự phòng: Mặc định 10 chu kỳ)
    #[serde(default = "default_mega_trend_max_cycles", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub mega_trend_max_cycles: usize,

    /// Tỷ Lệ Khóa Lợi Nhuận Đỉnh Sóng Khi Kích Hoạt Mega Trend (20% -> 80% đỉnh lãi, Step 5% -> 13 nấc)
    #[dna(min = 0.20, max = 0.80, step = 0.05)]
    pub mega_trend_lock_ratio: f64,

    /// Mức ROE Sàn Tối Thiểu Được Khóa Khi Kích Hoạt Mega Trend (10% -> 50% ROE, Step 5% -> 9 nấc)
    #[dna(min = 0.10, max = 0.50, step = 0.05)]
    pub mega_trend_min_lock_roe: f64,

    // =========================================================================
    // 7. THAM SỐ PHỤ TRỢ & TÍCH LŨY & COOLDOWN
    // =========================================================================
    /// Thời Gian Ngâm Tối Đa Trước Khi Giải Phóng Vốn (60 nến -> 720 nến / 12 tiếng, Step 30 nến -> 23 nấc)
    #[dna(min = 60.0, max = 720.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub max_stagnant_hold_bars: usize,

    /// Lợi Nhuận Tối Thiểu Để Giữ Lệnh Ngâm Không Bị Đóng Sớm (0.2% -> 5.0% giá, Step 0.2% -> 25 nấc)
    #[dna(min = 0.002, max = 0.050, step = 0.002)]
    pub stagnant_min_profit_pct: f64,

    /// Khoảng Cách Cắt Lỗ Ban Đầu Mặc Định Theo ATR (0.5 -> 6.0 ATR, Step 0.10 -> 56 nấc)
    #[dna(min = 0.5, max = 6.0, step = 0.10)]
    pub base_sl_atr: f64,

    /// Cờ Bật/Tắt Cơ Chế Giãn Cách Cooldown Sau Khi Hòa Vốn / Cắt Lỗ (true: BẬT, false: TẮT)
    #[serde(default = "default_true")]
    pub enable_re_entry_cooldown: bool,

    /// Thời Gian Giãn Cách Sau Khi Hòa Vốn Chống Cào Lệnh (0 -> 60 nến, Step 5 nến -> 13 nấc)
    #[dna(min = 0.0, max = 60.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub re_entry_cooldown_bars: usize,

    /// Hệ Số ATR Bứt Phá Định Hướng Để Giải Phóng Sớm Khỏi Cooldown (0.5 -> 4.0 ATR, Step 0.25 -> 15 nấc)
    #[dna(min = 0.5, max = 4.0, step = 0.25)]
    #[serde(default = "default_cooldown_breakout_atr_mult")]
    pub cooldown_breakout_atr_mult: f64,

    // =========================================================================
    // 8. CƠ CHẾ CHỐT LỜI ĐỘNG LAI (HYBRID TRAILING TAKE PROFIT - FLAW-TSL.1)
    // =========================================================================
    /// Ngưỡng ROE Kích Hoạt Radar Thả Diều Bám Đỉnh Cho Trend Scanners (10% -> 50% ROE, Step 5% -> 9 nấc)
    #[dna(min = 0.10, max = 0.50, step = 0.05)]
    #[serde(default = "default_ttp_activation_roe")]
    pub ttp_activation_roe: f64,

    /// Khoảng Cách Quay Đầu Từ Đỉnh Theo ATR Để Chốt Lời Khóa Lãi (0.4 -> 2.0 ATR, Step 0.10 -> 17 nấc)
    #[dna(min = 0.4, max = 2.0, step = 0.10)]
    #[serde(default = "default_ttp_callback_atr")]
    pub ttp_callback_atr: f64,

    /// Sàn Bảo Vệ Lãi Tối Thiểu Khi Dải Bollinger Co Hẹp Chống Chốt Lỗ (0.1% -> 0.8% giá, Step 0.05% -> 15 nấc)
    #[dna(min = 0.001, max = 0.008, step = 0.0005)]
    #[serde(default = "default_dynamic_tp_min_gain_pct")]
    pub dynamic_tp_min_gain_pct: f64,
}

fn default_cooldown_breakout_atr_mult() -> f64 {
    1.5
}

fn default_sl_roe_cap() -> f64 {
    0.20
}

fn default_ttp_activation_roe() -> f64 {
    0.20
}

fn default_ttp_callback_atr() -> f64 {
    0.80
}

fn default_dynamic_tp_min_gain_pct() -> f64 {
    0.002
}

fn default_tsl_buffer_floor_atr() -> f64 {
    1.0
}

fn default_mega_trend_max_cycles() -> usize {
    10
}

fn default_true() -> bool {
    true
}

fn default_pullback_sl_buffer_atr() -> f64 {
    0.5
}

fn default_momentum_sl_buffer_atr() -> f64 {
    0.50
}

fn default_momentum_min_clv() -> f64 {
    0.20
}

fn default_momentum_max_adverse_slope() -> f64 {
    0.15
}

fn default_momentum_w_breakout() -> f64 {
    0.40
}

fn default_momentum_w_volume() -> f64 {
    0.30
}

fn default_range_sl_buffer_atr() -> f64 {
    0.3
}

fn default_absorption_sl_buffer_atr() -> f64 {
    0.50
}

fn default_absorption_max_adverse_slope() -> f64 {
    0.15
}

fn default_absorption_spring_tolerance() -> f64 {
    0.10
}

fn default_absorption_min_clv() -> f64 {
    0.20
}

fn default_absorption_max_boundary_pct_b() -> f64 {
    0.40
}

fn default_absorption_squeeze_weight() -> f64 {
    0.50
}

fn default_range_max_bb_slope() -> f64 {
    0.10
}

fn default_range_tp_target_ratio() -> f64 {
    0.50
}

fn default_pullback_rsi_max_cap() -> f64 {
    68.0
}

fn default_pullback_max_dist_atr() -> f64 {
    2.0
}

fn default_pullback_invalidation_buffer_atr() -> f64 {
    0.50
}

fn default_reversal_min_clv() -> f64 {
    0.20
}

fn default_reversal_mfi_thresh() -> f64 {
    30.0
}

fn default_reversal_w_z_score() -> f64 {
    0.40
}

fn default_reversal_w_displacement() -> f64 {
    0.40
}

#[derive(Debug, Clone, Deserialize, Default, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}

