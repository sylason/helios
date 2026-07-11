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
    
    // --- ⚖️ CƠ CHẾ PHÂN XỬ XẾP HẠNG (ABSOLUTE THRESHOLD ARBITRATION) ---
    // Mỗi squad cạnh tranh công bằng dựa trên raw score.
    let mut candidates: Vec<(MarketRegime, f64)> = vec![
        (MarketRegime::Trending,   scores.trend),
        (MarketRegime::Squeeze,    scores.squeeze),
        (MarketRegime::Ranging,    scores.ranging),
        (MarketRegime::Absorption, scores.absorption),
        (MarketRegime::Anomaly,    scores.anomaly),
        (MarketRegime::Parabolic,  scores.parabolic),
    ];

    // 1. Sắp xếp giảm dần theo Score
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // 2. Chọn kẻ đứng đầu và áp dụng Cầu chì Damping
    let (best_squad, max_raw_score) = candidates[0];
    
    // [FIXED Task 2.2]: Final Score = Raw Score * Stability Index.
    // Nếu môi trường hỗn loạn (index thấp), điểm số sẽ bị kéo xuống tự nhiên.
    let final_score = max_raw_score * env.stability_index;

    let priority = &config.market_regime.priority;
    let min_thresh = priority.min_score_threshold;

    // Nếu confidence cuối cùng chưa đủ ngưỡng tối thiểu → đứng ngoài quan sát.
    // Optimizer sẽ tự học cách điều chỉnh threshold hoặc né vùng index thấp.
    if final_score <= min_thresh {
        return TacticalPlaybook::Wait;
    }

    // 3. Ánh xạ Squad chiến thắng sang Playbook tương ứng
    match best_squad {
        
        // 🌪️ THẾ TRẬN SQUEEZE
        MarketRegime::Squeeze => {
            let extreme_thresh = config.market_regime.phase.accel_min.max(0.5);
            if matrix_rsi > extreme_thresh || matrix_rsi < (1.0 - extreme_thresh) {
                TacticalPlaybook::SqueezeAggressive
            } else {
                TacticalPlaybook::SqueezeConfirmed
            }
        },

        // 🌊 THẾ TRẬN TRENDING
        MarketRegime::Trending => {
            // --- [BUG #7 - VẤN ĐỀ UNIT MISMATCH]: trend_angle vs angle_parabolic ---
            // Vấn đề: tham số `trend_angle` truyền vào hàm này là trend_angle_ADJ
            //   (giá trị percentile [0.0, 1.0]), KHÔNG PHẢI angle degrees.
            //   Nhưng angle_parabolic trong config là degrees (60.0 - 85.0).
            //   → So sánh: 0.035 > 60.0 → LUÔN FALSE → Parabolic filter không bao giờ kích hoạt.
            //
            // [FIX #7]: Dùng trend_angle_adj [0, 1] làm ngưỡng Parabolic thay vì degrees.
            //   Quy ước mới: trend_angle_adj > 0.85 = xu hướng đang ở vùng parabolic
            //   (tức là slope hiện tại nằm trong top 15% lịch sử → dốc bất thường)
            //   → Không deploy TrendPullback khi đang Parabolic (rủi ro bắt đỉnh)
            //
            // Cũ (sai unit): trend_angle.abs() > config.market_regime.trend.angle_parabolic
            // Mới (đúng):   trend_angle        > config.market_regime.parabolic.accel_angle_threshold
            //                                   (accel_angle_threshold range: 0.8 - 2.5, tuy nhiên
            //                                    vì trend_angle_adj max là 1.0, ta clamp tại 1.0)
            let parabolic_thresh = config.market_regime.parabolic.accel_angle_threshold.min(1.0);
            let is_parabolic = trend_angle > parabolic_thresh;

            if trend_dir == TrendDirection::Neutral || is_parabolic {
                return TacticalPlaybook::Wait;
            }
            let rsi_thresh = config.tactical_sniper.momentum.rsi_threshold;
            let rsi_climax = config.tactical_sniper.momentum.rsi_climax;
            let is_momentum = (matrix_rsi > rsi_thresh && matrix_rsi < rsi_climax) || 
                              (matrix_rsi < (1.0 - rsi_thresh) && matrix_rsi > (1.0 - rsi_climax));
            
            if is_momentum {
                TacticalPlaybook::MomentumIgnition
            } else {
                TacticalPlaybook::TrendPullback
            }
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
