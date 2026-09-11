use std::sync::Arc;
use std::path::Path;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::{AppConfig, EngineConfig, ExecutionMode};
use bot_engine::models::weight_monitor::WeightMonitor;
use bot_engine::models::repository::StrategyRepository;
use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::controller::asset_actor::AssetActor;
use bot_engine::execution::engine::ExecutionEngineFactory;
use bot_engine::optimizer::prospector_runner::ProspectorRunner;
use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::execution::ws_manager::WsManager;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[path = "common/mod.rs"]
mod common;

// ============================================================================
// ⚙️ BẢNG ĐIỀU KHIỂN & CẤU HÌNH TRUNG TÂM (CENTRAL HARNESS CONFIG)
// Đồng bộ 1:1 với walk_forward_smart_seed.rs
// ============================================================================
// 1. Nhận diện & Chế độ Thực thi
const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "live_smart_seed";

/// Công tắc 3 chế độ thực thi:
/// - ExecutionMode::Paper: Nhận nến WS Binance thật, khớp lệnh PaperTrader $60 local (Zero Risk)
/// - ExecutionMode::Testnet: Gửi lệnh thật tới Binance Futures Testnet
/// - ExecutionMode::RealTrade: Gửi lệnh thật tiền thật tới Binance Futures Mainnet
const EXECUTION_MODE: ExecutionMode = ExecutionMode::Paper;

// 2. Dữ liệu & Bộ nhớ Nến (Đảm bảo đủ 50 nến 4h warmup)
const HISTORICAL_SYNC_DAYS: i64 = 33;      // Số ngày nến lịch sử đồng bộ (33 ngày)
const TRAIN_WINDOW_DAYS: i64 = 30;         // Cửa sổ nến huấn luyện GA In-Sample (30 ngày)
const DNA_FRESHNESS_MINS: i64 = 15;        // Ngưỡng xác định DNA tươi (< 15 phút thì nạp ngay)
const MAX_LIVE_MEM_CANDLES: usize = 43100; // Số nến tối đa trong RAM (>= 43100 để đủ 50 nến 4h)
const BUFFER_PADDING_BARS: usize = 200;    // Số nến đệm an toàn
const WEIGHT_LIMIT: u32 = 2400;            // Giới hạn IP Rate Limit Binance

// 3. Chu kỳ Vòng lặp Tịnh tiến (GA Re-Optimizer)
const REOPT_INTERVAL_MINS: i64 = 240;      // Tần suất chạy GA Re-Optimizer (4 tiếng)

