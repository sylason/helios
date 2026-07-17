use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error, warn, debug};
use crate::models::models::{Candle, AssetState, EmergencyCmd, TradeSignal, Asset, StatusUpdate};
use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::models::weight_monitor::WeightMonitor;
use crate::optimizer::helios_bridge::HeliosBridge;
use ferro_trade::strategy::Strategy;
use crate::execution::paper_trader::PaperTrader;

use crate::models::account_state::{AccountState, PositionState, AccountEvent};

pub struct AssetActor {
    symbol: String,
    strategy_id: String,
    state: AssetState,
    asset: Asset,
    candle_repo: Arc<dyn CandleRepository>,
    strategy_repo: Arc<dyn StrategyRepository>,
    monitor: Arc<WeightMonitor>,
    dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
    data_rx: broadcast::Receiver<Candle>,
    account_rx: broadcast::Receiver<AccountEvent>,
    emergency_rx: mpsc::Receiver<EmergencyCmd>,
    status_tx: mpsc::Sender<StatusUpdate>,
    _trade_tx: mpsc::Sender<TradeSignal>,
    strategy: Option<Box<dyn Strategy + Send>>,
    paper_trader: PaperTrader,
    market_filter: crate::models::market_info::MarketFilter,
    
    // Khối 2: Account & Position State
    pub account_state: AccountState,
    pub position_state: PositionState,

    // Khối 3: Order Executor
    pub executor: Option<Arc<crate::execution::order_executor::OrderExecutor>>,
    pub allocated_capital: f64,
    pub leverage: f64,
}

impl AssetActor {
    pub fn new(
        symbol: String,
        strategy_id: String,
        candle_repo: Arc<dyn CandleRepository>,
        strategy_repo: Arc<dyn StrategyRepository>,
        monitor: Arc<WeightMonitor>,
        dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
        data_tx: broadcast::Sender<Candle>,
        emergency_rx: mpsc::Receiver<EmergencyCmd>,
        status_tx: mpsc::Sender<StatusUpdate>,
        trade_tx: mpsc::Sender<TradeSignal>,
        account_rx: broadcast::Receiver<AccountEvent>,
        market_filter: crate::models::market_info::MarketFilter,
        executor: Option<Arc<crate::execution::order_executor::OrderExecutor>>,
        allocated_capital: f64,
        leverage: f64,
    ) -> Self {
        Self {
            symbol: symbol.clone(),
            strategy_id,
            state: AssetState::InitialSync,
            asset: Asset::new(symbol.clone()),
            candle_repo,
            strategy_repo,
            monitor,
            dna_rx,
            data_rx: data_tx.subscribe(),
            account_rx,
            emergency_rx,
            status_tx,
            _trade_tx: trade_tx,
            strategy: None,
            paper_trader: PaperTrader::new(symbol.clone(), allocated_capital, 0.0004),
            market_filter,
            account_state: AccountState::default(),
            position_state: PositionState::new(&symbol),
            executor,
            allocated_capital,
            leverage,
        }
    }

