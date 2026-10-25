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
    #[serde(default)]
    #[dna(skip)]
    pub macro_regime: MacroRegimeConfig,
    #[dna(nest)]
    #[serde(default)]
    pub circuit_breaker: FlashCrashBreakerConfig,
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
    #[serde(default)]
    pub micro_natr: MicroNAtrParams,
    #[dna(nest)]
    pub mfi: MfiParams,
    #[dna(nest)]
    pub vol_ma: VolMaParams,
    #[dna(nest)]
    pub vwma: VwmaParams,

    #[dna(nest)]
    pub price_z_score: PriceZScoreParams,
    #[dna(nest)]
    pub vol_z_score: VolZScoreParams,
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
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

impl Default for KamaParams {
    fn default() -> Self {
        Self {
            short_period: 10,
            long_period: 30,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct AdxParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for AdxParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct RsiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize, 
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for RsiParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct BbandsParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub nbdev: f64,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for BbandsParams {
    fn default() -> Self {
        Self {
            period: 20,
            nbdev: 2.0,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct KeltnerParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(min = 1.0, max = 5.0, step = 1.0)]
    pub multiplier: f64,
    #[dna(skip)]
    #[serde(default = "default_timeframe")]
    pub timeframe: Timeframe,
    #[dna(options="SMA,EMA,WMA,HMA,RMA,DMA,DEMA,TMA,TEMA,WSMA,SMM,SWMA,TRIMA,LinReg,Vidya")]
    pub ma_type: MaType,
}

impl Default for KeltnerParams {
    fn default() -> Self {
        Self {
            period: 20,
            multiplier: 1.5,
            timeframe: Timeframe::T15m,
            ma_type: MaType::EMA,
        }
    }
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

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
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

impl Default for StochasticParams {
    fn default() -> Self {
        Self {
            period: 14,
            k_period: 3,
            d_period: 3,
            k_ma_type: MaType::SMA,
            d_ma_type: MaType::SMA,
            zone: 0.20,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct AtrParams {
    #[dna(min = 5.0, max = 60.0, step = 1.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "1m, 3m, 5m")]
    pub timeframe: Timeframe,
}

impl Default for AtrParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T5m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct NAtrParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for NAtrParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct MicroNAtrParams {
    #[dna(min = 5.0, max = 30.0, step = 1.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "1m, 3m, 5m")]
    pub timeframe: Timeframe,
}

impl Default for MicroNAtrParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T1m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct MfiParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for MfiParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T15m,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VolMaParams {
    #[serde(default = "default_vol_ma_period", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[serde(default = "default_vol_ma_timeframe")]
    pub timeframe: Timeframe,
}

fn default_vol_ma_period() -> usize { 20 }
fn default_vol_ma_timeframe() -> Timeframe { Timeframe::T1m }

impl Default for VolMaParams {
    fn default() -> Self {
        Self {
            period: default_vol_ma_period(),
            timeframe: default_vol_ma_timeframe(),
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

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct VwmaParams {
    #[dna(min = 8.0, max = 80.0, step = 3.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "5m, 15m, 1h, 4h")]
    pub timeframe: Timeframe,
}

impl Default for VwmaParams {
    fn default() -> Self {
        Self {
            period: 20,
            timeframe: Timeframe::T15m,
        }
    }
}

/// Cảm biến Độ lệch chuẩn Giá Vi mô (Dùng cho Boundary Guard & Reversal)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct PriceZScoreParams {
    #[dna(min = 10.0, max = 300.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub lookback: usize,
    #[dna(subset = "1m, 3m, 5m")]
    pub timeframe: Timeframe,
}

impl Default for PriceZScoreParams {
    fn default() -> Self {
        Self {
            lookback: 50,
            timeframe: Timeframe::T5m,
        }
    }
}

/// Cảm biến Đột biến Khối lượng Vi mô (Dùng cho Momentum Ignition & Absorption)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct VolZScoreParams {
    #[dna(min = 20.0, max = 400.0, step = 5.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub lookback: usize,
    #[dna(subset = "1m, 3m, 5m")]
    pub timeframe: Timeframe,
}

impl Default for VolZScoreParams {
    fn default() -> Self {
        Self {
            lookback: 50,
            timeframe: Timeframe::T5m,
        }
    }
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct MatrixConfig {
    /// 🚀 Hệ số co giãn độ dốc KAMA chuẩn hóa theo ATR qua Tanh scaling (0.02 -> 1.00, Step 0.02 -> 50 nấc)
    #[dna(min = 0.02, max = 1.00, step = 0.02)]
    #[serde(default = "default_kama_slope_scale_atr")]
    pub kama_slope_scale_atr: f64,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            kama_slope_scale_atr: default_kama_slope_scale_atr(),
        }
    }
}

fn default_kama_slope_scale_atr() -> f64 { 0.20 }

/// ============================================================================
/// PHẦN 2.5: BỘ PHÂN LOẠI CHẾ ĐỘ THỊ TRƯỜNG VĨ MÔ (MACRO REGIME CONFIG)
/// ============================================================================
/// Cấu hình các ngưỡng chuyển pha chế độ thị trường trực giao (KER, ADX, Squeeze).
/// Được cố định theo chuẩn vật lý và thống kê, không cấp gen DNA để bảo toàn 104 chiều.
#[derive(Debug, Clone, Deserialize)]
pub struct MacroRegimeConfig {
    /// Ngưỡng Kaufman Efficiency Ratio tối thiểu để xác nhận xu hướng có trớn (0.35)
    #[serde(default = "default_min_trending_er")]
    pub min_trending_er: f64,

    /// Ngưỡng Welles Wilder ADX tối thiểu để xác nhận xu hướng mạnh (25.0)
    #[serde(default = "default_min_trending_adx")]
    pub min_trending_adx: f64,

    /// Ngưỡng Squeeze tối thiểu để coi là thị trường nén chặt đi ngang (0.65)
    #[serde(default = "default_range_squeeze_threshold", alias = "range_squeeze_percentile")]
    pub range_squeeze_threshold: f64,

    /// Ngưỡng Squeeze tối đa để kích hoạt bứt phá nến đầu Breakout (0.35)
    #[serde(default = "default_breakout_squeeze_threshold", alias = "breakout_squeeze_percentile")]
    pub breakout_squeeze_threshold: f64,

    /// Ngưỡng Welles Wilder ADX tối thiểu để xác nhận xung lực bứt phá Breakout (20.0)
    #[serde(default = "default_min_breakout_adx")]
    pub min_breakout_adx: f64,
}

impl Default for MacroRegimeConfig {
    fn default() -> Self {
        Self {
            min_trending_er: default_min_trending_er(),
            min_trending_adx: default_min_trending_adx(),
            range_squeeze_threshold: default_range_squeeze_threshold(),
            breakout_squeeze_threshold: default_breakout_squeeze_threshold(),
            min_breakout_adx: default_min_breakout_adx(),
        }
    }
}

fn default_min_trending_er() -> f64 { 0.35 }
fn default_min_trending_adx() -> f64 { 25.0 }
fn default_range_squeeze_threshold() -> f64 { 0.65 }
fn default_breakout_squeeze_threshold() -> f64 { 0.35 }
fn default_min_breakout_adx() -> f64 { 20.0 }

/// ============================================================================
/// PHẦN 2.6: CẦU DAO KHẨN CẤP FLASH CRASH & LIQUIDITY SHOCK (CIRCUIT BREAKER CONFIG)
/// ============================================================================
/// Cấu hình các ngưỡng ngắt mạch khẩn cấp phát hiện nến sập/bơm dị thường và cạn kiệt sổ lệnh vi mô.
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct FlashCrashBreakerConfig {
    /// ⚡ Ngưỡng biên độ nến (span / atr) kết hợp volume để xác nhận bão thanh lý (2.5 -> 6.0, step 0.25)
    #[dna(min = 2.5, max = 6.0, step = 0.25)]
    #[serde(default = "default_atr_shock_threshold")]
    pub atr_shock_threshold: f64,

    /// ⚡ Ngưỡng đột biến khối lượng Z-Score để xác nhận xả lũ (2.5 -> 5.0, step 0.25)
    #[dna(min = 2.5, max = 5.0, step = 0.25)]
    #[serde(default = "default_vol_z_shock_threshold")]
    pub vol_z_shock_threshold: f64,

    /// ⚡ Ngưỡng biên độ nến rỗng thanh khoản độc lập không cần volume (3.5 -> 7.0, step 0.5)
    #[dna(min = 3.5, max = 7.0, step = 0.5)]
    #[serde(default = "default_vacuum_atr_threshold")]
    pub vacuum_atr_threshold: f64,

    /// ⚡ Số nến đóng băng khẩn cấp phong tỏa mở vị thế mới (10 -> 40 nến 1m, step 5)
    #[dna(min = 10.0, max = 40.0, step = 5.0)]
    #[serde(default = "default_shock_freeze_bars", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub freeze_bars: usize,

    /// 🌪️ Ngưỡng biên độ nến giãn nở tối thiểu (span / atr) để phát hiện cú sốc tiêu tán (1.0 -> 2.5, step 0.1)
    #[dna(min = 1.0, max = 2.5, step = 0.1)]
    #[serde(default = "default_dissipation_span_threshold")]
    pub dissipation_span_threshold: f64,

    /// 🌪️ Ngưỡng tỷ lệ thân nến tối đa (|close - open| / span) để phát hiện cạn kiệt động năng (0.15 -> 0.45, step 0.05)
    #[dna(min = 0.15, max = 0.45, step = 0.05)]
    #[serde(default = "default_dissipation_max_body_ratio")]
    pub dissipation_max_body_ratio: f64,
}

impl Default for FlashCrashBreakerConfig {
    fn default() -> Self {
        Self {
            atr_shock_threshold: default_atr_shock_threshold(),
            vol_z_shock_threshold: default_vol_z_shock_threshold(),
            vacuum_atr_threshold: default_vacuum_atr_threshold(),
            freeze_bars: default_shock_freeze_bars(),
            dissipation_span_threshold: default_dissipation_span_threshold(),
            dissipation_max_body_ratio: default_dissipation_max_body_ratio(),
        }
    }
}

fn default_atr_shock_threshold() -> f64 { 3.5 }
fn default_vol_z_shock_threshold() -> f64 { 4.0 }
fn default_vacuum_atr_threshold() -> f64 { 4.5 }
fn default_shock_freeze_bars() -> usize { 20 }
fn default_dissipation_span_threshold() -> f64 { 1.5 }
fn default_dissipation_max_body_ratio() -> f64 { 0.35 }


/// ============================================================================
/// PHẦN 3: BỘ TRỌNG TÀI & ĐẤU GIÁ TÍN HIỆU (ARBITER CONFIDENCE AUCTION CONFIG)
/// ============================================================================
/// Đã được De-overengineer sang mô hình chấm điểm tuyến tính O(1).
/// Loại bỏ 5 gen ma và 7 Multipliers phi tuyến.
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct ArbiterConfig {
    /// 🎯 NGƯỠNG PHÊ DUYỆT TỰ TIN TUYẾN TÍNH DUY NHẤT (0.05 -> 0.50, Step 0.01 -> 46 nấc)
    #[dna(min = 0.05, max = 0.50, step = 0.01)]
    #[serde(default = "default_min_confidence_threshold")]
    pub min_confidence_threshold: f64,

    /// 🌪️ NGƯỠNG CẮT CẦU DAO BÃO HỖN LOẠN (0.50 -> 0.85, Step 0.05 -> 8 nấc)
    #[dna(min = 0.50, max = 0.85, step = 0.05)]
    #[serde(default = "default_chaos_veto_threshold")]
    pub chaos_veto_threshold: f64,

    /// 📉 BIÊN LỢI NHUẬN RÒNG VI MÔ TỐI THIỂU (DNA Net Alpha) VƯỢT TRÊN SÀN CHI PHÍ SINH TỒN (0.0002 -> 0.0300, Step 0.0002 -> 150 nấc)
    /// Effective_Min_NATR = 2 * taker_fee_rate + slippage_allowance_pct + min_natr_threshold
    /// 🛡️ TODO-AUDIT-NEW-61 (H-CFG-06): Phủ trọn phổ biến động từ Forex (~0.02%) đến Memecoin (~3.00%).
    /// Sàn chi phí thực tế (roundtrip_fee_pct) trong gatekeeper.rs luôn là chốt chặn độc lập, không bao giờ bị thủng.
    #[dna(min = 0.0002, max = 0.0300, step = 0.0002)]
    #[serde(default = "default_min_natr_threshold")]
    pub min_natr_threshold: f64,

    /// 🛡️ BIÊN ĐỘ HỒI PHỤC AN TOÀN HYSTERESIS (Mặc định 0.10)
    #[serde(default = "default_chaos_hysteresis_delta")]
    pub chaos_hysteresis_delta: f64,

    /// 🌪️ TRẦN PHẠT BÃO HỖN LOẠN CŨ (ĐÃ BỎ GEN DNA ĐỂ DE-OVERENGINEER SANG HARD GATE)
    #[serde(default = "default_max_chaos_penalty")]
    pub max_chaos_penalty: f64,

    /// ⚖️ BIÊN ĐỘ PHÂN XỬ XUNG ĐỘT LONG VS SHORT (Mặc định 0.15)
    #[serde(default = "default_conflict_confidence_margin")]
    pub conflict_confidence_margin: f64,

    /// 💰 HỆ SỐ NGƯỠNG LỢI NHUẬN KỲ VỌNG TRÊN CHI PHÍ MA SÁT SÀN (TODO-AUDIT-NEW-30 / TODO-ARB-05)
    /// Yêu cầu biên độ lợi nhuận tối thiểu phải gấp min_ev_hurdle_ratio lần chi phí ma sát 2 chiều (2 * fee + slippage).
    /// Mặc định 2.50 (tương đương 0.25% biên độ giá cơ sở hay 5.0% ROE ở 20x đòn bẩy khi fee = 0.04%, slippage = 0.02%).
    #[serde(default = "default_min_ev_hurdle_ratio")]
    pub min_ev_hurdle_ratio: f64,

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

impl Default for ArbiterConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: default_min_confidence_threshold(),
            chaos_veto_threshold: default_chaos_veto_threshold(),
            min_natr_threshold: default_min_natr_threshold(),
            chaos_hysteresis_delta: default_chaos_hysteresis_delta(),
            max_chaos_penalty: default_max_chaos_penalty(),
            conflict_confidence_margin: default_conflict_confidence_margin(),
            min_ev_hurdle_ratio: default_min_ev_hurdle_ratio(),
            min_multiplier_veto: default_min_multiplier_veto(),
            scanner_weight: default_scanner_weight(),
            min_confidence_trending: default_min_confidence_trending(),
            min_confidence_ranging: default_min_confidence_ranging(),
            min_confidence_compression: default_min_confidence_compression(),
            min_confidence_chaos: default_min_confidence_chaos(),
        }
    }
}

fn default_min_confidence_threshold() -> f64 { 0.20 }
fn default_chaos_veto_threshold() -> f64 { 0.65 }
fn default_min_natr_threshold() -> f64 { 0.0012 }
fn default_chaos_hysteresis_delta() -> f64 { 0.10 }
fn default_max_chaos_penalty() -> f64 { 0.50 }
fn default_conflict_confidence_margin() -> f64 { 0.15 }
fn default_min_ev_hurdle_ratio() -> f64 { 2.50 }
fn default_min_multiplier_veto() -> f64 { 0.01 }
fn default_scanner_weight() -> f64 { 0.50 }
fn default_min_confidence_trending() -> f64 { 0.20 }
fn default_min_confidence_ranging() -> f64 { 0.20 }
fn default_min_confidence_compression() -> f64 { 0.20 }
fn default_min_confidence_chaos() -> f64 { 0.20 }
fn default_min_activation_score() -> f64 { 0.05 }

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
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalPullbackParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
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

    /// Ngưỡng kích hoạt mềm Trend Pullback (Sàn kỹ thuật 0.05)
    #[serde(default = "default_min_activation_score")]
    pub min_activation_score: f64,

    /// Đệm ATR bảo vệ qua KAMA / đáy nhịp hồi (0.10 -> 3.00 ATR, Step 0.15 -> 20 nấc)
    #[dna(min = 0.10, max = 3.00, step = 0.15)]
    #[serde(default = "default_pullback_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🛡️ CHỐT CHẶN BIÊN ĐỘ Z-SCORE (0.50 -> 3.00, Step 0.10 -> 26 nấc)
    /// Long cấm khi Z > +max_z (đu đỉnh Climax), Short cấm khi Z < -max_z (đu đáy hoảng loạn).
    /// Mặc định 1.60 sigma tiệm cận trần xu hướng tuyến tính sqrt(3) ≈ 1.732 sigma.
    #[dna(min = 0.50, max = 3.00, step = 0.10)]
    #[serde(default = "default_pullback_max_boundary_z")]
    pub max_boundary_z: f64,

    /// 🧱 CHỐT CHẶN VÙNG CO THẮT CHOP ZONE (0.50 -> 0.90, Step 0.05 -> 9 nấc)
    /// Cấm mở vị thế khi BB/KC Squeeze vượt ngưỡng (nhường sân cho range_osc).
    #[dna(min = 0.50, max = 0.90, step = 0.05)]
    #[serde(default = "default_pullback_max_squeeze_intensity")]
    pub max_squeeze_intensity: f64,

    /// 11. Ngưỡng Phân Vị Lực Nến Tối Thiểu CLV (Close Location Value từ 0.05 -> 0.50, Step 0.05 -> 10 nấc)
    #[dna(min = 0.05, max = 0.50, step = 0.05)]
    #[serde(default = "default_pullback_min_clv")]
    pub min_clv: f64,

    /// 12. Ngưỡng Cực Trị Thác Lũ Thanh Lý / Bắt Dao Rơi (5.0 -> 25.0, Step 2.0 -> 11 nấc)
    /// Chỉ khi Stochastic rơi vào vùng xả lũ cực đoan này mới kích hoạt Soft Hook Guard.
    #[dna(min = 5.0, max = 25.0, step = 2.0)]
    #[serde(default = "default_pullback_stoch_climax_thresh")]
    pub stoch_climax_thresh: f64,
}

impl Default for TacticalPullbackParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            rsi_max_cap: default_pullback_rsi_max_cap(),
            stoch_pullback_thresh: 30.0,
            kama_slope_threshold: 0.05,
            max_pullback_dist_atr: default_pullback_max_dist_atr(),
            invalidation_buffer_atr: default_pullback_invalidation_buffer_atr(),
            min_activation_score: default_min_activation_score(),
            struct_sl_buffer_atr: default_pullback_sl_buffer_atr(),
            max_boundary_z: default_pullback_max_boundary_z(),
            max_squeeze_intensity: default_pullback_max_squeeze_intensity(),
            min_clv: default_pullback_min_clv(),
            stoch_climax_thresh: default_pullback_stoch_climax_thresh(),
        }
    }
}

/// ⚡ 2. CHIẾN ĐỘI MOMENTUM IGNITION (Săn các cú bùng nổ xung lực Volume & Momentum)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalMomentumParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
    pub max_initial_sl_pct: f64,

    /// RSI Xung Lực Bùng Nổ (52.0 -> 88.0, Step 1.0 -> 37 nấc)
    #[dna(min = 52.0, max = 88.0, step = 1.0)]
    pub rsi_threshold: f64,

    /// Ngưỡng Z-Score bùng nổ xung lực khối lượng (1.00 -> 3.50, Step 0.10 -> 26 nấc)
    #[dna(min = 1.00, max = 3.50, step = 0.10)]
    pub vol_z_threshold: f64,

    /// Ngưỡng kích hoạt mềm Momentum Ignition (Sàn kỹ thuật 0.05)
    #[serde(default = "default_min_activation_score")]
    pub min_activation_score: f64,

    /// Đệm ATR dưới đáy nến bùng nổ xung lực (0.20 -> 2.00 ATR, Step 0.10 -> 19 nấc)
    #[dna(min = 0.20, max = 2.00, step = 0.10)]
    #[serde(default = "default_momentum_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🎯 Ngưỡng lực nến rút chân/đè đầu CLV tối thiểu (0.10 -> 0.60, Step 0.05 -> 11 nấc)
    #[dna(min = 0.10, max = 0.60, step = 0.05)]
    #[serde(default = "default_momentum_min_clv")]
    pub min_clv: f64,

    /// 🛡️ Ngưỡng dốc xu hướng KAMA tối đa cấm bùng nổ ngược sóng (0.00 -> 0.05, Step 0.01 -> 6 nấc)
    #[dna(min = 0.00, max = 0.05, step = 0.01)]
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

    /// ⏳ [TODO-AUDIT-NEW-47] Thời gian sống tối đa của xung lực Momentum Ignition trước khi kích hoạt Stagnation Exit (Mặc định: 4 nến)
    #[serde(default = "default_momentum_stagnation_bars", deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub stagnation_exit_bars: usize,

    /// 🎯 [TODO-AUDIT-NEW-47] Ngưỡng ROE tối đa để kích hoạt Stagnation Exit khi bứt phá xịt (Mặc định: +2.0% ROE)
    #[serde(default = "default_momentum_stagnation_max_roe")]
    pub stagnation_max_roe: f64,
}

impl Default for TacticalMomentumParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            rsi_threshold: 65.0,
            vol_z_threshold: 2.0,
            min_activation_score: default_min_activation_score(),
            struct_sl_buffer_atr: default_momentum_sl_buffer_atr(),
            min_clv: default_momentum_min_clv(),
            max_adverse_slope: default_momentum_max_adverse_slope(),
            w_breakout: default_momentum_w_breakout(),
            w_volume: default_momentum_w_volume(),
            stagnation_exit_bars: default_momentum_stagnation_bars(),
            stagnation_max_roe: default_momentum_stagnation_max_roe(),
        }
    }
}

