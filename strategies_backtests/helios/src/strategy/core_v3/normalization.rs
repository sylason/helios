// strategies_backtests/helios/src/strategy/core_v3/normalization.rs
//! # UNIFIED QUANT NORMALIZATION ENGINE
//!
//! Module toán học chuẩn hóa phi thứ nguyên dùng chung cho toàn bộ các Tactical Scanners trong Helios V3.
//!
//! ## Nguyên tắc thiết kế (First-Principles):
//! 1. **Thước đo phi thứ nguyên thuần túy (Dimensionless):** Sử dụng hàm bão hòa Tanh Half-Saturation.
//! 2. **Neo chuẩn đồng nhất (Uniform Benchmarks):**
//!    - Khi $x = x_{\text{thresh}}$ (đạt chuẩn cấu hình): Điểm luôn bằng $\tanh(1.0) \approx 0.7616$ (Tier B).
//!    - Khi $x = 2 \cdot x_{\text{thresh}}$ (vượt trội hoàn hảo): Điểm đạt $\tanh(2.0) \approx 0.9640$ (Tier A).
//!    - Khi $x \le 0$: Điểm bằng $0.0$.
//! 3. **Tổ hợp lồi an toàn (Convex Combination):** Tổng trọng số luôn được chuẩn hóa về 1.0, an toàn tuyệt đối với NaN.

/// Chuẩn hóa đại lượng dương không chặn trên (Z-Score, KAMA Slope, Volume Force, Displacement)
/// Công thức: $\tanh(\max(val, 0.0) / \max(thresh, 1e-5))$
#[inline]
pub fn sat_unbounded(val: f64, thresh: f64) -> f64 {
    if val.is_nan() || thresh.is_nan() || val <= 0.0 {
        return 0.0;
    }
    let t = thresh.max(1e-5);
    (val / t).tanh().clamp(0.0, 1.0)
}

/// Chuẩn hóa tỉ số nỗ lực vs kết quả (Effort vs Result của Wyckoff)
/// Áp dụng căn bậc hai $\sqrt{num / den} / thresh$ để chống bùng nổ điểm ảo khi nến nén thân hẹp.
#[inline]
pub fn sat_ratio_damped(numerator: f64, denominator: f64, thresh: f64) -> f64 {
    if numerator.is_nan() || denominator.is_nan() || thresh.is_nan() || numerator <= 0.0 {
        return 0.0;
    }
    let den = denominator.max(0.10); // Sàn biên độ tối thiểu 0.10 ATR
    let ratio = numerator / den;
    let damped = ratio.sqrt();
    let t = thresh.max(1e-5);
    (damped / t).tanh().clamp(0.0, 1.0)
}

/// Chuẩn hóa khoảng cách bứt phá (Breakout Penetration) so với mục tiêu tối thiểu
/// Khi vượt qua target_dist: Điểm đạt $\ge \tanh(1.0) \approx 0.76$
#[inline]
pub fn sat_penetration(dist: f64, target_dist: f64) -> f64 {
    if dist.is_nan() || target_dist.is_nan() || dist <= 0.0 {
        return 0.0;
    }
    let t = target_dist.max(1e-5);
    (dist / t).tanh().clamp(0.0, 1.0)
}

/// Chuẩn hóa độ gần của giá tới vùng hỗ trợ/kháng cự (Pullback Proximity)
/// - Khi dist = 0.0 (ngay tại hỗ trợ): Điểm đạt $\tanh(2.0) \approx 0.964$ (Tier A).
/// - Khi dist = 0.5 * max_dist (nửa đường): Điểm đạt $\tanh(1.0) \approx 0.762$ (Tier B).
/// - Khi dist >= max_dist (chạm ngưỡng vi phạm): Điểm về $0.0$.
#[inline]
pub fn sat_proximity(dist: f64, max_dist: f64) -> f64 {
    if dist.is_nan() || max_dist.is_nan() || max_dist <= 1e-5 {
        return 0.0;
    }
    let ratio = (dist.max(0.0) / max_dist).clamp(0.0, 1.0);
    let closeness = 1.0 - ratio;
    // Nhân 2.0 để khi closeness = 0.5 (nửa đường) thì đạt tanh(1.0) = 0.762
    (2.0 * closeness).tanh().clamp(0.0, 1.0)
}

