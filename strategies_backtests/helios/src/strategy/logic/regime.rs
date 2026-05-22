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

    // 1. ANOMALY SCORE ⚡ (Soft-Capped)
    let z_score_norm = price_z_score.abs() / reg_cfg.anomaly.z_score_threshold;
    let chaos_boost = chaos_ratio / 1.2; 
    scores.anomaly = (z_score_norm * 0.8 + chaos_boost * 0.2).tanh(); // Soft-cap tại 1.0

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
    scores.ranging = (1.0 - scores.trend.max(scores.squeeze).max(scores.anomaly)).max(0.01).min(1.0);

    // --- 🧭 XÁC ĐỊNH NHÃN CHÍNH (PRIMARY REGIME) ---
    // Dùng để tương thích ngược với Trailing Stop và Tagging.
    // Chọn nhãn có điểm cao nhất vượt ngưỡng 1.0, theo thứ tự ưu tiên truyền thống nếu xấp xỉ.
    let mut primary = MarketRegime::Ranging;
    let mut max_val = 1.0; // Chỉ công nhận nếu điểm > 1.0

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