/// 🌊 3. CHIẾN ĐỘI RANGE OSCILLATOR (Bắt đỉnh/đáy biên độ thị trường đi ngang)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalRangeParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
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

impl Default for TacticalRangeParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            bb_proximity_pct: 0.01,
            stoch_oversold: 20.0,
            stoch_overbought: 80.0,
            min_width_pct: 0.01,
            volume_max_threshold: 0.50,
            min_activation_score: 0.05,
            max_trend_strength: 0.30,
            max_adverse_slope: 0.15,
            struct_sl_buffer_atr: default_range_sl_buffer_atr(),
            max_bb_slope: default_range_max_bb_slope(),
            tp_target_ratio: default_range_tp_target_ratio(),
        }
    }
}

/// 🔄 4. CHIẾN ĐỘI CLIMAX REVERSAL (Bắt đỉnh/đáy kiệt sức Parabol)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalReversalParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
    pub max_initial_sl_pct: f64,

    /// Độ lệch chuẩn giá cực đại Z-Score (0.5 -> 5.0 sigma, Step 0.2 -> 23 nấc)
    #[dna(min = 0.5, max = 5.0, step = 0.2)]
    pub z_score_threshold: f64,

    /// Biên độ nến đảo chiều kiệt sức (0.5 -> 6.0 ATR, Step 0.25 -> 23 nấc)
    #[dna(min = 0.5, max = 6.0, step = 0.25)]
    pub min_reversal_displacement: f64,

    /// Ngưỡng tỷ số khối lượng bùng nổ cao trào Climax Reversal (1.00 -> 3.50 x SMA, Step 0.10 -> 26 nấc)
    #[dna(min = 1.00, max = 3.50, step = 0.10)]
    #[serde(default = "default_reversal_climax_vol_threshold")]
    pub climax_vol_threshold: f64,

    /// Đệm ATR bảo vệ qua râu nến (0.2 -> 6.0 ATR, Step 0.2 -> 30 nấc)
    #[dna(min = 0.20, max = 6.00, step = 0.20)]
    pub wick_sl_buffer_atr: f64,

    /// Ngưỡng kích hoạt mềm Climax Reversal (Sàn kỹ thuật 0.05)
    #[serde(default = "default_min_activation_score")]
    pub min_activation_score: f64,

    /// 🎯 Ngưỡng lực nến rút chân/đè đầu CLV tối thiểu (0.05 -> 0.60, Step 0.05 -> 12 nấc)
    #[dna(min = 0.05, max = 0.60, step = 0.05)]
    #[serde(default = "default_reversal_min_clv")]
    pub min_reversal_clv: f64,

    /// 🛡️ Ngưỡng tỷ lệ râu nến từ chối cực trị tối thiểu (Excess Wick: 0.10 -> 0.50, Step 0.05 -> 9 nấc)
    #[dna(min = 0.10, max = 0.50, step = 0.05)]
    #[serde(default = "default_reversal_min_excess_wick_ratio")]
    pub min_excess_wick_ratio: f64,

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

    /// 🎯 Ngưỡng tỷ lệ R:R tối thiểu (TP Dist / SL Dist >= 1.20)
    #[serde(default = "default_reversal_min_rr_ratio")]
    pub min_rr_ratio: f64,

    /// 🌪️ REV-MACRO-GATE: Độ dốc KAMA nghịch đảo tối đa cấm bắt dao rơi (0.02 -> 0.80, Step 0.02 -> 40 nấc, mặc định 0.20)
    #[dna(min = 0.02, max = 0.80, step = 0.02)]
    #[serde(default = "default_reversal_max_adverse_kama_slope")]
    pub max_adverse_kama_slope: f64,

    /// 🌪️ REV-MACRO-GATE: Cường độ xu hướng ADX nghịch chiều tối đa cấm cản đầu tàu (15.0 -> 75.0, Step 5.0 -> 13 nấc, mặc định 35.0)
    #[dna(min = 15.0, max = 75.0, step = 5.0)]
    #[serde(default = "default_reversal_max_adverse_adx")]
    pub max_adverse_adx: f64,

    /// ⚡ Chỉ số bão hỗn loạn Whipsaw tối đa (mặc định 0.65)
    #[serde(default = "default_reversal_max_chaos_index")]
    pub max_chaos_index: f64,
}

