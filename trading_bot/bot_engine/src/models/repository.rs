use crate::models::models::{Candle, OptimizedDna};
use crate::models::error::BotError;
use async_trait::async_trait;

/// Bản hợp đồng chung cho việc lưu trữ nến
#[async_trait]
pub trait CandleRepository: Send + Sync {
    /// Lưu một nến mới vào Database
    async fn save_candle(&self, candle: Candle) -> Result<(), BotError>;
    
    /// Lưu hàng loạt nến (dùng khi Sync dữ liệu lịch sử)
    async fn save_candles(&self, candles: Vec<Candle>) -> Result<(), BotError>;

    /// Lấy nến trong một khoảng thời gian (phục vụ Optimize)
    async fn get_candles_by_range(&self, symbol: &str, start_time: i64, end_time: i64) -> Result<Vec<Candle>, BotError>;

    /// Lấy nến mới nhất để Fill Gap
    async fn get_latest_candle(&self, symbol: &str) -> Result<Option<Candle>, BotError>;
    /// Lấy nến cũ nhất
    async fn get_oldest_candle(&self, symbol: &str) -> Result<Option<Candle>, BotError>;

    /// Đếm số lượng nến hiện có
    async fn count_candles(&self, symbol: &str) -> Result<usize, BotError>;

    /// Xóa nến cũ để giải phóng ổ cứng (ví dụ: giữ lại 30 ngày)
    async fn cleanup_old_data(&self, older_than_timestamp: i64) -> Result<(), BotError>;
}

/// Bản hợp đồng cho việc lưu trữ tham số chiến lược
#[async_trait]
pub trait StrategyRepository: Send + Sync {
    /// Lưu DNA tốt nhất vừa tìm được cho một symbol và strategy
    async fn save_best_dna(&self, dna: OptimizedDna) -> Result<(), BotError>;

    /// Lấy DNA tốt nhất của một symbol cho một strategy cụ thể
    async fn get_best_dna(&self, symbol: &str, strategy_id: &str) -> Result<Option<OptimizedDna>, BotError>;
}