// 4. Tài khoản, Đòn bẩy & Quản trị Vị thế (Tương thích 1:1 với walk_forward_smart_seed)
const INITIAL_CASH: f64 = 60.0;            // Vốn khởi tạo ($60)
const LEVERAGE: f64 = 20.0;                // Đòn bẩy tài khoản tiêu chuẩn (20x)
const NUKE_LEVERAGE: f64 = 50.0;           // Đòn bẩy chiến lược Nuke độc lập (50x)
const PERCENT_PER_TRADE: f64 = 0.20;       // Tỷ lệ vốn ký quỹ mỗi lệnh (20% vốn)
const MIN_ORDER_USD: f64 = 21.0;           // Kích thước lệnh tối thiểu sàn cho phép ($21)
const MAX_CAPITAL_FRACTION: f64 = 0.25;    // Trần tỷ lệ ký quỹ tối đa trên mỗi vị thế (25% vốn)
const COMMISSION_RATE: f64 = 0.0004;       // Tỷ lệ phí giao dịch (0.04% Maker/Taker Binance Futures)
const FRACTIONAL_UNIT: f64 = 0.0001;       // Đơn vị bước nhảy khối lượng

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // 1. Khởi tạo Logger (Console + File)
    let log_filename = format!("{}_bot.log", STRATEGY_ID);
    let file_appender = tracing_appender::rolling::daily("examples/logs", &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with(fmt::layer().with_writer(std::io::stdout)) // Ghi ra console
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // Ghi ra file
        .init();

    info!("🚀 Khởi động Trading Bot Demo ({})...", SYMBOL);
    info!("🎯 Chế độ thực thi: {:?}", EXECUTION_MODE);
    info!("💰 Vốn ban đầu: ${:.2} | Đòn bẩy: {:.0}x (Nuke: {:.0}x)", INITIAL_CASH, LEVERAGE, NUKE_LEVERAGE);

    // 2. Nạp cấu hình từ .env
    let _config = AppConfig::from_env();
    
    // 3. Khởi tạo Hệ thống Phòng ngự (Weight Monitor)
    let monitor = Arc::new(WeightMonitor::new(WEIGHT_LIMIT));
    info!("🛡️ Hệ thống phòng ngự (Weight Monitor) đã kích hoạt: {} req/min.", WEIGHT_LIMIT);

    // 4. Khởi tạo Database (SQLite)
    std::fs::create_dir_all("examples/storage")?;
    let db_path = format!("examples/storage/{}.db", STRATEGY_ID);
    let db_store = Arc::new(SqliteStore::new(&db_path)?);
    info!("📂 Database SQLite đã khởi tạo: {}", db_path);

    // 5. Đảm bảo dữ liệu nến lịch sử cho Optimizer (Backward Sync + Gap Filler)
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());
    
    // Đồng bộ ngược nếu DB chưa đủ số ngày cấu hình
    if let Err(e) = syncer.ensure_history(SYMBOL, HISTORICAL_SYNC_DAYS).await {
        warn!("⚠️ Lỗi đồng bộ dữ liệu lịch sử Backward: {}. Bot sẽ tiếp tục với dữ liệu hiện có.", e);
    }
    
    // Đồng bộ gap tiến lên hiện tại (Gap Filler)
    if let Err(e) = syncer.fill_gap(SYMBOL, HISTORICAL_SYNC_DAYS).await {
        warn!("⚠️ Lỗi đồng bộ dữ liệu lịch sử Forward: {}. Bot sẽ tiếp tục với dữ liệu hiện có.", e);
    }

    // 6. Lấy Market Filter từ sàn (Tick size, step size...)
    let market_filter = match syncer.fetch_market_filter(SYMBOL).await {
        Ok(filter) => filter,
        Err(e) => {
            warn!("⚠️ Không lấy được Market Filter cho {}: {}. Sử dụng mặc định.", SYMBOL, e);
            bot_engine::models::market_info::MarketFilter::default()
        }
    };
    info!("📊 Market Filter: Tick={}, Step={}, MinQty={}", market_filter.tick_size, market_filter.step_size, market_filter.min_qty);

    // 7. Thiết lập "Hệ thần kinh" (Event Bus)
    let (event_bus, _trade_rx, emergency_rx, mut status_rx) = EventBus::new();
    let data_tx = event_bus.data_tx.clone();
    let _trade_tx = event_bus.trade_tx.clone();
    let status_tx = event_bus.status_tx.clone();

    info!("🧠 Hệ thần kinh (Event Bus) đã sẵn sàng.");

    // [MỚI] Lắng nghe báo cáo trạng thái từ các Actor (Tạm thời in ra Console)
    tokio::spawn(async move {
        while let Some(update) = status_rx.recv().await {
            info!("📢 [UI_STATUS] [{}] {:?} -> {}", update.symbol, update.state, update.message);
        }
    });

    // [MỚI] Khởi tạo kênh truyền tin Event-Driven cho Optimizer -> Actor
    let (dna_tx, dna_rx) = tokio::sync::watch::channel(None);

    // 7.5 [T1.3] HUẤN LUYỆN INITIAL GA ĐỒNG BỘ TRƯỚC KHI TRADE (PRE-WARMUP & FRESHNESS GUARD)
    let config_path = common::resolve_strategy_config_path("helios");
    let now_ms = chrono::Utc::now().timestamp_millis();
    let freshness_threshold_ms = DNA_FRESHNESS_MINS * 60 * 1000;

    let existing_dna = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await?;
    let mut current_dna_vec = Vec::new();

    let is_dna_fresh = if let Some(ref champ) = existing_dna {
        let age_ms = now_ms - champ.updated_at;
        age_ms < freshness_threshold_ms
    } else {
        false
    };

    if is_dna_fresh {
        let champ = existing_dna.unwrap();
        let max_dd = champ.stats.get("max_drawdown").cloned().unwrap_or(0.0);
        info!("🛡️ [DNA FRESHNESS GUARD] Tìm thấy Champion DNA tươi (< {} phút). Return: {:.2}% | Max DD: {:.2}% | Dim: {}. Nạp ngay!", 
              DNA_FRESHNESS_MINS, champ.fitness, max_dd, champ.dna.len());
        current_dna_vec = champ.dna.clone();
        let _ = dna_tx.send(Some(champ));
    } else {
        info!("--------------------------------------------------------------------------------");
        info!("🧬 [T1.3 INITIAL OPTIMIZE] Không có DNA tươi. Bắt đầu GA huấn luyện ban đầu trên 30 ngày...");

        let prospector_db = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
        if Path::new(&prospector_db).exists() {
            let _ = std::fs::remove_file(&prospector_db);
        }

        let train_end_ts = now_ms;
        let train_start_ts = now_ms - (TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000);

        if let Err(e) = ProspectorRunner::run_optimization_cycle_range(
            SYMBOL,
            STRATEGY_ID,
            db_store.clone(),
            &config_path,
            &prospector_db,
            Some(dna_tx.clone()),
            train_start_ts,
            train_end_ts,
        ).await {
            warn!("⚠️ Lỗi trong quá trình Initial Optimize: {}", e);
        }

        if let Ok(Some(champ)) = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await {
            let max_dd = champ.stats.get("max_drawdown").cloned().unwrap_or(0.0);
            info!("🏆 [INITIAL CHAMPION SẴN SÀNG] In-Sample Return: {:.2}% | Max DD: {:.2}% | Dim: {}", 
                  champ.fitness, max_dd, champ.dna.len());
            current_dna_vec = champ.dna;
        } else {
            warn!("⚠️ Không trích xuất được Champion DNA ban đầu, bot sẽ khởi động với DNA mặc định.");
        }
        info!("--------------------------------------------------------------------------------");
    }

    // 7.6 [T1.4] KHỞI CHẠY RE-OPTIMIZER NGẦM ĐỊNH KỲ 4 TIẾNG (SMART SEEDING 0.5%)
    let db_for_opt = db_store.clone();
    let dna_tx_opt = dna_tx.clone();
    let config_path_opt = config_path.clone();
    let mut seed_dna_vec = current_dna_vec.clone();

    tokio::spawn(async move {
        let reopt_interval = tokio::time::Duration::from_secs((REOPT_INTERVAL_MINS * 60) as u64);
        loop {
            info!("💤 [Optimizer Scheduler] Tạm nghỉ {} phút trước chu kỳ Re-Opt tiếp theo.", REOPT_INTERVAL_MINS);
            tokio::time::sleep(reopt_interval).await;

            let now = chrono::Utc::now().timestamp_millis();
            let opt_end_ts = now;
            let opt_start_ts = now - (TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000);

            info!("🧬 [T1.4 RE-OPTIMIZE TRIGGERED] Bắt đầu chu kỳ Re-Opt 4h trên cửa sổ 30 ngày...");

            // Smart Seeding 0.5%: Lấy Rank 1 Champion từ chu kỳ trước làm hạt giống mồi
            let rank1_seed_dna = if !seed_dna_vec.is_empty() {
                info!("🌱 [SMART SEEDING 0.5%] Đã chọn Rank 1 Champion DNA ({} chiều) làm hạt giống mồi duy nhất.", seed_dna_vec.len());
                Some(seed_dna_vec.clone())
            } else {
                None
            };

            // Dọn dẹp cache đệm prospector_db cũ để 99.5% quần thể được làm mới ngẫu nhiên (LHS Sampling)
            let prospector_db = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
            if Path::new(&prospector_db).exists() {
                let _ = std::fs::remove_file(&prospector_db);
                tracing::debug!("🧹 [SMART SEEDING] Đã reset prospector_db để làm mới 99.5% quần thể qua LHS.");
            }

            if let Err(e) = ProspectorRunner::run_optimization_cycle_range_smart_seed(
                SYMBOL,
                STRATEGY_ID,
                db_for_opt.clone(),
                &config_path_opt,
                &prospector_db,
                Some(dna_tx_opt.clone()),
                opt_start_ts,
                opt_end_ts,
                rank1_seed_dna,
            ).await {
                tracing::error!("❌ Lỗi Re-Optimize ngầm: {}", e);
            } else if let Ok(Some(best_dna)) = db_for_opt.get_best_dna(SYMBOL, STRATEGY_ID).await {
                let max_dd = best_dna.stats.get("max_drawdown").cloned().unwrap_or(0.0);
                info!("🏆 [RE-OPT COMPLETED] Champion DNA mới: Return: {:.2}% | Max DD: {:.2}% -> Đã phát tới AssetActor.", 
                      best_dna.fitness, max_dd);
                seed_dna_vec = best_dna.dna;
            }
        }
    });

    let account_tx = event_bus.account_tx.clone();

    // 8. Cấu hình cờ Testnet dựa trên EXECUTION_MODE
    let use_testnet = match EXECUTION_MODE {
        ExecutionMode::Testnet => true,
        ExecutionMode::RealTrade => false,
        ExecutionMode::Paper => _config.use_testnet,
        _ => false,
    };

    // Khởi chạy WsManager (WebSocket)
    let ws_manager = WsManager::new(
        _config.api_key.clone(),
        _config.api_secret.clone(),
        use_testnet,
        monitor.clone(),
        db_store.clone(),
        data_tx.clone(),
        account_tx.clone()
    );

    ws_manager.start_market_data_stream(SYMBOL.to_string(), "1m".to_string());
    if EXECUTION_MODE != ExecutionMode::Paper {
        ws_manager.start_user_data_stream();
    }

    // Lấy account_rx sớm để không bị rớt mất message initial từ AccountManager
    let account_rx = event_bus.account_tx.subscribe();

    // 8.5 & 8.6 Khởi tạo ExecutionEngine theo ExecutionMode
    let execution_engine = match EXECUTION_MODE {
        ExecutionMode::Paper => {
            info!("📝 [ENGINE SETUP] Khởi tạo PaperExecutionEngine (Vốn: ${:.2}, Leverage: {:.0}x)...", INITIAL_CASH, LEVERAGE);
            ExecutionEngineFactory::create_paper(
                SYMBOL,
                INITIAL_CASH,
                LEVERAGE,
                COMMISSION_RATE,
                market_filter,
                Some(account_tx),
            )
        }
        ExecutionMode::Testnet | ExecutionMode::RealTrade => {
            info!("⚡ [ENGINE SETUP] Khởi tạo BinanceLiveExecutionEngine (Chế độ: {:?})...", EXECUTION_MODE);
            let account_manager = bot_engine::execution::account_manager::AccountManager::new(
                _config.api_key.clone(),
                _config.api_secret.clone(),
                use_testnet,
                account_tx.clone(),
                monitor.clone()
            );
            if let Err(e) = account_manager.initialize_account(SYMBOL, LEVERAGE as i32).await {
                tracing::error!("❌ Lỗi khởi tạo tài khoản ban đầu: {}", e);
            }

            let executor = Arc::new(bot_engine::execution::order_executor::OrderExecutor::new(
                _config.api_key.clone(),
                _config.api_secret.clone(),
                use_testnet,
                market_filter.clone(),
                db_store.clone(),
                monitor.clone()
            ));

            ExecutionEngineFactory::create_live(
                SYMBOL,
                INITIAL_CASH,
                LEVERAGE,
                market_filter,
                executor,
                account_tx,
            )
        }
        _ => panic!("Chế độ {:?} không được hỗ trợ", EXECUTION_MODE),
    };

    // 9. Khởi chạy Asset Actor (Nhạc trưởng) với HeliosStrategyV3 được cấu hình đầy đủ
    let mut strategy_inst = HeliosStrategyV3::new(&std::collections::HashMap::new());
    strategy_inst.debug_mode = true;
    strategy_inst.config.risk_management.risk_base.leverage = LEVERAGE;
    strategy_inst.config.risk_management.risk_base.nuke_leverage = NUKE_LEVERAGE;
    strategy_inst.config.risk_management.risk_base.percent_per_trade = PERCENT_PER_TRADE;
    strategy_inst.config.risk_management.risk_base.min_order_usd = MIN_ORDER_USD;
    strategy_inst.config.risk_management.risk_base.max_cap_fraction = MAX_CAPITAL_FRACTION;
    let strategy = Box::new(strategy_inst);

    let engine_config = EngineConfig {
        initial_cash: INITIAL_CASH,
        commission_rate: COMMISSION_RATE,
        margin_rate: 1.0 / LEVERAGE,
        fractional_unit: Some(FRACTIONAL_UNIT),
        max_live_mem_candles: MAX_LIVE_MEM_CANDLES,
        historical_sync_days: HISTORICAL_SYNC_DAYS as u64,
        buffer_padding_bars: BUFFER_PADDING_BARS,
        opt_num_threads: 4,
    };

    let actor = AssetActor::new(
        SYMBOL.to_string(),
        STRATEGY_ID.to_string(),
        db_store.clone(),
        db_store,
        dna_rx,
        data_tx,
        emergency_rx,
        status_tx,
        account_rx,
        execution_engine,
        engine_config,
    ).with_strategy(strategy);
    
    let actor_handle = tokio::spawn(async move {
        actor.run().await;
    });

    info!("📡 Hệ thống đang vận hành. Đợi nến 1m từ sàn (có thể mất 1-2 phút)...");
    
    // 8. Lắng nghe tín hiệu dừng (Ctrl+C)
    let emergency_tx = event_bus.emergency_tx.clone();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("🚨 Nhận lệnh dừng từ bàn phím. Đang dọn dẹp...");
            let _ = emergency_tx.send(bot_engine::models::models::EmergencyCmd::PanicCloseAll).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        _ = actor_handle => {
            warn!("⚠️ Luồng Actor đã kết thúc sớm.");
        }
    }

    Ok(())
}
