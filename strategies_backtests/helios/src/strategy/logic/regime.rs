// strategies_backtests/helios/src/strategy/logic/regime.rs

use crate::types::*;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;

/// PHÂN TÍCH THẾ TRẬN ĐA NHIỆM (Level 2: Multi-Regime Scoring)
/// Thay vì trả về 1 nhãn duy nhất, ta tính điểm "thèm khát" cho mọi Squad.
pub fn analyze_regime(
    config: &HeliosConfig,
    memory: &mut TacticalMemory,
    squeeze_intensity: f64,
    trend_strength: f64,
    trend_angle: f64,
    volume_force: f64,    
    bb_width_norm: f64,   
    price_z_score: f64,   
    chaos_ratio: f64,    
) -> (MarketRegime, MarketRegimeScores) {
    let reg_cfg = &config.market_regime;
    let mut scores = MarketRegimeScores::default();

    // --- 🛡️ KIỂM TRA DỮ LIỆU ---
    if squeeze_intensity.is_nan() || trend_strength.is_nan() || trend_angle.is_nan() {
        return (MarketRegime::Ranging, scores);
    }

    // 1. ANOMALY SCORE ⚡
    //
    // --- [BUG #9 - CŨ - ĐÃ VÔ HIỆU HÓA] ---
    // Vấn đề: Double Tanh — price_z_score ĐÃ được tanh() trong matrix.rs,
    //   nhân thêm z_score_threshold rồi tanh() lần nữa → signal bị nén 2 lần.
    //   Ngoài ra, z_score_threshold đóng vai trò MULTIPLIER (không phải threshold),
    //   tên biến sai → Optimizer hiểu nhầm semantic, tối ưu hóa biến vô nghĩa.
    //
    // Cụ thể:
    //   price_z_score đã ở dải [-1, 1] (sau tanh trong matrix)
    //   z_score_norm = |price_z_score| * z_score_threshold  → dải [0, ~0.9]
    //   anomaly_score = tanh(z_score_norm * 0.8 + chaos * 0.2) → bị nén thêm lần nữa
    //   Kết quả: anomaly_score gần như hằng số, không có gradient
    //
    // let z_score_norm = price_z_score.abs() * reg_cfg.anomaly.z_score_threshold;
    // let chaos_boost = chaos_ratio / 1.2;
    // scores.anomaly = (z_score_norm * 0.8 + chaos_boost * 0.2).tanh();

    // --- [FIX #9 - MỚI]: Dùng price_z_score trực tiếp, tanh một lần duy nhất ---
    //
    // Ý nghĩa mới của z_score_threshold:
    //   - Là ngưỡng phân loại thực sự: nếu |z_score| < threshold → anomaly score thấp
    //   - Khi |price_z_score| = threshold → score ≈ tanh(1.0) ≈ 0.76 (ngưỡng cảnh báo)
    //   - Khi |price_z_score| >> threshold → score tiến về 1.0 (anomaly rõ ràng)
    //
    // Đây là cách dùng đúng: threshold chia để "scale" tốc độ tăng của score.
    let chaos_boost = chaos_ratio / 1.2;
    let z_component  = price_z_score.abs() / reg_cfg.anomaly.z_score_threshold.max(0.01);
    let chaos_component = chaos_boost;
    scores.anomaly = (z_component * 0.8 + chaos_component * 0.2).tanh(); // Soft-cap tại 1.0

    // 2. TREND SCORE 🌊 (Soft-Capped)
    let is_currently_trending = memory.last_regime == MarketRegime::Trending;
    let threshold_buffer = if is_currently_trending { 0.8 } else { 1.0 };

    let angle_norm = trend_angle.abs() / reg_cfg.trend.angle_healthy; 
    let adx_norm = trend_strength / reg_cfg.trend.adx_moderate;
    
    let weight_sum = reg_cfg.confidence.weight_angle + reg_cfg.confidence.weight_adx;
    let confidence = if weight_sum > 0.0 {
        (angle_norm * reg_cfg.confidence.weight_angle + adx_norm * reg_cfg.confidence.weight_adx) / weight_sum
    } else {
        0.0
    };
    scores.trend = (confidence / (reg_cfg.confidence.min_threshold * threshold_buffer)).tanh();

    // 3. SQUEEZE SCORE 🔥 (Soft-Capped)
    scores.squeeze = (squeeze_intensity / reg_cfg.squeeze.threshold).tanh();

    // 4. ABSORPTION SCORE 🧽 (Soft-Capped)
    if bb_width_norm < reg_cfg.absorption.range_width_max {
        scores.absorption = (volume_force / reg_cfg.absorption.volume_threshold).tanh();
    }

    // 5. PARABOLIC SCORE 🌋 (Soft-Capped)
    scores.parabolic = (trend_angle.abs() / reg_cfg.parabolic.angle_threshold).tanh();

    // 6. RANGING SCORE 🧱 (Mặc định khi các thế trận khác yếu)
    scores.ranging = (1.0 - scores.trend.max(scores.squeeze).max(scores.anomaly).max(scores.absorption)).max(0.01).min(1.0);

    // --- 🧭 XÁC ĐỊNH NHÃN CHÍNH (PRIMARY REGIME) ---
    // Dùng để tương thích ngược với Trailing Stop và Tagging.
    // [FIX] Hạ ngưỡng max_val xuống 0.1 để các Squad có cửa thắng Ranging.
    let mut primary = MarketRegime::Ranging;
    let mut max_val = 0.1; 

    if scores.anomaly > max_val {
        primary = MarketRegime::Anomaly;
        max_val = scores.anomaly;
    }
    if scores.absorption > max_val {
        primary = MarketRegime::Absorption;
        max_val = scores.absorption;
    }
    if scores.parabolic > max_val {
        primary = MarketRegime::Parabolic;
        max_val = scores.parabolic;
    }
    if scores.squeeze > max_val {
        primary = MarketRegime::Squeeze;
        max_val = scores.squeeze;
    }
    if scores.trend > max_val {
        primary = MarketRegime::Trending;
    }

    update_regime_memory(memory, primary);
    (primary, scores)
}

/// Helper để cập nhật bộ nhớ Thế trận
fn update_regime_memory(memory: &mut TacticalMemory, current_regime: MarketRegime) {
    if memory.last_regime == current_regime {
        memory.bars_since_regime_change += 1;
    } else {
        memory.last_regime = current_regime;
        memory.bars_since_regime_change = 0;
    }
}

/// XÁC ĐỊNH HƯỚNG XU HƯỚNG
pub fn detect_trend_direction(trend_dir: f64) -> TrendDirection {
    let buffer = 0.15; 
    if trend_dir > buffer {
        TrendDirection::Bullish
    } else if trend_dir < -buffer {
        TrendDirection::Bearish
    } else {
        TrendDirection::Neutral
    }
}
