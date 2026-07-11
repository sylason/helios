// strategies_backtests/helios/src/strategy/core_v3/matrix.rs

use std::sync::Arc;
use crate::strategy::sensors::HeliosSensors;
use crate::utils::normalization;
use crate::strategy::config::HeliosConfig;

/// [LEVEL 2: MARKET MATRIX V3]
/// Chứa các dữ liệu đã được chuẩn hóa (Normalized 0.x hoặc -1.0 to 1.0).
/// Bổ sung thêm các Regime Multipliers phục vụ cho Arbiter.
#[derive(Clone, Debug, Default)]
pub struct MarketMatrixV3 {
    // --- 🧭 Hướng & Sức mạnh (Directional) ---
    pub trend_dir: Arc<Vec<f64>>,      // -1.0 (Bearish) to 1.0 (Bullish)
    pub trend_strength: Arc<Vec<f64>>, // 0.0 to 1.0 (ADX Scaled)
    pub trend_angle: Arc<Vec<f64>>,    // -90.0 to 90.0 (Degrees)
    pub trend_angle_adj: Arc<Vec<f64>>, // Góc hiệu chỉnh theo ATR
    
    // --- 🌪️ Năng lượng & Biến động (Chaos) ---
    pub squeeze_intensity: Arc<Vec<f64>>, // 0.0 to 1.0 (BB/KC Compression)
    pub chaos: Arc<Vec<f64>>,             // 0.0 to 0.10+ (NATR - Biến động % giá)
    pub chaos_ratio: Arc<Vec<f64>>,       // NATR / SMA(NATR)
    pub entropy: Arc<Vec<f64>>,           // 0.0 to 1.0 (Efficiency Ratio)
    pub liquidity: Arc<Vec<f64>>,         // 0.0 to 1.0 (MFI Scaled)
    pub volume_force: Arc<Vec<f64>>,      // 0.0 to 2.0+ (Volume / Vol_MA)
    pub bb_width_norm: Arc<Vec<f64>>,     // (Upper - Lower) / Middle
    
    // --- 🌊 Động lượng (Velocity) ---
    pub momentum: Arc<Vec<f64>>, // 0.0 to 1.0 (RSI Scaled)
    pub price_z_score: Arc<Vec<f64>>, // Z-Score thực của giá
    
    // --- ⚖️ REGIME MULTIPLIERS (Dành riêng cho V3 Arbiter) ---
    pub trend_multiplier: Arc<Vec<f64>>,   // Sức mạnh Trend
    pub squeeze_multiplier: Arc<Vec<f64>>, // Sức nén Volatility
    pub range_multiplier: Arc<Vec<f64>>,   // Phần bù của Trend (Sideway)
    pub anomaly_multiplier: Arc<Vec<f64>>, // Sự kết hợp của Chaos và Z-Score (Climax)
}

impl MarketMatrixV3 {
    pub fn from_sensors(config: &HeliosConfig, sensors: &HeliosSensors, volumes: &[f64], vol_ratio: f64) -> Self {
        let len = volumes.len();
        
        let mut trend_strength = Vec::with_capacity(len);
        let mut chaos = Vec::with_capacity(len);
        let mut bb_width_norm = Vec::with_capacity(len);
        let mut vol_raw = Vec::with_capacity(len);
        let mut sqz_raw = Vec::with_capacity(len);
        let mut adj_slope = Vec::with_capacity(len);

        for i in 0..len {
            trend_strength.push(sensors.adx[i] / 100.0);
            chaos.push(sensors.natr[i] / 100.0);
            
            let bu = sensors.bb_upper[i];
            let bl = sensors.bb_lower[i];
            let bm = (bu + bl) / 2.0;
            bb_width_norm.push(if bm > 0.0 { (bu - bl) / bm } else { 0.0 });
            
            let v = volumes[i];
            let ma = sensors.vol_ma_series[i];
            vol_raw.push(if ma > 0.0 { (v * vol_ratio) / ma } else { 1.0 });

            let ku = sensors.kc_upper[i];
            let kl = sensors.kc_lower[i];
            let kc_w = ku - kl;
            if kc_w < 1e-9 || bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
                sqz_raw.push(0.0);
            } else {
                let overlap = (ku - bu) + (bl - kl);
                sqz_raw.push((overlap / kc_w).max(0.0).min(1.0));
            }

            let s = sensors.linreg_slope[i];
            let atr = sensors.atr_sl[i];
            adj_slope.push(if s.is_nan() || atr.is_nan() || atr < 1e-9 { f64::NAN } else { s / atr });
        }

        let adj_slope_pct = normalization::rolling_percentile(&adj_slope, config.matrix.trend_lookback);
        let trend_dir: Vec<f64> = adj_slope_pct.iter().map(|&v| if v.is_nan() { f64::NAN } else { 2.0 * v - 1.0 }).collect();

        let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, config.matrix.squeeze_lookback);
        let momentum = normalization::rolling_percentile(&sensors.rsi, config.matrix.momentum_lookback);
        let liquidity = normalization::rolling_percentile(&sensors.mfi, config.matrix.momentum_lookback);
        let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, config.matrix.squeeze_lookback);
        let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, config.matrix.trend_lookback);
        let volume_force = normalization::rolling_percentile(&vol_raw, config.matrix.volume_lookback);

        let z_raw = normalization::calculate_z_score_series(&sensors.closes, config.indicators.z_score_lookback);
        let price_z_score = normalization::tanh_scale(&z_raw, config.matrix.z_score_sensitivity);

        // TÍNH TOÁN REGIME MULTIPLIERS (V3 MỚI)
        let mut trend_multiplier = Vec::with_capacity(len);
        let mut squeeze_multiplier = Vec::with_capacity(len);
        let mut range_multiplier = Vec::with_capacity(len);
        let mut anomaly_multiplier = Vec::with_capacity(len);

        for i in 0..len {
            // Trend mạnh lên khi ADX cao và Trend Angle ổn định
            let t_mul = trend_strength[i].max(0.0).min(1.0);
            trend_multiplier.push(t_mul);

            // Range mạnh lên khi Trend yếu
            range_multiplier.push(1.0 - t_mul);

            // Squeeze phụ thuộc trực tiếp vào Squeeze Intensity
            squeeze_multiplier.push(squeeze_intensity[i]);

            // Anomaly (bùng nổ hoặc kiệt sức) khi giá lệch chuẩn mạnh và hỗn loạn cao
            let z_abs = price_z_score[i].abs(); // 0.0 to 1.0
            anomaly_multiplier.push((z_abs * chaos_ratio[i]).max(0.0).min(1.0));
        }

        Self {
            trend_dir: Arc::new(trend_dir),
            trend_strength: Arc::new(trend_strength),
            trend_angle: sensors.linreg_angle.clone(),
            trend_angle_adj: Arc::new(adj_slope_pct),
            squeeze_intensity: Arc::new(squeeze_intensity),
            bb_width_norm: Arc::new(bb_width_norm),
            momentum: Arc::new(momentum),
            liquidity: Arc::new(liquidity),
            chaos: Arc::new(chaos),
            chaos_ratio: Arc::new(chaos_ratio), 
            entropy: Arc::new(entropy),
            volume_force: Arc::new(volume_force),
            price_z_score: Arc::new(price_z_score),
            trend_multiplier: Arc::new(trend_multiplier),
            squeeze_multiplier: Arc::new(squeeze_multiplier),
            range_multiplier: Arc::new(range_multiplier),
            anomaly_multiplier: Arc::new(anomaly_multiplier),
        }
    }
}
