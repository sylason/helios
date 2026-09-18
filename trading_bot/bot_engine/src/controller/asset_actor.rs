use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error, warn, debug};
use chrono::{TimeZone, Utc};
use crate::models::models::{Candle, AssetState, EmergencyCmd, TradeSignal, Asset, StatusUpdate};
use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::optimizer::helios_bridge::HeliosBridge;
use ferro_trade::strategy::Strategy;
use ferro_trade::broker::actions::Action;
use ferro_trade::optimizer::DnaMappable;
use helios::strategy::core_v3::config::HeliosConfigV3;
use crate::models::account_state::AccountEvent;
use crate::models::config::EngineConfig;
use crate::execution::engine::ExecutionEngine;
use crate::error::BotError;

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

/// 🛡️ CHỐT CHẶN COOLDOWN TẠI ENGINE (DUAL SENTINELS - FLAW-TSL.7)
/// Bảo vệ tài khoản chống bẫy cào lệnh (Revenge Trading) và bẫy vả 2 đầu (Alternating Whipsaw)
/// khi lệnh thật (Paper/Live) bị cắt lỗ hoặc hòa vốn danh nghĩa.
#[derive(Debug, Clone, Default)]
pub struct EngineCooldownGuard {
    pub locked_until_bar: usize,
    pub last_exit_price: f64,
    pub locked_side: Option<String>,
    pub consecutive_losses: usize,
    pub last_losing_side: Option<String>,
    pub last_losing_price: f64,
}

impl EngineCooldownGuard {
    /// Tính toán số nến cooldown phạt lũy tiến (Exponential Backoff) với trần tối đa 60 nến (~1 giờ)
    #[inline]
    pub fn calculate_exponential_cooldown(base_bars: usize, consecutive_losses: usize, max_cap: usize) -> usize {
        if base_bars == 0 || consecutive_losses == 0 {
            return 0;
        }
        let streak_power = (consecutive_losses.saturating_sub(1)).min(2) as u32;
        let multiplier = 1usize << streak_power; // 1, 2, hoặc 4
        let penalty = base_bars.saturating_mul(multiplier);
        let cap = if max_cap == 0 { 60 } else { max_cap };
        penalty.min(cap)
    }

    /// Kiểm tra xem một Action (Buy/Sell) có bị chặn bởi Cooldown hay không
    /// Trả về `true` nếu bị VETO (chặn), `false` nếu HỢP LỆ (được phép thực thi)
    pub fn should_veto(
        &self,
        is_buy: bool,
        current_bar_idx: usize,
        current_close: f64,
        current_atr: f64,
        enable_cooldown: bool,
        breakout_atr_mult: f64,
    ) -> bool {
        if !enable_cooldown {
            return false;
        }

        // 🛡️ 1. SAME-SIDE PRICE GUARD:
        // Nếu đang trong chuỗi thua cùng chiều: Cấm vào lệnh nếu giá hiện tại tồi tệ hơn giá vừa cắt lỗ!
        let side_str = if is_buy { "LONG" } else { "SHORT" };
        if self.consecutive_losses > 0 && self.last_losing_side.as_deref() == Some(side_str) {
            let is_worse_price = if is_buy {
                current_close < self.last_losing_price
            } else {
                current_close > self.last_losing_price
            };
            if is_worse_price {
                return true; // Veto bắt dao rơi / cản đầu xe tăng khi giá tồi tệ hơn điểm cắt lỗ
            }
        }

        // 🛡️ 2. UNIVERSAL BIDIRECTIONAL EXPONENTIAL COOLDOWN:
        if current_bar_idx < self.locked_until_bar {
            let mult = if breakout_atr_mult <= 0.0 { 1.5 } else { breakout_atr_mult };
            let threshold = mult * current_atr;

            let is_favorable_breakout = if is_buy {
                (current_close - self.last_exit_price) >= threshold
            } else {
                (self.last_exit_price - current_close) >= threshold
            };

            if !is_favorable_breakout {
                return true;
            }
        }

        false
    }
}