impl Default for TacticalReversalParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            z_score_threshold: 2.0,
            min_reversal_displacement: 1.5,
            climax_vol_threshold: default_reversal_climax_vol_threshold(),
            wick_sl_buffer_atr: 1.0,
            min_activation_score: default_min_activation_score(),
            min_reversal_clv: default_reversal_min_clv(),
            min_excess_wick_ratio: default_reversal_min_excess_wick_ratio(),
            mfi_reversal_thresh: default_reversal_mfi_thresh(),
            w_z_score: default_reversal_w_z_score(),
            w_displacement: default_reversal_w_displacement(),
            min_rr_ratio: default_reversal_min_rr_ratio(),
            max_adverse_kama_slope: default_reversal_max_adverse_kama_slope(),
            max_adverse_adx: default_reversal_max_adverse_adx(),
            max_chaos_index: default_reversal_max_chaos_index(),
        }
    }
}

/// 💥 5. CHIẾN ĐỘI ANOMALY FADE (Săn các râu nến quét thanh khoản dị thường)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalAnomalyParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
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

impl Default for TacticalAnomalyParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            wick_atr_threshold: 2.0,
            z_score_threshold: 2.5,
            wick_sl_buffer_atr: 1.0,
            min_activation_score: 0.05,
        }
    }
}

