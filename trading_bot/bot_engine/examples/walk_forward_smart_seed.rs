// trading_bot/bot_engine/examples/walk_forward_smart_seed.rs
//
// 🧪 WALK-FORWARD STRATEGY EVALUATOR WITH SMART SEEDING (0.5% CHAMPION SEED INJECTION)
// Luồng thực thi:
// 1. Nạp dữ liệu nến lịch sử từ Binance REST API vào SQLite.
// 2. Huấn luyện GA Optimizer ban đầu trên cửa sổ In-Sample (30 ngày).
// 3. Khởi chạy EventBus, PaperTrader & AssetActor ở chế độ `ExecutionMode::Paper`.
// 4. Bơm nến Out-Of-Sample liên tục theo thời gian thực.
// 5. [FEATURE A - SMART SEEDING 0.5%]: Định kỳ N phút, trước khi Re-Optimize, lấy duy nhất 1 cá thể Rank 1 Champion từ chu kỳ trước làm Seed tiêm vào Generation 0 (chiếm 0.5% quần thể), 99.5% còn lại là cá thể mới ngẫu nhiên.
// 6. [FEATURE B]: Định kỳ M phút, tự động trích xuất nến Rolling Buffer 30 ngày và gọi HeliosBridge::diagnostic_backtest để đo đạc hiệu suất Out-Of-Sample.

use std::path::Path;

use chrono::{TimeZone, Utc};
use tracing::{info, warn, error, debug};

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::models::repository::{StrategyRepository, CandleRepository};
use bot_engine::models::models::Candle;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::EngineConfig;
use bot_engine::controller::asset_actor::{AssetActor, ActorRunOutput};
use bot_engine::execution::engine::ExecutionEngineFactory;
use bot_engine::optimizer::prospector_runner::ProspectorRunner;
use bot_engine::optimizer::helios_bridge::HeliosBridge;
use bot_engine::telemetry::visual_report::VisualReportGenerator;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;

#[path = "common/mod.rs"]
mod common;

// ============================================================================
// ⚙️ BẢNG ĐIỀU KHIỂN & CẤU HÌNH TRUNG TÂM (CENTRAL TEST HARNESS CONFIG)
// ============================================================================
// 1. Nhận diện & Mục tiêu
// const SYMBOL: &str = "SOLUSDT";
const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "walk_forward_smart_seed";


// 2. Dữ liệu & Cửa sổ thời gian
const TOTAL_FETCH_DAYS: i64 = 45;        // Tổng số ngày dữ liệu tải về: 14 ngày Train + 8 ngày OOS Paper Trade test
const TRAIN_WINDOW_DAYS: i64 = 40;       // Độ dài cửa sổ nến huấn luyện In-Sample (14 ngày, 20.160 nến 1m)
const VALIDATION_SPAN_DAYS: i64 = 3;     // Độ dài cửa sổ nến kiểm định Out-Of-Sample WFE Gate (3 ngày = 4.320 nến 1m)
const VALIDATION_SPAN_BARS: usize = (VALIDATION_SPAN_DAYS * 24 * 60) as usize; // 4.320 nến (Tỷ lệ vàng 80/20)

// 3. Chu kỳ Vòng lặp Tịnh tiến
const REOPT_INTERVAL_MINS: i64 = 240;    // N MINUTES: Tần suất chạy lại GA Re-Optimizer (4h)
const BACKTEST_INTERVAL_MINS: i64 = 240;  // M MINUTES: Tần suất chạy Single Backtest chẩn đoán (4h đồng bộ)

// 4. Tài khoản & Đòn bẩy & Quản trị Vị thế (Account, Leverage & Sizing Settings)
const INITIAL_CASH: f64 = 60.0;          // Vốn khởi tạo ($60)
const LEVERAGE: f64 = 20.0;              // Đòn bẩy tài khoản tiêu chuẩn (20x)
const NUKE_LEVERAGE: f64 = 50.0;          // Đòn bẩy chiến lược Nuke độc lập (50x)
const PERCENT_PER_TRADE: f64 = 0.20;     // Tỷ lệ vốn ký quỹ mỗi lệnh (20% vốn)
const MIN_ORDER_USD: f64 = 21.0;         // Kích thước lệnh tối thiểu sàn cho phép ($21)
const MAX_CAPITAL_FRACTION: f64 = 0.25;  // Trần tỷ lệ ký quỹ tối đa trên mỗi vị thế (25% vốn)
const COMMISSION_RATE: f64 = 0.0004;     // Tỷ lệ phí giao dịch (0.04% Maker/Taker Binance Futures)
const FRACTIONAL_UNIT: f64 = 0.0001;     // Đơn vị bước nhảy khối lượng

