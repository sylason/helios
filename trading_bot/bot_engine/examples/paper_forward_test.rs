// trading_bot/bot_engine/examples/paper_forward_test.rs
//
// 🧪 FULL ENGINE PAPER FORWARD TEST (HỆ THỐNG GIAO DỊCH GIẢ LẬP VỚI THÍCH ỨNG RE-OPTIMIZE 15 PHÚT)
// Luồng thực thi:
// 1. Nạp N ngày dữ liệu nến lịch sử từ Binance REST API vào Database SQLite nội bộ.
// 2. Chạy GA Prospector Optimizer huấn luyện ban đầu trên 30 ngày đầu tiên (In-Sample Train).
// 3. Khởi tạo EventBus, PaperTrader & AssetActor ở chế độ `ExecutionMode::Paper`.
// 4. Khởi chạy Task Re-Optimizer ngầm định kỳ 15 phút, phát tín hiệu DNA mới qua `dna_tx`.
// 5. AssetActor nhận nến real-time qua EventBus, tính toán tín hiệu, Hot-Swap DNA ngầm qua `dna_rx`,
//    và thực thi lệnh giả lập (High/Low SL scanning & PnL tracking) qua PaperTrader.

use std::sync::Arc;
use std::path::Path;

use chrono::{TimeZone, Utc};
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::models::AssetState;
use bot_engine::models::repository::{StrategyRepository, CandleRepository};
use bot_engine::models::weight_monitor::WeightMonitor;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::ExecutionMode;
use bot_engine::controller::asset_actor::AssetActor;
use bot_engine::optimizer::prospector_runner::ProspectorRunner;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;

// ============================================================================
// ⚙️ BẢNG CẤU HÌNH TẬP TRUNG (TEST & RE-OPTIMIZER CONFIGURATION)
// ============================================================================
const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "paper_forward_v3";

// 1. Cấu hình Dữ liệu & Cửa sổ Thời gian
const TOTAL_FETCH_DAYS: i64 = 32;      // Tổng số ngày dữ liệu kéo từ Binance (33 ngày: 30 ngày train + 3 ngày forward test)
const TRAIN_WINDOW_DAYS: i64 = 30;     // Độ dài cửa sổ huấn luyện GA Optimizer (In-Sample: 30 ngày)

// 2. Cấu hình Tần suất Re-Optimizer định kỳ
const STEP_INTERVAL_MINS: i64 = 240;   // Tần suất Re-Optimize định kỳ (1440 phút = 24 giờ)
const ENABLE_SMART_SEEDING: bool = true; // Tiêm hạt giống Rank 1 Champion trước đó vào GA mới