/// 🧲 6. CHIẾN ĐỘI ABSORPTION SCOUT (Phát hiện dòng tiền cá mập gom/xả hàng trong vùng nén)
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct TacticalAbsorptionParams {
    /// Kẹp trần rủi ro ROE ban đầu (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
    pub max_initial_sl_pct: f64,

    /// Độ nén dải Bollinger/Keltner (Universal 1% -> 99%, Step 3% -> 33 nấc)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub squeeze_threshold: f64,

    /// Z-Score Volume hấp thụ dòng tiền (0.2 -> 5.0 sigma, Step 0.2 -> 25 nấc)
    #[dna(min = 0.20, max = 5.00, step = 0.20)]
    pub vol_z_threshold: f64,

    /// Ngưỡng kích hoạt mềm Absorption Scout (Sàn kỹ thuật 0.05)
    #[serde(default = "default_min_activation_score")]
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

impl Default for TacticalAbsorptionParams {
    fn default() -> Self {
        Self {
            max_initial_sl_pct: 0.15,
            squeeze_threshold: 0.50,
            vol_z_threshold: 2.0,
            min_activation_score: default_min_activation_score(),
            struct_sl_buffer_atr: default_absorption_sl_buffer_atr(),
            max_adverse_slope: default_absorption_max_adverse_slope(),
            spring_tolerance: default_absorption_spring_tolerance(),
            min_clv: default_absorption_min_clv(),
            max_boundary_pct_b: default_absorption_max_boundary_pct_b(),
            squeeze_weight: default_absorption_squeeze_weight(),
        }
    }
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

    /// 8. Tỷ Lệ Phí Taker 1 Chiều Của Sàn/Asset (Mặc định: 0.0004 = 0.04% Binance Futures)
    #[serde(default = "default_taker_fee_rate")]
    pub taker_fee_rate: f64,

    /// 9. Biên Độ Trượt Giá An Toàn Của Asset (Mặc định: 0.0002 = 0.02% cho Top Coins)
    #[serde(default = "default_slippage_allowance_pct")]
    pub slippage_allowance_pct: f64,

    /// 10. Tỷ Lệ Ký Quỹ Duy Trì Của Sàn/Asset (MMR: Mặc định 0.01 = 1.0% cho Tier 1 Binance Futures)
    #[serde(default = "default_maintenance_margin_rate")]
    pub maintenance_margin_rate: f64,

    /// 11. Hệ Số Vùng Đệm An Toàn Thanh Lý Co Giãn (Mặc định 0.15 = 15% khoảng cách thanh lý)
    #[serde(default = "default_liquidation_buffer_ratio")]
    pub liquidation_buffer_ratio: f64,
}

fn default_percent_per_trade() -> f64 { 0.20 }
fn default_sl_min_dist_pct() -> f64 { 0.005 }
fn default_base_lot_size() -> f64 { 0.01 }
fn default_leverage() -> f64 { 20.0 }
fn default_nuke_leverage() -> f64 { 50.0 }
fn default_min_order_usd() -> f64 { 21.0 }
fn default_max_cap_fraction() -> f64 { 0.25 }
fn default_taker_fee_rate() -> f64 { 0.0004 }
fn default_slippage_allowance_pct() -> f64 { 0.0002 }
fn default_maintenance_margin_rate() -> f64 { 0.01 }
fn default_liquidation_buffer_ratio() -> f64 { 0.15 }

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
            taker_fee_rate: default_taker_fee_rate(),
            slippage_allowance_pct: default_slippage_allowance_pct(),
            maintenance_margin_rate: default_maintenance_margin_rate(),
            liquidation_buffer_ratio: default_liquidation_buffer_ratio(),
        }
    }
}

