use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error};
use crate::error::BotError;
use crate::models::models::Candle;
use crate::models::account_state::{AccountState, PositionState, AccountEvent};
use crate::controller::asset_actor::LivePositionState;
use crate::execution::engine::{ExecutionEngine, ExecutionCore};
use crate::execution::order_executor::OrderExecutor;
use ferro_trade::broker::actions::Action;

/// Động cơ Thực thi Sàn thật / Testnet (Binance Live Execution Engine)
/// Gửi lệnh trực tiếp qua REST API và lắng nghe vị thế thời gian thực qua WebSocket.
pub struct BinanceLiveExecutionEngine {
    pub core: ExecutionCore,
    pub executor: Arc<OrderExecutor>,
    pub account_state: AccountState,
    pub position_state: PositionState,
    pub live_position: Option<LivePositionState>,
    pub is_submitting_order: bool,
    pub account_tx: broadcast::Sender<AccountEvent>,
}

impl BinanceLiveExecutionEngine {
    pub fn new(
        core: ExecutionCore,
        executor: Arc<OrderExecutor>,
        account_tx: broadcast::Sender<AccountEvent>,
    ) -> Self {
        let symbol = core.symbol.clone();
        Self {
            core,
            executor,
            account_state: AccountState::default(),
            position_state: PositionState::new(&symbol),
            live_position: None,
            is_submitting_order: false,
            account_tx,
        }
    }
}

#[async_trait]
impl ExecutionEngine for BinanceLiveExecutionEngine {
    async fn execute_action(&mut self, action: &Action, current_price: f64) -> Result<(), BotError> {
        let position_amt = self.position_state.position_amt;

        // Kiểm tra Exclusive Order
        if let Action::Buy { .. } = action {
            if position_amt > 0.0 || self.is_submitting_order {
                info!("ℹ️ [{}] Đang có vị thế Long hoặc đang gửi lệnh. Bỏ qua Action Buy.", self.core.symbol);
                return Ok(());
            }
        } else if let Action::Sell { .. } = action {
            if position_amt < 0.0 || self.is_submitting_order {
                info!("ℹ️ [{}] Đang có vị thế Short hoặc đang gửi lệnh. Bỏ qua Action Sell.", self.core.symbol);
                return Ok(());
            }
        }

        let usable_capital = self.account_state.available_balance;
        let leverage = self.core.leverage;
        let symbol = self.core.symbol.clone();
        let action_clone = action.clone();
        let executor_clone = self.executor.clone();
        let account_tx_clone = self.account_tx.clone();

        self.is_submitting_order = true;

        tokio::spawn(async move {
            if let Err(e) = executor_clone.execute_action(&symbol, action_clone, current_price, usable_capital, position_amt, leverage).await {
                error!("❌ [{}] Lỗi gửi lệnh REST API: {:?}", symbol, e);
                // Bắn event REJECTED để reset cờ chống Deadlock
                let _ = account_tx_clone.send(AccountEvent::OrderTradeUpdate {
                    symbol,
                    side: "".into(),
                    order_type: "".into(),
                    order_status: "REJECTED".into(),
                    avg_price: 0.0,
                    orig_qty: 0.0,
                    executed_qty: 0.0,
                    client_order_id: "".into(),
                    order_id: 0,
                });
            }
        });

        Ok(())
    }

    async fn on_candle(&mut self, _candle: &Candle) -> Result<(), BotError> {
        // Binance Server tự động quản lý lệnh Algo SL/TP trên Sàn, no-op tại local
        Ok(())
    }

    fn get_equity(&self) -> f64 {
        let u_pnl = self.position_state.unrealized_profit;
        (self.account_state.wallet_balance + u_pnl).max(self.core.allocated_capital)
    }

    fn get_position(&self) -> Option<LivePositionState> {
        self.live_position.clone()
    }

    async fn on_account_event(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::AccountUpdate { wallet_balance, available_balance } => {
                self.account_state.wallet_balance = *wallet_balance;
                self.account_state.available_balance = *available_balance;
                info!("💼 [{}] Live Wallet: {:.2} | Available: {:.2}", self.core.symbol, wallet_balance, available_balance);
            }
            AccountEvent::PositionUpdate { symbol, position_amt, entry_price, unrealized_profit } => {
                if symbol == &self.core.symbol {
                    self.position_state.position_amt = *position_amt;
                    self.position_state.entry_price = *entry_price;
                    self.position_state.unrealized_profit = *unrealized_profit;
                    self.is_submitting_order = false;

                    if *position_amt != 0.0 {
                        if let Some(ref mut live_pos) = self.live_position {
                            live_pos.position_amt = *position_amt;
                            live_pos.entry_price = *entry_price;
                        } else {
                            let current_time = chrono::Utc::now().timestamp_millis();
                            self.live_position = Some(LivePositionState {
                                symbol: symbol.clone(),
                                position_amt: *position_amt,
                                entry_price: *entry_price,
                                entry_timestamp: current_time,
                                live_entry_bar: 0,
                                current_sl_price: None,
                                current_tp_price: None,
                                opening_playbook_tag: None,
                            });
                        }
                    } else {
                        self.live_position = None;
                    }
                }
            }
            AccountEvent::OrderTradeUpdate { symbol, .. } => {
                if symbol == &self.core.symbol {
                    self.is_submitting_order = false;
                }
            }
        }
    }
}
