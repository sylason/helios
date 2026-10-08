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
use crate::execution::paper_trader::ExitReason;
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

/// 🛡️ CẦU DAO BẢO VỆ TÀI KHOẢN (ACCOUNT CIRCUIT BREAKER - UNIVERSAL PURE TTL)
/// Bảo vệ tài khoản chống bẫy cào lệnh trả thù (Revenge Trading) và bẫy vả 2 đầu (Whipsaw)
/// sau chuỗi lệnh thua liên tiếp. Tự động mở khóa 100% khi hết hạn nến phạt (Pure TTL).
#[derive(Debug, Clone, Default)]
pub struct EngineCooldownGuard {
    pub locked_until_bar: usize,
    pub consecutive_losses: usize,
}

impl EngineCooldownGuard {
    /// Tính toán số nến cooldown phạt lũy tiến (Exponential Backoff) với trần tối đa max_cap
    #[inline]
    pub fn calculate_exponential_cooldown(base_bars: usize, consecutive_losses: usize, max_cap: usize) -> usize {
        if base_bars == 0 || consecutive_losses == 0 {
            return 0;
        }
        let streak_power = (consecutive_losses.saturating_sub(1)).min(2) as u32;
        let multiplier = 1usize << streak_power; // 1, 2, hoặc 4
        let penalty = base_bars.saturating_mul(multiplier);
        let cap = if max_cap == 0 { 45 } else { max_cap };
        penalty.min(cap)
    }

