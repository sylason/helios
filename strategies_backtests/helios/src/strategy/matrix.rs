// strategies_backtests/helios/src/strategy/matrix.rs

use std::sync::Arc;
use crate::strategy::sensors::HeliosSensors;
use crate::utils::normalization;
use crate::strategy::config::HeliosConfig;

/// [LEVEL 2: MARKET MATRIX]
/// Chứa các dữ liệu đã được chuẩn hóa (Normalized 0.x hoặc -1.0 to 1.0).
/// Đây là "Nhận thức" của Helios, giúp các module Logic ra quyết định đồng nhất.
#[derive(Clone, Debug, Default)]
pub struct MarketMatrix {
    // --- 🧭 Hướng & Sức mạnh (Directional) ---
    pub trend_dir: Arc<Vec<f64>>,      // -1.0 (Bearish) to 1.0 (Bullish)
    pub trend_strength: Arc<Vec<f64>>, // 0.0 to 1.0 (ADX Scaled)
    pub trend_angle_adj: Arc<Vec<f64>>, // NEW: Góc hiệu chỉnh theo ATR (FIX #7)
    
    // --- 🌪️ Năng lượng & Biến động (Chaos) ---
    pub squeeze_intensity: Arc<Vec<f64>>, // 0.0 to 1.0 (BB/KC Compression)
    pub chaos: Arc<Vec<f64>>,             // 0.0 to 0.10+ (NATR - Biến động % giá)
    pub chaos_ratio: Arc<Vec<f64>>,       // NEW: NATR / SMA(NATR) (FIX #3, #23)
    pub entropy: Arc<Vec<f64>>,           // 0.0 to 1.0 (Efficiency Ratio - Độ thẳng của giá)
    pub liquidity: Arc<Vec<f64>>,         // 0.0 to 1.0 (MFI Scaled)
    pub volume_force: Arc<Vec<f64>>,      // 0.0 to 2.0+ (Volume / Vol_MA)
    pub bb_width_norm: Arc<Vec<f64>>,     // (Upper - Lower) / Middle
    
    // --- 🌊 Động lượng (Velocity) ---
    pub momentum: Arc<Vec<f64>>, // 0.0 to 1.0 (RSI Scaled)
    pub price_z_score: Arc<Vec<f64>>, // NEW: Z-Score thực của giá (FIX #6)
}

impl MarketMatrix {
    /// Tính toán ma trận từ dữ liệu cảm biến thô.
    pub fn from_sensors(config: &HeliosConfig, sensors: &HeliosSensors, volumes: &[f64], vol_ratio: f64) -> Self {
        let len = volumes.len();
        
        // --- [TỐI ƯU HÓA: LOOP FUSION] ---
        // Gộp 6 vòng lặp map() ngang thành 1 vòng lặp dọc duy nhất để tối ưu L1 Cache.
        let mut adx_raw = Vec::with_capacity(len);
        let mut natr_raw = Vec::with_capacity(len);
        let mut bb_width_raw = Vec::with_capacity(len);
        let mut vol_raw = Vec::with_capacity(len);
        let mut sqz_raw = Vec::with_capacity(len);
        let mut adj_slope = Vec::with_capacity(len);

        for i in 0..len {
            // 1. Trend Strength (ADX Raw)
            adx_raw.push(sensors.adx[i]);
            
            // 2. Chaos (NATR Raw)
            natr_raw.push(sensors.natr[i]);
            
            // 3. BB Width Norm
            let bu = sensors.bb_upper[i];
            let bl = sensors.bb_lower[i];
            let bm = (bu + bl) / 2.0;
            bb_width_raw.push(if bm > 0.0 { (bu - bl) / bm } else { 0.0 });
            
            // 4. Volume Force Raw
            let v = volumes[i];
            let ma = sensors.vol_sma[i];
            vol_raw.push(if ma > 0.0 { (v * vol_ratio) / ma } else { 1.0 });

            // 5. Squeeze Raw (Overlap Logic)
            let ku = sensors.kc_upper[i];
            let kl = sensors.kc_lower[i];
            let kc_w = ku - kl;
            if kc_w < 1e-9 || bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
                sqz_raw.push(0.0);
            } else {
                let overlap = (ku - bu) + (bl - kl);
                sqz_raw.push((overlap / kc_w).max(0.0).min(1.0));
            }

            // 6. Volatility Adjusted Slope
            let s = sensors.kama_slope[i];
            let atr = sensors.atr_sl[i];
            adj_slope.push(if s.is_nan() || atr.is_nan() || atr < 1e-9 { f64::NAN } else { s / atr });
        }

        // --- [BƯỚC 2: ROLLING & Z-SCORE] ---
        // Các phép toán yêu cầu cửa sổ trượt (trạng thái)
        let adj_slope_pct = normalization::rolling_percentile(&adj_slope, config.matrix.trend_lookback);
        
        let trend_dir: Vec<f64> = adj_slope_pct.iter().map(|&v| {
            if v.is_nan() { f64::NAN } else { 2.0 * v - 1.0 }
        }).collect();

        let trend_angle_adj = adj_slope_pct;

        let trend_strength = normalization::rolling_percentile(&adx_raw, config.matrix.trend_lookback);
        let chaos = normalization::rolling_percentile(&natr_raw, config.matrix.squeeze_lookback);
        let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, config.matrix.squeeze_lookback);
        let bb_width_norm = normalization::rolling_percentile(&bb_width_raw, config.matrix.squeeze_lookback);
        let momentum = normalization::rolling_percentile(&sensors.rsi_trend, config.matrix.momentum_lookback);
        let liquidity = normalization::rolling_percentile(&sensors.mfi, config.matrix.momentum_lookback);
        let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, config.matrix.squeeze_lookback);
        let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, config.matrix.trend_lookback);
        let volume_force = normalization::rolling_percentile(&vol_raw, config.matrix.volume_lookback);

        let z_raw = normalization::calculate_z_score_series(&sensors.closes, config.indicators.z_score_lookback);
        let price_z_score = normalization::tanh_scale(&z_raw, config.matrix.z_score_sensitivity);

        Self {
            trend_dir: Arc::new(trend_dir),
            trend_strength: Arc::new(trend_strength),
            trend_angle_adj: Arc::new(trend_angle_adj),
            squeeze_intensity: Arc::new(squeeze_intensity),
            bb_width_norm: Arc::new(bb_width_norm),
            momentum: Arc::new(momentum),
            liquidity: Arc::new(liquidity),
            chaos: Arc::new(chaos),
            chaos_ratio: Arc::new(chaos_ratio), 
            entropy: Arc::new(entropy),
            volume_force: Arc::new(volume_force),
            price_z_score: Arc::new(price_z_score),
        }
    }
}
