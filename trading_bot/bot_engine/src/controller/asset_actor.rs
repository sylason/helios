use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error, warn, debug};
use crate::models::models::{Candle, AssetState, EmergencyCmd, TradeSignal, Asset, StatusUpdate};
use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::optimizer::helios_bridge::HeliosBridge;
use ferro_trade::strategy::Strategy;
use crate::models::account_state::AccountEvent;
use crate::models::config::EngineConfig;
use crate::execution::engine::ExecutionEngine;

/// Trạng thái Vị thế Đang Mở (Live Position Snapshot)
/// Được chia sẻ giữa ExecutionEngine, AssetActor và Strategy
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

/// Actor quản lý một Tài sản Giao dịch (AssetActor)
/// Đóng vai trò Bộ não Điều phối (Orchestrator), tích hợp Strategy và ủy quyền thực thi sang ExecutionEngine.
pub struct AssetActor {
    pub symbol: String,
    pub strategy_id: String,
    pub state: AssetState,
    pub asset: Asset,
    pub candle_repo: Arc<dyn CandleRepository>,
    pub strategy_repo: Arc<dyn StrategyRepository>,
    pub dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
    pub data_rx: broadcast::Receiver<(Candle, bool)>,
    pub account_rx: broadcast::Receiver<AccountEvent>,
    pub emergency_rx: mpsc::Receiver<EmergencyCmd>,
    pub status_tx: mpsc::Sender<StatusUpdate>,
    pub _trade_tx: Option<mpsc::Sender<TradeSignal>>,
    pub strategy: Option<Box<dyn Strategy + Send>>,
    pub execution_engine: Box<dyn ExecutionEngine>,
    pub config: EngineConfig,
    pub initial_sync_until: Option<i64>,
}

impl AssetActor {
    pub fn new(
        symbol: String,
        strategy_id: String,
        candle_repo: Arc<dyn CandleRepository>,
        strategy_repo: Arc<dyn StrategyRepository>,
        dna_rx: tokio::sync::watch::Receiver<Option<crate::models::models::OptimizedDna>>,
        data_tx: broadcast::Sender<(Candle, bool)>,
        emergency_rx: mpsc::Receiver<EmergencyCmd>,
        status_tx: mpsc::Sender<StatusUpdate>,
        account_rx: broadcast::Receiver<AccountEvent>,
        execution_engine: Box<dyn ExecutionEngine>,
        config: EngineConfig,
    ) -> Self {
        Self {
            symbol: symbol.clone(),
            strategy_id,
            state: AssetState::InitialSync,
            asset: Asset::new(symbol),
            candle_repo,
            strategy_repo,
            dna_rx,
            data_rx: data_tx.subscribe(),
            account_rx,
            emergency_rx,
            status_tx,
            _trade_tx: None,
            strategy: None,
            execution_engine,
            config,
            initial_sync_until: None,
        }
    }

    /// Thiết lập Strategy cho Actor (Dependency Injection)
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy + Send>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Thiết lập mốc thời gian đồng bộ lịch sử ban đầu (phục vụ mô phỏng Forward Test)
    pub fn with_sync_until(mut self, until_ts: i64) -> Self {
        self.initial_sync_until = Some(until_ts);
        self
    }

    /// Thiết lập kênh TradeSignal (nếu cần)
    pub fn with_trade_tx(mut self, trade_tx: mpsc::Sender<TradeSignal>) -> Self {
        self._trade_tx = Some(trade_tx);
        self
    }

