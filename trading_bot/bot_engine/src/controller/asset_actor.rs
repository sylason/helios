use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error, warn, debug};
use crate::models::models::{Candle, AssetState, EmergencyCmd, TradeSignal, Asset, StatusUpdate};
use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::models::weight_monitor::WeightMonitor;
use crate::optimizer::helios_bridge::HeliosBridge;
use ferro_trade::strategy::Strategy;
use crate::execution::paper_trader::PaperTrader;
use ferro_trade::broker::actions::Action;

use crate::models::account_state::{AccountState, PositionState, AccountEvent};

pub const MAX_LIVE_MEM_CANDLES: usize = 15000;

#[derive(Debug, Clone)]
pub struct LivePositionState {
    pub symbol: String,
    pub position_amt: f64,
    pub entry_price: f64,
    pub entry_timestamp: i64,
    pub live_entry_bar: usize,
    pub current_sl_price: Option<f64>,
    pub current_tp_price: Option<f64>,
    pub opening_playbook_tag: Option<String>,
}


pub struct AssetActor {
    symbol: String,
    strategy_id: String,
    state: AssetState,
    asset: Asset,
    candle_repo: Arc<dyn CandleRepository>,
    strategy_repo: Arc<dyn StrategyRepository>,
    monitor: Arc<WeightMonitor>,
    dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
    data_rx: broadcast::Receiver<(Candle, bool)>,
    account_rx: broadcast::Receiver<AccountEvent>,
    account_tx: broadcast::Sender<AccountEvent>,
    emergency_rx: mpsc::Receiver<EmergencyCmd>,
    status_tx: mpsc::Sender<StatusUpdate>,
    _trade_tx: mpsc::Sender<TradeSignal>,
    strategy: Option<Box<dyn Strategy + Send>>,
    paper_trader: PaperTrader,
    market_filter: crate::models::market_info::MarketFilter,
    
    // Khối 2: Account & Position State
    pub account_state: AccountState,
    pub position_state: PositionState,
    pub live_position: Option<LivePositionState>,
    pub execution_mode: crate::models::config::ExecutionMode,
    // Khối 3: Order Executor
    pub executor: Option<Arc<crate::execution::order_executor::OrderExecutor>>,
    pub allocated_capital: f64,
    pub leverage: f64,
    pub max_live_mem_candles: usize,
    pub is_submitting_order: bool,
}

