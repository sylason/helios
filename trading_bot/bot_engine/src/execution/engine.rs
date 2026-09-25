use async_trait::async_trait;
use crate::error::BotError;
use crate::models::models::Candle;
use crate::controller::asset_actor::LivePositionState;
use ferro_trade::broker::actions::Action;
use crate::models::account_state::AccountEvent;

/// Bản hợp đồng chung cho tất cả các Động cơ Thực thi lệnh (ExecutionEngine)
/// Cho phép đa hình hóa giữa Paper Trading (Local Simulation) và Live Trading (Binance REST/WS).
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Thực thi một hành động giao dịch từ Strategy (Mua, Bán, Cập nhật SL/TP)
    async fn execute_action(&mut self, action: &Action, current_price: f64) -> Result<(), BotError>;

    /// Thực thi một hành động giao dịch với ngữ cảnh nến (timestamp, bar_index)
    async fn execute_action_ctx(
        &mut self, 
        action: &Action, 
        current_price: f64, 
        _timestamp: i64, 
        _bar_idx: usize
    ) -> Result<(), BotError> {
        self.execute_action(action, current_price).await
    }

    /// Quét nến mới (quét Trailing SL/TP local cho Paper, hoặc no-op cho Live)
    async fn on_candle(&mut self, candle: &Candle) -> Result<(), BotError>;

    /// Quét nến mới với chỉ số nến trong bộ đệm
    async fn on_candle_ctx(
        &mut self, 
        candle: &Candle, 
        _bar_idx: usize
    ) -> Result<(), BotError> {
        self.on_candle(candle).await
    }

    /// Lấy số dư vốn thực tế hiện tại (Equity)
    fn get_equity(&self) -> f64;

    /// Lấy ảnh chụp trạng thái vị thế hiện tại (Position Snapshot)
    fn get_position(&self) -> Option<LivePositionState>;

    /// Hook nhận sự kiện tài khoản từ EventBus (dành cho Live/Testnet engine cập nhật từ WS)
    async fn on_account_event(&mut self, event: &AccountEvent);

    /// Gán chỉ số nến mở vị thế chính xác (live_entry_bar) trong bộ đệm
    fn set_live_entry_bar(&mut self, _bar_idx: usize) {}

    /// Báo cáo tích lũy kết quả Paper Trading (No-op cho Live)
    fn log_paper_report(&self, _current_price: f64, _time_label: &str) {}

    /// Báo cáo chi tiết tất cả các lệnh đã đóng Paper Trading (No-op cho Live)
    fn log_all_trades_report(&self, _current_price: f64) {}

    /// Báo cáo tổng kết toàn diện 100% lịch sử Paper Trading (No-op cho Live)
    fn log_final_summary(&self, _current_price: f64) {}

    /// Rút thông tin lệnh vừa đóng gần nhất (nếu có) để phục vụ Cầu Dao và hạch toán PnL
    fn take_last_closed_trade(&mut self) -> Option<crate::execution::paper_trader::PaperTradeRecord> {
        None
    }
}

/// Thành phần cốt lõi dùng chung (ExecutionCore) cho các Engine
/// Đóng gói các logic làm tròn giá, khối lượng và kiểm tra ràng buộc sàn giao dịch.
#[derive(Debug, Clone)]
pub struct ExecutionCore {
    pub symbol: String,
    pub market_filter: crate::models::market_info::MarketFilter,
    pub allocated_capital: f64,
    pub leverage: f64,
}

impl ExecutionCore {
    pub fn new(
        symbol: String,
        market_filter: crate::models::market_info::MarketFilter,
        allocated_capital: f64,
        leverage: f64,
    ) -> Self {
        Self {
            symbol,
            market_filter,
            allocated_capital,
            leverage,
        }
    }

    /// Làm tròn giá và khối lượng theo quy định của Sàn
    pub fn validate_and_format(&self, price: f64, qty: f64) -> Result<(f64, f64), BotError> {
        let p = self.market_filter.round_price(price);
        let q = self.market_filter.round_qty(qty);
        self.market_filter.validate_order(p, q).map_err(|e| BotError::MarketFilterError(e.to_string()))?;
        Ok((p, q))
    }
}

/// Nhà máy Khởi tạo Động cơ Thực thi (ExecutionEngineFactory)
pub struct ExecutionEngineFactory;

impl ExecutionEngineFactory {
    /// Tạo Động cơ Giả lập (Paper Execution Engine)
    pub fn create_paper(
        symbol: &str,
        allocated_capital: f64,
        leverage: f64,
        commission_rate: f64,
        market_filter: crate::models::market_info::MarketFilter,
        account_tx: Option<tokio::sync::broadcast::Sender<AccountEvent>>,
    ) -> Box<dyn ExecutionEngine> {
        let core = ExecutionCore::new(symbol.to_string(), market_filter, allocated_capital, leverage);
        Box::new(crate::execution::paper_engine::PaperExecutionEngine::new(
            core,
            allocated_capital,
            commission_rate,
            account_tx,
        ))
    }

    /// Tạo Động cơ Sàn thật (Binance Live / Testnet Execution Engine)
    pub fn create_live(
        symbol: &str,
        allocated_capital: f64,
        leverage: f64,
        market_filter: crate::models::market_info::MarketFilter,
        executor: std::sync::Arc<crate::execution::order_executor::OrderExecutor>,
        account_tx: tokio::sync::broadcast::Sender<AccountEvent>,
    ) -> Box<dyn ExecutionEngine> {
        let core = ExecutionCore::new(symbol.to_string(), market_filter, allocated_capital, leverage);
        Box::new(crate::execution::live_engine::BinanceLiveExecutionEngine::new(
            core,
            executor,
            account_tx,
        ))
    }
}

