// strategies_backtests/helios/src/types.rs

use serde::{Deserialize, Serialize};

/// Cấp độ 1: Cảm biến môi trường (Màng lọc an toàn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketEnvironment {
    Orderly, // Trật tự - Cho phép giao dịch
    Chaotic, // Hỗn loạn - Đứng ngoài
}

impl Default for MarketEnvironment {
    fn default() -> Self {
        Self::Orderly
    }
}

/// MA TRẬN NGỮ CẢNH (Level 2 Matrix)
/// Chứa các chỉ số đã chuẩn hóa (0.0 -> 1.0 hoặc -1.0 -> 1.0) để Optimizer Micro-managing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketContext {
    // --- 📈 TREND DIMENSION ---
    pub trend_dir: f64,       // -1.0 (Bear) đến 1.0 (Bull)
    pub trend_strength: f64,  // 0.0 đến 1.0 (ADX Normalized)
    pub trend_consistency: f64, // 0.0 đến 1.0 (Độ mượt LinReg)

    // --- 🌀 COMPRESSION DIMENSION ---
    pub squeeze_intensity: f64, // 0.0 đến 1.0 (BB/KC Width ratio)
    pub squeeze_duration: f64,  // 0.0 đến 1.0 (Normalized count)

    // --- 🌪️ ENERGY & VOLATILITY ---
    pub volatility_chaos: f64,  // Z-Score (ATR)
    pub volume_force: f64,      // 0.0 đến 1.0 (Vol/Vol_MA)

    // --- ⚡ MOMENTUM DIMENSION ---
    pub mom_velocity: f64,      // RSI Normalized
    pub mom_accel: f64,         // -1.0 đến 1.0 (ROC of Momentum)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketRegimeScores {
    pub trend: f64,
    pub squeeze: f64,
    pub ranging: f64,
    pub absorption: f64,
    pub anomaly: f64,
    pub parabolic: f64,
}

/// Cấp độ 2: Phân loại thế trận (Dùng để hiển thị/debug nhanh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    Trending,   // Xu hướng ổn định
    Squeeze,    // Nén lò xo
    Ranging,    // Đi ngang biên độ
    Absorption, // Gom hàng / Phân phối
    Anomaly,    // Bất thường / Flash Crash
    Parabolic,  // Tăng hưng phấn / Kiệt sức (NEW)
}

impl Default for MarketRegime {
    fn default() -> Self {
        Self::Ranging
    }
}

/// Hướng của xu hướng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}

impl Default for TrendDirection {
    fn default() -> Self {
        Self::Neutral
    }
}

/// Cấp độ 3: Ma trận Playbook (Kế hoạch tác chiến / Đội SWAT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalPlaybook {
    // 🌪️ Đội Squeeze SWAT (Chuyên đánh phá vỡ)
    SqueezeAggressive, // Vào ngay khi chớm phá vỡ
    SqueezeConfirmed,  // Đợi xác nhận Retest hoặc nến đóng cửa

    // 🌊 Đội Trend SWAT (Chuyên đánh bám sóng)
    TrendPullback,     // Đợi giá hồi về vùng an toàn mới vào (NEW)

    // 🚀 Đội Momentum SWAT (Chuyên đánh đuổi theo xung lực)
    MomentumIgnition,  // Đuổi theo khi dòng tiền bùng nổ mạnh (NEW)

    // 🏓 Đội Range SWAT (Chuyên đánh đảo chiều biên độ)
    RangeOscillator,   // Đánh bật lại khi chạm trần/sàn hành lang (NEW)

    // 🌋 Đội Reversal SWAT (Chuyên đánh chặn đầu kiệt sức)
    ClimaxReversal,    // Bắt đỉnh/đáy khi giá bay thẳng đứng - Parabolic (NEW)

    // 🧽 Đội Absorption SWAT (Chuyên đánh gom hàng)
    AbsorptionScout,   // Vào lệnh thăm dò khi phát hiện dấu vết cá mập

    // ⚡ Đội Anomaly SWAT (Chuyên đánh phản ứng nhanh tai nạn)
    AnomalyFade,       // Bắt cú hồi kỹ thuật sau Flash Crash

    // 🧱 Đội Phòng thủ
    Wait,              // Đứng ngoài quan sát
}

impl Default for TacticalPlaybook {
    fn default() -> Self {
        Self::Wait
    }
}

/// Trạng thái tức thời của thị trường tại một nến.
#[derive(Debug, Clone, Default)]
pub struct MarketState {
    pub environment: MarketEnvironment,
    pub matrix: MarketContext, // MA TRẬN ĐÃ CHUẨN HÓA
    pub regime: MarketRegime,
    pub direction: TrendDirection,
    pub playbook: TacticalPlaybook,
    pub confidence_score: f64, // Score tổng hợp để quản lý vốn biến thiên
}
