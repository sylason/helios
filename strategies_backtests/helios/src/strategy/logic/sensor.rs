// strategies_backtests/helios/src/strategy/logic/sensor.rs

use crate::strategy::config::EnvironmentFuseConfig;
use crate::strategy::memory::TacticalMemory;
use crate::types::MarketEnvironment;

/// CẤP ĐỘ 1: ENVIRONMENT FUSE (CẦU CHÌ MÔI TRƯỜNG)
/// [FIXED Task 2.1]: Chuyển từ Binary Gate (0/1) sang Stability Index (0.0 -> 1.0).
/// Cung cấp gradient cho Optimizer thay vì cắt lệnh đột ngột.
pub fn analyze_environment(
    config: &EnvironmentFuseConfig,
    memory: &mut TacticalMemory,
    chaos: f64,
    volume_force: f64,
    entropy: f64,
) -> MarketEnvironment {
    // --- 0. KIỂM TRA ĐỦ DỮ LIỆU ---
    if chaos.is_nan() || volume_force.is_nan() || entropy.is_nan() {
        return MarketEnvironment { stability_index: 0.0 };
    }

    // --- 1. MÀNG LỌC HOẢNG LOẠN (VOLUME SPIKE) (FIX #25) ---
    // Nếu Volume vọt lên quá cao (gấp 5 lần trung bình), báo động Chaotic ngay.
    if volume_force > 5.0 {
        return MarketEnvironment { stability_index: 0.0 };
    }
    
    // --- 2. TÍNH TOÁN ĐIỂM SỐ THÀNH PHẦN (GRADIENT SCORES) ---
    // Càng gần ngưỡng vi phạm, điểm càng thấp.
    let chaos_score = (1.0 - (chaos / config.chaos_z_threshold)).clamp(0.0, 1.0);
    let liq_score = (volume_force / config.min_liquidity_ratio).clamp(0.0, 1.0);
    let ent_score = (entropy / config.er_threshold).clamp(0.0, 1.0);

    // --- 3. LOGIC XÁC NHẬN "BỘ ĐẾM KIÊN NHẪN" (FIX #24) ---
    // Phải tích lũy đủ điểm trật tự mới cho phép hoạt động hoàn toàn.
    let is_all_ok = chaos < config.chaos_z_threshold && 
                    volume_force > config.min_liquidity_ratio && 
                    entropy > config.er_threshold;

    if is_all_ok {
        memory.state_duration_counter = (memory.state_duration_counter + 1).min(config.min_orderly_bars * 2);
    } else {
        memory.state_duration_counter = memory.state_duration_counter.saturating_sub(1);
    }

    let persistence_factor = (memory.state_duration_counter as f64 / config.min_orderly_bars as f64).clamp(0.0, 1.0);

    // --- 4. TỔNG HỢP STABILITY INDEX ---
    // stability_index = (Trung bình cộng các chỉ số) * Hệ số kiên nhẫn.
    let raw_index = (chaos_score + liq_score + ent_score) / 3.0;
    let final_index = raw_index * persistence_factor;

    MarketEnvironment { stability_index: final_index }
}
