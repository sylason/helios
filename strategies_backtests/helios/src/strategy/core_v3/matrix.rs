// strategies_backtests/helios/src/strategy/core_v3/matrix.rs

use std::sync::Arc;
use crate::strategy::sensors::HeliosSensors;
use crate::utils::normalization;
use crate::strategy::core_v3::config::HeliosConfigV3;

/// [LEVEL 2: MARKET MATRIX V3.3]
/// Chứa các dữ liệu đã được chuẩn hóa (Normalized 0.x hoặc -1.0 to 1.0).
/// Bổ sung thêm các Regime Multipliers chuẩn Quant phục vụ cho Arbiter.
#[derive(Clone, Debug, Default)]
pub struct MarketMatrixV3 {
    // --- 🧭 Hướng & Sức mạnh (Directional) ---
    pub trend_dir: Arc<Vec<f64>>,         // -1.0 (Bearish) to 1.0 (Bullish)
    pub trend_strength: Arc<Vec<f64>>,    // 0.0 to 1.0 (ADX Tanh Scaled)
    pub kama_slope_adj: Arc<Vec<f64>>,    // KAMA Slope điều chỉnh theo ER
    pub bb_middle_slope_adj: Arc<Vec<f64>>, // 🛡️ FLAW-REG.10: Độ dốc chuẩn hóa của trục giữa BB Middle (SMA 20)
    
    // --- 🌪️ Năng lượng & Biến động (Chaos) ---
    pub squeeze_intensity: Arc<Vec<f64>>, // 0.0 to 1.0 (BB/KC Compression)
    pub chaos: Arc<Vec<f64>>,             // 0.0 to 0.10+ (NATR - Biến động % giá)
    pub chaos_ratio: Arc<Vec<f64>>,       // NATR / SMA(NATR)
    pub entropy: Arc<Vec<f64>>,           // 0.0 to 1.0 (Efficiency Ratio)
    pub liquidity: Arc<Vec<f64>>,         // 0.0 to 1.0 (MFI Scaled)
    pub volume_force: Arc<Vec<f64>>,      // 0.0 to 1.0 (Quant Volume Force: Z-Score + MFI)
    pub bb_width_norm: Arc<Vec<f64>>,     // (Upper - Lower) / Middle
    
    // --- 🌊 Động lượng (Velocity) ---
    pub price_z_score: Arc<Vec<f64>>,     // Z-Score thực của giá (Tanh Scaled)
    
    // --- ⚖️ REGIME MULTIPLIERS (Dành riêng cho V3.3 Arbiter) ---
    pub trend_multiplier: Arc<Vec<f64>>,   // Sức mạnh Trend (ADX Tanh)
    pub squeeze_multiplier: Arc<Vec<f64>>, // Sức nén Volatility
    pub range_multiplier: Arc<Vec<f64>>,   // Phần bù của Trend (1.0 - trend_multiplier)
    pub anomaly_multiplier: Arc<Vec<f64>>, // Chaos Ratio + Z-Score
    pub reversal_multiplier: Arc<Vec<f64>>, // Kiệt sức: Z-Score cao + Trend yếu
    pub absorption_multiplier: Arc<Vec<f64>>, // Squeeze + Volume Force
    pub volatility_expansion_multiplier: Arc<Vec<f64>>, // Chaos Ratio + Volume Force

    // --- 📐 VECTOR BỐI CẢNH CONTINUOUS SIMPLEX (Chuẩn Hóa Tổng = 1.00) ---
    pub w_trend_norm: Arc<Vec<f64>>,    // Trọng số Xu Hướng [0.0, 1.0]
    pub w_range_norm: Arc<Vec<f64>>,    // Trọng số Đi Ngang Sideway [0.0, 1.0]
    pub w_volatile_norm: Arc<Vec<f64>>, // Trọng số Bão Biến Động cũ (tương thích ngược) [0.0, 1.0]
    pub w_comp_norm: Arc<Vec<f64>>,     // Trọng số Nén Tích Lũy Squeeze Compression [0.0, 1.0]
    pub w_chaos_norm: Arc<Vec<f64>>,    // Trọng số Bão Hỗn Loạn Whipsaw Chaos [0.0, 1.0]
}

