// strategies_backtests/helios/src/strategy/config.rs

use serde::Deserialize;
use ferro_trade::optimizer::config_parser::deserialize_usize_from_f64;
use ferro_trade::indicators::identifier::MaType;

/// CẤU TRÚC TỔNG THỂ (Root Config)
/// Phân cấp logic để khớp 1:1 với file JSON của Prospector.
/// [STAGE 1]: Chỉ giữ lại các thành phần cốt lõi để chạy ổn định.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HeliosConfig {
    pub indicators: IndicatorsConfig,
    pub environment_fuse: EnvironmentFuseConfig,
    pub market_regime: MarketRegimeConfig,
    pub tactical_sniper: TacticalSniperConfig,
    pub risk_management: RiskManagementConfig,
    // pub scoring: ScoringConfig, // [STAGE 2+]
    // pub warmup: WarmupConfig,
}

// ==================================================================================
// TẦNG 0: INDICATORS (MẮT THẦN)
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IndicatorsConfig {
    pub kama: KamaParams,
    pub adx: AdxParams,
    pub rsi: RsiParams,
    pub bbands: BbandsParams,
    pub keltner: KeltnerParams,
    pub macd: MacdParams,
    pub stochastic: StochasticParams, 
    pub atr: AtrParams,
    pub natr: NAtrParams,
    pub mfi: MfiParams,
    pub vol_ma: VolMaParams,
    pub linreg: LinRegParams,
    
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub z_score_lookback: usize, // NEW: Số nến dùng để tính Z-Score (Optimize [20-100])
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MfiParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KamaParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub short_period: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub long_period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdxParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RsiParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BbandsParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub nbdev: f64,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct KeltnerParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub multiplier: f64,
    pub timeframe: String,
    pub ma_type: MaType,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MacdParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub fast: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub slow: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub signal: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct StochasticParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub k_period: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub d_period: usize,
    pub k_ma_type: MaType, 
    pub d_ma_type: MaType,
    pub zone: f64,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AtrParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NAtrParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct VolMaParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LinRegParams {
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub period: usize,
    pub timeframe: String,
}

// ==================================================================================
// TẦNG 1: ENVIRONMENT FUSE (CẦU CHÌ MÔI TRƯỜNG)
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EnvironmentFuseConfig {
    pub er_threshold: f64,
    pub chaos_z_threshold: f64,
    pub min_liquidity_ratio: f64,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub min_orderly_bars: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub percentile_lookback: usize,
    #[serde(deserialize_with = "deserialize_usize_from_f64")]
    pub min_data_points: usize,
}

// ==================================================================================
// TẦNG 2: MARKET REGIME (LA BÀN THẾ TRẬN)
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MarketRegimeConfig {
    pub squeeze: RegimeSqueezeParams,
    pub trend: RegimeTrendParams,
    pub phase: RegimePhaseParams,
    pub absorption: RegimeAbsorptionParams, 
    pub anomaly: RegimeAnomalyParams,       
    pub parabolic: RegimeParabolicParams,   // NEW: Thế trận hưng phấn
    pub confidence: RegimeConfidenceParams,
    pub priority: RegimePriorityParams,      // NEW: Trọng số phân xử
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimePriorityParams {
    pub trend: f64,
    pub squeeze: f64,
    pub ranging: f64,
    pub absorption: f64,
    pub anomaly: f64,
    pub parabolic: f64,
    pub min_score_threshold: f64, // Ngưỡng tối thiểu để một squad được phép hoạt động
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeSqueezeParams {
    pub threshold: f64,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeTrendParams {
    pub adx_strong: f64,
    pub adx_moderate: f64,
    pub angle_healthy: f64,
    pub angle_parabolic: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimePhaseParams {
    pub accel_min: f64,
    pub accel_max: f64,
    pub climax: f64,
    pub spike_mult: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeAbsorptionParams {
    pub volume_threshold: f64, // Volume Force > threshold (e.g. 1.5)
    pub range_width_max: f64,  // BB Width < threshold
    pub bias_threshold: f64,   // Ngưỡng lệch để xác định phe nào đang gom (e.g. 0.55) (FIX #12)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeAnomalyParams {
    pub z_score_threshold: f64, // Price Z-Score (e.g. 3.0)
    pub natr_spike_mult: f64,   // NATR hiện tại / NATR trung bình (e.g. 2.0)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeParabolicParams {
    pub angle_threshold: f64,      // Góc tối thiểu (e.g. 75.0)
    pub accel_angle_threshold: f64, // Gia tốc góc (Góc hiện tại / Góc trung bình > 1.2) (FIX #15)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RegimeConfidenceParams {
    pub min_threshold: f64,
    pub weight_angle: f64,
    pub weight_adx: f64,
}

// ==================================================================================
// TẦNG 3: TACTICAL SNIPER (NGÒI NỔ BÓP CÒ)
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalSniperConfig {
    pub squeeze: TacticalSqueezeParams,     // Đội Squeeze
    pub pullback: TacticalPullbackParams,   // Đội Trend
    pub momentum: TacticalMomentumParams,   // Đội Momentum
    pub range: TacticalRangeParams,         // Đội Range
    pub reversal: TacticalReversalParams,   // Đội Reversal
    pub anomaly: TacticalAnomalyParams,     // Đội Anomaly
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalSqueezeParams {
    pub break_confirm_mult: f64, // Hệ số nhân khoảng cách phá vỡ (FIX #27)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalPullbackParams {
    pub rsi_buy_thresh: f64,     // Ngưỡng RSI hồi (e.g. 40-45 cho Long)
    pub kama_tolerance_pct: f64, // Khoảng cách cho phép so với đường KAMA
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalMomentumParams {
    pub rsi_threshold: f64,      // RSI vượt ngưỡng (e.g. 60 cho Long) (FIX #11)
    pub volume_spike_min: f64,   // Volume Force tối thiểu
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalRangeParams {
    pub stoch_oversold: f64,     // Stochastic quá bán
    pub stoch_overbought: f64,    // Stochastic quá mua
    pub min_width_pct: f64,      // NEW: Độ rộng tối thiểu tính theo % giá (e.g. 0.0624 cho 3% ròng)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalReversalParams {
    pub rsi_extreme: f64,        // RSI cực hạn (e.g. 80 cho Short)
    pub exhaustion_vol_drop: f64, // Tỷ lệ giảm Volume khi kiệt sức (FIX #28)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TacticalAnomalyParams {
    pub rsi_climax: f64,         // RSI cao điểm để xác nhận cú hồi (FIX #28)
    pub min_rejection_pct: f64,  // Tỷ lệ rút chân tối thiểu
}

// ==================================================================================
// TẦNG 4: RISK MANAGEMENT (QUẢN TRỊ RỦI RO)
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RiskManagementConfig {
    pub risk_base: RiskBaseParams,
    pub stop_management: RiskStopParams,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RiskBaseParams {
    pub percent_per_trade: f64,
    pub sl_min_dist_pct: f64,
    pub use_breakeven: f64,      // Đổi sang f64: > 0.5 là dùng, <= 0.5 là không (FIX #18)
    pub be_profit_target_rr: f64, // Đạt RR bao nhiêu thì dời SL về BE
    pub base_lot_size: f64,      // NEW: Khối lượng giao dịch gốc (FIX #27)
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RiskStopParams {
    // --- 🧬 GENE GỐC (Base Genes) ---
    pub base_sl_atr: f64,      // Dừng lỗ gốc theo ATR
    pub base_tp_rr: f64,       // Chốt lời gốc theo tỷ lệ R:R

    // --- 🎭 HỆ SỐ ĐỘI ĐẶC NHIỆM (Squad Factors) ---
    pub squeeze_sl_factor: f64,
    pub trend_sl_factor: f64,
    pub momentum_sl_factor: f64,
    pub range_sl_factor: f64,
    pub reversal_sl_factor: f64,
    pub anomaly_sl_factor: f64,

    // --- 🛡️ TRAILING STOP (Áo giáp co giãn) ---
    pub tsl_atr_tight: f64,    // Siết chặt (Cho Parabolic)
    pub tsl_atr_normal: f64,   // Bình thường
    pub tsl_atr_loose: f64,    // Nới rộng (Cho Squeeze lúc đầu)
}
// #[derive(Debug, Clone, Deserialize, Default)] pub struct RiskPyramidingParams { pub min_profit_1r: f64, pub risk_budget_factor: f64, pub anchor_lookback: usize }

// ==================================================================================
// TẦNG 5: SCORING (STAGE 2+)
// ==================================================================================
/*
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScoringConfig {
    pub base_score: f64,
    pub bonus: ScoringBonusParams,
    pub penalty: ScoringPenaltyParams,
}
#[derive(Debug, Clone, Deserialize, Default)] pub struct ScoringBonusParams { pub trending_vol: f64, pub trending_mom: f64, pub squeeze_vol: f64 }
#[derive(Debug, Clone, Deserialize, Default)] pub struct ScoringPenaltyParams { pub trending_div: f64, pub trending_vol_div: f64, pub ranging_vol: f64 }
*/

// ==================================================================================
// GLOBAL: WARMUP
// ==================================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WarmupConfig {
    pub num: usize,
}
