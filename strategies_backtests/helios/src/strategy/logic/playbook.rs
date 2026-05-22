// strategies_backtests/helios/src/strategy/logic/playbook.rs

use crate::types::*;
use crate::strategy::config::HeliosConfig;

/// HỘI ĐỒNG ĐIỀU PHỐI ĐỘI ĐẶC NHIỆM (Level 3: Playbook Arbitrator)
/// Phân xử và chọn ra Kế hoạch tác chiến tối ưu dựa trên Bảng điểm và Trọng số ưu tiên.
pub fn select_playbook(
    config: &HeliosConfig,
    env: MarketEnvironment,
    scores: &MarketRegimeScores, // NEW: Nhận bảng điểm thay vì 1 nhãn duy nhất
    trend_dir: TrendDirection,
    trend_angle: f64, 
    matrix_rsi: f64, 
) -> TacticalPlaybook {
    
    // --- 🛡️ BẢO VỆ TUYỆT ĐỐI ---
    if env == MarketEnvironment::Chaotic {
        return TacticalPlaybook::Wait;
    }

    let priority = &config.market_regime.priority;
    let min_thresh = priority.min_score_threshold;

    // --- ⚖️ CƠ CHẾ PHÂN XỬ (ARBITRATION) ---
    // 1. Tính điểm trọng số (Weighted Scores) cho từng Squad
    let w_anomaly = scores.anomaly * priority.anomaly;
    let w_absorption = scores.absorption * priority.absorption;
    let w_parabolic = scores.parabolic * priority.parabolic;
    let w_squeeze = scores.squeeze * priority.squeeze;
    let w_trend = scores.trend * priority.trend;
    let w_ranging = scores.ranging * priority.ranging;

    // 2. Tìm Squad có điểm trọng số cao nhất vượt ngưỡng tối thiểu
    let mut best_squad = MarketRegime::Ranging;
    let mut max_weighted_score = min_thresh;

    // --- 🕵️ DIAGNOSTIC LOG (Chỉ in khi điểm cao hoặc có lệnh) ---
    if w_trend > 0.4 || w_anomaly > 0.4 {
         println!("[COUNCIL] T:{:.2} (R:{:.2}*W:{:.2}) | A:{:.2} (R:{:.2}*W:{:.2}) | Thresh:{:.2}", 
            w_trend, scores.trend, priority.trend,
            w_anomaly, scores.anomaly, priority.anomaly,
            min_thresh
         );
    }

    // Ưu tiên theo thứ tự (nếu điểm bằng nhau): Anomaly -> Parabolic -> Squeeze -> Trend -> Ranging
    if w_ranging > max_weighted_score {
        best_squad = MarketRegime::Ranging;
        max_weighted_score = w_ranging;
    }
    if w_trend > max_weighted_score {
        best_squad = MarketRegime::Trending;
        max_weighted_score = w_trend;
    }
    if w_squeeze > max_weighted_score {
        best_squad = MarketRegime::Squeeze;
        max_weighted_score = w_squeeze;
    }
    if w_parabolic > max_weighted_score {
        best_squad = MarketRegime::Parabolic;
        max_weighted_score = w_parabolic;
    }
    if w_absorption > max_weighted_score {
        best_squad = MarketRegime::Absorption;
        max_weighted_score = w_absorption;
    }
    if w_anomaly > max_weighted_score {
        best_squad = MarketRegime::Anomaly;
        max_weighted_score = w_anomaly;
    }

    // Nếu không có squad nào vượt ngưỡng tối thiểu, đứng ngoài quan sát
    if max_weighted_score <= min_thresh {
        return TacticalPlaybook::Wait;
    }

    // 3. Ánh xạ Squad chiến thắng sang Playbook tương ứng
    match best_squad {
        
        // 🌪️ THẾ TRẬN SQUEEZE
        MarketRegime::Squeeze => {
            if matrix_rsi > config.market_regime.phase.accel_min || matrix_rsi < (1.0 - config.market_regime.phase.accel_min) {
                TacticalPlaybook::SqueezeAggressive
            } else {
                TacticalPlaybook::SqueezeConfirmed
            }
        },

        // 🌊 THẾ TRẬN TRENDING
        MarketRegime::Trending => {
            if trend_dir == TrendDirection::Neutral || trend_angle.abs() > config.market_regime.trend.angle_parabolic {
                return TacticalPlaybook::Wait;
            }
            TacticalPlaybook::TrendPullback
        },

        // 🧱 THẾ TRẬN RANGING
        MarketRegime::Ranging => TacticalPlaybook::RangeOscillator,

        // 🧽 THẾ TRẬN ABSORPTION
        MarketRegime::Absorption => {
            if matrix_rsi > config.market_regime.absorption.bias_threshold || matrix_rsi < (1.0 - config.market_regime.absorption.bias_threshold) {
                TacticalPlaybook::AbsorptionScout
            } else {
                TacticalPlaybook::Wait
            }
        },

        // ⚡ THẾ TRẬN ANOMALY
        MarketRegime::Anomaly => TacticalPlaybook::AnomalyFade,

        // 🌋 THẾ TRẬN PARABOLIC
        MarketRegime::Parabolic => TacticalPlaybook::ClimaxReversal,
    }
}