/// Tổ hợp lồi an toàn tuyệt đối cho N thành phần điểm
/// Công thức: $\frac{\sum w_i \cdot s_i}{\sum w_i}$
/// Đảm bảo luôn nằm trong $[0.0, 1.0]$ và miễn nhiễm hoàn toàn với NaN.
#[inline]
pub fn convex_combine(scores: &[f64], weights: &[f64]) -> f64 {
    if scores.is_empty() || weights.is_empty() || scores.len() != weights.len() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut sum_w = 0.0;

    for (&s, &w) in scores.iter().zip(weights.iter()) {
        if s.is_nan() || w.is_nan() || w <= 0.0 {
            continue;
        }
        let valid_s = s.clamp(0.0, 1.0);
        weighted_sum += valid_s * w;
        sum_w += w;
    }

    if sum_w <= 1e-9 {
        0.0
    } else {
        (weighted_sum / sum_w).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sat_unbounded_benchmarks() {
        let thresh = 2.0;
        // Điểm 0
        assert_eq!(sat_unbounded(0.0, thresh), 0.0);
        assert_eq!(sat_unbounded(-1.0, thresh), 0.0);
        assert_eq!(sat_unbounded(f64::NAN, thresh), 0.0);

        // Tier B: x = thresh -> tanh(1.0) ~ 0.7616
        let score_b = sat_unbounded(2.0, thresh);
        assert!((score_b - 1.0_f64.tanh()).abs() < 1e-6);

        // Tier A: x = 2 * thresh -> tanh(2.0) ~ 0.9640
        let score_a = sat_unbounded(4.0, thresh);
        assert!((score_a - 2.0_f64.tanh()).abs() < 1e-6);
    }

    #[test]
    fn test_sat_proximity_benchmarks() {
        let max_dist = 1.0;
        // dist = 0.0 -> closeness = 1.0 -> tanh(2.0) ~ 0.9640 (Tier A)
        let at_support = sat_proximity(0.0, max_dist);
        assert!((at_support - 2.0_f64.tanh()).abs() < 1e-6);

        // dist = 0.5 -> closeness = 0.5 -> tanh(1.0) ~ 0.7616 (Tier B)
        let halfway = sat_proximity(0.5, max_dist);
        assert!((halfway - 1.0_f64.tanh()).abs() < 1e-6);

        // dist = 1.0 -> closeness = 0.0 -> 0.0
        let at_boundary = sat_proximity(1.0, max_dist);
        assert_eq!(at_boundary, 0.0);

        // dist > 1.0 -> clamp 0.0
        assert_eq!(sat_proximity(1.5, max_dist), 0.0);
    }

    #[test]
    fn test_sat_ratio_damped_prevents_explosion() {
        let thresh = 1.5;
        // norm_spread rất nhỏ (0.10) và volume lớn (2.0) -> ratio = 20.0
        // damped = sqrt(20.0) ~ 4.47 -> tanh(4.47 / 1.5) ~ tanh(2.98) ~ 0.995 (không bị blow up quá sớm)
        let score = sat_ratio_damped(2.0, 0.10, thresh);
        assert!(score < 1.0);
        assert!(score > 0.90);
    }

    #[test]
    fn test_convex_combine_robustness() {
        let scores = [0.80, 0.60, f64::NAN];
        let weights = [0.50, 0.50, 0.20];
        let res = convex_combine(&scores, &weights);
        // Bỏ qua NaN: (0.80 * 0.50 + 0.60 * 0.50) / (0.50 + 0.50) = 0.70
        assert!((res - 0.70).abs() < 1e-6);
    }
}