    /// Thiết lập Strategy cho Actor (Dependency Injection)
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy + Send>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub async fn run(mut self) {
        info!("🎬 Actor [{}] started with strategy '{}'.", self.symbol, self.strategy_id);

        // Báo cáo trạng thái khởi đầu
        self.update_state(AssetState::InitialSync, "Bắt đầu đồng bộ dữ liệu lịch sử...").await;

        // 1. Initial Handshake: Đồng bộ dữ liệu lịch sử
        if let Err(e) = self.handle_initial_sync().await {
            error!("❌ [{}] Initial Sync failed: {}", self.symbol, e);
            self.update_state(AssetState::Stopped, &format!("Lỗi đồng bộ: {}", e)).await;
            return;
        }

        // 2. Nạp tham số DNA tốt nhất từ Database (Nếu có)
        if let Err(e) = self.load_best_params().await {
            warn!("⚠️ [{}] Failed to load optimized DNA: {}. Using defaults.", self.symbol, e);
        }

        // 3. Chạy chu kỳ tối ưu hóa đầu tiên nếu chưa có DNA
        self.update_state(AssetState::Optimizing, "Đang tìm kiếm bộ tham số tối ưu (GA)...").await;
        // if self.needs_optimization() { ... }

        // 4. Chuyển sang chế độ Trading và bắt đầu vòng lặp chính
        self.update_state(AssetState::Trading, "Hệ thống Helios đã sẵn sàng!").await;
        info!("✅ [{}] is now TRADING.", self.symbol);

        loop {
            tokio::select! {
                // Lắng nghe lệnh khẩn cấp (Ưu tiên cao nhất)
                Some(cmd) = self.emergency_rx.recv() => {
                    if self.handle_emergency(cmd).await { break; }
                }

                // Lắng nghe DNA mới từ Optimizer
                Ok(_) = self.dna_rx.changed() => {
                    let new_dna_opt = self.dna_rx.borrow().clone();
                    if let Some(dna) = new_dna_opt {
                        info!("🧬 [{}] Nhận được DNA mới từ Optimizer thông qua Event Bus!", self.symbol);
                        if let Some(ref mut strategy) = self.strategy {
                            strategy.update_from_dna(&dna.dna);
                            info!("✨ [{}] Đã cập nhật bộ não Helios với DNA mới (Fitness: {:.4})", self.symbol, dna.fitness);
                        }
                    }
                }

                // Lắng nghe trạng thái tài khoản và vị thế
                Ok(acc_evt) = self.account_rx.recv() => {
                    self.on_account_event(acc_evt).await;
                }

                // Lắng nghe nến real-time từ MarketData
                Ok(candle) = self.data_rx.recv() => {
                    if candle.symbol == self.symbol {
                        self.on_new_candle(candle).await;
                    }
                }
            }
        }

        self.update_state(AssetState::Stopped, "Actor đã dừng.").await;
        info!("🏁 Actor [{}] stopped.", self.symbol);
    }

    /// Cập nhật trạng thái và gửi thông báo lên hệ thống UI
    async fn update_state(&mut self, new_state: AssetState, msg: &str) {
        self.state = new_state.clone();
        let update = StatusUpdate {
            symbol: self.symbol.clone(),
            state: new_state,
            message: msg.to_string(),
        };
        
        // Gửi báo cáo vào Channel (không block logic chính)
        if let Err(e) = self.status_tx.send(update).await {
            warn!("⚠️ [{}] Failed to send status update: {}", self.symbol, e);
        }
    }

    /// Xử lý sự kiện cập nhật số dư hoặc vị thế
    async fn on_account_event(&mut self, event: AccountEvent) {
        match event {
            AccountEvent::AccountUpdate { wallet_balance, available_balance } => {
                self.account_state.wallet_balance = wallet_balance;
                self.account_state.available_balance = available_balance;
                info!("💼 [{}] Account Update: Wallet = {:.2}, Available = {:.2}", 
                      self.symbol, wallet_balance, available_balance);
            }
            AccountEvent::PositionUpdate { symbol, position_amt, entry_price, unrealized_profit } => {
                if symbol == self.symbol {
                    self.position_state.position_amt = position_amt;
                    self.position_state.entry_price = entry_price;
                    self.position_state.unrealized_profit = unrealized_profit;
                    info!("📈 [{}] Position Update: Amt = {:.4}, Entry = {:.2}, uPnL = {:.2}", 
                          self.symbol, position_amt, entry_price, unrealized_profit);
                }
            }
        }
    }

    /// Truy vấn Database để lấy bộ DNA tốt nhất đã được lưu
    async fn load_best_params(&mut self) -> Result<(), crate::error::BotError> {
        info!("🧬 [{}] Searching for best DNA in DB...", self.symbol);
        
        let dna_opt = self.strategy_repo.get_best_dna(&self.symbol, &self.strategy_id).await
            .map_err(|e| crate::error::BotError::Internal(e.to_string()))?;

        if let Some(dna_data) = dna_opt {
            if let Some(ref mut strategy) = self.strategy {
                strategy.update_from_dna(&dna_data.dna);
                info!("✨ [{}] DNA loaded successfully! (Fitness: {:.4})", self.symbol, dna_data.fitness);
            }
        } else {
            info!("ℹ️ [{}] No optimized DNA found for strategy '{}'.", self.symbol, self.strategy_id);
        }
        
        Ok(())
    }