// 3. Cấu hình Mô phỏng Execution & Bộ nhớ Trượt
const INITIAL_CAPITAL: f64 = 60.0;     // Quỹ vốn giả lập ($60 USD)
const LEVERAGE: f64 = 20.0;            // Đòn bẩy giả lập (20x)
const MAX_LIVE_MEM_CANDLES: usize = 4000; // Cửa sổ bộ nhớ trượt cho strategy.init (Tối ưu 30x tốc độ)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Khởi tạo Hệ thống Log (Console + File log chuyên dụng)
    std::fs::create_dir_all("examples/logs")?;
    let log_filename = format!("{}.log", STRATEGY_ID);
    let file_appender = tracing_appender::rolling::never("examples/logs", &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("================================================================================");
    info!("🧪 BẮT ĐẦU THỬ NGHIỆM FULL ENGINE PAPER FORWARD TEST (15-MIN RE-OPT)");
    info!("   Symbol: {}, Strategy: {}", SYMBOL, STRATEGY_ID);
    info!("   Total Fetch: {} days | Train Window: {} days | Re-Opt Frequency: {} mins", 
          TOTAL_FETCH_DAYS, TRAIN_WINDOW_DAYS, STEP_INTERVAL_MINS);
    info!("   Execution Mode: ExecutionMode::Paper (Sliding Window Memory: {} candles)", MAX_LIVE_MEM_CANDLES);
    info!("================================================================================");

    // 2. Khởi tạo Database SQLite & Weight Monitor
    std::fs::create_dir_all("examples/storage")?;
    let db_path = format!("examples/storage/{}.db", STRATEGY_ID);
    let db_store = Arc::new(SqliteStore::new(&db_path)?);
    let monitor = Arc::new(WeightMonitor::new(2400));

    // 3. Nạp dữ liệu lịch sử N ngày từ Binance REST API
    info!("📥 [Data Ingestion] Đang đồng bộ dữ liệu {} ngày từ Binance...", TOTAL_FETCH_DAYS);
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());
    if let Err(e) = syncer.ensure_history(SYMBOL, TOTAL_FETCH_DAYS).await {
        warn!("⚠️ Lỗi đồng bộ dữ liệu Backward: {}. Tiếp tục với dữ liệu hiện có.", e);
    }
    if let Err(e) = syncer.fill_gap(SYMBOL, TOTAL_FETCH_DAYS).await {
        warn!("⚠️ Lỗi đồng bộ Gap Forward: {}. Tiếp tục với dữ liệu hiện có.", e);
    }

    // 4. Lấy phạm vi nến có sẵn trong Database
    let candles = db_store.get_candles_by_range(SYMBOL, 0, i64::MAX).await?;
    if candles.is_empty() {
        error!("❌ Không tìm thấy dữ liệu nến nào trong Database! Dừng chương trình.");
        return Ok(());
    }

    let min_ts = candles.first().unwrap().timestamp;
    let max_ts = candles.last().unwrap().timestamp;
    let total_candles = candles.len();

    let start_date_str = Utc.timestamp_millis_opt(min_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let end_date_str = Utc.timestamp_millis_opt(max_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    info!("📊 [Data Summary] Tổng số nến: {} | Từ: {} -> Đến: {}", total_candles, start_date_str, end_date_str);

    // 5. GIAI ĐOẠN 1: HUẤN LUYỆN ĐẦU TIÊN (IN-SAMPLE 30 NGÀY ĐẦU)
    let train_start_ts = min_ts;
    let train_end_ts = min_ts + (TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000);

    let train_start_str = Utc.timestamp_millis_opt(train_start_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let train_end_str = Utc.timestamp_millis_opt(train_end_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    info!("--------------------------------------------------------------------------------");
    info!("🧬 [PHASE 1 - INITIAL TRAIN] Khởi chạy Prospector GA Optimizer ban đầu...");
    info!("   📌 Dữ liệu Huấn Luyện (In-Sample): [{}] -> [{}]", train_start_str, train_end_str);

    let prospector_db = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
    if Path::new(&prospector_db).exists() {
        let _ = std::fs::remove_file(&prospector_db);
    }

    let config_local = "examples/helios_optimizer_config.json";
    let config_path = if Path::new(config_local).exists() {
        config_local
    } else {
        r#"E:\Projects\Automations\trading_backtest\strategies_backtests\trading_bot_demo\examples\helios_optimizer_config.json"#
    };

    // Khởi tạo kênh truyền tín hiệu Event-Driven cho DNA Hot-Swapping (Optimizer -> AssetActor)
    let (dna_tx, dna_rx) = tokio::sync::watch::channel(None);

    if let Err(e) = ProspectorRunner::run_optimization_cycle_range(
        SYMBOL,
        STRATEGY_ID,
        db_store.clone(),
        config_path,
        &prospector_db,
        Some(dna_tx.clone()),
        train_start_ts,
        train_end_ts
    ).await {
        error!("❌ Lỗi khi chạy Optimizer ban đầu: {}", e);
        return Ok(());
    }

    let initial_champion = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await?;

    // 6. KHỞI TẠO BỘ MÁY ENGINE (ASSET ACTOR IN PAPER MODE)
    info!("--------------------------------------------------------------------------------");
    info!("🚀 [PHASE 2 - ENGINE SETUP] Khởi tạo EventBus & AssetActor (ExecutionMode::Paper)...");

    let (event_bus, _trade_rx, emergency_rx, mut status_rx) = EventBus::new();
    let data_tx = event_bus.data_tx.clone();
    let trade_tx = event_bus.trade_tx.clone();
    let status_tx = event_bus.status_tx.clone();
    let account_tx = event_bus.account_tx.clone();
    let account_rx = event_bus.account_tx.subscribe();

    // Spawn listener ghi nhận Status Update từ AssetActor
    tokio::spawn(async move {
        while let Some(update) = status_rx.recv().await {
            info!("📢 [UI_STATUS] [{}] {:?} -> {}", update.symbol, update.state, update.message);
        }
    });

    let market_filter = syncer.fetch_market_filter(SYMBOL).await.unwrap_or_default();
    let strategy = Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new()));

    // Khởi tạo AssetActor ở chế độ ExecutionMode::Paper
    let mut actor = AssetActor::new(
        SYMBOL.to_string(),
        STRATEGY_ID.to_string(),
        db_store.clone(),
        db_store.clone(),
        monitor.clone(),
        dna_rx,
        data_tx.clone(),
        emergency_rx,
        status_tx,
        trade_tx,
        account_rx,
        account_tx.clone(),
        market_filter,
        None, // executor = None (Không dùng Binance REST Trade API)
        INITIAL_CAPITAL,
        LEVERAGE,
        ExecutionMode::Paper, // Chế độ Paper Trader Giả lập
    )
    .with_strategy(strategy)
    .with_max_mem_candles(MAX_LIVE_MEM_CANDLES);

    // Nạp dữ liệu lịch sử vào Rolling Buffer ĐẾN MỐC BẮT ĐẦU OUT-OF-SAMPLE
    if let Err(e) = actor.handle_initial_sync_until(train_end_ts).await {
        error!("❌ Lỗi nạp dữ liệu lịch sử cho AssetActor: {}", e);
    }

    if let Some(ref champ) = initial_champion {
        actor.update_strategy_dna(&champ.dna);
    }
    actor.update_state(AssetState::Trading, "Hệ thống Helios đã sẵn sàng!").await;

    // 7. GIAI ĐOẠN 3: BƠM NẾN REAL-TIME GIẢ LẬP & RE-OPTIMIZE ĐỊNH KỲ
    info!("--------------------------------------------------------------------------------");
    info!("🔄 [PHASE 3 - FORWARD EXECUTION] Khởi chạy luồng mô phỏng đồng bộ & Re-Optimize ngầm...");
    info!("--------------------------------------------------------------------------------");

    let step_ms = STEP_INTERVAL_MINS * 60 * 1000;
    let eval_span_ms = TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000;

    let oos_start_ts = train_end_ts + (60 * 1000);
    let oos_candles = db_store.get_candles_by_range(SYMBOL, oos_start_ts, max_ts).await?;
    info!("📊 Số nến Out-Of-Sample sẽ bơm: {} nến (Từ {} đến {})", 
          oos_candles.len(), 
          Utc.timestamp_millis_opt(oos_start_ts).unwrap().format("%Y-%m-%d %H:%M"),
          end_date_str);

    let mut last_reopt_ts = train_end_ts;
    let mut step_count = 0usize;

    for candle in &oos_candles {
        let current_ts = candle.timestamp;

        // Cập nhật DNA mới từ Re-Optimizer ngầm nếu có
        actor.check_and_update_dna();

        // Xử lý nến trực tiếp qua AssetActor
        actor.on_new_candle(candle.clone(), false).await;

        // Tự động kích hoạt Re-Optimizer định kỳ
        if current_ts - last_reopt_ts >= step_ms {
            step_count += 1;
            last_reopt_ts = current_ts;

            let opt_start_ts = current_ts - eval_span_ms;
            let opt_start_str = Utc.timestamp_millis_opt(opt_start_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();
            let opt_end_str = Utc.timestamp_millis_opt(current_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();

            let all_start_str = Utc.timestamp_millis_opt(train_start_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();
            info!("📊 [CUMULATIVE PERIODIC REPORT #{:04}] Báo cáo kết quả Paper Trades tích lũy từ mốc khởi đầu [{}] -> [{}]", 
                  step_count, all_start_str, opt_end_str);
            actor.log_paper_report(candle.close, &opt_end_str);
            actor.log_all_trades_report(candle.close);

            info!("🧬 [#{:04} RE-OPTIMIZE TRIGGERED] Chạy GA Optimizer tuần tự trên cửa sổ [{} -> {}]", 
                  step_count, opt_start_str, opt_end_str);

            let db_opt = db_store.clone();
            let dna_tx_clone = dna_tx.clone();
            let step_prospector_db = format!("examples/storage/{}_prospector_step_{}.db", STRATEGY_ID, step_count);
            let config_path_owned = config_path.to_string();

            let seed_dna = if ENABLE_SMART_SEEDING {
                match db_opt.get_best_dna(SYMBOL, STRATEGY_ID).await {
                    Ok(Some(dna_data)) => Some(dna_data.dna),
                    _ => None,
                }
            } else {
                None
            };

            // Chạy Re-Optimizer TUẦN TỰ (.await) để tạm dừng luồng bơm nến, nạp DNA mới rồi mới bơm nến tiếp
            if let Err(e) = ProspectorRunner::run_optimization_cycle_range_smart_seed(
                SYMBOL,
                STRATEGY_ID,
                db_opt,
                &config_path_owned,
                &step_prospector_db,
                Some(dna_tx_clone),
                opt_start_ts,
                current_ts,
                seed_dna,
            ).await {
                warn!("⚠️ Lỗi Re-Optimize tại step {}: {}", step_count, e);
            }

            // Cập nhật ngay bộ não Helios với DNA mới nhất vừa tìm được
            actor.check_and_update_dna();
        }
    }

    // Cho AssetActor 1 giây để xử lý nốt các nến còn lại trong hàng đợi
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    if let Some(last_candle) = oos_candles.last() {
        actor.log_paper_report(last_candle.close, &end_date_str);
        actor.log_all_trades_report(last_candle.close);
    }

    info!("================================================================================");
    info!("✅ KẾT THÚC BÀI THỬ NGHIỆM PAPER FORWARD TEST THÀNH CÔNG!");
    info!("   Tổng số lượt Re-Optimize định kỳ: {}", step_count);
    info!("   Log chi tiết đã được lưu đầy đủ tại: examples/logs/{}.log", STRATEGY_ID);
    info!("================================================================================");

    Ok(())
}