    /// Thiết lập cấu hình Động cơ (EngineConfig)
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    /// Thiết lập kích thước cửa sổ nến bộ nhớ tối đa
    pub fn with_max_mem_candles(mut self, max_candles: usize) -> Self {
        self.config.max_live_mem_candles = max_candles;
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

        // 3. Chuyển sang trạng thái sẵn sàng Trading
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
                    }
                }

                // Lắng nghe trạng thái tài khoản và vị thế
                Ok(acc_evt) = self.account_rx.recv() => {
                    self.execution_engine.on_account_event(&acc_evt).await;
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

    /// In báo cáo tích lũy kết quả Paper Trader
    pub fn log_paper_report(&self, current_price: f64, time_label: &str) {
        self.execution_engine.log_paper_report(current_price, time_label);
    }

    /// Xuất báo cáo vị thế và lịch sử tất cả lệnh cho Paper Trader
    pub fn log_all_trades_report(&self, current_price: f64) {
        self.execution_engine.log_all_trades_report(current_price);
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

    pub async fn handle_initial_sync_until(&mut self, until_ts: i64) -> Result<(), String> {
        info!("📥 [{}] Syncing history until timestamp {} into Rolling Buffer...", self.symbol, until_ts);
        
        let start_time = until_ts - (self.config.historical_sync_days as i64 * 24 * 60 * 60 * 1000);
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
        if let Some(until_ts) = self.initial_sync_until {
            return self.handle_initial_sync_until(until_ts).await;
        }

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

        // 2. Lưu nến vào Database nếu không phải nến lịch sử
        if !is_historical {
            if let Err(e) = self.candle_repo.save_candle(candle.clone()).await {
                error!("❌ [{}] Failed to save candle: {}", self.symbol, e);
            }
        }

        // 3. Nếu đang ở trạng thái Trading, thực thi chiến lược
        if self.state == AssetState::Trading {
            let full_buffer = self.asset.get_candles_vec();
            let live_position = self.execution_engine.get_position();
            
            // Cắt nến bộ nhớ: Đảm bảo cửa sổ luôn chứa nến mở vị thế (live_entry_bar) và padding an toàn
            let max_mem = if let Some(ref pos) = live_position {
                let pos_age = full_buffer.len().saturating_sub(pos.live_entry_bar);
                self.config.max_live_mem_candles.max(pos_age + self.config.buffer_padding_bars)
            } else {
                self.config.max_live_mem_candles
            };

            let slice_start = full_buffer.len().saturating_sub(max_mem);
            let rolling_buffer = &full_buffer[slice_start..];

            // BƯỚC 3.1: Quét nến real-time cho ExecutionEngine (Paper Trader tự kiểm tra Trailing SL/TP tại đây)
            let _ = self.execution_engine.on_candle(&candle).await;

            // BƯỚC 3.2: Lấy số dư vốn thực tế và trạng thái vị thế mới nhất
            let current_equity = self.execution_engine.get_equity();
            let current_position = self.execution_engine.get_position();

            // BƯỚC 3.3: Gọi HeliosBridge::evaluate_live_tick để đánh giá tín hiệu và dời Trailing SL cho nến tiếp theo
            if let Some(ref mut strategy) = self.strategy {
                let actions = HeliosBridge::evaluate_live_tick(
                    &self.symbol,
                    rolling_buffer,
                    slice_start,
                    current_position.as_ref(),
                    strategy.as_mut(),
                    current_equity,
                );
                
                if !actions.is_empty() {
                    info!("🎯 [{}] Strategy generated {} actions on candle {}!", self.symbol, actions.len(), current_close);
                    for action in actions {
                        if let Err(e) = self.execution_engine.execute_action(&action, current_close).await {
                            error!("❌ [{}] Lỗi thực thi action {:?}: {:?}", self.symbol, action, e);
                        }
                    }
                    // Gán chính xác nến mở vị thế theo chỉ số nến hiện tại trong full_buffer
                    let current_bar_idx = full_buffer.len().saturating_sub(1);
                    self.execution_engine.set_live_entry_bar(current_bar_idx);
                } else {
                    debug!("🧠 [{}] Strategy evaluated. No actions generated on close {}.", self.symbol, current_close);
                }
            }

            // Đồng bộ AccountEvent vừa phát ra ngay lập tức
            while let Ok(acc_evt) = self.account_rx.try_recv() {
                self.execution_engine.on_account_event(&acc_evt).await;
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
