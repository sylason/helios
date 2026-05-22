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
    pub trend_angle: Arc<Vec<f64>>,    // -90.0 to 90.0 (Degrees)
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
        // 1. Trend Strength (ADX / 100)
        let trend_strength = sensors.adx.iter().map(|&v| v / 100.0).collect();

        // 2. Trend Direction (Tanh Scale LinReg Slope)
        let trend_dir = normalization::tanh_scale(&sensors.linreg_slope, 100.0);

        // 3. Trend Angle (Lấy trực tiếp từ cảm biến)
        let trend_angle = sensors.linreg_angle.as_ref().clone();

        // 4. Trend Angle Adjusted (FIX #7)
        let adj_slope = normalization::volatility_adjusted_slope(&sensors.linreg_slope, &sensors.atr_sl);
        // Chuyển Adjusted Slope sang Angle (Degrees)
        let trend_angle_adj = adj_slope.iter().map(|&s| s.atan().to_degrees()).collect();

        // 5. Squeeze Intensity (BB / KC Compression)
        let squeeze_intensity = normalization::calculate_squeeze_intensity(
            &sensors.bb_upper,
            &sensors.bb_lower,
            &sensors.kc_upper,
            &sensors.kc_lower,
        );

        // 6. BB Width Normalized
        let bb_width_norm: Vec<f64> = sensors.bb_upper.iter().enumerate().map(|(i, &u)| {
            let l = sensors.bb_lower[i];
            let m = (u + l) / 2.0;
            if m > 0.0 { (u - l) / m } else { 0.0 }
        }).collect();

        // 7. Momentum (RSI / 100)
        let momentum = sensors.rsi.iter().map(|&v| v / 100.0).collect();

        // 8. Liquidity (MFI / 100)
        let liquidity = sensors.mfi.iter().map(|&v| v / 100.0).collect();

        // 9. Chaos (Sử dụng NATR làm thước đo biến động chuẩn hóa)
        let chaos = sensors.natr.as_ref().clone();

        // 10. Chaos Ratio (NATR / SMA(NATR, 100)) - FIX #3, #23
        let chaos_ratio = sensors.chaos_ratio.as_ref().clone();

        // 11. Entropy (Độ thẳng của xu hướng - Efficiency Ratio)
        let entropy = sensors.efficiency_ratio.as_ref().clone();

        // 12. Volume Force (Volume * Ratio / Volume MA)
        let volume_force: Vec<f64> = volumes
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let ma = sensors.vol_ma_series[i];
                if ma > 0.0 { (v * vol_ratio) / ma } else { 1.0 }
            })
            .collect();

        // 13. Price Z-Score (FIX #6)
        let price_z_score = normalization::calculate_z_score_series(&sensors.closes, config.indicators.z_score_lookback);

        Self {
            trend_dir: Arc::new(trend_dir),
            trend_strength: Arc::new(trend_strength),
            trend_angle: Arc::new(trend_angle),
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
