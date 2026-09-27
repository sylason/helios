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
    // --- 🚀 3 TRỤC ĐO VẬT LÝ TRỰC GIAO (DE-OVERENGINEERED ORTHOGONAL AXES) ---
    // (vol_axis đã được xóa bỏ hoàn toàn do trùng lặp 100% với squeeze_intensity)
    pub kama_slope_atr: Arc<Vec<f64>>,   // Độ dốc KAMA chuẩn hóa theo ATR phi thứ nguyên [-1.0, 1.0]
    pub trend_axis: Arc<Vec<f64>>,       // Trục xu hướng ADX chuẩn hóa theo adx_trend_threshold [0.0, 1.0]
    pub chaos_index: Arc<Vec<f64>>,      // Chỉ số bão hỗn loạn Whipsaw [0.0, 1.0]

    // --- 🧭 Hướng & Sức mạnh (Directional) ---
    pub trend_strength: Arc<Vec<f64>>,    // 0.0 to 1.0 (ADX Tanh Scaled)
    pub kama_slope_adj: Arc<Vec<f64>>,    // KAMA Slope điều chỉnh theo ER
    pub bb_middle_slope_adj: Arc<Vec<f64>>, // 🛡️ FLAW-REG.10: Độ dốc chuẩn hóa của trục giữa BB Middle (SMA 20)
    
    // --- 🌪️ Năng lượng & Biến động (Chaos) ---
    pub squeeze_intensity: Arc<Vec<f64>>, // 0.0 to 1.0 (BB/KC Compression)
    pub chaos: Arc<Vec<f64>>,             // 0.0 to 0.10+ (NATR - Biến động % giá)
    pub chaos_ratio: Arc<Vec<f64>>,       // NATR / SMA(NATR)
    pub entropy: Arc<Vec<f64>>,           // 0.0 to 1.0 (Efficiency Ratio)
    pub liquidity: Arc<Vec<f64>>,         // 0.0 to 1.0 (MFI Scaled)
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
            let bb_w = bu - bl;
            if kc_w < 1e-9 || kc_w.is_nan() || bb_w.is_nan() || bu.is_nan() || bl.is_nan() || ku.is_nan() || kl.is_nan() {
                sqz_raw.push(0.0);
            } else {
                // 🚀 TASK-MAT-02 (Inverse Quadratic Soft Squeeze):
                // sqz_raw = 1.0 / (1.0 + (BB_w / KC_w)^2)
                // - Khi BB_w << KC_w (nén cực đại): sqz_raw -> 1.0
                // - Khi BB_w == KC_w (ngưỡng Carter): sqz_raw = 0.50
                // - Khi BB_w >> KC_w (bung nén/bão): sqz_raw -> 0.0 (không bao giờ kẹt 0.0 phẳng lỳ)
                let ratio = (bb_w.max(0.0) / kc_w).max(0.0);
                let sqz_continuous = 1.0 / (1.0 + ratio * ratio);
                sqz_raw.push(sqz_continuous.clamp(0.0, 1.0));
            }
        }

        // TÍNH TOÁN CÁC DÒNG CHUẨN HÓA V3.3
        let z_sens = if config.matrix.z_score_sensitivity <= 0.0 { 1.0 } else { config.matrix.z_score_sensitivity };
        let sqz_lb = if config.matrix.squeeze_lookback == 0 { 20 } else { config.matrix.squeeze_lookback };
        let trend_lb = if config.matrix.trend_lookback == 0 { 20 } else { config.matrix.trend_lookback };
        let mom_lb = if config.matrix.momentum_lookback == 0 { 20 } else { config.matrix.momentum_lookback };

        // 🚀 TASK-MAT-03: Chuẩn hóa tuyến tính thuần khiết ADX / 100.0 (Occam's Scalpel)
        // - Triệt tiêu vĩnh viễn số âm, đưa dải đo về chuẩn vô hướng [0.0, 1.0] (ADX 25.0 = 0.25).
        // - Loại bỏ hoàn toàn vector đệm adx_centered và chấm dứt ô nhiễm chéo z_score_sensitivity.
        let mut trend_strength = Vec::with_capacity(len);
        for i in 0..len {
            let adx_v = sensors.adx.get(i).copied().unwrap_or(0.0);
            let val = if adx_v.is_nan() || adx_v <= 0.0 {
                0.0
            } else {
                (adx_v / 100.0).min(1.0)
            };
            trend_strength.push(val);
        }

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

        // 🚀 3. CHỈ SỐ BÃO HỖN LOẠN WHIPSAW (chaos_index)
        // chaos_index = c_r * (1.0 - entropy)
        // Trong đó:
        // - c_r: Chaos Ratio (tỷ lệ biến động rối loạn, từ sensors.chaos_ratio)
        // - entropy: Kaufman Efficiency Ratio (ER), đo độ mượt của xu hướng (1.0 = đi thẳng một mạch, 0.0 = hoàn toàn ngẫu nhiên)
        // - (1.0 - entropy): Mức độ nhiễu loạn/ziczac (Noise/Whipsaw)
        // Khi Super Trend bùng nổ: c_r cao nhưng entropy ~ 1.0 -> (1.0 - entropy) ~ 0.0 -> chaos_index ~ 0.0 (không bị phạt oan)
        // Khi Bão Whipsaw: c_r cao và entropy ~ 0.0 -> (1.0 - entropy) ~ 1.0 -> chaos_index cao (kích hoạt phòng vệ)
        let mut chaos_index = Vec::with_capacity(len);
        for i in 0..len {
            let c_r_raw = chaos_ratio.get(i).copied().unwrap_or(f64::NAN);
            let ent_raw = entropy.get(i).copied().unwrap_or(f64::NAN);

            let ch = if c_r_raw.is_nan() || ent_raw.is_nan() {
                0.0
            } else {
                let c_r = c_r_raw.clamp(0.0, 1.0);
                let noise = (1.0 - ent_raw.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                c_r * noise
            };

            chaos_index.push(ch);
        }
        
        Self {
            kama_slope_atr: Arc::new(kama_slope_atr),
            trend_axis: Arc::new(trend_axis),
            chaos_index: Arc::new(chaos_index),
            trend_strength: Arc::new(trend_strength),
            kama_slope_adj: sensors.kama_slope_adj.clone(),
            bb_middle_slope_adj: sensors.bb_middle_slope_adj.clone(),
            squeeze_intensity: Arc::new(squeeze_intensity),
            bb_width_norm: Arc::new(bb_width_norm),
            liquidity: Arc::new(liquidity),
            chaos: Arc::new(chaos),
            chaos_ratio: Arc::new(chaos_ratio), 
            entropy: Arc::new(entropy),
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
            let sqz = matrix.squeeze_intensity[i];
            let chaos = matrix.chaos_index[i];

            assert!(!slope.is_nan(), "kama_slope_atr không được là NaN tại index {}", i);
            assert!(!trend.is_nan(), "trend_axis không được là NaN tại index {}", i);
            assert!(!sqz.is_nan(), "squeeze_intensity không được là NaN tại index {}", i);
            assert!(!chaos.is_nan(), "chaos_index không được là NaN tại index {}", i);

            assert!(slope >= -1.0 && slope <= 1.0, "slope phải trong [-1.0, 1.0]");
            assert!(trend >= 0.0 && trend <= 1.0, "trend phải trong [0.0, 1.0]");
            assert!(sqz >= 0.0 && sqz <= 1.0, "sqz phải trong [0.0, 1.0]");
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
        
        // Giả lập bối cảnh: 15 nến cuối bão Whipsaw bùng nổ (chaos_ratio = 3.5, z_score = 2.0, ER sụt giảm mạnh về 0.05)
        let mut chaos_vec = vec![0.8; len];
        let mut z_vec = vec![0.2; len];
        let mut er_vec = vec![0.8; len];
        for k in (len - 15)..len {
            chaos_vec[k] = 3.5;
            z_vec[k] = 2.0;
            er_vec[k] = 0.05;
        }
        sensors.chaos_ratio = Arc::new(chaos_vec);
        sensors.efficiency_ratio = Arc::new(er_vec);
        sensors.z_score = Arc::new(z_vec);
        sensors.vol_z_score = Arc::new(vec![0.1; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // Kiểm tra tại các nến bùng nổ bão Whipsaw (nến 35..45): chaos_index phải tăng cao (> 0.50)
        for i in (len - 15)..(len - 5) {
            let chaos = matrix.chaos_index[i];
            assert!(chaos > 0.50, "chaos_index ({}) phải tăng cao (> 0.50) khi bão Whipsaw bùng nổ", chaos);
        }
        // Toàn bộ giai đoạn bão (15 nến cuối) phải duy trì mức cảnh báo cao (> 0.35)
        for i in (len - 15)..len {
            let chaos = matrix.chaos_index[i];
            assert!(chaos > 0.35, "chaos_index ({}) phải duy trì mức cảnh báo (> 0.35) trong suốt cơn bão", chaos);
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
        }
    }

    #[test]
    fn test_super_trend_low_chaos() {
        // Kiểm chứng tính đúng đắn toán học của chaos_index mới:
        // Khi có Super Trend: c_r cao (biến động lớn), nhưng entropy ~ 1.0 (xu hướng đi thẳng, cực mượt)
        // => noise = 1.0 - entropy = 0.0 => chaos_index = 0.0 (KHÔNG bị phạt oan!)
        let mut sensors = HeliosSensors::default();
        let len = 20;
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.adx = Arc::new(vec![40.0; len]);
        sensors.kama_slope_adj = Arc::new(vec![0.20; len]);
        sensors.vol_z_score = Arc::new(vec![1.0; len]);
        sensors.mfi = Arc::new(vec![50.0; len]);
        sensors.kc_upper = Arc::new(vec![106.0; len]);
        sensors.kc_lower = Arc::new(vec![94.0; len]);
        sensors.bb_upper = Arc::new(vec![105.0; len]);
        sensors.bb_lower = Arc::new(vec![95.0; len]);
        sensors.bb_middle = Arc::new(vec![100.0; len]);

        // Giả lập chuỗi ER tăng dần để percentile tại index cuối tiệm cận 1.0
        let mut er_vec = Vec::with_capacity(len);
        let mut cr_vec = Vec::with_capacity(len);
        for i in 0..len {
            er_vec.push(i as f64 / len as f64);
            cr_vec.push(i as f64 / len as f64);
        }
        sensors.efficiency_ratio = Arc::new(er_vec);
        sensors.chaos_ratio = Arc::new(cr_vec);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // Tại phần tử cuối cùng: entropy percentile tiệm cận 1.0 (>= 0.95) => noise <= 0.05 => chaos_index ~ 0.024
        let last_idx = len - 1;
        assert!(matrix.entropy[last_idx] >= 0.95, "Entropy percentile phải tiệm cận 1.0 (>= 0.95)");
        assert!(matrix.chaos_ratio[last_idx] >= 0.95, "Chaos ratio percentile phải tiệm cận 1.0 (>= 0.95)");
        assert!(
            matrix.chaos_index[last_idx] < 0.03,
            "Khi xu hướng cực mượt (entropy >= 0.95), chaos_index ({}) phải tiệm cận 0.0 (< 0.03) bất kể biến động lớn!",
            matrix.chaos_index[last_idx]
        );
    }

    #[test]
    fn test_squeeze_continuous_no_half_stuck() {
        let mut sensors = HeliosSensors::default();
        let len = 50;
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![110.0; len]);
        sensors.lows = Arc::new(vec![90.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.natr = Arc::new(vec![2.0; len]);
        sensors.adx = Arc::new(vec![25.0; len]);
        sensors.kama_slope_adj = Arc::new(vec![0.0; len]);
        sensors.vol_z_score = Arc::new(vec![0.0; len]);
        sensors.mfi = Arc::new(vec![50.0; len]);
        sensors.chaos_ratio = Arc::new(vec![1.0; len]);
        sensors.efficiency_ratio = Arc::new(vec![0.5; len]);
        sensors.z_score = Arc::new(vec![0.0; len]);

        // Giả lập 30 nến đầu: Nén cực mạnh (BB = 4.0, KC = 10.0 => ratio = 0.4 => sqz ~ 0.86)
        // 20 nến sau: Bung nén bão lớn dần (BB tăng từ 12.0 đến 31.0, KC = 10.0 => ratio > 1.0)
        let mut bb_u = Vec::with_capacity(len);
        let mut bb_l = Vec::with_capacity(len);
        let mut kc_u = Vec::with_capacity(len);
        let mut kc_l = Vec::with_capacity(len);

        for i in 0..len {
            kc_u.push(105.0);
            kc_l.push(95.0); // KC width = 10.0
            if i < 30 {
                // Squeeze mạnh: BB width = 4.0
                bb_u.push(102.0);
                bb_l.push(98.0);
            } else {
                // Bung nén mở rộng: BB width = 12.0 + (i - 30) = 12..31
                let half_w = 6.0 + (i - 30) as f64 * 0.5;
                bb_u.push(100.0 + half_w);
                bb_l.push(100.0 - half_w);
            }
        }

        sensors.bb_upper = Arc::new(bb_u);
        sensors.bb_lower = Arc::new(bb_l);
        sensors.bb_middle = Arc::new(vec![100.0; len]);
        sensors.kc_upper = Arc::new(kc_u);
        sensors.kc_lower = Arc::new(kc_l);

        let mut config = HeliosConfigV3::default();
        config.matrix.squeeze_lookback = 20;

        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        // Nến cuối cùng (index 49) là lúc bung nén mạnh nhất (BB = 31, KC = 10, ratio = 3.1)
        // Với công thức cũ: cả 20 nến sau đều bị kẹt 0.0 => percentile kẹt cứng ở 0.50!
        // Với công thức mới: sqz_raw giảm liên tục từ 0.40 về 0.09, là giá trị thấp nhất trong window!
        let last_sqz = matrix.squeeze_intensity[len - 1];
        assert!(
            last_sqz < 0.20,
            "Khi biến động bung mạnh nhất, Squeeze Intensity ({}) phải ở vùng đáy (< 0.20), không được kẹt ở 0.50!",
            last_sqz
        );
    }

    #[test]
    fn test_trend_strength_linear_bounds() {
        let mut sensors = HeliosSensors::default();
        let test_adx = vec![f64::NAN, -5.0, 0.0, 15.0, 25.0, 50.0, 100.0, 120.0];
        let len = test_adx.len();

        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);
        sensors.adx = Arc::new(test_adx);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        assert_eq!(matrix.trend_strength[0], 0.0, "NaN phải trả về 0.0");
        assert_eq!(matrix.trend_strength[1], 0.0, "Số âm phải kẹp về 0.0");
        assert_eq!(matrix.trend_strength[2], 0.0, "ADX 0 phải trả về 0.0");
        assert!((matrix.trend_strength[3] - 0.15).abs() < 1e-6, "ADX 15 phải là 0.15 (không được âm!)");
        assert!((matrix.trend_strength[4] - 0.25).abs() < 1e-6, "ADX 25 phải là 0.25");
        assert!((matrix.trend_strength[5] - 0.50).abs() < 1e-6, "ADX 50 phải là 0.50");
        assert_eq!(matrix.trend_strength[6], 1.0, "ADX 100 phải là 1.0");
        assert_eq!(matrix.trend_strength[7], 1.0, "ADX > 100 phải kẹp trần 1.0");
    }
}
