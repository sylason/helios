// strategies_backtests/helios/src/strategy/memory.rs

use crate::types::*;

/// Bộ nhớ chiến thuật của Helios (Tactical Memory).
/// Lưu trữ các dấu vết để "Mahoraga" có thể học và thích nghi với thị trường.
#[derive(Debug, Clone)]
pub struct TacticalMemory {
    /// Trạng thái của thị trường tại nến ngay trước đó (t-1).
    pub previous_market_state: MarketState,

    /// Thế trận của nến trước đó (phục vụ bộ lọc nhiễu chuyển trạng thái) (FIX #20)
    pub last_regime: MarketRegime,
    
    /// Số nến đã trôi qua kể từ khi thế trận thay đổi (FIX #20)
    pub bars_since_regime_change: usize,

    /// Giá cao nhất (đối với Long) hoặc thấp nhất (đối với Short) kể từ khi mở lệnh (FIX #16)
    pub peak_price: Option<f64>,

    /// Kết quả của lệnh giao dịch gần nhất (Lợi nhuận %) (FIX #9)
    pub last_trade_result: f64,

    /// Biến động trung bình làm hệ quy chiếu (SMA của ATR) (FIX #10)
    pub volatility_benchmark: f64,
    
    /// Bộ đếm số nến liên tiếp duy trì trạng thái hiện tại.
    /// Giúp lọc nhiễu cho khung thời gian 1-phút (Confirmation duration).
    pub state_duration_counter: usize,
    
    /// Thời gian hồi phục (tính bằng số nến) kể từ giao dịch cuối cùng.
    /// Triết lý Quyền Anh: Không tung đòn liên tục khi vừa bị choáng.
    pub recovery_period_counter: usize,
    
    /// Vị trí (index) của nến khi bắt đầu vào lệnh hiện tại.
    pub current_entry_index: Option<usize>,

    /// Đánh dấu lệnh hiện tại đã được dời SL về điểm hòa vốn hay chưa (FIX #18)
    pub is_breakeven_active: bool,
}

impl Default for TacticalMemory {
    fn default() -> Self {
        Self {
            previous_market_state: MarketState::default(),
            last_regime: MarketRegime::Ranging,
            bars_since_regime_change: 0,
            peak_price: None,
            last_trade_result: 0.0,
            volatility_benchmark: 0.0,
            state_duration_counter: 0,
            recovery_period_counter: 0,
            current_entry_index: None,
            is_breakeven_active: false,
        }
    }
}