/// 🛡️ CƠ CHẾ CẮT LỖ & TRAILING STOP THÍCH ỨNG
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct RiskStopParams {
    /// 0. Khoảng Đệm ATR Tối Thiểu Khi Dời TSL Chống Râu Nến Liếm (0.4 -> 2.5 ATR, Step 0.10 -> 22 nấc)
    #[dna(min = 0.4, max = 2.5, step = 0.10)]
    #[serde(default = "default_tsl_buffer_floor_atr")]
    pub tsl_buffer_floor_atr: f64,

    /// 1. Sàn Cắt Lỗ Khởi Tạo Tối Thiểu (0.15% -> 4.0% giá, Step 0.05% -> 78 nấc)
    #[dna(min = 0.0015, max = 0.040, step = 0.0005)]
    pub min_initial_sl_pct: f64,

    /// 1.1 Trần Cắt Lỗ ROE Mặc Định Cho Lệnh Untagged / Adopted (4.0% -> 15.0% ROE, Step 1.0% -> 12 nấc)
    #[dna(min = 0.04, max = 0.15, step = 0.01)]
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

    // --- 4.1 NHÓM XU HƯỚNG (TREND REGIME: TREND_PB) ---
    /// Ngưỡng kích hoạt BE cho Trend (Lãi đạt 1.0 -> 4.5 ATR mới kéo SL, Step 0.10)
    #[dna(min = 1.0, max = 4.5, step = 0.10)]
    pub be_atr_trigger_trend: f64,
    /// Khoảng đệm bảo vệ vốn dưới đỉnh cho Trend (Giữ khoảng cách 0.8 -> 3.0 ATR cho giá thở, Step 0.10)
    #[dna(min = 0.8, max = 3.0, step = 0.10)]
    pub be_buffer_atr_trend: f64,

    // --- 4.1.1 NHÓM HẤP THỤ VI MÔ (ABSORPTION REGIME) ---
    /// Ngưỡng kích hoạt BE cho Absorption (0.8 -> 3.0 ATR, Step 0.10)
    #[dna(min = 0.8, max = 3.0, step = 0.10)]
    pub be_atr_trigger_absorption: f64,
    /// Khoảng đệm bảo vệ vốn cho Absorption (0.4 -> 2.0 ATR, Step 0.10)
    #[dna(min = 0.4, max = 2.0, step = 0.10)]
    pub be_buffer_atr_absorption: f64,

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
    // 5. BỘ 3 PHÂN TẦNG ATR TRAILING STOP THỨ BẬC BẤT BIẾN (TODO-AUDIT-NEW-60: TIGHT <= NORMAL <= LOOSE)
    // =========================================================================
    /// - Phân Tầng Siết Chặt: Dành cho thị trường Range / Reversal / Đỉnh sóng (0.40 -> 6.00 ATR, Step 0.10 -> 57 nấc)
    #[dna(min = 0.40, max = 6.00, step = 0.10)]
    pub tsl_atr_tight: f64,

    /// - Phân Tầng Chuẩn: Dành cho xu hướng bậc thang Trend Pullback & Absorption (0.40 -> 6.00 ATR, Step 0.10 -> 57 nấc)
    #[dna(min = 0.40, max = 6.00, step = 0.10)]
    pub tsl_atr_normal: f64,

    /// - Phân Tầng Nới Lỏng: Dành cho siêu sóng bùng nổ Momentum & Anomaly (0.40 -> 6.00 ATR, Step 0.10 -> 57 nấc)
    #[dna(min = 0.40, max = 6.00, step = 0.10)]
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
    /// Thời Gian Ngâm Tối Đa Trước Khi Giải Phóng Vốn (30 nến -> 360 nến / 6 tiếng, Step 30 nến -> 12 nấc)
    #[dna(min = 30.0, max = 360.0, step = 30.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub max_stagnant_hold_bars: usize,

    /// Lợi Nhuận Tối Thiểu Để Giữ Lệnh Ngâm Không Bị Đóng Sớm (0.2% -> 5.0% giá, Step 0.2% -> 25 nấc)
    #[dna(min = 0.002, max = 0.050, step = 0.002)]
    pub stagnant_min_profit_pct: f64,

    /// Khoảng Cách Cắt Lỗ Ban Đầu Mặc Định Theo ATR (Bỏ khỏi DNA theo TODO-RSK-02)
    #[dna(skip)]
    #[serde(default = "default_base_sl_atr")]
    pub base_sl_atr: f64,

    /// 🛡️ CƠ CHẾ CẦU DAO BẢO VỆ TÀI KHOẢN TẠI BOT ENGINE (ENGINE COOLDOWN SENTINEL - PURE TTL)
    /// Cờ Bật/Tắt Cơ Chế Giãn Cách Cooldown Tại Bot Engine (true: BẬT, false: TẮT)
    #[serde(default = "default_true")]
    pub enable_re_entry_cooldown: bool,

    /// Thời Gian Giãn Cách Cơ Sở Sau Lệnh Dính Hard SL (Cố định: 15 nến 1m = 15 phút)
    #[serde(default = "default_base_cooldown_bars")]
    pub re_entry_cooldown_bars: usize,

    /// Trần Thời Gian Giãn Cách Tối Đa Khi Dính Chuỗi Thua Liên Tiếp (Cố định: 45 nến = 45 phút)
    #[serde(default = "default_max_cooldown_cap_bars")]
    pub max_cooldown_cap_bars: usize,

    /// Thời Gian Giãn Cách Ngắn Sau Lệnh Hòa Vốn True BE Để Ổn Định Nến (Cố định: 3 nến = 3 phút)
    #[serde(default = "default_be_tolerance_cooldown_bars")]
    pub be_tolerance_cooldown_bars: usize,

    /// Khoảng Đệm Bù Phí Sàn 2 Chiều + Trượt Giá Khi Dời SL Hòa Vốn (Mặc định: 0.13% = 0.0013)
    #[serde(default = "default_true_be_roundtrip_fee_pct")]
    pub true_be_roundtrip_fee_pct: f64,

    /// Ngưỡng ROE Phân Định Lệnh Thua Nặng Cần Kích Hoạt Cầu Dao Đầy Đủ (Mặc định: -5.0% ROE)
    #[serde(default = "default_hard_loss_roe_threshold")]
    pub hard_loss_roe_threshold: f64,

    /// Hệ Số ATR Bứt Phá Định Hướng (Dự phòng cho phân tích ngoại vi)
    #[serde(default = "default_cooldown_breakout_atr_mult")]
    pub cooldown_breakout_atr_mult: f64,

    // =========================================================================
    // 8. CƠ CHẾ CHỐT LỜI ĐỘNG LAI (HYBRID TRAILING TAKE PROFIT - FLAW-TSL.1)
    // =========================================================================
    /// Ngưỡng ROE Kích Hoạt Radar Thả Diều Bám Đỉnh Cho Trend Scanners (5% -> 50% ROE, Step 1% -> 46 nấc)
    #[dna(min = 0.05, max = 0.50, step = 0.01)]
    #[serde(default = "default_ttp_activation_roe")]
    pub ttp_activation_roe: f64,

    /// Khoảng Cách Quay Đầu Từ Đỉnh Theo ATR Để Chốt Lời Khóa Lãi (0.3 -> 2.5 ATR, Step 0.05 -> 45 nấc)
    #[dna(min = 0.30, max = 2.50, step = 0.05)]
    #[serde(default = "default_ttp_callback_atr")]
    pub ttp_callback_atr: f64,

    /// Sàn Bảo Vệ Lãi Tối Thiểu Khi Dải Bollinger Co Hẹp Chống Chốt Lỗ (0.1% -> 0.8% giá, Step 0.05% -> 15 nấc)
    #[dna(min = 0.001, max = 0.008, step = 0.0005)]
    #[serde(default = "default_dynamic_tp_min_gain_pct")]
    pub dynamic_tp_min_gain_pct: f64,
}