// 5. Bộ nhớ Nến & Hiệu năng Động cơ
const TRAIN_WINDOW_BARS: usize = (TRAIN_WINDOW_DAYS * 24 * 60) as usize;
const MAX_LIVE_MEM_CANDLES: usize = TRAIN_WINDOW_BARS; // Bằng đúng số nến cửa sổ huấn luyện để slice_start = 0, loại bỏ nến cụt
const BUFFER_PADDING_BARS: usize = 200;    // Số nến dự phòng an toàn
const WEIGHT_LIMIT: u32 = 2400;            // Giới hạn IP Rate Limit Binance
const CHANNEL_CAPACITY: usize = 64;        // Dung lượng kênh Backpressure Async Channel

// 6. Công tắc Tính năng
const ENABLE_PAPER_TRADE: bool = true;    // true: Bật Paper Trading ($60 vốn thật giả lập); false: Tắt (chỉ chạy Diagnostic Eval)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Khởi tạo Log hệ thống từ module dùng chung
    let guard = common::init_logging(STRATEGY_ID)?;

    info!("================================================================================");
    info!("🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%");
    info!("   Symbol: {}, Strategy: {}", SYMBOL, STRATEGY_ID);
    info!("   Total Fetch: {} days | Train Window: {} days | WFE OOS: {} days ({} bars)", TOTAL_FETCH_DAYS, TRAIN_WINDOW_DAYS, VALIDATION_SPAN_DAYS, VALIDATION_SPAN_BARS);
    info!("   📌 [N MINS] Tần suất Re-Optimize GA:       {} phút", REOPT_INTERVAL_MINS);
    info!("   📌 [M MINS] Tần suất Single Backtest Eval:  {} phút", BACKTEST_INTERVAL_MINS);
    info!("   💰 [ACCOUNT] Initial Cash: ${:.2} | Leverage: {:.0}x | Comm: {:.4}", 
          INITIAL_CASH, LEVERAGE, COMMISSION_RATE);
    info!("   🧠 [MEMORY] Max Live Candles: {} (Đảm bảo 50+ nến MTF 4h)", MAX_LIVE_MEM_CANDLES);
    info!("   🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới");
    info!("================================================================================");

    // 0. Dọn dẹp sạch sẽ storage cũ trước khi khởi động bài test mới (Zero Stale State)
    let prospector_db_init = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
    let main_db_init = format!("examples/storage/{}.db", STRATEGY_ID);
    for path in [&prospector_db_init, &main_db_init] {
        if Path::new(path).exists() {
            let _ = std::fs::remove_file(path);
        }
        let shm = format!("{}-shm", path);
        let wal = format!("{}-wal", path);
        let _ = std::fs::remove_file(&shm);
        let _ = std::fs::remove_file(&wal);
    }

    // 2. Khởi tạo Database SQLite & Weight Monitor qua module dùng chung
    let (db_store, monitor) = common::setup_sqlite_and_monitor(STRATEGY_ID, WEIGHT_LIMIT)?;

    // 3. Nạp dữ liệu lịch sử N ngày
    info!("📥 [Data Ingestion] Đang đồng bộ dữ liệu {} ngày từ Binance...", TOTAL_FETCH_DAYS);
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());
    let _ = syncer.ensure_history(SYMBOL, TOTAL_FETCH_DAYS).await;
    let _ = syncer.fill_gap(SYMBOL, TOTAL_FETCH_DAYS).await;

    // 4. Lấy dải nến lịch sử có sẵn
    let candles = db_store.get_candles_by_range(SYMBOL, 0, i64::MAX).await?;
    if candles.is_empty() {
        error!("❌ Không tìm thấy dữ liệu nến trong Database! Dừng chương trình.");
        return Ok(());
    }

    let min_ts = candles.first().unwrap().timestamp;
    let max_ts = candles.last().unwrap().timestamp;

    let start_date_str = Utc.timestamp_millis_opt(min_ts).unwrap().format("%Y-%m-%d %H:%M UTC").to_string();
    let end_date_str = Utc.timestamp_millis_opt(max_ts).unwrap().format("%Y-%m-%d %H:%M UTC").to_string();

    info!("📊 [Data Summary] Tổng số nến: {} | Từ {} -> Đến {}", candles.len(), start_date_str, end_date_str);

    // 5. HUẤN LUYỆN ĐẦU TIÊN (IN-SAMPLE 30 NGÀY ĐẦU)
    let train_start_ts = min_ts;
    let train_end_ts = min_ts + (TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000);

    info!("--------------------------------------------------------------------------------");
    info!("🧬 [INITIAL OPTIMIZE] Chạy GA Optimizer huấn luyện ban đầu trên 30 ngày...");

    let prospector_db = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
    let walk_forward_smart_seed_db = format!("examples/storage/{}.db", STRATEGY_ID);
    if Path::new(&prospector_db).exists() {
        let _ = std::fs::remove_file(&prospector_db);
    }

    // Tự động tìm kiếm file cấu hình chiến lược qua Config Resolver dùng chung
    let config_path = common::resolve_strategy_config_path("helios");

    let (dna_tx, dna_rx) = tokio::sync::watch::channel(None);

    ProspectorRunner::run_optimization_cycle_range_custom(
        SYMBOL,
        STRATEGY_ID,
        db_store.clone(),
        &config_path,
        &prospector_db,
        Some(dna_tx.clone()),
        train_start_ts,
        train_end_ts,
        Some(VALIDATION_SPAN_BARS),
    ).await?;

    let initial_champ = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await?;
    let mut current_dna_vec = if let Some(ref champ) = initial_champ {
        let max_dd = champ.stats.get("max_drawdown").cloned().unwrap_or(0.0);
        info!("🏆 [INITIAL CHAMPION] In-Sample Return: {:.2}% | Max DD: {:.2}%", champ.fitness, max_dd);
        champ.dna.clone()
    } else {
        Vec::new()
    };

    // 6. KHỞI TẠO BỘ MÁY ASSETACTOR PAPER MODE
    let (sync_tx, actor_handle) = if ENABLE_PAPER_TRADE {
        info!("--------------------------------------------------------------------------------");
        info!("🚀 [ENGINE SETUP] Khởi tạo AssetActor ở chế độ ExecutionMode::Paper...");

        let (event_bus, _trade_rx, emergency_rx, mut status_rx) = EventBus::new();
        let data_tx = event_bus.data_tx.clone();
        let status_tx = event_bus.status_tx.clone();
        let account_tx = event_bus.account_tx.clone();
        let account_rx = event_bus.account_tx.subscribe();

        tokio::spawn(async move {
            while let Some(update) = status_rx.recv().await {
                info!("📢 [UI_STATUS] [{}] {:?} -> {}", update.symbol, update.state, update.message);
            }
        });

        let market_filter = syncer.fetch_market_filter(SYMBOL).await.unwrap_or_default();
        let mut strategy_inst = HeliosStrategyV3::new(&std::collections::HashMap::new());
        strategy_inst.debug_mode = true; // BẬT TELEMETRY CHO ASSET ACTOR TRONG WALK FORWARD (GA Optimizer KHÔNG bị ảnh hưởng)
        strategy_inst.config.risk_management.risk_base.leverage = LEVERAGE;
        strategy_inst.config.risk_management.risk_base.nuke_leverage = NUKE_LEVERAGE;
        strategy_inst.config.risk_management.risk_base.percent_per_trade = PERCENT_PER_TRADE;
        strategy_inst.config.risk_management.risk_base.min_order_usd = MIN_ORDER_USD;
        strategy_inst.config.risk_management.risk_base.max_cap_fraction = MAX_CAPITAL_FRACTION;
        strategy_inst.config.risk_management.risk_base.taker_fee_rate = COMMISSION_RATE;
        let strategy = Box::new(strategy_inst);

        let mut engine_config = EngineConfig::for_days(TRAIN_WINDOW_DAYS as u64);
        engine_config.initial_cash = INITIAL_CASH;
        engine_config.commission_rate = COMMISSION_RATE;
        engine_config.margin_rate = 1.0 / LEVERAGE;
        engine_config.fractional_unit = Some(FRACTIONAL_UNIT);
        engine_config.buffer_padding_bars = BUFFER_PADDING_BARS;
        engine_config.opt_num_threads = 4;

        let paper_engine = ExecutionEngineFactory::create_paper(
            SYMBOL,
            INITIAL_CASH,
            LEVERAGE,
            COMMISSION_RATE,
            market_filter,
            Some(account_tx),
        );

        let (tx, rx) = tokio::sync::mpsc::channel::<(Candle, bool)>(CHANNEL_CAPACITY);

        let actor = AssetActor::new(
            SYMBOL.to_string(),
            STRATEGY_ID.to_string(),
            db_store.clone(),
            db_store.clone(),
            dna_rx,
            data_tx.clone(),
            emergency_rx,
            status_tx,
            account_rx,
            paper_engine,
            engine_config,
        )
        .with_strategy(strategy)
        .with_sync_until(train_end_ts)
        .with_sync_data_rx(rx);

        let handle = tokio::spawn(async move {
            actor.run().await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        (Some(tx), Some(handle))
    } else {
        (None, None)
    };

    // 7. GIAI ĐOẠN OUT-OF-SAMPLE WITH PERIODIC RE-OPT & BACKTEST EVALUATION (CÓ SMART SEEDING)
    info!("--------------------------------------------------------------------------------");
    info!("🔄 [FORWARD EVALUATION LOOP] Bơm nến & Đánh giá hiệu suất định kỳ (Smart Seeding 0.5%)...");
    info!("--------------------------------------------------------------------------------");

    let oos_candles = db_store.get_candles_by_range(SYMBOL, train_end_ts, max_ts).await?;
    let reopt_span_ms = REOPT_INTERVAL_MINS * 60 * 1000;
    let backtest_span_ms = BACKTEST_INTERVAL_MINS * 60 * 1000;
    let eval_window_ms = TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000;

    let mut last_reopt_ts = train_end_ts;
    let mut last_backtest_ts = train_end_ts;

    let mut reopt_count = 0usize;
    let mut backtest_count = 0usize;
    let mut reopt_events: Vec<(i64, String)> = Vec::new();

    for candle in &oos_candles {
        let current_ts = candle.timestamp;

        // Bơm nến đồng bộ có Backpressure (is_historical = true để tắt I/O SQLite thừa thãi)
        if let Some(ref tx) = sync_tx {
            let _ = tx.send((candle.clone(), true)).await;
        }

        // ------------------------------------------------------------------------
        // [FEATURE B] ĐỊNH KỲ M PHÚT: CHẠY SINGLE BACKTEST ĐÁNH GIÁ CHẨN ĐOÁN
        // ------------------------------------------------------------------------
        if current_ts - last_backtest_ts >= backtest_span_ms {
            backtest_count += 1;
            last_backtest_ts = current_ts;

            let eval_start_ts = current_ts - eval_window_ms;
            let window_candles = db_store.get_candles_by_range(SYMBOL, eval_start_ts, current_ts).await?;

            if !window_candles.is_empty() && !current_dna_vec.is_empty() {
                let current_time_str = Utc.timestamp_millis_opt(current_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();
                
                let eval_start_instant = std::time::Instant::now();
                match HeliosBridge::diagnostic_backtest(&window_candles, &current_dna_vec) {
                    Ok(result) => {
                        let eval_duration = eval_start_instant.elapsed();
                        info!("📊 [SINGLE BACKTEST EVAL #{:04}] Time: {} | Window: 30D ({} candles) | Return: {:.2}% | Max DD: {:.2}% | Trades: {} | Win Rate: {:.2}% | Duration: {:.2}ms | Paper Trade Active: {}", 
                              backtest_count, 
                              current_time_str,
                              window_candles.len(), 
                              result.stats.return_pct, 
                              result.stats.max_drawdown_pct, 
                              result.stats.total_trades,
                              result.stats.win_rate_pct,
                              eval_duration.as_secs_f64() * 1000.0,
                              ENABLE_PAPER_TRADE);
                    }
                    Err(e) => {
                        error!("❌ Lỗi chạy Single Backtest Eval #{}: {}", backtest_count, e);
                    }
                }
            }
        }

        // ------------------------------------------------------------------------
        // [FEATURE A] ĐỊNH KỲ N PHÚT: CHẠY LẠI RE-OPTIMIZER VỚI KỸ THUẬT SMART SEEDING (0.5%)
        // ------------------------------------------------------------------------
        if current_ts - last_reopt_ts >= reopt_span_ms {
            reopt_count += 1;
            last_reopt_ts = current_ts;

            let opt_start_ts = current_ts - eval_window_ms;
            let opt_start_str = Utc.timestamp_millis_opt(opt_start_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();
            let opt_end_str = Utc.timestamp_millis_opt(current_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();

            info!("🧬 [RE-OPTIMIZE TRIGGERED #{:04}] Cửa sổ tối ưu GA: [{} -> {}]", 
                  reopt_count, opt_start_str, opt_end_str);
            reopt_events.push((current_ts, format!("Re-Opt #{:04}", reopt_count)));

            // [SMART SEEDING]: Lấy 1 cá thể Rank 1 Champion duy nhất từ kết quả trước đó
            let rank1_seed_dna = if !current_dna_vec.is_empty() {
                info!("🌱 [SMART SEEDING 0.5%] Đã chọn Rank 1 Champion DNA ({} chiều) làm hạt giống tinh nhuệ duy nhất.", current_dna_vec.len());
                Some(current_dna_vec.clone())
            } else {
                None
            };

            // Dọn dẹp cache đệm prospector_db cũ để 99.5% quần thể được làm mới ngẫu nhiên (LHS Sampling)
            if Path::new(&prospector_db).exists() {
                let _ = std::fs::remove_file(&prospector_db);
                let _ = std::fs::remove_file(&walk_forward_smart_seed_db);
                debug!("🧹 [SMART SEEDING] Đã reset prospector_db để làm mới 99.5% quần thể còn lại.");
            }

            let db_opt = db_store.clone();
            let dna_tx_clone = dna_tx.clone();
            let prospector_db_clone = prospector_db.clone();
            let config_path_owned = config_path.to_string();

            if let Err(e) = ProspectorRunner::run_optimization_cycle_range_smart_seed_custom(
                SYMBOL,
                STRATEGY_ID,
                db_opt,
                &config_path_owned,
                &prospector_db_clone,
                Some(dna_tx_clone),
                opt_start_ts,
                current_ts,
                rank1_seed_dna,
                Some(VALIDATION_SPAN_BARS),
            ).await {
                warn!("⚠️ Lỗi Re-Optimize lần {}: {}", reopt_count, e);
            } else if let Ok(Some(best_dna)) = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await {
                current_dna_vec = best_dna.dna.clone();
            }
        }
    }

    // Đóng kênh truyền nến để AssetActor hoàn tất các nến còn đọng và xuất báo cáo tổng kết
    drop(sync_tx);
    let run_output = if let Some(handle) = actor_handle {
        handle.await.unwrap_or_default()
    } else {
        ActorRunOutput::default()
    };

    info!("================================================================================");
    info!("✅ KẾT THÚC BÀI ĐÁNH GIÁ WALK-FORWARD EVALUATOR VỚI SMART SEEDING!");
    info!("   Tổng số lần chạy Re-Optimize (N mins): {}", reopt_count);
    info!("   Tổng số lần chạy Single Backtest Eval (M mins): {}", backtest_count);
    info!("================================================================================");

    // 8. TẠO BÁO CÁO TRỰC QUAN OOS VISUAL REPORT (HTML CANVAS 60FPS)
    info!("🎨 [VISUAL REPORT] Đang khởi tạo báo cáo tương tác OOS HTML tại {}...", guard.log_dir);
    match VisualReportGenerator::generate(
        SYMBOL,
        &oos_candles,
        &run_output.closed_trades,
        &reopt_events,
        run_output.gatekeeper_zones_json.as_deref(),
        INITIAL_CASH,
        &guard.log_dir,
    ) {
        Ok(html_path) => {
            info!("🌟 [VISUAL REPORT] Xuất báo cáo HTML thành công: {}", html_path);
        }
        Err(e) => {
            error!("❌ [VISUAL REPORT] Lỗi khi tạo báo cáo HTML: {}", e);
        }
    }

    // Dọn dẹp sạch sẽ các file database SQLite trong examples/storage sau khi bài test hoàn tất
    info!("🧹 [STORAGE CLEANUP] Đang dọn dẹp các database trong examples/storage...");
    let prospector_db_end = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
    let main_db_end = format!("examples/storage/{}.db", STRATEGY_ID);
    for path in [&prospector_db_end, &main_db_end] {
        if Path::new(path).exists() {
            let _ = std::fs::remove_file(path);
        }
        let shm = format!("{}-shm", path);
        let wal = format!("{}-wal", path);
        let _ = std::fs::remove_file(&shm);
        let _ = std::fs::remove_file(&wal);
    }
    info!("✅ [STORAGE CLEANUP] Đã dọn dẹp sạch sẽ database storage.");

    Ok(())
}
