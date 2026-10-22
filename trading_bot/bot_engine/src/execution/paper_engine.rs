use async_trait::async_trait;
use crate::error::BotError;
use crate::models::models::Candle;
use crate::controller::asset_actor::LivePositionState;
use crate::execution::engine::{ExecutionEngine, ExecutionCore};
use crate::execution::paper_trader::{PaperTrader, PositionType};
use ferro_trade::broker::actions::Action;
use crate::models::account_state::AccountEvent;
use tokio::sync::broadcast;
use tracing::debug;

/// Động cơ Thực thi Giả lập (Paper Execution Engine)
/// Xử lý khớp lệnh, quét Trailing SL/TP và kế toán Vốn hoàn toàn nội bộ (Zero-delay Local Simulation).
pub struct PaperExecutionEngine {
    pub core: ExecutionCore,
    pub paper_trader: PaperTrader,
    pub live_position: Option<LivePositionState>,
    pub account_tx: Option<broadcast::Sender<AccountEvent>>,
    pub last_close: f64,
}

impl PaperExecutionEngine {
    pub fn new(
        core: ExecutionCore,
        initial_equity: f64,
        commission_rate: f64,
        account_tx: Option<broadcast::Sender<AccountEvent>>,
    ) -> Self {
        let mut paper_trader = PaperTrader::new(core.symbol.clone(), initial_equity, commission_rate);
        paper_trader.leverage = core.leverage;

        Self {
            core,
            paper_trader,
            live_position: None,
            account_tx,
            last_close: 0.0,
        }
    }
}

#[async_trait]
impl ExecutionEngine for PaperExecutionEngine {
    async fn execute_action_ctx(
        &mut self, 
        action: &Action, 
        current_price: f64, 
        timestamp: i64, 
        bar_idx: usize
    ) -> Result<(), BotError> {
        debug!("📝 [{}] [PAPER ENGINE] Process Action: {:?}", self.core.symbol, action);

        // Trích xuất tag từ action (SL/TP được đọc trực tiếp từ paper_trader sau khi qua Ratchet Guard)
        let playbook_tag = match action {
            Action::Buy { tag, .. } | Action::Sell { tag, .. } => tag.clone(),
            _ => None,
        };

        // Cập nhật giá thị trường gần nhất để tính PnL thả nổi tức thì
        if current_price.is_finite() && current_price > 0.0 {
            self.last_close = current_price;
        }

        // Xử lý action trong PaperTrader (đã tích hợp Monotonic Ratchet Guard & Pessimistic Matching)
        self.paper_trader.process_action(action.clone(), current_price, timestamp, bar_idx, self.account_tx.as_ref());

        // Đồng bộ hóa trạng thái vị thế sang LivePositionState cho Strategy
        if self.paper_trader.position_type != PositionType::None {
            let pos_amt = match self.paper_trader.position_type {
                PositionType::Long => self.paper_trader.position_size,
                PositionType::Short => -self.paper_trader.position_size,
                PositionType::None => 0.0,
            };
            let (highest_fav, peak_ts) = match self.paper_trader.position_type {
                PositionType::Long => (self.paper_trader.highest_price_seen, self.paper_trader.peak_timestamp),
                PositionType::Short => (self.paper_trader.lowest_price_seen, self.paper_trader.peak_timestamp),
                PositionType::None => (0.0, 0),
            };

            if let Some(ref mut pos) = self.live_position {
                pos.position_amt = pos_amt;
                pos.entry_price = self.paper_trader.entry_price;
                if playbook_tag.is_some() { pos.opening_playbook_tag = playbook_tag; }
                // Luôn đồng bộ chuẩn xác mức SL/TP đã qua bộ lọc Monotonic Ratchet Guard của PaperTrader
                pos.current_sl_price = self.paper_trader.current_sl_price;
                pos.current_tp_price = self.paper_trader.current_tp_price;
                pos.highest_favorable_price = highest_fav;
                pos.peak_timestamp = peak_ts;
            } else {
                self.live_position = Some(LivePositionState {
                    symbol: self.core.symbol.clone(),
                    position_amt: pos_amt,
                    entry_price: self.paper_trader.entry_price,
                    entry_timestamp: timestamp,
                    live_entry_bar: bar_idx,
                    current_sl_price: self.paper_trader.current_sl_price,
                    current_tp_price: self.paper_trader.current_tp_price,
                    opening_playbook_tag: playbook_tag,
                    highest_favorable_price: highest_fav,
                    peak_timestamp: peak_ts,
                });
            }
        } else {
            self.live_position = None;
        }

        Ok(())
    }

    async fn execute_action(&mut self, action: &Action, current_price: f64) -> Result<(), BotError> {
        let now_ts = chrono::Utc::now().timestamp_millis();
        self.execute_action_ctx(action, current_price, now_ts, 0).await
    }

    async fn on_candle_ctx(&mut self, candle: &Candle, bar_idx: usize) -> Result<(), BotError> {
        // Cập nhật giá đóng cửa gần nhất từ nến cho việc định giá PnL thả nổi
        if candle.close.is_finite() && candle.close > 0.0 {
            self.last_close = candle.close;
        }

        // Quét nến real-time kiểm tra Trailing SL/TP
        if self.paper_trader.on_candle(candle, bar_idx, self.account_tx.as_ref()) {
            if self.paper_trader.position_type == PositionType::None {
                self.live_position = None;
            }
        }
        Ok(())
    }

    async fn on_candle(&mut self, candle: &Candle) -> Result<(), BotError> {
        self.on_candle_ctx(candle, 0).await
    }