impl MarketMatrixV3 {
    pub fn from_sensors(config: &HeliosConfigV3, sensors: &HeliosSensors) -> Self {
        let len = sensors.closes.len();
        if len == 0 {
            return Self::default();
        }
        
        let mut chaos = Vec::with_capacity(len);
        let mut bb_width_norm = Vec::with_capacity(len);
        let mut sqz_raw = Vec::with_capacity(len);
        let mut adx_centered = Vec::with_capacity(len);

        for i in 0..len {
            chaos.push(sensors.natr[i] / 100.0);
            
            let bu = sensors.bb_upper[i];
            let bl = sensors.bb_lower[i];
            let bm = sensors.bb_middle[i];
            bb_width_norm.push(if bm > 0.0 { (bu - bl) / bm } else { 0.0 });

            let ku = sensors.kc_upper[i];
            let kl = sensors.kc_lower[i];
            let kc_w = ku - kl;
            if kc_w < 1e-9 || bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
                sqz_raw.push(0.0);
            } else {
                let overlap = (ku - bu) + (bl - kl);
                sqz_raw.push((overlap / kc_w).max(0.0).min(1.0));
            }

            // ADX Centered tại ngưỡng 20.0 để dùng Tanh Scale
            let adx_v = sensors.adx[i];
            adx_centered.push(if adx_v.is_nan() { 0.0 } else { adx_v - 20.0 });
        }

        // TÍNH TOÁN CÁC DÒNG CHUẨN HÓA V3.3
        let z_sens = if config.matrix.z_score_sensitivity <= 0.0 { 1.0 } else { config.matrix.z_score_sensitivity };
        let sqz_lb = if config.matrix.squeeze_lookback == 0 { 20 } else { config.matrix.squeeze_lookback };
        let trend_lb = if config.matrix.trend_lookback == 0 { 20 } else { config.matrix.trend_lookback };
        let mom_lb = if config.matrix.momentum_lookback == 0 { 20 } else { config.matrix.momentum_lookback };

        let trend_strength = normalization::tanh_scale(&adx_centered, z_sens);
        