impl RiskStopParams {
    /// 🛡️ TODO-AUDIT-NEW-60 (H-CFG-05): Khóa bất biến thứ bậc phân tầng Trailing Stop (tight <= normal <= loose)
    #[inline]
    pub fn enforce_tsl_hierarchy(&mut self) {
        if self.tsl_atr_tight > self.tsl_atr_normal {
            std::mem::swap(&mut self.tsl_atr_tight, &mut self.tsl_atr_normal);
        }
        if self.tsl_atr_normal > self.tsl_atr_loose {
            std::mem::swap(&mut self.tsl_atr_normal, &mut self.tsl_atr_loose);
        }
        if self.tsl_atr_tight > self.tsl_atr_normal {
            std::mem::swap(&mut self.tsl_atr_tight, &mut self.tsl_atr_normal);
        }
    }
}

impl Default for RiskStopParams {
    fn default() -> Self {
        Self {
            tsl_buffer_floor_atr: default_tsl_buffer_floor_atr(),
            min_initial_sl_pct: 0.01,
            default_sl_roe_cap: default_sl_roe_cap(),
            time_countdown_bars: 120,
            tighten_step_multiplier: 0.80,
            base_be_roe_trigger: 0.10,
            be_atr_trigger_trend: 2.0,
            be_buffer_atr_trend: 1.0,
            be_atr_trigger_absorption: 1.2,
            be_buffer_atr_absorption: 0.8,
            be_atr_trigger_range: 1.5,
            be_buffer_atr_range: 0.8,
            be_atr_trigger_volatile: 2.0,
            be_buffer_atr_volatile: 1.0,
            tsl_atr_tight: 1.5,
            tsl_atr_normal: 2.5,
            tsl_atr_loose: 3.5,
            mega_trend_pnl_trigger: 0.05,
            mega_trend_max_cycles: default_mega_trend_max_cycles(),
            mega_trend_lock_ratio: 0.50,
            mega_trend_min_lock_roe: 0.20,
            max_stagnant_hold_bars: 180,
            stagnant_min_profit_pct: 0.01,
            base_sl_atr: default_base_sl_atr(),
            enable_re_entry_cooldown: default_true(),
            re_entry_cooldown_bars: default_base_cooldown_bars(),
            max_cooldown_cap_bars: default_max_cooldown_cap_bars(),
            be_tolerance_cooldown_bars: default_be_tolerance_cooldown_bars(),
            true_be_roundtrip_fee_pct: default_true_be_roundtrip_fee_pct(),
            hard_loss_roe_threshold: default_hard_loss_roe_threshold(),
            cooldown_breakout_atr_mult: default_cooldown_breakout_atr_mult(),
            ttp_activation_roe: default_ttp_activation_roe(),
            ttp_callback_atr: default_ttp_callback_atr(),
            dynamic_tp_min_gain_pct: default_dynamic_tp_min_gain_pct(),
        }
    }
}