    fn get_equity(&self) -> f64 {
        if self.paper_trader.position_type != PositionType::None 
            && self.last_close.is_finite() 
            && self.last_close > 0.0 
        {
            self.paper_trader.get_total_equity(self.last_close).max(0.0)
        } else {
            self.paper_trader.equity.max(0.0)
        }
    }

    fn get_position(&self) -> Option<LivePositionState> {
        self.live_position.clone()
    }

    fn set_live_entry_bar(&mut self, bar_idx: usize) {
        if let Some(ref mut pos) = self.live_position {
            if pos.live_entry_bar == 0 {
                pos.live_entry_bar = bar_idx;
            }
        }
        if self.paper_trader.entry_bar_index == 0 {
            self.paper_trader.entry_bar_index = bar_idx;
        }
    }

    async fn on_account_event(&mut self, _event: &AccountEvent) {
        // Paper Engine tự chủ 100%, bỏ qua các sự kiện từ ngoài để tránh ô nhiễm state
    }

    fn log_paper_report(&self, current_price: f64, time_label: &str) {
        self.paper_trader.log_cumulative_report(current_price, time_label);
    }

    fn log_all_trades_report(&self, current_price: f64) {
        self.paper_trader.log_all_trades(current_price);
    }

    fn log_final_summary(&self, current_price: f64) {
        self.paper_trader.log_final_summary(current_price);
    }

    fn take_last_closed_trade(&mut self) -> Option<crate::execution::paper_trader::PaperTradeRecord> {
        self.paper_trader.take_last_closed_trade()
    }

    fn get_closed_trades(&self) -> Vec<crate::execution::paper_trader::PaperTradeRecord> {
        self.paper_trader.closed_trades.clone()
    }

    fn get_market_filter(&self) -> Option<crate::models::market_info::MarketFilter> {
        Some(self.core.market_filter.clone())
    }

    fn update_market_filter(&mut self, filter: crate::models::market_info::MarketFilter) {
        self.core.market_filter = filter;
    }

    fn get_allocated_capital(&self) -> f64 {
        self.core.allocated_capital
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::market_info::MarketFilter;
    use ferro_trade::broker::actions::Action;

    fn create_test_engine(initial_equity: f64, leverage: f64) -> PaperExecutionEngine {
        let core = ExecutionCore::new(
            "BTCUSDT".to_string(),
            MarketFilter::default(),
            initial_equity,
            leverage,
        );
        PaperExecutionEngine::new(core, initial_equity, 0.0, None)
    }

    fn create_test_candle(close: f64) -> Candle {
        Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1_000,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }

    #[tokio::test]
    async fn test_paper_engine_get_equity_unrealized_pnl() {
        let mut engine = create_test_engine(100.0, 1.0);
        assert_eq!(engine.get_equity(), 100.0);

        // Mở Long 1.0 BTC tại giá 100.0 (1x leverage)
        let buy_action = Action::Buy {
            size: 1.0,
            limit: None,
            stop: None,
            policy: None,
            tag: Some("test_buy".to_string()),
        };
        engine.execute_action_ctx(&buy_action, 100.0, 1_000, 1).await.unwrap();
        assert_eq!(engine.get_equity(), 100.0);

        // Giá tăng lên 110.0 (+10.0 PnL thả nổi)
        let candle_up = create_test_candle(110.0);
        engine.on_candle_ctx(&candle_up, 2).await.unwrap();
        assert_eq!(engine.get_equity(), 110.0);

        // Giá giảm xuống 90.0 (-10.0 PnL thả nổi)
        let candle_down = create_test_candle(90.0);
        engine.on_candle_ctx(&candle_down, 3).await.unwrap();
        assert_eq!(engine.get_equity(), 90.0);

        // Đóng vị thế tại 105.0 -> Chốt lãi thực tế thành tiền mặt 105.0
        let close_action = Action::ClosePosition { portion: 1.0 };
        engine.execute_action_ctx(&close_action, 105.0, 2_000, 4).await.unwrap();
        assert_eq!(engine.get_equity(), 105.0);
    }

    #[tokio::test]
    async fn test_paper_engine_get_equity_isolated_margin_zero_floor() {
        // Vốn 100.0, Đòn bẩy 2x -> Vị thế mở 2.0 BTC @ 100.0
        let mut engine = create_test_engine(100.0, 2.0);
        let buy_action = Action::Buy {
            size: 1.0,
            limit: None,
            stop: None,
            policy: None,
            tag: Some("test_buy_2x".to_string()),
        };
        engine.execute_action_ctx(&buy_action, 100.0, 1_000, 1).await.unwrap();

        // Giá sụt về 40.0: uPnL = (40 - 100) * 2.0 = -120.0 -> Total Equity = 100 - 120 = -20.0
        // Cơ chế bảo vệ sàn Isolated Margin kẹp sàn tại 0.0 (không bị âm nợ)
        let candle_crash = create_test_candle(40.0);
        engine.on_candle_ctx(&candle_crash, 2).await.unwrap();
        assert_eq!(engine.get_equity(), 0.0);
    }

    #[tokio::test]
    async fn test_paper_engine_get_equity_invalid_price_fallback() {
        let mut engine = create_test_engine(100.0, 1.0);
        let buy_action = Action::Buy {
            size: 1.0,
            limit: None,
            stop: None,
            policy: None,
            tag: None,
        };
        engine.execute_action_ctx(&buy_action, 100.0, 1_000, 1).await.unwrap();

        // Giả lập giá bất thường: 0.0, NaN, số âm
        engine.last_close = 0.0;
        assert_eq!(engine.get_equity(), 100.0);

        engine.last_close = f64::NAN;
        assert_eq!(engine.get_equity(), 100.0);

        engine.last_close = -50.0;
        assert_eq!(engine.get_equity(), 100.0);
    }
}

