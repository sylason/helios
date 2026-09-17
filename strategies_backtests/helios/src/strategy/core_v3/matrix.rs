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
    // --- 🚀 4 TRỤC ĐO VẬT LÝ TRỰC GIAO MỚI (DE-OVERENGINEERED ORTHOGONAL AXES) ---
    pub kama_slope_atr: Arc<Vec<f64>>,   // Độ dốc KAMA chuẩn hóa theo ATR phi thứ nguyên [-1.0, 1.0]
    pub trend_axis: Arc<Vec<f64>>,       // Trục xu hướng ADX chuẩn hóa theo adx_trend_threshold [0.0, 1.0]
    pub vol_axis: Arc<Vec<f64>>,         // Trục nén/bung biến động (Squeeze / Expansion) [0.0, 1.0]
    pub flow_axis: Arc<Vec<f64>>,        // Trục dòng tiền Z-Score Volume chuẩn hóa theo vol_z_sensitivity [0.0, 1.0]
    pub chaos_index: Arc<Vec<f64>>,      // Chỉ số bão hỗn loạn Whipsaw [0.0, 1.0]

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
            let natr_val = sensors.natr.get(i).copied().unwrap_or(0.0);
            chaos.push(natr_val / 100.0);
            
            let bu = sensors.bb_upper.get(i).copied().unwrap_or(0.0);
            let bl = sensors.bb_lower.get(i).copied().unwrap_or(0.0);
            let bm = sensors.bb_middle.get(i).copied().unwrap_or(0.0);
            bb_width_norm.push(if bm > 0.0 { (bu - bl) / bm } else { 0.0 });

            let ku = sensors.kc_upper.get(i).copied().unwrap_or(0.0);
            let kl = sensors.kc_lower.get(i).copied().unwrap_or(0.0);
            let kc_w = ku - kl;
            if kc_w < 1e-9 || bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
                sqz_raw.push(0.0);
            } else {
                let overlap = (ku - bu) + (bl - kl);
                sqz_raw.push((overlap / kc_w).max(0.0).min(1.0));
            }

            // ADX Centered tại ngưỡng 20.0 để dùng Tanh Scale
            let adx_v = sensors.adx.get(i).copied().unwrap_or(0.0);
            adx_centered.push(if adx_v.is_nan() { 0.0 } else { adx_v - 20.0 });
        }

        // TÍNH TOÁN CÁC DÒNG CHUẨN HÓA V3.3
        let z_sens = if config.matrix.z_score_sensitivity <= 0.0 { 1.0 } else { config.matrix.z_score_sensitivity };
        let sqz_lb = if config.matrix.squeeze_lookback == 0 { 20 } else { config.matrix.squeeze_lookback };
        let trend_lb = if config.matrix.trend_lookback == 0 { 20 } else { config.matrix.trend_lookback };
        let mom_lb = if config.matrix.momentum_lookback == 0 { 20 } else { config.matrix.momentum_lookback };

        let trend_strength = normalization::tanh_scale(&adx_centered, z_sens);

        let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
        let liquidity = normalization::rolling_percentile(&sensors.mfi, mom_lb);
        let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, sqz_lb);
        let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, trend_lb);
        let price_z_score = normalization::tanh_scale(&sensors.z_score, z_sens);

        // 🚀 1. DỐC KAMA CHUẨN HÓA THEO ATR (PHI THỨ NGUYÊN - ASSET AGNOSTIC)
        let scale_atr = if config.matrix.kama_slope_scale_atr <= 0.0 { 0.20 } else { config.matrix.kama_slope_scale_atr };
        let kama_slope_atr: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
            if s.is_nan() { 0.0 } else { (s / scale_atr).max(-1.0).min(1.0) }
        }).collect();

        // 🚀 2. TRỤC XU HƯỚNG TRỰC GIAO (ADX THEO GEN adx_trend_threshold)
        let adx_thresh = if config.matrix.adx_trend_threshold <= 0.0 { 25.0 } else { config.matrix.adx_trend_threshold };
        let trend_axis: Vec<f64> = sensors.adx.iter().map(|&adx_v| {
            if adx_v.is_nan() { 0.0 } else { (adx_v / adx_thresh).max(0.0).min(1.0) }
        }).collect();

        // 🚀 3. TRỤC BIẾN ĐỘNG / NÉN SQUEEZE (vol_axis)
        let vol_axis = squeeze_intensity.clone();

        // 🚀 4. TRỤC DÒNG TIỀN / KHỐI LƯỢNG THEO vol_z_sensitivity (flow_axis)
        let vol_z_sens = if config.matrix.vol_z_sensitivity <= 0.0 { 1.50 } else { config.matrix.vol_z_sensitivity };
        let vol_z_scale = normalization::tanh_scale(&sensors.vol_z_score, 1.0 / vol_z_sens);
        let mut flow_axis = Vec::with_capacity(len);
        for i in 0..len {
            let mfi_val = sensors.mfi.get(i).copied().unwrap_or(50.0);
            let mfi_norm = (mfi_val / 100.0).max(0.0).min(1.0);
            let z_val = vol_z_scale.get(i).copied().unwrap_or(0.0);
            let vf = 0.5 * (z_val.abs()) + 0.5 * mfi_norm;
            flow_axis.push(vf.max(0.0).min(1.0));
        }

        // 🚀 5. CHỈ SỐ BÃO HỖN LOẠN WHIPSAW (chaos_index)
        let mut chaos_index = Vec::with_capacity(len);
        for i in 0..len {
            let c_r_raw = chaos_ratio.get(i).copied().unwrap_or(0.0);
            let c_r = if c_r_raw.is_nan() { 0.0 } else { c_r_raw };
            let z_abs = price_z_score.get(i).copied().unwrap_or(0.0).abs();
            let ch = 0.60 * c_r.max(0.0).min(1.0) + 0.40 * z_abs.max(0.0).min(1.0);
            chaos_index.push(ch.max(0.0).min(1.0));
        }
        
        let trend_dir = kama_slope_atr.clone();

        let volume_force = flow_axis.clone();

        Self {
            kama_slope_atr: Arc::new(kama_slope_atr),
            trend_axis: Arc::new(trend_axis),
            vol_axis: Arc::new(vol_axis),
            flow_axis: Arc::new(flow_axis),
            chaos_index: Arc::new(chaos_index),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::sensors::HeliosSensors;
    use crate::strategy::core_v3::config::HeliosConfigV3;

    #[test]
    fn test_orthogonal_matrix_nan_resilience() {
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
            let slope = matrix.kama_slope_atr[i];
            let trend = matrix.trend_axis[i];
            let vol = matrix.vol_axis[i];
            let flow = matrix.flow_axis[i];
            let chaos = matrix.chaos_index[i];

            assert!(!slope.is_nan(), "kama_slope_atr không được là NaN tại index {}", i);
            assert!(!trend.is_nan(), "trend_axis không được là NaN tại index {}", i);
            assert!(!vol.is_nan(), "vol_axis không được là NaN tại index {}", i);
            assert!(!flow.is_nan(), "flow_axis không được là NaN tại index {}", i);
            assert!(!chaos.is_nan(), "chaos_index không được là NaN tại index {}", i);

            assert!(slope >= -1.0 && slope <= 1.0, "slope phải trong [-1.0, 1.0]");
            assert!(trend >= 0.0 && trend <= 1.0, "trend phải trong [0.0, 1.0]");
            assert!(vol >= 0.0 && vol <= 1.0, "vol phải trong [0.0, 1.0]");
            assert!(flow >= 0.0 && flow <= 1.0, "flow phải trong [0.0, 1.0]");
            assert!(chaos >= 0.0 && chaos <= 1.0, "chaos phải trong [0.0, 1.0]");
        }
    }

    #[test]
    fn test_whipsaw_discrimination_chaos_index() {
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
        sensors.adx = Arc::new(vec![12.0; len]);
        sensors.kama_slope_adj = Arc::new(vec![0.0; len]);
        sensors.mfi = Arc::new(vec![50.0; len]);
        
        // Giả lập bối cảnh: 15 nến cuối bão Whipsaw bùng nổ (chaos_ratio = 3.5, z_score = 2.0)
        let mut chaos_vec = vec![0.8; len];
        let mut z_vec = vec![0.2; len];
        for k in (len - 15)..len {
            chaos_vec[k] = 3.5;
            z_vec[k] = 2.0;
        }
        sensors.chaos_ratio = Arc::new(chaos_vec);
        sensors.efficiency_ratio = Arc::new(vec![0.1; len]);
        sensors.z_score = Arc::new(z_vec);
        sensors.vol_z_score = Arc::new(vec![0.1; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in (len - 10)..len {
            let chaos = matrix.chaos_index[i];
            assert!(chaos > 0.50, "chaos_index ({}) phải tăng cao (> 0.50) khi bão Whipsaw", chaos);
        }
    }

    #[test]
    fn test_kama_slope_atr_scale_asset_agnostic() {
        // Kiểm chứng tính phi thứ nguyên:
        // Cùng tỷ lệ delta_kama / atr thì dù coin giá to hay nhỏ, kết quả kama_slope_atr phải như nhau!
        let mut sensors_eth = HeliosSensors::default();
        let mut sensors_doge = HeliosSensors::default();
        let len = 10;
        
        sensors_eth.closes = Arc::new(vec![2500.0; len]);
        sensors_eth.kama_slope_adj = Arc::new(vec![0.10; len]);
        
        sensors_doge.closes = Arc::new(vec![0.10; len]);
        sensors_doge.kama_slope_adj = Arc::new(vec![0.10; len]);

        let mut config = HeliosConfigV3::default();
        config.matrix.kama_slope_scale_atr = 0.20;

        let matrix_eth = MarketMatrixV3::from_sensors(&config, &sensors_eth);
        let matrix_doge = MarketMatrixV3::from_sensors(&config, &sensors_doge);

        for i in 0..len {
            let slope_eth = matrix_eth.kama_slope_atr[i];
            let slope_doge = matrix_doge.kama_slope_atr[i];
            assert_eq!(slope_eth, slope_doge, "kama_slope_atr phải giống nhau bất kể thị giá");
            assert!((slope_eth - 0.50).abs() < 1e-6, "0.10 / 0.20 = 0.50");
        }
    }

    #[test]
    fn test_matrix_independence_no_zero_sum() {
        // Kiểm chứng tính độc lập của 4 trục đo:
        // Khi Trend tăng mạnh, trục Vol và Flow không bị ép triệt tiêu về 0
        let mut sensors = HeliosSensors::default();
        let len = 20;
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.adx = Arc::new(vec![50.0; len]); // ADX siêu mạnh
        sensors.kama_slope_adj = Arc::new(vec![0.30; len]); // Trend tăng dốc
        sensors.vol_z_score = Arc::new(vec![2.5; len]); // Volume cũng bùng nổ cực mạnh!
        sensors.mfi = Arc::new(vec![80.0; len]);
        sensors.kc_upper = Arc::new(vec![106.0; len]);
        sensors.kc_lower = Arc::new(vec![94.0; len]);
        sensors.bb_upper = Arc::new(vec![105.0; len]);
        sensors.bb_lower = Arc::new(vec![95.0; len]);
        sensors.bb_middle = Arc::new(vec![100.0; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in 0..len {
            assert!(matrix.trend_axis[i] > 0.8, "Trend axis phải mạnh khi ADX = 50");
            assert!(matrix.flow_axis[i] > 0.6, "Flow axis phải độc lập duy trì mức cao, không bị Trend triệt tiêu!");
        }
    }
}