fn default_cooldown_breakout_atr_mult() -> f64 {
    1.5
}

fn default_sl_roe_cap() -> f64 {
    0.15
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
    0.02
}

fn default_momentum_w_breakout() -> f64 {
    0.40
}

fn default_momentum_w_volume() -> f64 {
    0.30
}

fn default_momentum_stagnation_bars() -> usize {
    4
}

fn default_momentum_stagnation_max_roe() -> f64 {
    0.02
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

fn default_pullback_max_boundary_z() -> f64 {
    1.60
}

fn default_pullback_max_squeeze_intensity() -> f64 {
    0.75
}

fn default_pullback_min_clv() -> f64 {
    0.20
}

fn default_pullback_stoch_climax_thresh() -> f64 {
    10.0
}

fn default_reversal_climax_vol_threshold() -> f64 {
    1.50
}

fn default_reversal_min_clv() -> f64 {
    0.20
}

fn default_reversal_min_excess_wick_ratio() -> f64 {
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

fn default_reversal_min_rr_ratio() -> f64 {
    1.20
}

fn default_reversal_max_adverse_kama_slope() -> f64 {
    0.40
}

fn default_reversal_max_adverse_adx() -> f64 {
    35.0
}

fn default_reversal_max_chaos_index() -> f64 {
    0.65
}

#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct WarmupConfig {
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub num: usize,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self { num: 50 }
    }
}

