// strategies_backtests/helios/src/strategy/sensors.rs

use std::sync::Arc;

/// [LEVEL 1: RAW SENSORS]
/// Chứa các con trỏ dữ liệu thô (Raw Indicators) lấy trực tiếp từ Factory.
/// Đây là "Đôi mắt" của Helios V3.3, được trang bị hệ thống Cảm biến Độc lập Per-Playbook.
#[derive(Clone, Debug, Default)]
pub struct HeliosSensors {
    // --- Raw Price & Volume Data ---
    pub closes: Arc<Vec<f64>>, 
    pub highs: Arc<Vec<f64>>,
    pub lows: Arc<Vec<f64>>,
    pub volumes: Arc<Vec<f64>>,

    // --- Environment & Chaos ---
    pub natr: Arc<Vec<f64>>,
    pub chaos_ratio: Arc<Vec<f64>>,
    pub efficiency_ratio: Arc<Vec<f64>>,
    pub mfi: Arc<Vec<f64>>,
    pub z_score: Arc<Vec<f64>>,
    pub vol_z_score: Arc<Vec<f64>>,
    
    // --- Trend & Regime (KAMA & KAMA Slope) ---
    pub adx: Arc<Vec<f64>>,
    pub kama_mid: Arc<Vec<f64>>,
    pub kama_slope: Arc<Vec<f64>>,
    pub kama_slope_adj: Arc<Vec<f64>>,
    
    // --- Volatility & Squeeze ---
    pub bb_upper: Arc<Vec<f64>>,
    pub bb_middle: Arc<Vec<f64>>,
    pub bb_middle_slope_adj: Arc<Vec<f64>>,
    pub bb_lower: Arc<Vec<f64>>,
    pub kc_upper: Arc<Vec<f64>>,
    pub kc_lower: Arc<Vec<f64>>,

    // --- Dedicated Per-Playbook RSI Sensors ---
    pub rsi_trend: Arc<Vec<f64>>,
    pub rsi_reversal: Arc<Vec<f64>>,
    pub rsi_momentum: Arc<Vec<f64>>,
    pub rsi_anomaly: Arc<Vec<f64>>,
    
    // --- Dedicated Per-Playbook Stochastic Sensors ---
    pub stoch_trend_k: Arc<Vec<f64>>,
    pub stoch_trend_d: Arc<Vec<f64>>,
    pub stoch_range_k: Arc<Vec<f64>>,
    pub stoch_range_d: Arc<Vec<f64>>,
    
    // --- Absorption Sensor (VWMA & Volume SMA) ---
    pub vwma: Arc<Vec<f64>>,
    pub vol_sma: Arc<Vec<f64>>,
    
    // --- MACD Sensors ---
    pub macd_main: Arc<Vec<f64>>,
    pub macd_signal: Arc<Vec<f64>>,
    pub macd_hist: Arc<Vec<f64>>,
    pub stoch_k: Arc<Vec<f64>>,
    
    // --- Risk & Execution ---
    pub atr_sl: Arc<Vec<f64>>,
}
