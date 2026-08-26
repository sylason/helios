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

use std::sync::Arc;
use std::path::Path;

use chrono::{TimeZone, Utc};
use tracing::{info, warn, error};

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::repository::{StrategyRepository, CandleRepository};
use bot_engine::models::weight_monitor::WeightMonitor;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::EngineConfig;
use bot_engine::controller::asset_actor::AssetActor;
use bot_engine::execution::engine::ExecutionEngineFactory;
use bot_engine::optimizer::prospector_runner::ProspectorRunner;
use bot_engine::optimizer::helios_bridge::HeliosBridge;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;

#[path = "common/mod.rs"]
mod common;

// ============================================================================
// ⚙️ CẤU HÌNH TẦN SUẤT CHẠY OPTIMIZER (N MINS) VÀ SINGLE BACKTEST (M MINS)
// ============================================================================
const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "walk_forward_smart_seed";

const TOTAL_FETCH_DAYS: i64 = 91;        // Tổng số ngày dữ liệu tải về (365 ngày)
const TRAIN_WINDOW_DAYS: i64 = 30;       // Cửa sổ nến huấn luyện GA (30 ngày)

const REOPT_INTERVAL_MINS: i64 = 240;    // N MINUTES: Tần suất chạy lại GA Optimizer (VD: 360 phút / 6 giờ, 44640 / 31 days)
const BACKTEST_INTERVAL_MINS: i64 = 3;  // M MINUTES: Tần suất chạy Single Backtest chẩn đoán (VD: 60 phút / 1 giờ, 1440 phút / 24 giờ)
const ENABLE_PAPER_TRADE: bool = false;   // CÔNG TẮC TẮT/MỞ PAPER TRADE: true = Bật Paper Trading ($60 initial equity), false = Tắt (chỉ chạy Diagnostic)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Khởi tạo Log hệ thống từ module dùng chung
    let _guard = common::init_logging(STRATEGY_ID)?;

    info!("================================================================================");
    info!("🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%");
    info!("   Symbol: {}, Strategy: {}", SYMBOL, STRATEGY_ID);
    info!("   Total Fetch: {} days | Train Window: {} days", TOTAL_FETCH_DAYS, TRAIN_WINDOW_DAYS);
    info!("   📌 [N MINS] Tần suất Re-Optimize GA:      {} phút", REOPT_INTERVAL_MINS);
    info!("   📌 [M MINS] Tần suất Single Backtest Eval: {} phút", BACKTEST_INTERVAL_MINS);
    info!("   🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới");
    info!("================================================================================");

    // 2. Khởi tạo Database SQLite & Weight Monitor
    std::fs::create_dir_all("examples/storage")?;
    let db_path = format!("examples/storage/{}.db", STRATEGY_ID);
    let db_store = Arc::new(SqliteStore::new(&db_path)?);
    let monitor = Arc::new(WeightMonitor::new(2400));

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
    if Path::new(&prospector_db).exists() {
        let _ = std::fs::remove_file(&prospector_db);
    }

    let config_local = "examples/helios_optimizer_config.json";
    let config_path = if Path::new(config_local).exists() {
        config_local
    } else {
        r#"E:\Projects\Automations\trading_backtest\strategies_backtests\trading_bot_demo\examples\helios_optimizer_config.json"#
    };

    let (dna_tx, dna_rx) = tokio::sync::watch::channel(None);

    ProspectorRunner::run_optimization_cycle_range(
        SYMBOL,
        STRATEGY_ID,
        db_store.clone(),
        config_path,
        &prospector_db,
        Some(dna_tx.clone()),
        train_start_ts,
        train_end_ts
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
    info!("--------------------------------------------------------------------------------");
    info!("🚀 [ENGINE SETUP] Khởi tạo AssetActor ở chế độ ExecutionMode::Paper...");

    let (event_bus, _trade_rx, emergency_rx, mut status_rx) = EventBus::new();
    let data_tx = event_bus.data_tx.clone();
    let _trade_tx = event_bus.trade_tx.clone();
    let status_tx = event_bus.status_tx.clone();

    let account_tx = event_bus.account_tx.clone();
    let account_rx = event_bus.account_tx.subscribe();

    tokio::spawn(async move {
        while let Some(update) = status_rx.recv().await {
            info!("📢 [UI_STATUS] [{}] {:?} -> {}", update.symbol, update.state, update.message);
        }
    });

    let market_filter = syncer.fetch_market_filter(SYMBOL).await.unwrap_or_default();
    let strategy = Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new()));

    let engine_config = EngineConfig {
        initial_cash: 60.0,
        commission_rate: 0.0004,
        margin_rate: 1.0 / 20.0,
        fractional_unit: Some(0.0001),
        max_live_mem_candles: 15000,
        historical_sync_days: TOTAL_FETCH_DAYS as u64,
        buffer_padding_bars: 200,
        opt_num_threads: 4,
    };

    let paper_engine = ExecutionEngineFactory::create_paper(
        SYMBOL,
        60.0,
        20.0,
        0.0004,
        market_filter,
        Some(account_tx),
    );

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
    ).with_strategy(strategy);

    tokio::spawn(async move {
        actor.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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

    for candle in oos_candles {
        let current_ts = candle.timestamp;

        // Bơm nến real-time qua EventBus
        let _ = data_tx.send((candle.clone(), false));

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
                
                match HeliosBridge::diagnostic_backtest(&window_candles, &current_dna_vec) {
                    Ok(result) => {
                        info!("📊 [SINGLE BACKTEST EVAL #{:04}] Time: {} | Window: 30D ({} candles) | Return: {:.2}% | Max DD: {:.2}% | Trades: {} | Win Rate: {:.2}% | Paper Trade Active: {}", 
                              backtest_count, 
                              current_time_str,
                              window_candles.len(), 
                              result.stats.return_pct, 
                              result.stats.max_drawdown_pct, 
                              result.stats.total_trades,
                              result.stats.win_rate_pct,
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

            let all_start_str = Utc.timestamp_millis_opt(train_start_ts).unwrap().format("%Y-%m-%d %H:%M").to_string();
            info!("📊 [CUMULATIVE PERIODIC REPORT #{:04}] Báo cáo kết quả Paper Trades tích lũy từ mốc khởi đầu [{}] -> [{}]", 
                  reopt_count, all_start_str, opt_end_str);

            info!("🧬 [RE-OPTIMIZE TRIGGERED #{:04}] Chạy GA Optimizer trên cửa sổ [{} -> {}]", 
                  reopt_count, opt_start_str, opt_end_str);

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
                info!("🧹 [SMART SEEDING] Đã reset prospector_db để làm mới 99.5% quần thể còn lại.");
            }

            let db_opt = db_store.clone();
            let dna_tx_clone = dna_tx.clone();
            let prospector_db_clone = prospector_db.clone();
            let config_path_owned = config_path.to_string();

            if let Err(e) = ProspectorRunner::run_optimization_cycle_range_smart_seed(
                SYMBOL,
                STRATEGY_ID,
                db_opt,
                &config_path_owned,
                &prospector_db_clone,
                Some(dna_tx_clone),
                opt_start_ts,
                current_ts,
                rank1_seed_dna,
            ).await {
                warn!("⚠️ Lỗi Re-Optimize lần {}: {}", reopt_count, e);
            } else if let Ok(Some(best_dna)) = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await {
                current_dna_vec = best_dna.dna.clone();
                info!("✨ [RE-OPTIMIZE COMPLETE #{:04}] DNA mới cập nhật thành công! (Fitness: {:.2}%)", 
                      reopt_count, best_dna.fitness);
            }
        }
    }

    info!("================================================================================");
    info!("✅ KẾT THÚC BÀI ĐÁNH GIÁ WALK-FORWARD EVALUATOR VỚI SMART SEEDING!");
    info!("   Tổng số lần chạy Re-Optimize (N mins): {}", reopt_count);
    info!("   Tổng số lần chạy Single Backtest Eval (M mins): {}", backtest_count);
    info!("   File log chi tiết: examples/logs/{}.log", STRATEGY_ID);
    info!("================================================================================");

    Ok(())
}
