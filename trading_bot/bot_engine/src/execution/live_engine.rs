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

use crate::execution::paper_trader::{PaperTradeRecord, PositionType, ExitReason};

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
    /// Lưu trữ lệnh vừa đóng hoàn chỉnh để cung cấp cho AssetActor và Strategy
    pub recently_closed_trade: Option<PaperTradeRecord>,
    /// Mảnh 1: Ảnh chụp vị thế vừa chuyển từ có sang 0
    pub last_closed_pos_snapshot: Option<LivePositionState>,
    /// Mảnh 2: Thông tin fill từ OrderTradeUpdate (giá thoát, realized pnl, loại lệnh, timestamp)
    pub pending_closed_fill: Option<(f64, f64, String, i64)>,
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
            recently_closed_trade: None,
            last_closed_pos_snapshot: None,
            pending_closed_fill: None,
        }
    }

    /// Ghép 2 mảnh sự kiện (Vị thế về 0 + Lệnh khớp mang rp) để tạo thành ClosedTradeRecord hoàn chỉnh
    pub fn try_assemble_closed_trade(&mut self) {
        if let (Some(pos), Some((exit_price, realized_pnl, order_type, exit_ts))) = (
            self.last_closed_pos_snapshot.as_ref(),
            self.pending_closed_fill.as_ref(),
        ) {
            let is_long = pos.position_amt > 0.0;
            let pos_type = if is_long { PositionType::Long } else { PositionType::Short };
            let entry_price = pos.entry_price;
            let exit_p = *exit_price;
            let r_pnl = *realized_pnl;

            let raw_price_ret = if is_long {
                (exit_p - entry_price) / entry_price.max(1e-5)
            } else {
                (entry_price - exit_p) / entry_price.max(1e-5)
            };
            let leverage = self.core.leverage.max(1.0);
            let pnl_pct = raw_price_ret * leverage * 100.0;

            let exit_reason = match order_type.as_str() {
                "STOP_MARKET" => {
                    if r_pnl > 0.0 {
                        ExitReason::TrailingStopLoss
                    } else {
                        ExitReason::HardStopLoss
                    }
                }
                "TAKE_PROFIT_MARKET" | "TAKE_PROFIT" => ExitReason::TakeProfit,
                _ => ExitReason::StrategySignalClose,
            };

            let trade_record = PaperTradeRecord {
                id: 1,
                pos_type,
                entry_price,
                exit_price: exit_p,
                entry_timestamp: pos.entry_timestamp,
                exit_timestamp: *exit_ts,
                bars_held: 0,
                exit_reason,
                opening_playbook_tag: pos.opening_playbook_tag.clone(),
                net_pnl: r_pnl,
                pnl_pct,
                equity_after: self.account_state.wallet_balance,
                mfe_pct: 0.0,
                mae_pct: 0.0,
            };

            info!(
                "🎯 [BINANCE LIVE CLOSED TRADE ASSEMBLED] Symbol: {} | Side: {:?} | Entry: {:.2} | Exit: {:.2} | Net PnL: {:+.2} USDT ({:+.2}%) | Reason: {:?}",
                self.core.symbol, pos_type, entry_price, exit_p, r_pnl, pnl_pct, exit_reason
            );

            self.recently_closed_trade = Some(trade_record);
            self.last_closed_pos_snapshot = None;
            self.pending_closed_fill = None;
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
                    realized_pnl: 0.0,
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
                        if let Some(pos) = self.live_position.take() {
                            self.last_closed_pos_snapshot = Some(pos);
                            self.try_assemble_closed_trade();
                        }
                    }
                }
            }
            AccountEvent::OrderTradeUpdate { symbol, order_status, avg_price, order_type, realized_pnl, .. } => {
                if symbol == &self.core.symbol {
                    self.is_submitting_order = false;
                    // Nếu lệnh đóng khớp thành công (FILLED và có realized_pnl hoặc đang có/vừa có vị thế)
                    if order_status == "FILLED" && (*realized_pnl != 0.0 || self.live_position.is_some() || self.last_closed_pos_snapshot.is_some()) {
                        let now_ts = chrono::Utc::now().timestamp_millis();
                        self.pending_closed_fill = Some((*avg_price, *realized_pnl, order_type.clone(), now_ts));
                        self.try_assemble_closed_trade();
                    }
                }
            }
        }
    }

    fn take_last_closed_trade(&mut self) -> Option<PaperTradeRecord> {
        self.recently_closed_trade.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::market_info::MarketFilter;

    fn create_test_engine() -> BinanceLiveExecutionEngine {
        let filter = MarketFilter::default();
        let core = ExecutionCore::new(
            "BTCUSDT".to_string(),
            filter,
            100.0,
            20.0,
        );
        let db_store = Arc::new(crate::data_engine::sqlite_impl::SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = Arc::new(OrderExecutor::new("dummy".into(), "dummy".into(), true, filter, db_store, monitor));
        let (account_tx, _) = broadcast::channel(100);
        BinanceLiveExecutionEngine::new(core, executor, account_tx)
    }

    #[tokio::test]
    async fn test_live_engine_assemble_closed_trade_order_after_position() {
        let mut engine = create_test_engine();

        // 1. Mở vị thế Long 1.0 @ 50,000
        engine.on_account_event(&AccountEvent::PositionUpdate {
            symbol: "BTCUSDT".into(),
            position_amt: 1.0,
            entry_price: 50_000.0,
            unrealized_profit: 0.0,
        }).await;
        assert!(engine.live_position.is_some());

        // 2. Kịch bản B: PositionUpdate(0.0) đến TRƯỚC
        engine.on_account_event(&AccountEvent::PositionUpdate {
            symbol: "BTCUSDT".into(),
            position_amt: 0.0,
            entry_price: 50_000.0,
            unrealized_profit: 0.0,
        }).await;
        assert!(engine.live_position.is_none());
        assert!(engine.recently_closed_trade.is_none(), "Chưa có mảnh fill -> Chưa hoàn tất đóng lệnh");

        // 3. OrderTradeUpdate(FILLED) với realized_pnl đến SAU
        engine.on_account_event(&AccountEvent::OrderTradeUpdate {
            symbol: "BTCUSDT".into(),
            side: "SELL".into(),
            order_type: "STOP_MARKET".into(),
            order_status: "FILLED".into(),
            avg_price: 49_000.0,
            orig_qty: 1.0,
            executed_qty: 1.0,
            client_order_id: "sl-123".into(),
            order_id: 999,
            realized_pnl: -20.0,
        }).await;

        // 4. Kiểm tra ClosedTradeRecord đã được ghép hoàn chỉnh
        let closed = engine.take_last_closed_trade();
        assert!(closed.is_some());
        let trade = closed.unwrap();
        assert_eq!(trade.pos_type, PositionType::Long);
        assert_eq!(trade.entry_price, 50_000.0);
        assert_eq!(trade.exit_price, 49_000.0);
        assert_eq!(trade.net_pnl, -20.0);
        assert_eq!(trade.exit_reason, ExitReason::HardStopLoss);

        // 5. Kiểm tra Drain semantics
        assert!(engine.take_last_closed_trade().is_none());
    }

    #[tokio::test]
    async fn test_live_engine_assemble_closed_trade_order_before_position() {
        let mut engine = create_test_engine();

        // 1. Mở vị thế Short -0.5 @ 3,000
        engine.on_account_event(&AccountEvent::PositionUpdate {
            symbol: "BTCUSDT".into(),
            position_amt: -0.5,
            entry_price: 3_000.0,
            unrealized_profit: 0.0,
        }).await;

        // 2. Kịch bản A: OrderTradeUpdate(FILLED) đến TRƯỚC
        engine.on_account_event(&AccountEvent::OrderTradeUpdate {
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            order_type: "STOP_MARKET".into(),
            order_status: "FILLED".into(),
            avg_price: 2_900.0,
            orig_qty: 0.5,
            executed_qty: 0.5,
            client_order_id: "tsl-456".into(),
            order_id: 888,
            realized_pnl: 50.0, // Chốt lãi Trailing SL
        }).await;

        // 3. PositionUpdate(0.0) đến SAU
        engine.on_account_event(&AccountEvent::PositionUpdate {
            symbol: "BTCUSDT".into(),
            position_amt: 0.0,
            entry_price: 3_000.0,
            unrealized_profit: 0.0,
        }).await;

        // 4. Kiểm tra ClosedTradeRecord
        let closed = engine.take_last_closed_trade();
        assert!(closed.is_some());
        let trade = closed.unwrap();
        assert_eq!(trade.pos_type, PositionType::Short);
        assert_eq!(trade.net_pnl, 50.0);
        assert_eq!(trade.exit_reason, ExitReason::TrailingStopLoss);
    }
}