impl AssetActor {
    pub fn new(
        symbol: String,
        strategy_id: String,
        candle_repo: Arc<dyn CandleRepository>,
        strategy_repo: Arc<dyn StrategyRepository>,
        monitor: Arc<WeightMonitor>,
        dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
        data_tx: broadcast::Sender<(Candle, bool)>,
        emergency_rx: mpsc::Receiver<EmergencyCmd>,
        status_tx: mpsc::Sender<StatusUpdate>,
        trade_tx: mpsc::Sender<TradeSignal>,
        account_rx: broadcast::Receiver<AccountEvent>,
        account_tx: broadcast::Sender<AccountEvent>,
        market_filter: crate::models::market_info::MarketFilter,
        executor: Option<Arc<crate::execution::order_executor::OrderExecutor>>,
        allocated_capital: f64,
        leverage: f64,
        execution_mode: crate::models::config::ExecutionMode,
    ) -> Self {
        let mut paper_trader = PaperTrader::new(symbol.clone(), allocated_capital, 0.0004);
        paper_trader.leverage = leverage;

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
            account_tx,
            emergency_rx,
            status_tx,
            _trade_tx: trade_tx,
            strategy: None,
            paper_trader,
            market_filter,
            account_state: AccountState::default(),
            position_state: PositionState::new(&symbol),
            live_position: None,
            execution_mode,
            executor,
            allocated_capital,
            leverage,
            max_live_mem_candles: MAX_LIVE_MEM_CANDLES,
            is_submitting_order: false,
        }
    }

    /// Thiết lập Strategy cho Actor (Dependency Injection)
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy + Send>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Thiết lập kích thước cửa sổ nến bộ nhớ tối đa
    pub fn with_max_mem_candles(mut self, max_candles: usize) -> Self {
        self.max_live_mem_candles = max_candles;
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

                        // Chạy Backtest chẩn đoán 1 lần duy nhất khi nhận được DNA mới
                        let rolling_buffer = self.asset.get_candles_vec();
                        match HeliosBridge::diagnostic_backtest(&rolling_buffer, &dna.dna) {
                            Ok(result) => {
                                info!("📊 [DIAGNOSTIC BACKTEST] Window: 30D + Ticks | Return: {:.2}% | Max DD: {:.2}% | Trades: {}", 
                                    result.stats.return_pct, result.stats.max_drawdown_pct, result.stats.total_trades);
                            }
                            Err(e) => {
                                error!("❌ [DIAGNOSTIC BACKTEST] Failed to run: {}", e);
                            }
                        }

                        if self.execution_mode == crate::models::config::ExecutionMode::Paper {
                            let last_close = rolling_buffer.last().map(|c| c.close).unwrap_or(0.0);
                            let time_str = chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string();
                            self.paper_trader.log_cumulative_report(last_close, &time_str);
                        }
                    }
                }

                // Lắng nghe trạng thái tài khoản và vị thế
                Ok(acc_evt) = self.account_rx.recv() => {
                    self.on_account_event(acc_evt).await;
                }

                // Lắng nghe nến real-time từ MarketData
                res = self.data_rx.recv() => {
                    match res {
                        Ok((candle, is_historical)) => {
                            if candle.symbol == self.symbol {
                                self.on_new_candle(candle, is_historical).await;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("⚠️ [{}] AssetActor data_rx LAGGED! Dropped {} candles!", self.symbol, skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("ℹ️ [{}] data_rx channel closed. Stopping AssetActor loop.", self.symbol);
                            break;
                        }
                    }
                }
            }
        }

        self.update_state(AssetState::Stopped, "Actor đã dừng.").await;
        info!("🏁 Actor [{}] stopped.", self.symbol);
    }

    /// Cập nhật trạng thái và gửi thông báo lên hệ thống UI
    pub async fn update_state(&mut self, new_state: AssetState, detail: &str) {
        self.state = new_state.clone();
        let update = StatusUpdate {
            symbol: self.symbol.clone(),
            state: new_state,
            message: detail.to_string(),
        };
        
        // Gửi báo cáo vào Channel (không block logic chính)
        if let Err(e) = self.status_tx.send(update).await {
            warn!("⚠️ [{}] Failed to send status update: {}", self.symbol, e);
        }
    }

    /// Cập nhật bộ DNA cho chiến thuật hiện tại
    pub fn update_strategy_dna(&mut self, dna: &[f64]) {
        if let Some(ref mut strategy) = self.strategy {
            strategy.update_from_dna(dna);
        }
    }

    /// In báo cáo tích lũy kết quả Paper Trader
    pub fn log_paper_report(&self, current_price: f64, time_label: &str) {
        self.paper_trader.log_cumulative_report(current_price, time_label);
    }

    /// Xuất báo cáo vị thế và lịch sử tất cả lệnh cho Paper Trader
    pub fn log_all_trades_report(&self, current_price: f64) {
        self.paper_trader.log_all_trades(current_price);
    }

    /// Lắng nghe và cập nhật trạng thái tài khoản / vị thế thời gian thực
    async fn on_account_event(&mut self, evt: AccountEvent) {
        match evt {
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
                    self.is_submitting_order = false;
                    info!("📈 [{}] Position Update: Amt = {:.4}, Entry = {:.2}, uPnL = {:.2}", 
                          self.symbol, position_amt, entry_price, unrealized_profit);

                    // Đồng bộ LivePositionState cho Trailing SL
                    if position_amt != 0.0 {
                        let current_bar = self.asset.get_candles_vec().len().saturating_sub(1);
                        if let Some(ref mut live_pos) = self.live_position {
                            live_pos.position_amt = position_amt;
                            live_pos.entry_price = entry_price;
                        } else {
                            let current_time = chrono::Utc::now().timestamp_millis();
                            self.live_position = Some(LivePositionState {
                                symbol: symbol.clone(),
                                position_amt,
                                entry_price,
                                entry_timestamp: current_time,
                                live_entry_bar: current_bar,
                                current_sl_price: None,
                                current_tp_price: None,
                                opening_playbook_tag: None,
                            });

                            info!("🛡️ [{}] Khởi tạo LivePositionState mới cho Trailing SL (entry_bar: {})", symbol, current_bar);
                        }
                    } else {
                        if self.live_position.is_some() {
                            info!("🧹 [{}] Vị thế đã đóng trên Binance. Clear LivePositionState.", symbol);
                        }
                        self.live_position = None;
                    }
                }
            }
            AccountEvent::OrderTradeUpdate { symbol, side: _, order_type: _, order_status: _, avg_price: _, orig_qty: _, executed_qty: _, client_order_id: _, order_id: _ } => {
                if symbol == self.symbol {
                    self.is_submitting_order = false;
                }
            }
        }
    }

    /// Truy vấn Database để lấy bộ DNA tốt nhất đã được lưu
    pub async fn load_best_params(&mut self) -> Result<(), crate::error::BotError> {
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

    /// Kiểm tra và cập nhật DNA từ Receiver nếu có DNA mới từ Re-Optimizer
    pub fn check_and_update_dna(&mut self) {
        if self.dna_rx.has_changed().unwrap_or(false) {
            let dna_data = self.dna_rx.borrow_and_update();
            if let Some(ref dna) = *dna_data {
                if let Some(ref mut strategy) = self.strategy {
                    strategy.update_from_dna(&dna.dna);
                    info!("✨ [{}] Đã cập nhật bộ não Helios với DNA mới từ Re-Optimizer! (Fitness: {:.4})", self.symbol, dna.fitness);
                }
            }
        }
    }

    pub async fn handle_initial_sync_until(&mut self, until_ts: i64) -> Result<(), String> {
        info!("📥 [{}] Syncing history until timestamp {} into Rolling Buffer...", self.symbol, until_ts);
        
        let start_time = until_ts - (33 * 24 * 60 * 60 * 1000);
        let historical_candles = self.candle_repo.get_candles_by_range(&self.symbol, start_time, until_ts).await.map_err(|e| e.to_string())?;
        
        let count = historical_candles.len();
        for c in historical_candles {
            self.asset.push_candle(c);
        }

        info!("✅ [{}] Synchronized {} historical 1m candles into Rolling Buffer (Total buffer: {}).", 
              self.symbol, count, self.asset.get_candles_vec().len());
        Ok(())
    }

    pub async fn handle_initial_sync(&mut self) -> Result<(), String> {
        let latest = self.candle_repo.get_latest_candle(&self.symbol).await.map_err(|e| e.to_string())?;
        if let Some(latest_candle) = latest {
            self.handle_initial_sync_until(latest_candle.timestamp).await
        } else {
            warn!("⚠️ [{}] Database trống! Bot sẽ phải thu thập nến từ từ (có thể mất rất lâu mới trade được).", self.symbol);
            Ok(())
        }
    }

    pub async fn on_new_candle(&mut self, candle: Candle, is_historical: bool) {
        debug!("📬 [{}] Received new candle: {} (historical: {})", self.symbol, candle.timestamp, is_historical);
        
        // 1. Đẩy vào Rolling Buffer (Trí nhớ 30 ngày)
        self.asset.push_candle(candle.clone());

        let current_close = candle.close;

        // 2. Chỉ lưu nến vào Database nếu không phải Paper mode và không phải nến lịch sử
        if self.execution_mode != crate::models::config::ExecutionMode::Paper && !is_historical {
            if let Err(e) = self.candle_repo.save_candle(candle.clone()).await {
                error!("❌ [{}] Failed to save candle: {}", self.symbol, e);
            }
        }

        // 3. Nếu đang ở trạng thái Trading, thực thi chiến lược
        if self.state == AssetState::Trading {
            let full_buffer = self.asset.get_candles_vec();
            
            // Cắt nến bộ nhớ: Đảm bảo cửa sổ luôn chứa nến mở vị thế (live_entry_bar)
            let max_mem = if let Some(ref pos) = self.live_position {
                let pos_age = full_buffer.len().saturating_sub(pos.live_entry_bar);
                self.max_live_mem_candles.max(pos_age + 200)
            } else {
                self.max_live_mem_candles
            };

            let slice_start = full_buffer.len().saturating_sub(max_mem);
            let rolling_buffer = &full_buffer[slice_start..];

            // BƯỚC 3.1: Quét nến real-time cho PaperTrader xem có dính Trailing SL/TP từ nến trước hay không (CHẠY TRƯỚC khi Strategy dời SL)
            if self.execution_mode == crate::models::config::ExecutionMode::Paper {
                if self.paper_trader.on_candle(&candle, Some(&self.account_tx)) {
                    if self.paper_trader.position_type == crate::execution::paper_trader::PositionType::None {
                        self.live_position = None;
                    }
                }
            }

            // BƯỚC 3.2: Gọi HeliosBridge::evaluate_live_tick để đánh giá tín hiệu và dời Trailing SL cho nến tiếp theo
            if let Some(ref mut strategy) = self.strategy {
                let actions = HeliosBridge::evaluate_live_tick(&self.symbol, rolling_buffer, slice_start, self.live_position.as_ref(), strategy.as_mut());
                
                if !actions.is_empty() {
                    info!("🎯 [{}] Strategy generated {} actions on candle {}!", self.symbol, actions.len(), current_close);
                    for action in actions {
                        use ferro_trade::broker::policies::RiskPolicy;
                        use crate::execution::paper_trader::PositionType;

                        // Trích xuất tag, initial_sl & initial_tp từ action
                        let (playbook_tag, initial_sl, initial_tp) = match &action {
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
                                pos.current_sl_price = new_sl_price;
                                pos.current_tp_price = new_tp_price;
                            }
                        }

                        // --- ENFORCE EXCLUSIVE ORDER RULE ---
                        let position_amt = self.position_state.position_amt;
                        if let Action::Buy { .. } = action {
                            if position_amt > 0.0 || self.is_submitting_order {
                                info!("ℹ️ [{}] Đang có vị thế Long ({}) hoặc đang gửi lệnh, bỏ qua tín hiệu Buy (Exclusive Order).", self.symbol, position_amt);
                                continue;
                            }
                        } else if let Action::Sell { .. } = action {
                            if position_amt < 0.0 || self.is_submitting_order {
                                info!("ℹ️ [{}] Đang có vị thế Short ({}) hoặc đang gửi lệnh, bỏ qua tín hiệu Sell (Exclusive Order).", self.symbol, position_amt);
                                continue;
                            }
                        }
                        
                        // Phân định chế độ thực thi qua ExecutionMode
                        use crate::models::config::ExecutionMode;
                        match self.execution_mode {
                            ExecutionMode::Paper => {
                                info!("📝 [{}] [PAPER MODE] Process Action: {:?}", self.symbol, action);
                                self.paper_trader.process_action(action.clone(), current_close, Some(&self.account_tx));

                                // Cập nhật live_position để Helios nhận đúng tag, initial_sl & initial_tp ở các nến sau
                                if self.paper_trader.position_type != PositionType::None {
                                    let current_bar = full_buffer.len().saturating_sub(1);
                                    let pos_amt = match self.paper_trader.position_type {
                                        PositionType::Long => self.paper_trader.position_size,
                                        PositionType::Short => -self.paper_trader.position_size,
                                        PositionType::None => 0.0,
                                    };
                                    if let Some(ref mut pos) = self.live_position {
                                        pos.position_amt = pos_amt;
                                        pos.entry_price = self.paper_trader.entry_price;
                                        if playbook_tag.is_some() {
                                            pos.opening_playbook_tag = playbook_tag;
                                        }
                                        if initial_sl.is_some() {
                                            pos.current_sl_price = initial_sl;
                                        }
                                        if initial_tp.is_some() {
                                            pos.current_tp_price = initial_tp;
                                        }
                                    } else {
                                        let current_time = chrono::Utc::now().timestamp_millis();
                                        self.live_position = Some(LivePositionState {
                                            symbol: self.symbol.clone(),
                                            position_amt: pos_amt,
                                            entry_price: self.paper_trader.entry_price,
                                            entry_timestamp: current_time,
                                            live_entry_bar: current_bar,
                                            current_sl_price: initial_sl,
                                            current_tp_price: initial_tp,
                                            opening_playbook_tag: playbook_tag,
                                        });
                                    }
                                } else {
                                    self.live_position = None;
                                }
                            }

                            ExecutionMode::Testnet | ExecutionMode::RealTrade => {
                                if is_historical {
                                    info!("🕰️ [{}] Nến Lịch sử. Bỏ qua Action {:?} (Gap Filler Order Spam Prevention).", self.symbol, action);
                                } else if let Some(ref executor) = self.executor {
                                    let usable_capital = self.account_state.available_balance;
                                    let leverage = self.leverage;
                                    let symbol = self.symbol.clone();
                                    let action_clone = action.clone();
                                    let executor_clone = executor.clone();
                                    self.is_submitting_order = true;
                                    
                                    tokio::spawn(async move {
                                        if let Err(e) = executor_clone.execute_action(&symbol, action_clone, current_close, usable_capital, position_amt, leverage).await {
                                            error!("❌ [{}] Lỗi thực thi lệnh qua OrderExecutor: {:?}", symbol, e);
                                        }
                                    });
                                } else {
                                    warn!("⚠️ [{}] [{:?}] Executor bị None nhưng execution_mode là Real/Testnet!", self.symbol, self.execution_mode);
                                }
                            }
                            ExecutionMode::DryRun => {
                                info!("🧪 [{}] [DRY_RUN MODE] Would execute Action: {:?} @ {:.2}", self.symbol, action, current_close);
                            }
                            ExecutionMode::Diagnostic => {
                                info!("📊 [{}] [DIAGNOSTIC MODE] Action logged: {:?} @ {:.2}", self.symbol, action, current_close);
                            }
                        }
                    }
                } else {
                    debug!("🧠 [{}] Strategy evaluated. No actions generated on close {}.", self.symbol, current_close);
                }
            }

            // Đồng bộ AccountEvent vừa phát ra ngay lập tức
            while let Ok(acc_evt) = self.account_rx.try_recv() {
                self.on_account_event(acc_evt).await;
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