fn default_base_cooldown_bars() -> usize {
    15
}

fn default_max_cooldown_cap_bars() -> usize {
    45
}

fn default_be_tolerance_cooldown_bars() -> usize {
    3
}

fn default_true_be_roundtrip_fee_pct() -> f64 {
    0.0013
}

fn default_hard_loss_roe_threshold() -> f64 {
    -0.05
}

fn default_base_sl_atr() -> f64 {
    2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_trade::optimizer::DnaMappable;

    #[test]
    fn test_helios_config_v3_default_has_no_zero_periods() {
        let config = HeliosConfigV3::default();
        assert!(config.indicators.kama.short_period > 0);
        assert!(config.indicators.kama.long_period > 0);
        assert!(config.indicators.adx.period > 0);
        assert!(config.indicators.rsi_trend.period > 0);
        assert!(config.indicators.rsi_reversal.period > 0);
        assert!(config.indicators.rsi_momentum.period > 0);
        assert!(config.indicators.bbands.period > 0);
        assert!(config.indicators.keltner.period > 0);
        assert!(config.indicators.macd.fast > 0);
        assert!(config.indicators.macd.slow > 0);
        assert!(config.indicators.macd.signal > 0);
        assert!(config.indicators.stoch_trend.period > 0);
        assert!(config.indicators.atr.period > 0);
        assert!(config.indicators.natr.period > 0);
        assert!(config.indicators.mfi.period > 0);
        assert!(config.indicators.vol_ma.period > 0);
        assert!(config.indicators.vwma.period > 0);
        assert!(config.indicators.price_z_score.lookback > 0);
        assert!(config.indicators.vol_z_score.lookback > 0);
        assert!(config.matrix.kama_slope_scale_atr > 0.0);
        assert!(config.tactical_sniper.pullback.max_initial_sl_pct > 0.0);
        assert!(config.tactical_sniper.momentum.max_initial_sl_pct > 0.0);
        assert!(config.tactical_sniper.reversal.max_initial_sl_pct > 0.0);
        assert!(config.tactical_sniper.absorption.max_initial_sl_pct > 0.0);
        assert!(config.risk_management.stop_management.time_countdown_bars > 0);
    }

    #[test]
    fn test_price_and_vol_zscore_params_dna_independence() {
        assert_eq!(PriceZScoreParams::DNA_WIDTH, 2);
        assert_eq!(VolZScoreParams::DNA_WIDTH, 2);
        assert_eq!(AtrParams::DNA_WIDTH, 2);

        // Kiểm tra parameter definitions
        let p_defs = PriceZScoreParams::parameter_defs();
        assert_eq!(p_defs.len(), 2);
        assert_eq!(p_defs[0].0, "lookback");
        assert_eq!(p_defs[1].0, "timeframe");

        let v_defs = VolZScoreParams::parameter_defs();
        assert_eq!(v_defs.len(), 2);
        assert_eq!(v_defs[0].0, "lookback");
        assert_eq!(v_defs[1].0, "timeframe");

        // Kiểm tra IndicatorsConfig có chứa các khóa độc lập
        let ind_defs = IndicatorsConfig::parameter_defs();
        let has_price_lookback = ind_defs.iter().any(|(k, _)| k == "price_z_score.lookback");
        let has_price_tf = ind_defs.iter().any(|(k, _)| k == "price_z_score.timeframe");
        let has_vol_lookback = ind_defs.iter().any(|(k, _)| k == "vol_z_score.lookback");
        let has_vol_tf = ind_defs.iter().any(|(k, _)| k == "vol_z_score.timeframe");

        assert!(has_price_lookback, "Phải có indicators.price_z_score.lookback");
        assert!(has_price_tf, "Phải có indicators.price_z_score.timeframe");
        assert!(has_vol_lookback, "Phải có indicators.vol_z_score.lookback");
        assert!(has_vol_tf, "Phải có indicators.vol_z_score.timeframe");
    }

    #[test]
    fn test_micro_sensors_timeframe_subset() {
        let p_defs = PriceZScoreParams::parameter_defs();
        let p_tf_def = &p_defs[1].1;
        if let ferro_trade::optimizer::ParamDefinition::Options { values } = p_tf_def {
            let str_values: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
            assert!(str_values.contains(&"1m"));
            assert!(str_values.contains(&"3m"));
            assert!(str_values.contains(&"5m"));
            assert!(!str_values.contains(&"1h"), "Price Z-score không được chứa 1h");
            assert!(!str_values.contains(&"4h"), "Price Z-score không được chứa 4h");
        } else {
            panic!("timeframe phải là ParamDefinition::Options");
        }

        let atr_defs = AtrParams::parameter_defs();
        let atr_tf_def = &atr_defs[1].1;
        if let ferro_trade::optimizer::ParamDefinition::Options { values } = atr_tf_def {
            let str_values: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
            assert!(str_values.contains(&"1m"));
            assert!(str_values.contains(&"3m"));
            assert!(str_values.contains(&"5m"));
            assert!(!str_values.contains(&"1h"), "AtrParams không được chứa 1h");
            assert!(!str_values.contains(&"4h"), "AtrParams không được chứa 4h");
        } else {
            panic!("timeframe phải là ParamDefinition::Options");
        }

        let natr_defs = NAtrParams::parameter_defs();
        let natr_tf_def = &natr_defs[1].1;
        if let ferro_trade::optimizer::ParamDefinition::Options { values } = natr_tf_def {
            let str_values: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
            assert!(str_values.contains(&"15m"));
            assert!(str_values.contains(&"1h"));
            assert!(str_values.contains(&"4h"));
            assert!(!str_values.contains(&"1m"), "NAtrParams không được chứa 1m");
            assert!(!str_values.contains(&"5m"), "NAtrParams không được chứa 5m");
        } else {
            panic!("timeframe phải là ParamDefinition::Options");
        }
    }
}