        let trend_dir: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
            if s.is_nan() { 0.0 } else { (s / 0.50).max(-1.0).min(1.0) }
        }).collect();

        let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
        let liquidity = normalization::rolling_percentile(&sensors.mfi, mom_lb);
        let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, sqz_lb);
        let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, trend_lb);

        let price_z_score = normalization::tanh_scale(&sensors.z_score, z_sens);

        // VOLUME FORCE CHUẨN QUANT: 0.5 * Tanh(Vol_Z_Score) + 0.5 * (MFI / 100)
        let vol_z_scale = normalization::tanh_scale(&sensors.vol_z_score, 0.5);
        let mut volume_force = Vec::with_capacity(len);
        for i in 0..len {
            let mfi_norm = (sensors.mfi[i] / 100.0).max(0.0).min(1.0);
            let vf = 0.5 * (vol_z_scale[i].abs()) + 0.5 * mfi_norm;
            volume_force.push(vf.max(0.0).min(1.0));
        }

        // TÍNH TOÁN REGIME MULTIPLIERS (V3.3 SẠCH - KHÔNG TRỌNG SỐ THỪA)
        let mut trend_multiplier = Vec::with_capacity(len);
        let mut squeeze_multiplier = Vec::with_capacity(len);
        let mut range_multiplier = Vec::with_capacity(len);
        let mut anomaly_multiplier = Vec::with_capacity(len);
        let mut reversal_multiplier = Vec::with_capacity(len);
        let mut absorption_multiplier = Vec::with_capacity(len);
        let mut volatility_expansion_multiplier = Vec::with_capacity(len);

        // TÍNH TOÁN VECTOR BỐI CẢNH CONTINUOUS SIMPLEX 4 TRẠNG THÁI (Chuẩn hóa Tổng = 1.00)
        let mut w_trend_norm = Vec::with_capacity(len);
        let mut w_range_norm = Vec::with_capacity(len);
        let mut w_volatile_norm = Vec::with_capacity(len);
        let mut w_comp_norm = Vec::with_capacity(len);
        let mut w_chaos_norm = Vec::with_capacity(len);

        for i in 0..len {
            let t_mul = trend_strength[i].max(0.0).min(1.0);
            trend_multiplier.push(t_mul);

            // 🛡️ FLAW-REG.10: Hệ số phẳng của trục giữa Bollinger Bands (SMA 20)
            let bb_slope = if sensors.bb_middle_slope_adj.is_empty() || i >= sensors.bb_middle_slope_adj.len() {
                0.0
            } else {
                let s = sensors.bb_middle_slope_adj[i];
                if s.is_nan() { 0.0 } else { s }
            };
            let flatness_factor = (1.0 - (bb_slope.abs() / 0.10).tanh()).max(0.0).min(1.0);
            range_multiplier.push((1.0 - t_mul) * flatness_factor);

            squeeze_multiplier.push(squeeze_intensity[i]);

            let z_abs = price_z_score[i].abs();
            let c_ratio = if chaos_ratio[i].is_nan() { 0.0 } else { chaos_ratio[i] };
            let ano_score = 0.5 * z_abs + 0.5 * c_ratio;
            anomaly_multiplier.push(ano_score.max(0.0).min(1.0));

            let rev_trend_factor = if t_mul > 0.80 { 0.20 } else { 1.0 - t_mul };
            let rev_score = rev_trend_factor * z_abs;
            reversal_multiplier.push(rev_score.max(0.0).min(1.0));

            let abs_score = 0.5 * squeeze_intensity[i] + 0.5 * volume_force[i];
            absorption_multiplier.push(abs_score.max(0.0).min(1.0));

            let v_force = if volume_force[i].is_nan() { 0.5 } else { volume_force[i] };

            let vol_exp_score = 0.5 * c_ratio + 0.5 * v_force;
            volatility_expansion_multiplier.push(vol_exp_score.max(0.0).min(1.0));

            // =========================================================================
            // 4-STATE CONTINUOUS SIMPLEX VECTOR: w_trend + w_range + w_comp + w_chaos ≡ 1.00
            // =========================================================================
            // 1. Xu hướng định hướng: Trend Strength kết hợp độ dốc KAMA
            let slope_val = sensors.kama_slope_adj[i];
            let slope_factor = if slope_val.is_nan() { 0.5 } else { (1.0 + (slope_val.abs() / 0.10).tanh()) / 2.0 };
            let w_trend_raw = t_mul * slope_factor;

            // 2. Bão Hỗn loạn Whipsaw / Micro-noise: Chaos Ratio kết hợp bước giật giá Z-Score
            let c_r = c_ratio.max(0.0).min(1.0);
            let z_clipped = z_abs.max(0.0).min(1.0);
            let w_chaos_raw = (0.70 * c_r + 0.30 * z_clipped).max(0.0).min(1.0);

            // 3. Nén Tích lũy Dòng tiền: Squeeze co thắt trong điều kiện trật tự (không bị bão Whipsaw phá vỡ)
            let sqz = squeeze_intensity[i].max(0.0).min(1.0);
            let w_comp_raw = (sqz * (1.0 - c_r)).max(0.0).min(1.0);

            // 4. Đi ngang êm đềm (Clean Range): Không Trend, không Bão Chaos, không Nén Squeeze VÀ Trục giữa BB Middle phải phẳng!
            let w_range_raw = ((1.0 - t_mul) * (1.0 - c_r) * (1.0 - sqz) * flatness_factor).max(0.05).min(1.0);

            let w_t = if w_trend_raw.is_nan() { 0.0 } else { w_trend_raw.max(0.0) };
            let w_r = if w_range_raw.is_nan() { 0.0 } else { w_range_raw.max(0.0) };
            let w_c = if w_comp_raw.is_nan() { 0.0 } else { w_comp_raw.max(0.0) };
            let w_ch = if w_chaos_raw.is_nan() { 0.0 } else { w_chaos_raw.max(0.0) };

            let sum = w_t + w_r + w_c + w_ch;
            if sum <= 1e-6 || sum.is_nan() {
                // Fallback an toàn tuyệt đối: Chia đều 1/4 cho cả 4 chế độ (Simplex 100%)
                w_trend_norm.push(0.25);
                w_range_norm.push(0.25);
                w_comp_norm.push(0.25);
                w_chaos_norm.push(0.25);
                w_volatile_norm.push(0.25); // Tương thích ngược
            } else {
                let norm_t = (w_t / sum).max(0.0).min(1.0);
                let norm_r = (w_r / sum).max(0.0).min(1.0);
                let norm_c = (w_c / sum).max(0.0).min(1.0);
                let norm_ch = (w_ch / sum).max(0.0).min(1.0);

                w_trend_norm.push(norm_t);
                w_range_norm.push(norm_r);
                w_comp_norm.push(norm_c);
                w_chaos_norm.push(norm_ch);
                w_volatile_norm.push(norm_ch);
            }
        }

        Self {
            trend_dir: Arc::new(trend_dir),
            trend_strength: Arc::new(trend_strength),
            kama_slope_adj: sensors.kama_slope_adj.clone(),
            bb_middle_slope_adj: sensors.bb_middle_slope_adj.clone(),
            squeeze_intensity: Arc::new(squeeze_intensity),
            bb_width_norm: Arc::new(bb_width_norm),
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
            reversal_multiplier: Arc::new(reversal_multiplier),
            absorption_multiplier: Arc::new(absorption_multiplier),
            volatility_expansion_multiplier: Arc::new(volatility_expansion_multiplier),
            w_trend_norm: Arc::new(w_trend_norm),
            w_range_norm: Arc::new(w_range_norm),
            w_volatile_norm: Arc::new(w_volatile_norm),
            w_comp_norm: Arc::new(w_comp_norm),
            w_chaos_norm: Arc::new(w_chaos_norm),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::sensors::HeliosSensors;
    use crate::strategy::core_v3::config::HeliosConfigV3;

    #[test]
    fn test_simplex_normalization_nan_resilience() {
        let mut sensors = HeliosSensors::default();
        let len = 10;
        // Giả lập dữ liệu có chứa NaN trong chaos_ratio và kama_slope_adj
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.natr = Arc::new(vec![f64::NAN; len]);
        sensors.bb_upper = Arc::new(vec![105.0; len]);
        sensors.bb_lower = Arc::new(vec![95.0; len]);
        sensors.bb_middle = Arc::new(vec![100.0; len]);
        sensors.kc_upper = Arc::new(vec![106.0; len]);
        sensors.kc_lower = Arc::new(vec![94.0; len]);
        sensors.adx = Arc::new(vec![f64::NAN; len]);
        sensors.kama_slope_adj = Arc::new(vec![f64::NAN; len]);
        sensors.mfi = Arc::new(vec![f64::NAN; len]);
        sensors.chaos_ratio = Arc::new(vec![f64::NAN; len]);
        sensors.efficiency_ratio = Arc::new(vec![f64::NAN; len]);
        sensors.z_score = Arc::new(vec![f64::NAN; len]);
        sensors.vol_z_score = Arc::new(vec![f64::NAN; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in 0..len {
            let wt = matrix.w_trend_norm[i];
            let wr = matrix.w_range_norm[i];
            let wv = matrix.w_volatile_norm[i];
            let wc = matrix.w_comp_norm[i];
            let wch = matrix.w_chaos_norm[i];

            assert!(!wt.is_nan(), "w_trend không được là NaN tại index {}", i);
            assert!(!wr.is_nan(), "w_range không được là NaN tại index {}", i);
            assert!(!wv.is_nan(), "w_volatile không được là NaN tại index {}", i);
            assert!(!wc.is_nan(), "w_comp không được là NaN tại index {}", i);
            assert!(!wch.is_nan(), "w_chaos không được là NaN tại index {}", i);

            let sum = wt + wr + wc + wch;
            assert!((sum - 1.0).abs() < 1e-4, "Tổng 4-State Simplex ({}) phải xấp xỉ 1.0 tại index {}", sum, i);
        }
    }

    #[test]
    fn test_whipsaw_discrimination_prevents_fake_range() {
        let mut sensors = HeliosSensors::default();
        let len = 50;
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.natr = Arc::new(vec![3.0; len]);
        sensors.bb_upper = Arc::new(vec![105.0; len]);
        sensors.bb_lower = Arc::new(vec![95.0; len]);
        sensors.bb_middle = Arc::new(vec![100.0; len]);
        sensors.kc_upper = Arc::new(vec![104.0; len]);
        sensors.kc_lower = Arc::new(vec![96.0; len]);
        // Bão Whipsaw: ADX thấp (không có trend), KAMA slope phẳng
        sensors.adx = Arc::new(vec![12.0; len]);
        sensors.kama_slope_adj = Arc::new(vec![0.0; len]);
        sensors.mfi = Arc::new(vec![50.0; len]);
        // Giả lập bối cảnh: Các nến trước bình thường (0.8), 15 nến cuối bão Whipsaw bùng nổ (3.5)
        let mut chaos_vec = vec![0.8; len];
        for k in (len - 15)..len {
            chaos_vec[k] = 3.5;
        }
        sensors.chaos_ratio = Arc::new(chaos_vec);
        sensors.efficiency_ratio = Arc::new(vec![0.1; len]);
        sensors.z_score = Arc::new(vec![0.2; len]);
        sensors.vol_z_score = Arc::new(vec![0.1; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in (len - 10)..len {
            let wr = matrix.w_range_norm[i];
            let wch = matrix.w_chaos_norm[i];
            let wc = matrix.w_comp_norm[i];
            let wt = matrix.w_trend_norm[i];
            let sum = wt + wr + wc + wch;
            assert!((sum - 1.0).abs() < 1e-4, "Tổng 4-state ({}) phải = 1.0", sum);
            assert!(wch > wr, "w_chaos ({}) phải lớn hơn w_range ({}) khi bão Whipsaw", wch, wr);
            assert!(wr < 0.35, "w_range ({}) không được ngộ nhận bão thành Clean Range (> 0.50)", wr);
        }
    }
}
