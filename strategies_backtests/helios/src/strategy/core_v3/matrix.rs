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
    // --- 🚀 3 TRỤC ĐO VẬT LÝ TRỰC GIAO ---
    pub kama_slope_atr: Arc<Vec<f64>>,   // Độ dốc KAMA chuẩn hóa theo ATR phi thứ nguyên [-1.0, 1.0]
    pub chaos_index: Arc<Vec<f64>>,      // Chỉ số bão hỗn loạn Whipsaw [0.0, 1.0]
    pub trend_strength: Arc<Vec<f64>>,   // Sức mạnh xu hướng ADX tuyến tính [0.0, 1.0]

    // --- 🧭 Hướng & Biến động Dải Bollinger/Keltner ---
    pub bb_middle_slope_adj: Arc<Vec<f64>>, // 🛡️ FLAW-REG.10: Độ dốc chuẩn hóa của trục giữa BB Middle (SMA 20) [-2.0, 2.0]
    pub squeeze_intensity: Arc<Vec<f64>>,   // Cường độ nén BB/KC [0.0, 1.0]
    pub bb_width_norm: Arc<Vec<f64>>,       // (Upper - Lower) / Middle kẹp trần [0.0, 1.0]
}

impl MarketMatrixV3 {
    pub fn from_sensors(config: &HeliosConfigV3, sensors: &HeliosSensors) -> Self {
        let len = sensors.closes.len();
        if len == 0 {
            return Self::default();
        }
        
        let mut bb_width_norm = Vec::with_capacity(len);
        let mut sqz_raw = Vec::with_capacity(len);

        for i in 0..len {
            let bu = sensors.bb_upper.get(i).copied().unwrap_or(f64::NAN);
            let bl = sensors.bb_lower.get(i).copied().unwrap_or(f64::NAN);
            let bm = sensors.bb_middle.get(i).copied().unwrap_or(f64::NAN);
            let bb_w = bu - bl;

            // 🛡️ TASK-MAT-08: Kẹp trần 1.0 cho bb_width_norm chống bùng nổ vô tận khi flash crash
            let norm = if bm > 1e-9 && bb_w.is_finite() {
                (bb_w.max(0.0) / bm).clamp(0.0, 1.0)
            } else {
                0.0
            };
            bb_width_norm.push(norm);

            let ku = sensors.kc_upper.get(i).copied().unwrap_or(f64::NAN);
            let kl = sensors.kc_lower.get(i).copied().unwrap_or(f64::NAN);
            let kc_w = ku - kl;

            // 🚀 TASK-MAT-02 & TASK-MAT-06 (Safe Math Soft Squeeze):
            // - Mẫu số kc_w > 1e-9 và tử số bb_w hữu hạn
            // - Khi kc_w <= 1e-9 hoặc dữ liệu không hợp lệ -> sqz_raw = 0.0 (không nén)
            let sqz_continuous = if kc_w > 1e-9 && bb_w.is_finite() {
                let ratio = (bb_w.max(0.0) / kc_w).max(0.0);
                let val = 1.0 / (1.0 + ratio * ratio);
                val.clamp(0.0, 1.0)
            } else {
                0.0
            };
            sqz_raw.push(sqz_continuous);
        }

        // TÍNH TOÁN CÁC DÒNG CHUẨN HÓA V3.3
        let sqz_lb = if config.matrix.squeeze_lookback == 0 { 20 } else { config.matrix.squeeze_lookback };
        let trend_lb = if config.matrix.trend_lookback == 0 { 20 } else { config.matrix.trend_lookback };

        // 🚀 TASK-MAT-03 & TASK-MAT-06: Chuẩn hóa tuyến tính thuần khiết ADX / 100.0 (Occam's Scalpel)
        let mut trend_strength = Vec::with_capacity(len);
        for i in 0..len {
            let adx_v = sensors.adx.get(i).copied().unwrap_or(f64::NAN);
            let val = if adx_v.is_finite() && adx_v > 0.0 {
                (adx_v / 100.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            trend_strength.push(val);
        }

        let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
        // 🛡️ TASK-MAT-07: chaos_ratio và entropy là biến nội bộ tạm thời để tính chaos_index
        let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, sqz_lb);
        let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, trend_lb);

        // 🚀 1. DỐC KAMA CHUẨN HÓA THEO ATR (PHI THỨ NGUYÊN - ASSET AGNOSTIC)
        let scale_atr = if config.matrix.kama_slope_scale_atr <= 0.0 { 0.20 } else { config.matrix.kama_slope_scale_atr };
        let kama_slope_atr: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
            if s.is_finite() { (s / scale_atr).clamp(-1.0, 1.0) } else { 0.0 }
        }).collect();

        // 🚀 2. CHỈ SỐ BÃO HỖN LOẠN WHIPSAW (chaos_index = c_r * (1.0 - entropy))
        let mut chaos_index = Vec::with_capacity(len);
        for i in 0..len {
            let c_r_raw = chaos_ratio.get(i).copied().unwrap_or(f64::NAN);
            let ent_raw = entropy.get(i).copied().unwrap_or(f64::NAN);

            let ch = if c_r_raw.is_finite() && ent_raw.is_finite() {
                let c_r = c_r_raw.clamp(0.0, 1.0);
                let noise = (1.0 - ent_raw.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                c_r * noise
            } else {
                0.0
            };

            chaos_index.push(ch);
        }
        
        // 🚀 3. ĐỘ DỐC TRỤC GIỮA BB MIDDLE (KẸP BIÊN [-2.0, 2.0] THEO TASK-MAT-08)
        let bb_middle_slope_adj: Vec<f64> = sensors.bb_middle_slope_adj.iter().map(|&s| {
            if s.is_finite() { s.clamp(-2.0, 2.0) } else { 0.0 }
        }).collect();

        Self {
            kama_slope_atr: Arc::new(kama_slope_atr),
            chaos_index: Arc::new(chaos_index),
            trend_strength: Arc::new(trend_strength),
            bb_middle_slope_adj: Arc::new(bb_middle_slope_adj),
            squeeze_intensity: Arc::new(squeeze_intensity),
            bb_width_norm: Arc::new(bb_width_norm),
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
            let trend = matrix.trend_strength[i];
            let sqz = matrix.squeeze_intensity[i];
            let chaos = matrix.chaos_index[i];

            assert!(!slope.is_nan(), "kama_slope_atr không được là NaN tại index {}", i);
            assert!(!trend.is_nan(), "trend_strength không được là NaN tại index {}", i);
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
            assert!(matrix.trend_strength[i] >= 0.50, "Trend strength phải mạnh khi ADX = 50");
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

    #[test]
    fn test_matrix_safe_math_nan_resilience() {
        // 🛡️ TASK-MAT-06: Kiểm chứng Safe Math Guards triệt tiêu hoàn toàn NaN, Infinity, NegInfinity
        let len = 20;
        let mut sensors = HeliosSensors::default();
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![105.0; len]);
        sensors.lows = Arc::new(vec![95.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);

        // Trộn lẫn các giá trị dị tật toán học
        sensors.natr = Arc::new(vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 2.0].into_iter().cycle().take(len).collect());
        sensors.bb_upper = Arc::new(vec![f64::NAN, 110.0, f64::INFINITY, 105.0].into_iter().cycle().take(len).collect());
        sensors.bb_lower = Arc::new(vec![90.0, f64::NAN, 95.0, f64::NEG_INFINITY].into_iter().cycle().take(len).collect());
        sensors.bb_middle = Arc::new(vec![0.0, f64::NAN, 100.0, f64::INFINITY].into_iter().cycle().take(len).collect());
        sensors.kc_upper = Arc::new(vec![f64::NAN, 108.0, 104.0, f64::INFINITY].into_iter().cycle().take(len).collect());
        sensors.kc_lower = Arc::new(vec![92.0, f64::NAN, 96.0, f64::NEG_INFINITY].into_iter().cycle().take(len).collect());
        sensors.adx = Arc::new(vec![f64::NAN, f64::INFINITY, -10.0, 35.0].into_iter().cycle().take(len).collect());
        sensors.kama_slope_adj = Arc::new(vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.5].into_iter().cycle().take(len).collect());
        sensors.chaos_ratio = Arc::new(vec![f64::NAN, f64::INFINITY, 1.2, 0.8].into_iter().cycle().take(len).collect());
        sensors.efficiency_ratio = Arc::new(vec![f64::NAN, 0.5, f64::INFINITY, 0.9].into_iter().cycle().take(len).collect());

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in 0..len {
            assert!(matrix.bb_width_norm[i].is_finite(), "bb_width_norm[{}] phải là số hữu hạn", i);
            assert!(matrix.bb_width_norm[i] >= 0.0 && matrix.bb_width_norm[i] <= 1.0, "bb_width_norm[{}] phải trong [0.0, 1.0]", i);

            assert!(matrix.squeeze_intensity[i].is_finite(), "squeeze_intensity[{}] phải là số hữu hạn", i);
            assert!(matrix.squeeze_intensity[i] >= 0.0 && matrix.squeeze_intensity[i] <= 1.0, "squeeze_intensity[{}] phải trong [0.0, 1.0]", i);

            assert!(matrix.trend_strength[i].is_finite(), "trend_strength[{}] phải là số hữu hạn", i);
            assert!(matrix.trend_strength[i] >= 0.0 && matrix.trend_strength[i] <= 1.0, "trend_strength[{}] phải trong [0.0, 1.0]", i);

            assert!(matrix.kama_slope_atr[i].is_finite(), "kama_slope_atr[{}] phải là số hữu hạn", i);
            assert!(matrix.kama_slope_atr[i] >= -1.0 && matrix.kama_slope_atr[i] <= 1.0, "kama_slope_atr[{}] phải trong [-1.0, 1.0]", i);

            assert!(matrix.chaos_index[i].is_finite(), "chaos_index[{}] phải là số hữu hạn", i);
            assert!(matrix.chaos_index[i] >= 0.0 && matrix.chaos_index[i] <= 1.0, "chaos_index[{}] phải trong [0.0, 1.0]", i);
        }
    }

    #[test]
    fn test_bb_width_norm_clamped() {
        // 🛡️ TASK-MAT-08: Kiểm chứng kẹp trần 1.0 khi Flash Crash bung band cực đại
        let len = 5;
        let mut sensors = HeliosSensors::default();
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![1000.0; len]);
        sensors.lows = Arc::new(vec![10.0; len]);
        sensors.volumes = Arc::new(vec![1000.0; len]);

        // BB Upper = 1000.0, Lower = 10.0, Middle = 50.0 => (1000 - 10) / 50 = 19.8 => phải kẹp về 1.0
        sensors.bb_upper = Arc::new(vec![1000.0; len]);
        sensors.bb_lower = Arc::new(vec![10.0; len]);
        sensors.bb_middle = Arc::new(vec![50.0; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in 0..len {
            assert_eq!(matrix.bb_width_norm[i], 1.0, "bb_width_norm phải được kẹp trần chính xác tại 1.0 khi bung band cực đại");
        }
    }

    #[test]
    fn test_zero_volatility_flat_bars() {
        // 🛡️ Safe Math Guard: Thị trường đứng im phẳng lì (High = Low = Close, BB_w = 0, KC_w = 0)
        let len = 10;
        let mut sensors = HeliosSensors::default();
        sensors.closes = Arc::new(vec![100.0; len]);
        sensors.highs = Arc::new(vec![100.0; len]);
        sensors.lows = Arc::new(vec![100.0; len]);
        sensors.volumes = Arc::new(vec![0.0; len]);

        sensors.bb_upper = Arc::new(vec![100.0; len]);
        sensors.bb_lower = Arc::new(vec![100.0; len]);
        sensors.bb_middle = Arc::new(vec![100.0; len]);
        sensors.kc_upper = Arc::new(vec![100.0; len]);
        sensors.kc_lower = Arc::new(vec![100.0; len]);

        let config = HeliosConfigV3::default();
        let matrix = MarketMatrixV3::from_sensors(&config, &sensors);

        for i in 0..len {
            assert_eq!(matrix.bb_width_norm[i], 0.0, "Độ rộng BB nến phẳng lì phải là 0.0");
            assert!(matrix.squeeze_intensity[i].is_finite(), "KC_w = 0 không được panic chia cho 0");
            assert!(matrix.squeeze_intensity[i] >= 0.0 && matrix.squeeze_intensity[i] <= 1.0, "squeeze_intensity phải trong [0.0, 1.0]");
            // Khi toàn bộ mảng sqz_raw đều bằng 0.0 (tất cả phần tử bằng nhau), rolling_percentile trả về 0.50 (trung vị thống kê)
            assert_eq!(matrix.squeeze_intensity[i], 0.5, "Khi mọi phần tử đều bằng nhau (0.0), rolling percentile rơi vào trung vị 0.50");
        }
    }
}
