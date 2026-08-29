use async_trait::async_trait;
use crate::error::BotError;
use crate::models::models::Candle;
use crate::controller::asset_actor::LivePositionState;
use crate::execution::engine::{ExecutionEngine, ExecutionCore};
use crate::execution::paper_trader::{PaperTrader, PositionType};
use ferro_trade::broker::actions::Action;
use ferro_trade::broker::policies::RiskPolicy;
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

        // Trích xuất tag, initial_sl & initial_tp từ action
        let (playbook_tag, initial_sl, initial_tp) = match action {
            Action::Buy { policy, tag, .. } | Action::Sell { policy, tag, .. } => {
                let (sl, tp) = match policy {
                    Some(RiskPolicy::FixedPrice { sl_price, tp_price }) => (*sl_price, *tp_price),
                    _ => (None, None),
                };
                (tag.clone(), sl, tp)
            }
            _ => (None, None, None),
        };

        // Cập nhật giá SL/TP nội bộ nếu có Action::UpdateTradeSlTp
        if let Action::UpdateTradeSlTp { new_sl_price, new_tp_price, .. } = action {
            if let Some(ref mut pos) = self.live_position {
                pos.current_sl_price = *new_sl_price;
                pos.current_tp_price = *new_tp_price;
            }
        }

        // Xử lý action trong PaperTrader
        self.paper_trader.process_action(action.clone(), current_price, timestamp, bar_idx, self.account_tx.as_ref());

        // Đồng bộ hóa trạng thái vị thế sang LivePositionState cho Strategy
        if self.paper_trader.position_type != PositionType::None {
            let pos_amt = match self.paper_trader.position_type {
                PositionType::Long => self.paper_trader.position_size,
                PositionType::Short => -self.paper_trader.position_size,
                PositionType::None => 0.0,
            };

            if let Some(ref mut pos) = self.live_position {
                pos.position_amt = pos_amt;
                pos.entry_price = self.paper_trader.entry_price;
                if playbook_tag.is_some() { pos.opening_playbook_tag = playbook_tag; }
                if initial_sl.is_some() { pos.current_sl_price = initial_sl; }
                if initial_tp.is_some() { pos.current_tp_price = initial_tp; }
            } else {
                self.live_position = Some(LivePositionState {
                    symbol: self.core.symbol.clone(),
                    position_amt: pos_amt,
                    entry_price: self.paper_trader.entry_price,
                    entry_timestamp: timestamp,
                    live_entry_bar: bar_idx,
                    current_sl_price: initial_sl,
                    current_tp_price: initial_tp,
                    opening_playbook_tag: playbook_tag,
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
        self.paper_trader.equity
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
}

