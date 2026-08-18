use std::sync::Arc;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::AppConfig;
use bot_engine::models::weight_monitor::WeightMonitor;
use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::controller::asset_actor::AssetActor;
use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::execution::ws_manager::WsManager;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // KHAI BÁO STRATEGY ID ĐANG DÙNG (Dùng để định danh log và db)
    let strategy_id = "helios_v3";

    // 1. Khởi tạo Logger (Console + File)
    let log_filename = format!("{}_bot.log", strategy_id);
    let file_appender = tracing_appender::rolling::daily("examples/logs", &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with(fmt::layer().with_writer(std::io::stdout)) // Ghi ra console
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // Ghi ra file
        .init();

    info!("🚀 Khởi động Trading Bot Demo (ETHUSDT)...");

    // 2. Nạp cấu hình từ .env
    let _config = AppConfig::from_env();
    
    // 3. Khởi tạo Hệ thống Phòng ngự (Weight Monitor)
    // Hạn mức mặc định của Binance là 2400 weight/phút
    let monitor = Arc::new(WeightMonitor::new(2400));
    info!("🛡️ Hệ thống phòng ngự (Weight Monitor) đã kích hoạt.");

    // 4. Khởi tạo Database (SQLite)
    std::fs::create_dir_all("examples/storage")?;
    let db_path = format!("examples/storage/{}.db", strategy_id);
    let db_store = Arc::new(SqliteStore::new(&db_path)?);
    info!("📂 Database SQLite đã khởi tạo.");

    // 5. Bước quan trọng: Đảm bảo dữ liệu 33 ngày cho Optimizer (Backward Sync)
    let symbol = "ETHUSDT".to_string();
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());
    
    // Đồng bộ ngược nếu DB chưa đủ 33 ngày
    if let Err(e) = syncer.ensure_history(&symbol, 33).await {
        warn!("⚠️ Lỗi đồng bộ dữ liệu lịch sử Backward: {}. Bot sẽ tiếp tục với dữ liệu hiện có.", e);
    }
    
    // Đồng bộ gap tiến lên hiện tại (Gap Filler)
    if let Err(e) = syncer.fill_gap(&symbol, 30).await {
        warn!("⚠️ Lỗi đồng bộ dữ liệu lịch sử Forward: {}. Bot sẽ tiếp tục với dữ liệu hiện có.", e);
    }

    // 6. Lấy Market Filter từ sàn (Tick size, step size...)
    let market_filter = match syncer.fetch_market_filter(&symbol).await {
        Ok(filter) => filter,
        Err(e) => {
            warn!("⚠️ Không lấy được Market Filter cho {}: {}. Sử dụng mặc định.", symbol, e);
            bot_engine::models::market_info::MarketFilter::default()
        }
    };
    info!("📊 Market Filter: Tick={}, Step={}, MinQty={}", market_filter.tick_size, market_filter.step_size, market_filter.min_qty);

    // 7. Thiết lập "Hệ thần kinh" (Event Bus)
    let (event_bus, _trade_rx, emergency_rx, mut status_rx) = EventBus::new();
    let data_tx = event_bus.data_tx.clone();
    let trade_tx = event_bus.trade_tx.clone();
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

    // [MỚI] Khởi chạy Optimizer ngầm (Mỗi 15 phút chạy 1 lần)
    let db_for_opt = db_store.clone();
    let symbol_opt = symbol.clone();
    tokio::spawn(async move {
        // Optimizer chạy ngay lập tức vì ta đã đảm bảo xong 33 ngày data
        loop {
            // Chạy optimizer
            let local_config = "helios_optimizer_config.json";
            let config_in_dir = "config/helios_optimizer_config.json";
            let default_path = r#"E:\Projects\Automations\trading_backtest\strategies_backtests\trading_bot_demo\examples\helios_optimizer_config.json"#;
            let relative_1 = "strategies_backtests/trading_bot_demo/examples/helios_optimizer_config.json";
            let relative_2 = "../strategies_backtests/trading_bot_demo/examples/helios_optimizer_config.json";
            let relative_3 = "../../strategies_backtests/trading_bot_demo/examples/helios_optimizer_config.json";

            let config_path = if std::path::Path::new(local_config).exists() {
                local_config
            } else if std::path::Path::new(config_in_dir).exists() {
                config_in_dir
            } else if std::path::Path::new(relative_1).exists() {
                relative_1
            } else if std::path::Path::new(relative_2).exists() {
                relative_2
            } else if std::path::Path::new(relative_3).exists() {
                relative_3
            } else {
                default_path
            };

            let db_opt_path = format!("examples/storage/{}_prospector.db", strategy_id);
            if let Err(e) = bot_engine::optimizer::prospector_runner::ProspectorRunner::run_optimization_cycle_smart_seed(
                &symbol_opt,
                strategy_id,
                db_for_opt.clone(),
                config_path,
                &db_opt_path,
                Some(dna_tx.clone())
            ).await {
                tracing::error!("❌ Lỗi chạy Optimizer ngầm: {}", e);
            }
            
            // Chờ 5 phút cho chu kỳ tiếp theo
            info!("💤 [Optimizer] Tạm nghỉ 5 phút chờ chu kỳ mới.");
            tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
        }
    });

    let account_tx = event_bus.account_tx.clone();

    // 8. Khởi chạy WsManager (WebSocket)
    let ws_manager = WsManager::new(
        _config.api_key.clone(),
        _config.api_secret.clone(),
        _config.use_testnet,
        monitor.clone(),
        db_store.clone(),
        data_tx.clone(),
        account_tx.clone()
    );

    ws_manager.start_market_data_stream(symbol.clone(), "1m".to_string());
    ws_manager.start_user_data_stream();

    // Lấy account_rx sớm để không bị rớt mất message initial từ AccountManager
    let account_rx = event_bus.account_tx.subscribe();

    // 8.5 Khởi tạo Account Manager (Gọi REST API 1 lần lúc startup)
    let account_manager = bot_engine::execution::account_manager::AccountManager::new(
        _config.api_key.clone(),
        _config.api_secret.clone(),
        _config.use_testnet,
        account_tx.clone(),
        monitor.clone()
    );
    if let Err(e) = account_manager.initialize_account(&symbol, 20).await {
        tracing::error!("❌ Lỗi khởi tạo tài khoản ban đầu: {}", e);
    }

    // 8.6 Khởi tạo Order Executor cho bot trade thật
    let executor = Arc::new(bot_engine::execution::order_executor::OrderExecutor::new(
        _config.api_key.clone(),
        _config.api_secret.clone(),
        _config.use_testnet,
        market_filter.clone(),
        db_store.clone(),
        monitor.clone()
    ));

    // 9. Khởi chạy Asset Actor (Nhạc trưởng) với HeliosStrategyV3
    let test_strategy = Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new()));

    let execution_mode = if _config.use_testnet {
        bot_engine::models::config::ExecutionMode::Testnet
    } else {
        bot_engine::models::config::ExecutionMode::RealTrade
    };

    let account_tx = event_bus.account_tx.clone();

    let actor = AssetActor::new(
        symbol,
        strategy_id.to_string(),
        db_store.clone(),
        db_store,
        monitor,
        dna_rx,
        data_tx,
        emergency_rx,
        status_tx,
        trade_tx,
        account_rx,
        account_tx,
        market_filter,
        Some(executor),
        60.0,
        20.0,
        execution_mode,
    ).with_strategy(test_strategy);
    
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