    /// Kiểm tra xem một Action (Buy/Sell) có bị chặn bởi Cầu Dao hay không (Universal Pure TTL)
    /// Trả về `true` nếu bị VETO (đang trong thời gian phạt), `false` nếu HỢP LỆ (hết hạn hoặc tắt cooldown)
    #[inline]
    pub fn should_veto(&self, current_bar_idx: usize, enable_cooldown: bool) -> bool {
        if !enable_cooldown {
            return false;
        }
        current_bar_idx < self.locked_until_bar
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
    /// Bộ đếm số lượng nến đơn điệu tăng dần (Monotonic Sequence Counter)
    /// Đảm bảo không bao giờ bị bão hòa trần khi bộ đệm trượt ohlcv đạt 43,200 nến
    pub processed_candles_count: usize,
    /// Bản sao cấu hình chiến lược Helios V3 để đọc động tham số DNA
    pub helios_config: HeliosConfigV3,
    /// Ảnh chụp vị thế chu kỳ trước phục vụ Unified Lifecycle Sentinel nhận diện đóng lệnh
    pub previous_position: Option<LivePositionState>,
    /// Bộ đệm tạm thời chứa lệnh vừa đóng để truyền vào HeliosBridge trong nến đóng vị thế
    pub last_closed_trade_for_bridge: Option<crate::execution::paper_trader::PaperTradeRecord>,
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
            processed_candles_count: 0,
            helios_config: HeliosConfigV3::default(),
            previous_position: None,
            last_closed_trade_for_bridge: None,
        }
    }

    /// Thiết lập kênh nến đồng bộ có Backpressure (dành cho mô phỏng Walk-Forward)
    pub fn with_sync_data_rx(mut self, sync_rx: mpsc::Receiver<(Candle, bool)>) -> Self {
        self.sync_data_rx = Some(sync_rx);
        self
    }

    /// Thiết lập Strategy cho Actor (Dependency Injection)
    pub fn with_strategy(mut self, mut strategy: Box<dyn Strategy + Send>) -> Self {
        strategy.set_telemetry_enabled(true);
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

    /// 🛡️ ĐIỂM CHỐT DUY NHẤT: ĐỒNG BỘ VÀ XỬ LÝ VỊ THẾ VỪA ĐÓNG (UNIFIED LIFECYCLE SENTINEL)
    /// Trả về true nếu vừa có vị thế đóng trong nhịp xử lý này.
    pub fn sync_and_handle_closed_positions(&mut self, current_bar_idx: usize, current_close: f64) -> bool {
        let current_pos = self.execution_engine.get_position();
        let pos_closed = self.previous_position.is_some() && current_pos.is_none();
        
        // NGUỒN 1: Kiểm tra xem ExecutionEngine có ClosedTrade hoàn chỉnh không (PaperTrader)
        let engine_closed_trade = self.execution_engine.take_last_closed_trade();

        if pos_closed || engine_closed_trade.is_some() {
            // 🎯 LƯU VẾT CHO HELIOS BRIDGE TRUYỀN VÀO STRATEGY QUA TICKCONTEXT
            if let Some(ref trade) = engine_closed_trade {
                self.last_closed_trade_for_bridge = Some(trade.clone());
            }

            let stop_cfg = &self.helios_config.risk_management.stop_management;
            
            // Phân định thắng/thua bằng Dual-Feed (tự động trừ phí sàn, không bẫy mệnh giá):
            let (is_net_win, is_true_be, pnl_display, side_str) = if let Some(ref trade) = engine_closed_trade {
                let is_win = trade.net_pnl > 0.0;
                let display = format!("{:+.2} USD ({:+.2}%)", trade.net_pnl, trade.pnl_pct);
                let side = format!("{:?}", trade.pos_type);
                // True Break-Even: không phải Hard SL và mức lỗ chỉ là phí nhỏ (> hard_loss_roe_threshold * 100)
                let is_be = !is_win 
                    && trade.exit_reason != ExitReason::HardStopLoss 
                    && trade.pnl_pct > (stop_cfg.hard_loss_roe_threshold * 100.0);
                (is_win, is_be, display, side)
            } else if let Some(ref prev) = self.previous_position {
                let side = if prev.position_amt > 0.0 { "LONG" } else { "SHORT" };
                let raw_ret = if prev.position_amt > 0.0 {
                    (current_close - prev.entry_price) / prev.entry_price.max(1e-5)
                } else {
                    (prev.entry_price - current_close) / prev.entry_price.max(1e-5)
                };
                let roundtrip_fee = 2.0 * self.config.commission_rate; // ~0.08%
                let net_ret = raw_ret - roundtrip_fee;
                let is_win = net_ret > 0.0;
                let leverage = self.helios_config.risk_management.risk_base.leverage.max(1.0);
                let hard_loss_thresh = stop_cfg.hard_loss_roe_threshold / leverage;
                let is_be = !is_win && net_ret > hard_loss_thresh;
                (is_win, is_be, format!("{:+.2}% Net", net_ret * 100.0), side.to_string())
            } else {
                (false, false, "0.00 (Unknown)".to_string(), "UNKNOWN".to_string())
            };

            if stop_cfg.enable_re_entry_cooldown {
                if is_net_win {
                    // KỊCH BẢN 1: THẮNG THỰC TẾ -> Reset toàn bộ streak & cooldown
                    self.cooldown_guard.consecutive_losses = 0;
                    self.cooldown_guard.locked_until_bar = 0;
                    info!(
                        "🎉 [COOLDOWN FULL RESET] Symbol: {} | Side: {} | PnL: {} > 0 -> Reset streak & Cooldown to 0!",
                        self.symbol, side_str, pnl_display
                    );
                } else if is_true_be {
                    // KỊCH BẢN 2: HÒA VỐN TRUE BREAK-EVEN / TRƯỢT GIÁ NHẸ -> Giữ nguyên streak, phạt nhẹ để nến ổn định
                    let be_penalty = stop_cfg.be_tolerance_cooldown_bars.max(1);
                    self.cooldown_guard.locked_until_bar = current_bar_idx + be_penalty;
                    info!(
                        "🛡️ [TRUE BE TOLERANCE] Symbol: {} | Side: {} | PnL: {} | Streak maintained: {} | Cooldown: {} bars | Locked until bar_idx: {}",
                        self.symbol, side_str, pnl_display, self.cooldown_guard.consecutive_losses, be_penalty, self.cooldown_guard.locked_until_bar
                    );
                } else {
                    // KỊCH BẢN 3: THUA NẶNG CẤU TRÚC / HARD SL -> Tăng streak & phạt lũy tiến đầy đủ với trần max_cap
                    self.cooldown_guard.consecutive_losses = self.cooldown_guard.consecutive_losses.saturating_add(1);
                    let effective_cooldown = EngineCooldownGuard::calculate_exponential_cooldown(
                        stop_cfg.re_entry_cooldown_bars,
                        self.cooldown_guard.consecutive_losses,
                        stop_cfg.max_cooldown_cap_bars,
                    );
                    self.cooldown_guard.locked_until_bar = current_bar_idx + effective_cooldown;
                    warn!(
                        "🛑 [CIRCUIT BREAKER ACTIVATED] Symbol: {} | Side: {} | PnL: {} <= 0 | Streak: {} | Penalty: {} bars | Locked until bar_idx: {}",
                        self.symbol, side_str, pnl_display, self.cooldown_guard.consecutive_losses, effective_cooldown, self.cooldown_guard.locked_until_bar
                    );
                }
            }

            // Tự động kích hoạt Pending DNA mới (nếu có)
            if let Some(pending) = self.pending_dna.take() {
                let fitness = pending.fitness;
                let dna_len = pending.dna.len();
                if self.apply_dna(&pending.dna, fitness, "Position Closed") {
                    info!(
                        "🔄 [PENDING DNA ACTIVATED] Symbol: {} | Position Closed -> Deployed new DNA (Fitness: {:.2}%, Dim: {})",
                        self.symbol, fitness, dna_len
                    );
                }
            }

            self.previous_position = current_pos;
            return true;
        }

        self.previous_position = current_pos;
        false
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

        if let Some(ref mut strategy) = self.strategy {
            strategy.set_telemetry_enabled(true);
        }

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
                            let current_bars = self.processed_candles_count.saturating_sub(pos.live_entry_bar);
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
                    let last_close = self.asset.get_candles_vec().last().map(|c| c.close).unwrap_or(0.0);
                    let bar_idx = self.processed_candles_count.saturating_sub(1);
                    self.sync_and_handle_closed_positions(bar_idx, last_close);
                }

                // Lắng nghe nến đồng bộ có Backpressure (MPSC - Walk-Forward / Fast Simulation)
                res = async {
                    if let Some(ref mut rx) = sync_rx {
                        rx.recv().await
                    } else {
                        std::future::pending().await
                    }
                }, if has_sync_rx => {
                    match res {
                        Some((candle, is_historical)) => {
                            if candle.symbol == self.symbol {
                                self.on_new_candle(candle, is_historical).await;
                            }
                        }
                        None => {
                            info!("ℹ️ [{}] sync_rx channel closed. Stopping AssetActor loop gracefully.", self.symbol);
                            break;
                        }
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
        self.log_final_summary(last_price);
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
                    let current_bars = self.processed_candles_count.saturating_sub(pos.live_entry_bar);
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

    /// Xuất báo cáo tổng kết toàn diện 100% lịch sử giao dịch khi kết thúc phiên
    pub fn log_final_summary(&self, current_price: f64) {
        self.execution_engine.log_final_summary(current_price);
        if let Some(summary) = self.strategy.as_ref().and_then(|s| s.get_defense_summary()) {
            info!(target: "quant_telemetry", "\n{}", summary);
        }
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
        self.processed_candles_count = self.asset.get_candles_vec().len();

        info!("✅ [{}] Synchronized {} historical 1m candles into Rolling Buffer (Total buffer: {}).", 
            self.symbol, count, self.asset.get_candles_vec().len());
        Ok(())
    }

    pub async fn on_new_candle(&mut self, candle: Candle, is_historical: bool) {
        debug!("📬 [{}] Received new candle: {} (historical: {})", self.symbol, candle.timestamp, is_historical);
        
        // 1. Đẩy vào Rolling Buffer (Trí nhớ 30 ngày)
        self.asset.push_candle(candle.clone());
        self.processed_candles_count = self.processed_candles_count.saturating_add(1);

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
            let current_bar_idx = self.processed_candles_count.saturating_sub(1);
            
            // Cắt nến bộ nhớ: Đảm bảo cửa sổ luôn chứa nến mở vị thế (live_entry_bar) và padding an toàn
            let max_mem = if let Some(ref pos) = live_position {
                let pos_age = current_bar_idx.saturating_sub(pos.live_entry_bar);
                self.config.max_live_mem_candles.max(pos_age + self.config.buffer_padding_bars)
            } else {
                self.config.max_live_mem_candles
            };

            let slice_start = full_buffer.len().saturating_sub(max_mem);
            let rolling_buffer = &full_buffer[slice_start..];

            // BƯỚC 3.1: Quét nến real-time cho ExecutionEngine (Paper Trader tự kiểm tra Trailing SL/TP tại đây)
            let prev_side_before = self.execution_engine.get_position().map(|p| if p.position_amt > 0.0 { "LONG" } else { "SHORT" }.to_string());
            let _ = self.execution_engine.on_candle_ctx(&candle, current_bar_idx).await;
            let closed_intra_candle = self.sync_and_handle_closed_positions(current_bar_idx, current_close);
            let pos_after = self.execution_engine.get_position();

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
                    self.last_closed_trade_for_bridge.as_ref(),
                );
                self.last_closed_trade_for_bridge = None;

                // 🛡️ BƯỚC 2: CHỐT CHẶN REVENGE RE-ENTRY & COOLDOWN GUARD (DUAL SENTINELS)
                let mut valid_actions = Vec::new();

                for action in actions {
                    match &action {
                        Action::Buy { .. } | Action::Sell { .. } => {
                            // TẦNG 1: CẤM TUYỆT ĐỐI MỞ LỆNH TRÊN CÙNG CÂY NẾN VỪA ĐÓNG VỊ THẾ (No Same-Bar Re-entry)
                            if closed_intra_candle {
                                let prev_side = prev_side_before.as_deref().unwrap_or("UNKNOWN");
                                warn!(
                                    "🛡️ [REVENGE RE-ENTRY BLOCKED] Symbol: {} | Position {} was closed intra-candle at bar_idx {} -> SUPPRESSED new order {:?} on the SAME candle!",
                                    self.symbol, prev_side, current_bar_idx, action
                                );
                                continue;
                            }

                            // TẦNG 2: CẦU DAO BẢO VỆ TÀI KHOẢN (Pure TTL Circuit Breaker)
                            let stop_cfg = &self.helios_config.risk_management.stop_management;

                            if self.cooldown_guard.should_veto(current_bar_idx, stop_cfg.enable_re_entry_cooldown) {
                                let remaining = self.cooldown_guard.locked_until_bar.saturating_sub(current_bar_idx);
                                warn!(
                                    "🛡️ [ENGINE COOLDOWN VETO] Symbol: {} | Action {:?} BLOCKED by Circuit Breaker! \
                                    Locked until bar_idx: {} (Remaining: {} bars) | Streak: {}",
                                    self.symbol, action, self.cooldown_guard.locked_until_bar, remaining, self.cooldown_guard.consecutive_losses
                                );
                                continue;
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
                        } else {
                            // 🎯 QUANT DECISION TELEMETRY: Xuất thẻ kiểm toán quyết định ra target quant_telemetry
                            match action {
                                Action::Buy { .. } | Action::Sell { .. } => {
                                    if let Some(card) = self.strategy.as_mut().and_then(|s| s.take_decision_telemetry()) {
                                        info!(target: "quant_telemetry", "\n{}", card);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    // 🛡️ QUANT DECISION TELEMETRY: Xuất Thẻ Đỏ VETO hoặc cảnh báo Chuyển đổi trạng thái bão nếu có
                    if let Some(card) = self.strategy.as_mut().and_then(|s| s.take_decision_telemetry()) {
                        info!(target: "quant_telemetry", "\n{}", card);
                    }
                    debug!("🧠 [{}] Strategy evaluated. No actions executed on close {}.", self.symbol, current_close);
                }
            }

            // Đồng bộ AccountEvent vừa phát ra ngay lập tức
            while let Ok(acc_evt) = self.account_rx.try_recv() {
                self.execution_engine.on_account_event(&acc_evt).await;
            }

            // Đồng bộ hóa trạng thái vị thế sau khi thực thi Action (Strategy ClosePosition / Sàn)
            self.sync_and_handle_closed_positions(current_bar_idx, current_close);
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
    use crate::execution::paper_trader::{PaperTradeRecord, PositionType, ExitReason};

    #[test]
    fn test_calculate_exponential_cooldown_with_hard_cap_45() {
        // Base bars = 15, Max Cap = 45
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 0, 45), 0);
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 1, 45), 15);
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 2, 45), 30);
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 3, 45), 45); // Chặn cứng tại Cap 45
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 4, 45), 45); // Vẫn ở Cap 45
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(15, 10, 45), 45); // Không bao giờ vượt quá 45 bars

        // Base bars = 20, Max Cap = 60
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(20, 1, 60), 20);
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(20, 2, 60), 40);
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(20, 3, 60), 60);

        // Edge case: base_bars = 0
        assert_eq!(EngineCooldownGuard::calculate_exponential_cooldown(0, 5, 45), 0);
    }

    #[test]
    fn test_should_veto_pure_ttl() {
        let guard = EngineCooldownGuard {
            locked_until_bar: 100,
            consecutive_losses: 2,
        };

        // 1. Đang trong thời gian phạt (current_bar < locked_until_bar): Bị Veto
        assert!(
            guard.should_veto(95, true),
            "Bar 95 < 100 phải bị VETO!"
        );

        // 2. Chạm đúng mốc hết hạn (current_bar == locked_until_bar): Mở khóa
        assert!(
            !guard.should_veto(100, true),
            "Bar 100 >= 100 phải được MỞ KHÓA!"
        );

        // 3. Sau thời gian phạt (current_bar > locked_until_bar): Mở khóa
        assert!(
            !guard.should_veto(105, true),
            "Bar 105 > 100 phải được MỞ KHÓA!"
        );

        // 4. Khi enable_cooldown = false: Luôn mở khóa
        assert!(
            !guard.should_veto(95, false),
            "Khi enable_cooldown = false, không bao giờ được VETO!"
        );
    }

    #[test]
    fn test_pure_ttl_unlock_no_deadlock() {
        // Mô phỏng VULN-AA-09: kiểm tra tính bảo đảm không bị deadlock vĩnh viễn
        let guard = EngineCooldownGuard {
            locked_until_bar: 50,
            consecutive_losses: 3,
        };

        // Qua 50 bars, bảo đảm 100% được mở khóa bất kể giá thị trường
        assert!(!guard.should_veto(50, true));
        assert!(!guard.should_veto(51, true));
    }

    #[test]
    fn test_true_be_classification_logic() {
        let hard_loss_thresh_roe = -0.05; // -5.0%

        // Kịch bản A: Lệnh dính TrailingStopLoss ở sát Entry, lỗ nhẹ phí sàn -0.08%
        let be_trade = PaperTradeRecord {
            id: 1,
            pos_type: PositionType::Long,
            entry_price: 100.0,
            exit_price: 100.05,
            entry_timestamp: 1000,
            exit_timestamp: 2000,
            bars_held: 10,
            exit_reason: ExitReason::TrailingStopLoss,
            opening_playbook_tag: Some("TREND_PB".to_string()),
            net_pnl: -0.05,
            pnl_pct: -0.08,
            equity_after: 59.95,
            mfe_pct: 1.5,
            mae_pct: -0.2,
        };

        let is_win = be_trade.net_pnl > 0.0;
        let is_be = !is_win 
            && be_trade.exit_reason != ExitReason::HardStopLoss 
            && be_trade.pnl_pct > (hard_loss_thresh_roe * 100.0);

        assert!(!is_win, "Không phải lệnh win thực tế");
        assert!(is_be, "Phải được nhận diện là True Break-Even dung sai phí sàn!");

        // Kịch bản B: Lệnh dính HardStopLoss (-12.0% ROE)
        let hard_sl_trade = PaperTradeRecord {
            id: 2,
            pos_type: PositionType::Long,
            entry_price: 100.0,
            exit_price: 99.4,
            entry_timestamp: 1000,
            exit_timestamp: 1500,
            bars_held: 3,
            exit_reason: ExitReason::HardStopLoss,
            opening_playbook_tag: Some("MOMENTUM_IGN".to_string()),
            net_pnl: -1.5,
            pnl_pct: -12.0,
            equity_after: 58.5,
            mfe_pct: 0.1,
            mae_pct: -12.0,
        };

        let is_hard_win = hard_sl_trade.net_pnl > 0.0;
        let is_hard_be = !is_hard_win 
            && hard_sl_trade.exit_reason != ExitReason::HardStopLoss 
            && hard_sl_trade.pnl_pct > (hard_loss_thresh_roe * 100.0);

        assert!(!is_hard_win);
        assert!(!is_hard_be, "Lệnh HardStopLoss tuyệt đối không được coi là True BE!");
    }

    #[test]
    fn test_monotonic_sequence_counter_cooldown_saturated_buffer() {
        // Mô phỏng TODO-ACT-01: Bộ đệm đã bão hòa ở ngưỡng 43,200 nến
        let mut processed_candles_count: usize = 43_200;
        let mut guard = EngineCooldownGuard::default();

        // Lệnh bị dính StopLoss tại cây nến 43,199 (current_bar_idx = processed_candles_count - 1)
        let current_bar_idx: usize = processed_candles_count.saturating_sub(1); // 43,199
        let cooldown_penalty: usize = 15;
        guard.locked_until_bar = current_bar_idx + cooldown_penalty; // 43,214

        // Tại các cây nến tiếp theo trong thời gian phạt: VETO phải chặn
        for _ in 0..15 {
            processed_candles_count += 1;
            let bar_idx = processed_candles_count.saturating_sub(1);
            if bar_idx < guard.locked_until_bar {
                assert!(guard.should_veto(bar_idx, true), "Phải chặn vào lệnh khi chưa hết cooldown");
            }
        }

        // Cây nến thứ 16 sau lệnh thua: bar_idx = 43,214 -> guard mở khóa hoàn toàn
        let unlock_bar_idx = processed_candles_count.saturating_sub(1);
        assert_eq!(unlock_bar_idx, 43_214);
        assert!(!guard.should_veto(unlock_bar_idx, true), "Cooldown phải tự giải phóng sau 15 nến!");
    }

    #[test]
    fn test_pos_age_calculation_monotonic_counter_saturated_buffer() {
        // Mô phỏng TODO-ACT-01: Tính tuổi vị thế khi buffer ohlcv đã bão hòa 43,200
        let buffer_len_capped: usize = 43_200;
        let mut processed_candles_count: usize = 50_000;

        // Vị thế được mở tại bar_idx = 50,000
        let live_entry_bar: usize = processed_candles_count;

        // Trôi qua 120 nến mới trong khi buffer ohlcv vẫn bị trần ở 43,200
        processed_candles_count += 120;
        let current_bar_idx: usize = processed_candles_count.saturating_sub(1); // 50,119

        // Logic cũ (lỗi): buffer_len_capped.saturating_sub(live_entry_bar) -> 43,200 - 50,000 = 0
        let old_broken_age = buffer_len_capped.saturating_sub(live_entry_bar);
        assert_eq!(old_broken_age, 0, "Logic cũ bị bug bóp nghẹt pos_age về 0!");

        // Logic mới (chuẩn): current_bar_idx.saturating_sub(live_entry_bar) -> 50,119 - 50,000 = 119
        let new_fixed_age = current_bar_idx.saturating_sub(live_entry_bar);
        assert_eq!(new_fixed_age, 119, "Logic mới phản ánh chuẩn xác 119 nến đã trôi qua!");
    }
}
