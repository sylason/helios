pub mod models;
pub mod data_engine;
pub mod optimizer;
pub mod execution;
pub mod controller;
pub mod interface;
pub mod strategy_runner;
pub mod error;

use std::sync::Arc;
use models::bus::EventBus;
use models::config::AppConfig;
use models::weight_monitor::WeightMonitor;
use data_engine::sqlite_impl::SqliteStore;
use controller::asset_actor::AssetActor;
use execution::engine::ExecutionEngineFactory;
use models::config::EngineConfig;
use data_engine::binance_rest::InitialSync;
use execution::ws_manager::WsManager;
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
    let file_appender = tracing_appender::rolling::daily("logs", &log_filename);
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
    std::fs::create_dir_all("storage")?;
    let db_path = format!("storage/{}.db", strategy_id);
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
            crate::models::market_info::MarketFilter::default()
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

    // [MỚI] Khởi chạy Optimizer ngầm (Mỗi 15 phút chạy 1 lần)
    let db_for_opt = db_store.clone();
    let symbol_opt = symbol.clone();
    tokio::spawn(async move {
        // Optimizer chạy ngay lập tức vì ta đã đảm bảo xong 33 ngày data
        loop {
            // Chạy optimizer
            let config_path = crate::models::config::PathResolver::resolve_strategy_config("helios");

            let db_opt_path = format!("storage/{}_prospector.db", strategy_id);
            if let Err(e) = crate::optimizer::prospector_runner::ProspectorRunner::run_optimization_cycle_smart_seed(
                &symbol_opt,
                strategy_id,
                db_for_opt.clone(),
                &config_path,
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
    // let (ws_cmd_tx, ws_cmd_rx) = tokio::sync::mpsc::channel(100);
    // ws_manager.start_websocket_api(ws_cmd_rx);

    // Lấy account_rx sớm để không bị rớt mất message initial từ AccountManager
    let account_rx = event_bus.account_tx.subscribe();

    // 8.5 Khởi tạo Account Manager (Gọi REST API 1 lần lúc startup)
    let account_manager = crate::execution::account_manager::AccountManager::new(
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
    let executor = Arc::new(crate::execution::order_executor::OrderExecutor::new(
        _config.api_key.clone(),
        _config.api_secret.clone(),
        _config.use_testnet, // Đã sửa: lấy cấu hình đồng bộ từ .env thay vì hardcode true
        market_filter.clone(),
        db_store.clone(),
        monitor.clone()
    ));

    // 9. Khởi chạy Asset Actor (Nhạc trưởng) với HeliosStrategyV3
    let test_strategy = Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new()));

    let account_tx = event_bus.account_tx.clone();

    let engine_config = EngineConfig {
        initial_cash: 60.0,
        commission_rate: 0.0004,
        margin_rate: 1.0 / 20.0,
        fractional_unit: Some(0.0001),
        max_live_mem_candles: 15000,
        historical_sync_days: 33,
        buffer_padding_bars: 200,
        opt_num_threads: 4,
    };

    let live_engine = ExecutionEngineFactory::create_live(
        &symbol,
        60.0,
        20.0,
        market_filter,
        executor,
        account_tx,
    );

    let actor = AssetActor::new(
        symbol,
        strategy_id.to_string(), // strategy_id
        db_store.clone(),
        db_store,
        dna_rx,  // Channel nhận DNA từ Optimizer
        data_tx,
        emergency_rx,
        status_tx, // Truyền Sender trạng thái vào đây
        account_rx,
        live_engine,
        engine_config,
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
            let _ = emergency_tx.send(crate::models::models::EmergencyCmd::PanicCloseAll).await;
            // Đợi một chút để các Actor xử lý xong
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        // _ = ingestion_handle => { // Tạm bỏ do WsManager chạy ẩn
        //     warn!("⚠️ Luồng Ingestion đã kết thúc sớm.");
        // }
        _ = actor_handle => {
            warn!("⚠️ Luồng Actor đã kết thúc sớm.");
        }
    }

    Ok(())
}
