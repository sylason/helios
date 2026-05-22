// strategies_backtests/helios/src/strategy/sensors.rs

use std::sync::Arc;

/// [LEVEL 1: RAW SENSORS]
/// Chứa các con trỏ dữ liệu thô (Raw Indicators) lấy trực tiếp từ Factory.
/// Đây là "Đôi mắt" của Helios, nhìn thấy dữ liệu ở định dạng gốc của thư viện.
#[derive(Clone, Debug, Default)]
pub struct HeliosSensors {
    // --- Raw Price Data (FIX #10) ---
    pub closes: Arc<Vec<f64>>, 

    // --- Environment & Chaos ---
    pub natr: Arc<Vec<f64>>,
    pub chaos_ratio: Arc<Vec<f64>>,
    pub vol_ma_series: Arc<Vec<f64>>, 
    pub efficiency_ratio: Arc<Vec<f64>>,
    pub mfi: Arc<Vec<f64>>,
    
    // --- Trend & Regime ---
    pub adx: Arc<Vec<f64>>,
    pub linreg_slope: Arc<Vec<f64>>,
    pub linreg_angle: Arc<Vec<f64>>, // NEW
    pub kama_mid: Arc<Vec<f64>>,
    
    // --- Volatility & Squeeze ---
    pub bb_upper: Arc<Vec<f64>>,
    pub bb_lower: Arc<Vec<f64>>,
    pub kc_upper: Arc<Vec<f64>>,
    pub kc_lower: Arc<Vec<f64>>,
    
    // --- Momentum & Tactical ---
    pub rsi: Arc<Vec<f64>>,
    pub macd_main: Arc<Vec<f64>>,
    pub macd_signal: Arc<Vec<f64>>,
    pub macd_hist: Arc<Vec<f64>>,
    
    // --- Sniper Triggers (Stochastic) ---
    pub stoch_k: Arc<Vec<f64>>,
    pub stoch_d: Arc<Vec<f64>>,
    pub stoch_cross: Arc<Vec<f64>>,
    pub stoch_k_zone: Arc<Vec<f64>>,
    pub stoch_d_zone: Arc<Vec<f64>>,
    
    // --- Risk & Execution ---
    pub atr_sl: Arc<Vec<f64>>,
}