/// Tính toán True Range trung bình (ATR) 14 nến trên bộ đệm nến gần nhất phục vụ Directional Early Release
pub fn calculate_rolling_atr(candles: &[Candle], period: usize) -> f64 {
    let p = period.max(1);
    if candles.len() < 2 {
        return 1.0;
    }
    let start = candles.len().saturating_sub(p + 1);
    let slice = &candles[start..];
    if slice.len() < 2 {
        return 1.0;
    }
    let mut sum_tr = 0.0;
    let mut count = 0;
    for idx in 1..slice.len() {
        let h = slice[idx].high;
        let l = slice[idx].low;
        let prev_c = slice[idx - 1].close;
        let tr = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
        sum_tr += tr;
        count += 1;
    }
    if count > 0 {
        (sum_tr / count as f64).max(1e-5)
    } else {
        1.0
    }
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
    pub sync_data_rx: Option<mpsc::Receiver<(Candle, bool)>>,
    pub account_rx: broadcast::Receiver<AccountEvent>,
    pub emergency_rx: mpsc::Receiver<EmergencyCmd>,
    pub status_tx: mpsc::Sender<StatusUpdate>,
    pub _trade_tx: Option<mpsc::Sender<TradeSignal>>,
    pub strategy: Option<Box<dyn Strategy + Send>>,
    pub execution_engine: Box<dyn ExecutionEngine>,
    pub config: EngineConfig,
    pub initial_sync_until: Option<i64>,
    /// Bộ DNA mới đang chờ nạp khi vị thế hiện tại đóng hoàn toàn (Lock DNA on Active Trade)
    pub pending_dna: Option<crate::models::models::OptimizedDna>,
    /// Chốt chặn Cooldown hai chiều tại Engine chống bẫy Alternating Whipsaw (Dual Sentinels)
    pub cooldown_guard: EngineCooldownGuard,
    /// Bản sao cấu hình chiến lược Helios V3 để đọc động tham số DNA
    pub helios_config: HeliosConfigV3,
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
            sync_data_rx: None,
            account_rx,
            emergency_rx,
            status_tx,
            _trade_tx: None,
            strategy: None,
            execution_engine,
            config,
            initial_sync_until: None,
            pending_dna: None,
            cooldown_guard: EngineCooldownGuard::default(),
            helios_config: HeliosConfigV3::default(),
        }
    }

    /// Thiết lập kênh nến đồng bộ có Backpressure (dành cho mô phỏng Walk-Forward)
    pub fn with_sync_data_rx(mut self, sync_rx: mpsc::Receiver<(Candle, bool)>) -> Self {
        self.sync_data_rx = Some(sync_rx);
        self
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
        if let Err(e) = self.sync_history().await {
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

        let has_sync_rx = self.sync_data_rx.is_some();
        let mut sync_rx = self.sync_data_rx.take();

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
                        // 🧬 SENSOR PROBE & LOCK DNA ON ACTIVE TRADE:
                        // Nếu đang có vị thế mở -> Khóa vị thế với DNA cũ, lưu DNA mới vào pending_dna để kích hoạt sau khi vị thế đóng.
                        if let Some(pos) = self.execution_engine.get_position() {
                            let side_str = if pos.position_amt > 0.0 { "LONG" } else { "SHORT" };
                            let current_bars = self.asset.ohlcv.len().saturating_sub(pos.live_entry_bar);
                            warn!(
                                "🔒 [LOCK DNA ON ACTIVE TRADE] Symbol: {} | Active Position: {} @ {:.2} (Age: {} bars) \
                                | DNA HOT-SWAP RECEIVED MID-TRADE! Locking active trade with existing DNA. Storing new DNA (Fitness: {:.2}%, Dim: {}) in pending_dna!",
                                self.symbol, side_str, pos.entry_price, current_bars, dna.fitness, dna.dna.len()
                            );
                            self.pending_dna = Some(dna.clone());
                        } else {
                            // Không có vị thế (Flat) -> Cập nhật trực tiếp vào strategy ngay lập tức
                            if let Some(ref mut strategy) = self.strategy {
                                strategy.update_from_dna(&dna.dna);
                                info!(
                                    "🧬 [DNA HOT-SWAP APPLIED] Symbol: {} | Account FLAT -> New DNA Deployed (Fitness: {:.2}%, Dim: {})",
                                    self.symbol, dna.fitness, dna.dna.len()
                                );
                            }
                            self.helios_config.update_from_dna(&dna.dna);
                            self.pending_dna = None;
                        }

                        let candles_vec = self.asset.get_candles_vec();
                        let current_close = candles_vec.last().map(|c| c.close).unwrap_or(0.0);
                        let current_time_str = candles_vec.last()
                            .and_then(|c| Utc.timestamp_millis_opt(c.timestamp).single())
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "N/A".to_string());

                        // 1. Xuất báo cáo Paper Trade tích lũy & Bảng trạng thái vị thế tại mốc Re-Opt này
                        self.log_paper_report(current_close, &format!("Re-Opt Update @ {}", current_time_str));
                        self.log_all_trades_report(current_close);

                        // 2. Chạy Backtest chẩn đoán 1 lần duy nhất khi nhận được DNA mới
                        match HeliosBridge::diagnostic_backtest(&candles_vec, &dna.dna) {
                            Ok(result) => {
                                info!("📊 [DIAGNOSTIC BACKTEST] Window: 30D + Ticks | Return: {:.2}% | Max DD: {:.2}% | Trades: {} (Win Rate: {:.1}%)", 
                                    result.stats.return_pct, result.stats.max_drawdown_pct, result.stats.total_trades, result.stats.win_rate_pct);
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
                    // Nếu sau sự kiện tài khoản vị thế đã đóng -> Kích hoạt ngay Pending DNA nếu có
                    if self.execution_engine.get_position().is_none() {
                        if let Some(pending) = self.pending_dna.take() {
                            if let Some(ref mut strategy) = self.strategy {
                                strategy.update_from_dna(&pending.dna);
                                info!(
                                    "🔄 [PENDING DNA ACTIVATED] Symbol: {} | Account Event (Position closed) -> Swapped in pending DNA (Fitness: {:.2}%, Dim: {})",
                                    self.symbol, pending.fitness, pending.dna.len()
                                );
                            }
                            self.helios_config.update_from_dna(&pending.dna);
                        }
                    }
                }

                // Lắng nghe nến đồng bộ có Backpressure (MPSC - Walk-Forward / Fast Simulation)
                Some((candle, is_historical)) = async {
                    if let Some(ref mut rx) = sync_rx {
                        rx.recv().await
                    } else {
                        std::future::pending().await
                    }
                }, if has_sync_rx => {
                    if candle.symbol == self.symbol {
                        self.on_new_candle(candle, is_historical).await;
                    }
                }

                // Lắng nghe nến real-time từ Broadcast (Binance WebSocket - Live Trading)
                res = self.data_rx.recv(), if !has_sync_rx => {
                    match res {
                        Ok((candle, is_historical)) => {
                            if candle.symbol == self.symbol {
                                self.on_new_candle(candle, is_historical).await;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            debug!("⚠️ [{}] AssetActor data_rx LAGGED! Dropped {} candles!", self.symbol, skipped);
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
        let last_price = self.asset.get_candles_vec().last().map(|c| c.close).unwrap_or(0.0);
        self.log_paper_report(last_price, "FINAL RUN SUMMARY");
        self.log_all_trades_report(last_price);
        info!("🏁 Actor [{}] stopped.", self.symbol);
    }

    /// Áp dụng bộ DNA vào cả Strategy và helios_config với kiểm tra tính hợp lệ và số chiều an toàn tuyệt đối
    pub fn apply_dna(&mut self, dna_slice: &[f64], fitness: f64, context: &str) -> bool {
        let expected_dim = HeliosConfigV3::DNA_WIDTH;
        if dna_slice.len() != expected_dim {
            warn!(
                "⚠️ [{}] Bỏ qua DNA ({}) do không tương thích số chiều: {} gen != {} gen mong đợi! Giữ nguyên cấu hình an toàn.",
                self.symbol, context, dna_slice.len(), expected_dim
            );
            return false;
        }

        // Kiểm tra tính hợp lệ của số thực (chặn NaN / Inf - FLAW-7.5)
        for (idx, &val) in dna_slice.iter().enumerate() {
            if val.is_nan() || val.is_infinite() {
                warn!(
                    "⚠️ [{}] Bỏ qua DNA ({}) do chứa giá trị không hợp lệ (NaN/Inf) tại gen #{}: {}!",
                    self.symbol, context, idx, val
                );
                return false;
            }
        }

        if let Some(ref mut strategy) = self.strategy {
            strategy.update_from_dna(dna_slice);
        }
        self.helios_config.update_from_dna(dna_slice);
        info!(
            "✨ [{}] Áp dụng DNA thành công [{}]! (Fitness: {:.2}%, Dim: {})",
            self.symbol, context, fitness, dna_slice.len()
        );
        true
    }

    /// Tải cấu hình DNA tốt nhất từ SQLite
    pub async fn load_best_params(&mut self) -> Result<(), BotError> {
        info!("🧬 [{}] Searching for best DNA in DB...", self.symbol);
        if let Some(optimized) = self.strategy_repo.get_best_dna(&self.symbol, &self.strategy_id).await? {
            if self.apply_dna(&optimized.dna, optimized.fitness, "DB Best DNA") {
                return Ok(());
            } else {
                warn!("⚠️ [{}] DNA trong DB không tương thích số chiều. Bỏ qua và sử dụng cấu hình mặc định.", self.symbol);
                return Ok(());
            }
        }
        warn!("⚠️ [{}] No saved DNA found. Using default parameters.", self.symbol);
        Ok(())
    }

    /// Cập nhật trạng thái và bắn event ra EventBus
    pub async fn update_state(&mut self, new_state: AssetState, reason: &str) {
        self.state = new_state.clone();
        let update = StatusUpdate {
            symbol: self.symbol.clone(),
            state: new_state,
            message: reason.to_string(),
        };
        let _ = self.status_tx.send(update).await;
    }

    /// Cập nhật bộ DNA cho chiến thuật hiện tại
    pub fn update_strategy_dna(&mut self, dna: &[f64]) {
        self.apply_dna(dna, 0.0, "Manual / Forward Test Update");
    }

    /// Kiểm tra và cập nhật DNA từ Receiver nếu có DNA mới từ Re-Optimizer
    pub fn check_and_update_dna(&mut self) {
        if self.dna_rx.has_changed().unwrap_or(false) {
            let dna_data = self.dna_rx.borrow_and_update();
            if let Some(ref dna) = *dna_data {
                if let Some(pos) = self.execution_engine.get_position() {
                    let side_str = if pos.position_amt > 0.0 { "LONG" } else { "SHORT" };
                    let current_bars = self.asset.ohlcv.len().saturating_sub(pos.live_entry_bar);
                    warn!(
                        "🔒 [LOCK DNA ON ACTIVE TRADE] Symbol: {} | Active Position: {} @ {:.2} (Age: {} bars) \
                        | Postponing DNA update (Fitness: {:.2}%) until position closes! Storing in pending_dna.",
                        self.symbol, side_str, pos.entry_price, current_bars, dna.fitness
                    );
                    self.pending_dna = Some(dna.clone());
                } else {
                    if let Some(ref mut strategy) = self.strategy {
                        strategy.update_from_dna(&dna.dna);
                        info!("✨ [{}] Đã cập nhật bộ não Helios với DNA mới từ Re-Optimizer! (Fitness: {:.4})", self.symbol, dna.fitness);
                    }
                    self.helios_config.update_from_dna(&dna.dna);
                    self.pending_dna = None;
                }
            }
        }
    }

    /// Xử lý đồng bộ dữ liệu lịch sử đến mốc thời gian cụ thể
    pub async fn handle_initial_sync_until(&mut self, until_ts: i64) -> Result<(), BotError> {
        self.initial_sync_until = Some(until_ts);
        self.sync_history().await
    }

    /// Xuất báo cáo hiệu suất Paper Trading tích lũy
    pub fn log_paper_report(&self, current_price: f64, time_label: &str) {
        self.execution_engine.log_paper_report(current_price, time_label);
    }

    /// Xuất báo cáo danh sách tất cả các lệnh đã thực hiện của Paper Trading
    pub fn log_all_trades_report(&self, current_price: f64) {
        self.execution_engine.log_all_trades_report(current_price);
    }

    /// Xử lý đồng bộ dữ liệu nến lịch sử vào bộ nhớ (Rolling Buffer)
    pub async fn sync_history(&mut self) -> Result<(), BotError> {
        let now = Utc::now().timestamp_millis();
        let target_sync_ts = self.initial_sync_until.unwrap_or(now);
        let start_time = target_sync_ts - (30 * 24 * 60 * 60 * 1000); // 30 ngày trước target_sync_ts

        info!("📥 [{}] Syncing history until timestamp {} into Rolling Buffer...", self.symbol, target_sync_ts);
        
        let candles = self.candle_repo.get_candles_by_range(&self.symbol, start_time, target_sync_ts).await?;
        let count = candles.len();
        for candle in candles {
            self.asset.push_candle(candle);
        }

        info!("✅ [{}] Synchronized {} historical 1m candles into Rolling Buffer (Total buffer: {}).", 
            self.symbol, count, self.asset.get_candles_vec().len());
        Ok(())
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
            // 0. Đồng bộ AccountEvent dồn đọng từ WebSocket/sàn trước khi xử lý nến mới
            while let Ok(acc_evt) = self.account_rx.try_recv() {
                self.execution_engine.on_account_event(&acc_evt).await;
            }

            let full_buffer = self.asset.get_candles_vec();
            let live_position = self.execution_engine.get_position();
            let current_bar_idx = full_buffer.len().saturating_sub(1);
            
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
            let pos_before = self.execution_engine.get_position();
            let _ = self.execution_engine.on_candle_ctx(&candle, current_bar_idx).await;
            let pos_after = self.execution_engine.get_position();

            // KIỂM CHỨNG LỖ HỔNG 2: Vị thế vừa bị đóng trong chính cây nến này (SL/TP)
            let closed_intra_candle = pos_before.is_some() && pos_after.is_none();
            if closed_intra_candle {
                if let Some(ref p) = pos_before {
                    let side_str = if p.position_amt > 0.0 { "LONG" } else { "SHORT" };
                    let exit_price = if p.position_amt > 0.0 {
                        p.current_sl_price.unwrap_or(candle.low)
                    } else {
                        p.current_sl_price.unwrap_or(candle.high)
                    };
                    let trade_pnl = if p.position_amt > 0.0 {
                        exit_price - p.entry_price
                    } else {
                        p.entry_price - exit_price
                    };

                    info!(
                        "⚡ [INTRA-CANDLE EXIT] Symbol: {} | Position {} entry {:.2} was CLOSED during candle {} (bar_idx: {}) at est. {:.2} (Est PnL: {:+.2})",
                        self.symbol, side_str, p.entry_price, candle.timestamp, current_bar_idx, exit_price, trade_pnl
                    );

                    // 🛡️ DUAL SENTINELS: Kích hoạt Cooldown tại Engine khi lệnh đóng lỗ hoặc hòa vốn danh nghĩa
                    let stop_cfg = &self.helios_config.risk_management.stop_management;
                    if stop_cfg.enable_re_entry_cooldown && stop_cfg.re_entry_cooldown_bars > 0 {
                        if trade_pnl <= 0.001 {
                            // 1. Cập nhật chuỗi thua:
                            if self.cooldown_guard.last_losing_side.as_deref() == Some(side_str) {
                                self.cooldown_guard.consecutive_losses = self.cooldown_guard.consecutive_losses.saturating_add(1);
                            } else {
                                self.cooldown_guard.consecutive_losses = 1;
                            }
                            self.cooldown_guard.last_losing_side = Some(side_str.to_string());
                            self.cooldown_guard.last_losing_price = exit_price;

                            // 2. Tính Exponential Cooldown (trần 60 nến ~ 1 giờ):
                            let effective_cooldown = EngineCooldownGuard::calculate_exponential_cooldown(
                                stop_cfg.re_entry_cooldown_bars,
                                self.cooldown_guard.consecutive_losses,
                                60,
                            );

                            self.cooldown_guard.locked_until_bar = current_bar_idx + effective_cooldown;
                            self.cooldown_guard.last_exit_price = exit_price;
                            self.cooldown_guard.locked_side = Some(side_str.to_string());
                            warn!(
                                "🛑 [ENGINE EXPONENTIAL COOLDOWN ACTIVATED] Symbol: {} | Side: {} | Streak: {} | Penalty: {} bars (Cap: 60) | Exit: {:.2} | PnL: {:+.4} <= 0.001 | Locked until bar_idx: {}",
                                self.symbol, side_str, self.cooldown_guard.consecutive_losses, effective_cooldown, exit_price, trade_pnl, self.cooldown_guard.locked_until_bar
                            );
                        } else {
                            // 3. RESET TOÀN BỘ KHI CÓ LỢI NHUẬN THỰC TẾ:
                            self.cooldown_guard.consecutive_losses = 0;
                            self.cooldown_guard.locked_until_bar = 0;
                            self.cooldown_guard.locked_side = None;
                            self.cooldown_guard.last_losing_side = None;
                            info!(
                                "🎉 [ENGINE COOLDOWN RESET - REAL PROFIT] Symbol: {} | Side: {} | PnL: {:+.4} > 0.001 -> Reset streak & Cooldown to 0!",
                                self.symbol, side_str, trade_pnl
                            );
                        }
                    }
                }

                // 🔄 LOCK DNA ON ACTIVE TRADE: Kích hoạt ngay Pending DNA nếu vị thế vừa đóng bởi Trailing SL/TP
                if let Some(pending) = self.pending_dna.take() {
                    if let Some(ref mut strategy) = self.strategy {
                        strategy.update_from_dna(&pending.dna);
                        info!(
                            "🔄 [PENDING DNA ACTIVATED] Symbol: {} | Position closed (Intra-candle SL/TP) -> Swapped in pending DNA (Fitness: {:.2}%, Dim: {}) for subsequent trades!",
                            self.symbol, pending.fitness, pending.dna.len()
                        );
                    }
                    self.helios_config.update_from_dna(&pending.dna);
                }
            }

            // BƯỚC 3.2: Lấy số dư vốn thực tế và trạng thái vị thế mới nhất
            let current_equity = self.execution_engine.get_equity();
            let current_position = pos_after.clone();

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

                // 🛡️ BƯỚC 2: CHỐT CHẶN REVENGE RE-ENTRY & COOLDOWN GUARD (DUAL SENTINELS)
                let mut valid_actions = Vec::new();

                for action in actions {
                    match &action {
                        Action::Buy { .. } | Action::Sell { .. } => {
                            // TẦNG 1: CẤM TUYỆT ĐỐI MỞ LỆNH TRÊN CÙNG CÂY NẾN VỪA ĐÓNG VỊ THẾ (No Same-Bar Re-entry)
                            if closed_intra_candle {
                                let prev_side = pos_before.as_ref().map(|p| if p.position_amt > 0.0 { "LONG" } else { "SHORT" }).unwrap_or("UNKNOWN");
                                warn!(
                                    "🛡️ [REVENGE RE-ENTRY BLOCKED] Symbol: {} | Position {} was closed intra-candle at bar_idx {} -> SUPPRESSED new order {:?} on the SAME candle!",
                                    self.symbol, prev_side, current_bar_idx, action
                                );
                                continue;
                            }

                            // TẦNG 2: ENGINE COOLDOWN GUARD (Universal Bidirectional Cooldown + Directional Early Release)
                            let stop_cfg = &self.helios_config.risk_management.stop_management;
                            let is_buy = matches!(action, Action::Buy { .. });
                            let current_atr = calculate_rolling_atr(&full_buffer, 14);

                            if self.cooldown_guard.should_veto(
                                is_buy,
                                current_bar_idx,
                                current_close,
                                current_atr,
                                stop_cfg.enable_re_entry_cooldown,
                                stop_cfg.cooldown_breakout_atr_mult,
                            ) {
                                let diff = if is_buy {
                                    current_close - self.cooldown_guard.last_exit_price
                                } else {
                                    self.cooldown_guard.last_exit_price - current_close
                                };
                                let sig_str = if is_buy { "LONG" } else { "SHORT" };
                                let mult = if stop_cfg.cooldown_breakout_atr_mult <= 0.0 { 1.5 } else { stop_cfg.cooldown_breakout_atr_mult };
                                let threshold = mult * current_atr;
                                warn!(
                                    "🛡️ [ENGINE COOLDOWN VETO] Symbol: {} | Action {} BLOCKED by Engine Bidirectional Cooldown! \
                                    Locked until bar_idx: {} (Remaining: {} bars) | LastExit: {:.2} | Curr: {:.2} | \
                                    FavorableDiff: {:+.2} < {:.2}*ATR ({:.2})",
                                    self.symbol, sig_str, self.cooldown_guard.locked_until_bar,
                                    self.cooldown_guard.locked_until_bar.saturating_sub(current_bar_idx),
                                    self.cooldown_guard.last_exit_price, current_close, diff, mult, threshold
                                );
                                continue;
                            } else if stop_cfg.enable_re_entry_cooldown && current_bar_idx < self.cooldown_guard.locked_until_bar {
                                let mult = if stop_cfg.cooldown_breakout_atr_mult <= 0.0 { 1.5 } else { stop_cfg.cooldown_breakout_atr_mult };
                                let threshold = mult * current_atr;
                                info!(
                                    "🚀 [ENGINE COOLDOWN EARLY RELEASE] Symbol: {} | Action {:?} released early! Favorable breakout >= {:.2}*ATR ({:.2})",
                                    self.symbol, action, mult, threshold
                                );
                            }

                            valid_actions.push(action);
                        }
                        _ => {
                            valid_actions.push(action);
                        }
                    }
                }
                
                if !valid_actions.is_empty() {
                    debug!("🎯 [{}] Executing {} valid actions on candle {}!", self.symbol, valid_actions.len(), current_close);
                    for action in valid_actions {
                        if let Err(e) = self.execution_engine.execute_action_ctx(&action, current_close, candle.timestamp, current_bar_idx).await {
                            error!("❌ [{}] Lỗi thực thi action {:?}: {:?}", self.symbol, action, e);
                        }
                    }
                } else {
                    debug!("🧠 [{}] Strategy evaluated. No actions executed on close {}.", self.symbol, current_close);
                }
            }

            // Đồng bộ AccountEvent vừa phát ra ngay lập tức
            while let Ok(acc_evt) = self.account_rx.try_recv() {
                self.execution_engine.on_account_event(&acc_evt).await;
            }

            // 🔄 LOCK DNA ON ACTIVE TRADE: Kích hoạt Pending DNA nếu vị thế đã đóng hoàn toàn (Flat) sau khi thực thi Action
            let pos_post_action = self.execution_engine.get_position();
            if pos_after.is_some() && pos_post_action.is_none() {
                // Vị thế vừa được đóng bởi Action::ClosePosition hoặc sàn
                if let Some(ref p) = pos_after {
                    let side_str = if p.position_amt > 0.0 { "LONG" } else { "SHORT" };
                    let exit_price = current_close;
                    let trade_pnl = if p.position_amt > 0.0 {
                        exit_price - p.entry_price
                    } else {
                        p.entry_price - exit_price
                    };
                    let stop_cfg = &self.helios_config.risk_management.stop_management;
                    if stop_cfg.enable_re_entry_cooldown && stop_cfg.re_entry_cooldown_bars > 0 && trade_pnl <= 0.001 {
                        self.cooldown_guard.locked_until_bar = current_bar_idx + stop_cfg.re_entry_cooldown_bars;
                        self.cooldown_guard.last_exit_price = exit_price;
                        self.cooldown_guard.locked_side = Some(side_str.to_string());
                        warn!(
                            "🛑 [ENGINE COOLDOWN ACTIVATED - POST ACTION] Symbol: {} | Side: {} | Entry: {:.2} | Exit: {:.2} | PnL: {:+.4} <= 0.001 | Locked until bar_idx: {} ({} bars)",
                            self.symbol, side_str, p.entry_price, exit_price, trade_pnl, self.cooldown_guard.locked_until_bar, stop_cfg.re_entry_cooldown_bars
                        );
                    }
                }
            }
            if pos_post_action.is_none() {
                if let Some(pending) = self.pending_dna.take() {
                    if let Some(ref mut strategy) = self.strategy {
                        strategy.update_from_dna(&pending.dna);
                        info!(
                            "🔄 [PENDING DNA ACTIVATED] Symbol: {} | Position is now FLAT (Post-Action) -> Swapped in pending DNA (Fitness: {:.2}%, Dim: {}) for subsequent trades!",
                            self.symbol, pending.fitness, pending.dna.len()
                        );
                    }
                    self.helios_config.update_from_dna(&pending.dna);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rolling_atr_basic() {
        let mut candles = Vec::new();
        for i in 0..20 {
            candles.push(Candle {
                symbol: "BTCUSDT".to_string(),
                timestamp: i * 60_000,
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
                volume: 10.0,
            });
        }
        let atr = calculate_rolling_atr(&candles, 14);
        assert!((atr - 15.0).abs() < 1e-6, "ATR phải xấp xỉ 15.0, thực tế: {}", atr);
    }

    #[test]
    fn test_engine_cooldown_guard_blocks_both_sides_in_chop() {
        let guard = EngineCooldownGuard {
            locked_until_bar: 115,
            last_exit_price: 2000.0,
            locked_side: Some("LONG".to_string()),
            ..Default::default()
        };

        let current_bar = 105; // Còn 10 bars trong cooldown
        let current_atr = 10.0;
        let mult = 1.5; // threshold = 15.0

        // 1. Long trong vùng chop: giá 2005 (diff = +5 < 15) -> VETO!
        assert!(
            guard.should_veto(true, current_bar, 2005.0, current_atr, true, mult),
            "Long trong vùng biên độ nhỏ phải bị Veto!"
        );

        // 2. Short trong vùng chop: giá 1995 (diff = +5 < 15) -> VETO!
        assert!(
            guard.should_veto(false, current_bar, 1995.0, current_atr, true, mult),
            "Short trong vùng biên độ nhỏ phải bị Veto!"
        );
    }

    #[test]
    fn test_engine_cooldown_guard_directional_early_release() {
        let guard = EngineCooldownGuard {
            locked_until_bar: 115,
            last_exit_price: 2000.0,
            locked_side: Some("LONG".to_string()),
            ..Default::default()
        };

        let current_bar = 105;
        let current_atr = 10.0;
        let mult = 1.5; // threshold = 15.0

        // 1. Long bứt phá mạnh: giá 2020.0 (diff = +20 >= 15) -> EARLY RELEASE (should_veto = false)!
        assert!(
            !guard.should_veto(true, current_bar, 2020.0, current_atr, true, mult),
            "Long bứt phá >= 1.5 ATR phải được mở khóa sớm!"
        );

        // 2. Short sập gãy mạnh: giá 1980.0 (diff = 2000 - 1980 = 20 >= 15) -> EARLY RELEASE!
        assert!(
            !guard.should_veto(false, current_bar, 1980.0, current_atr, true, mult),
            "Short sập gãy >= 1.5 ATR phải được mở khóa sớm!"
        );

        // 3. Falling knife: Giá sập 1980.0 nhưng lại ra tín hiệu BUY -> BỊ VETO (diff = 1980 - 2000 = -20 < 15)!
        assert!(
            guard.should_veto(true, current_bar, 1980.0, current_atr, true, mult),
            "Long bắt dao rơi khi giá sập sâu phải bị VETO!"
        );

        // 4. Short cản tàu: Giá tăng 2020.0 nhưng lại ra tín hiệu SELL -> BỊ VETO (diff = 2000 - 2020 = -20 < 15)!
        assert!(
            guard.should_veto(false, current_bar, 2020.0, current_atr, true, mult),
            "Short cản tàu khi giá tăng mạnh phải bị VETO!"
        );
    }

    #[test]
    fn test_engine_cooldown_guard_expired_and_disabled() {
        let guard = EngineCooldownGuard {
            locked_until_bar: 115,
            last_exit_price: 2000.0,
            locked_side: Some("LONG".to_string()),
            ..Default::default()
        };

        // 1. Hết hạn cooldown (bar 116 >= 115): Cho phép mở lệnh kể cả biến động nhỏ
        assert!(
            !guard.should_veto(true, 116, 2002.0, 10.0, true, 1.5),
            "Khi hết thời gian cooldown, phải cho phép mở lệnh!"
        );

        // 2. Cooldown bị tắt (enable_cooldown = false)
        assert!(
            !guard.should_veto(true, 105, 2002.0, 10.0, false, 1.5),
            "Khi enable_cooldown = false, không được Veto!"
        );
    }

    #[test]
    fn test_engine_cooldown_same_side_price_guard() {
        // Vừa cắt lỗ Long tại 2000.0, streak = 1
        let guard = EngineCooldownGuard {
            locked_until_bar: 100, // Đã qua thời gian khóa bar
            last_exit_price: 2000.0,
            locked_side: Some("LONG".to_string()),
            consecutive_losses: 1,
            last_losing_side: Some("LONG".to_string()),
            last_losing_price: 2000.0,
        };

        // Long ở giá thấp hơn giá cắt lỗ (1990.0 < 2000.0) -> PHẢI BỊ VETO bởi Same-Side Price Guard
        assert!(
            guard.should_veto(true, 105, 1990.0, 10.0, true, 1.5),
            "Long ở giá thấp hơn giá cắt lỗ trong chuỗi thua PHẢI BỊ VETO!"
        );

        // Long ở giá hồi phục cao hơn (2005.0 > 2000.0) -> CHO PHÉP
        assert!(
            !guard.should_veto(true, 105, 2005.0, 10.0, true, 1.5),
            "Long ở giá hồi phục cao hơn được phép vào"
        );
    }
}
