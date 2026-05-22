// strategies_backtests/helios/src/strategy/logic/sensor.rs

use crate::strategy::config::EnvironmentFuseConfig;
use crate::strategy::memory::TacticalMemory;
use crate::types::MarketEnvironment;

/// CẤP ĐỘ 1: ENVIRONMENT FUSE (CẦU CHÌ MÔI TRƯỜNG)
/// Hệ thống "Cầu chì" bảo vệ tài khoản khỏi hoảng loạn và nhiễu.
pub fn analyze_environment(
    config: &EnvironmentFuseConfig,
    memory: &mut TacticalMemory,
    chaos: f64,
    volume_force: f64,
    entropy: f64,
) -> MarketEnvironment {
    // --- 0. KIỂM TRA ĐỦ DỮ LIỆU ---
    if chaos.is_nan() || volume_force.is_nan() || entropy.is_nan() {
        return MarketEnvironment::Chaotic;
    }

    // --- 1. MÀNG LỌC HOẢNG LOẠN (VOLUME SPIKE) (FIX #25) ---
    // Nếu Volume vọt lên quá cao (gấp 5 lần trung bình), báo động Chaotic ngay.
    if volume_force > 5.0 {
        return MarketEnvironment::Chaotic;
    }
    
    // --- 2. KIỂM TRA 3 VAN AN TOÀN (HARD STOP) ---
    let chaos_ok = chaos < config.chaos_z_threshold;
    let liquidity_ok = volume_force > config.min_liquidity_ratio;
    let entropy_ok = entropy > config.er_threshold;

    // --- 3. LOGIC XÁC NHẬN "BỘ ĐẾM KIÊN NHẪN" (FIX #24) ---
    if chaos_ok && liquidity_ok && entropy_ok {
        // Nếu tất cả van đều mở, tăng điểm tích lũy
        memory.state_duration_counter = (memory.state_duration_counter + 1).min(config.min_orderly_bars * 2);
    } else {
        // Thay vì reset về 0, ta phạt điểm để Bot kiên nhẫn hơn với nhiễu nhẹ
        memory.state_duration_counter = memory.state_duration_counter.saturating_sub(3);
    }

    // Phải tích lũy đủ điểm trật tự mới cho phép hoạt động
    if memory.state_duration_counter >= config.min_orderly_bars {
        MarketEnvironment::Orderly
    } else {
        MarketEnvironment::Chaotic
    }
}