    async fn handle_initial_sync(&mut self) -> Result<(), crate::error::BotError> {
        info!("📥 [{}] Syncing 30 days of history from Database into Rolling Buffer...", self.symbol);
        
        let latest = self.candle_repo.get_latest_candle(&self.symbol).await.map_err(|e| crate::error::BotError::Internal(e.to_string()))?;
        if let Some(latest_candle) = latest {
            let end_time = latest_candle.timestamp;
            // Lấy dư ra 33 ngày để đảm bảo Warmup
            let start_time = end_time - (33 * 24 * 60 * 60 * 1000);
            
            let historical_candles = self.candle_repo.get_candles_by_range(&self.symbol, start_time, end_time).await.map_err(|e| crate::error::BotError::Internal(e.to_string()))?;
            
            let count = historical_candles.len();
            for c in historical_candles {
                self.asset.push_candle(c);
            }
            info!("✅ [{}] Trí nhớ đã được nạp {} nến lịch sử. Sẵn sàng chiến đấu!", self.symbol, count);
        } else {
            warn!("⚠️ [{}] Database trống! Bot sẽ phải thu thập nến từ từ (có thể mất rất lâu mới trade được).", self.symbol);
        }
        Ok(())
    }

    async fn on_new_candle(&mut self, candle: Candle) {
        debug!("📬 [{}] Received new candle: {}", self.symbol, candle.timestamp);
        
        // 1. Đẩy vào Rolling Buffer (Trí nhớ 30 ngày)
        self.asset.push_candle(candle.clone());

        let current_close = candle.close;

        // 2. Lưu nến vào Database (Bền vững)
        if let Err(e) = self.candle_repo.save_candle(candle).await {
            error!("❌ [{}] Failed to save candle: {}", self.symbol, e);
        }

        // 3. Nếu đang ở trạng thái Trading, thực thi chiến lược
        if self.state == AssetState::Trading {
            let rolling_buffer = self.asset.get_candles_vec();

            // Lấy DNA mới nhất hiện tại từ channel
            let current_dna_opt = self.dna_rx.borrow().clone();

            // Chạy Backtest chẩn đoán 1:1 với ferro_trade::backtest::engine 
            // (Đúng như yêu cầu của user, gọi run_with_dna ở mỗi tick)
            if let Some(dna_data) = current_dna_opt {
                match HeliosBridge::diagnostic_backtest(&rolling_buffer, &dna_data.dna) {
                    Ok(result) => {
                        info!("📊 [DIAGNOSTIC BACKTEST] Window: 30D + Ticks | Return: {:.2}% | Max DD: {:.2}% | Trades: {}", 
                            result.stats.return_pct, result.stats.max_drawdown_pct, result.stats.total_trades);
                    }
                    Err(e) => {
                        error!("❌ [DIAGNOSTIC BACKTEST] Failed to run: {}", e);
                    }
                }
            }

            // Gọi HeliosBridge::analyze để trích xuất tín hiệu (Actions) cho nến HIỆN TẠI
            if let Some(ref mut strategy) = self.strategy {
                let (actions, _stats_opt) = HeliosBridge::analyze(&self.symbol, &rolling_buffer, strategy.as_mut());
                
                if !actions.is_empty() {
                    info!("🎯 [{}] Strategy generated {} actions on candle {}!", self.symbol, actions.len(), current_close);
                    for action in actions {
                        // Vẫn giữ PaperTrader để chạy giả lập song song (lưu log console)
                        self.paper_trader.process_action(action.clone(), current_close);
                        
                        // Bắn lệnh thật nếu có Executor
                        if let Some(ref executor) = self.executor {
                            let position_amt = self.position_state.position_amt;
                            let allocated = self.allocated_capital;
                            let leverage = self.leverage;
                            let symbol = self.symbol.clone();
                            
                            // Ta cần Clone action vì execute_action là hàm async lấy giá trị Action
                            let action_clone = action.clone();
                            let executor_clone = executor.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = executor_clone.execute_action(&symbol, action_clone, current_close, allocated, position_amt, leverage).await {
                                    error!("❌ [{}] Lỗi thực thi lệnh qua OrderExecutor: {:?}", symbol, e);
                                }
                            });
                        }
                    }
                } else {
                    debug!("🧠 [{}] Strategy evaluated. No actions generated on close {}.", self.symbol, current_close);
                }
            }
        }
    }

    async fn handle_emergency(&mut self, cmd: EmergencyCmd) -> bool {
        match cmd {
            EmergencyCmd::PanicCloseAll => {
                warn!("🚨 [{}] PANIC RECEIVED! Stopping...", self.symbol);
                true
            }
            EmergencyCmd::StopAsset(s) if s == self.symbol => {
                info!("🛑 [{}] Stopping as requested.", self.symbol);
                true
            }
            _ => false
        }
    }
}
