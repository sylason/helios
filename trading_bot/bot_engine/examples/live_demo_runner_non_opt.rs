// trading_bot/bot_engine/examples/live_demo_runner_non_opt.rs
//
// 🧪 KHAI THÁC THÍ NGHIỆM DNA DECAY FORWARD TEST (OUT-OF-SAMPLE ROLLING SLIDING WINDOW)
// Luồng thực thi:
// 1. Đồng bộ N ngày dữ liệu từ Binance REST API vào Database SQLite (Ví dụ: 90 ngày).
// 2. Chạy GA Optimizer 1 LẦN DUY NHẤT trên 30 ngày dữ liệu đầu tiên (In-Sample Train).
// 3. Trích xuất Champion DNA duy nhất.
// 4. Tịnh tiến cửa sổ Out-of-Sample 30 ngày (Rolling Window) mỗi 15 phút.
// 5. Tái sử dụng hàm `HeliosBridge::diagnostic_backtest` có sẵn trong `bot_engine` để kiểm tra kết quả nhanh & chuẩn xác.

use std::path::Path;

use chrono::{TimeZone, Utc};
use tracing::{info, warn, error};

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::models::repository::{StrategyRepository, CandleRepository};
use bot_engine::optimizer::prospector_runner::ProspectorRunner;
use bot_engine::optimizer::helios_bridge::HeliosBridge;

#[path = "common/mod.rs"]
mod common;

// ============================================================================
// ⚙️ CẤU HÌNH THỬ NGHIỆM FORWARD TEST (NGƯỜI DÙNG CÓ THỂ ĐIỀU CHỈNH)
// ============================================================================
const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "helios_v3_decay_test";

const TOTAL_FETCH_DAYS: i64 = 90;      // Tổng số ngày dữ liệu lịch sử tải về (VD: 90 ngày)
const TRAIN_WINDOW_DAYS: i64 = 30;     // Số ngày huấn luyện GA Optimizer ban đầu (VD: 30 ngày đầu)
const EVAL_WINDOW_DAYS: i64 = 30;      // Độ dài cửa sổ đánh giá Out-Of-Sample (30 ngày)
const STEP_INTERVAL_MINS: i64 = 15;    // Bước tịnh tiến Forward Backtest (15 phút)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Khởi tạo hệ thống Log (Console + File log chuyên dụng)
    let _guard = common::init_logging(STRATEGY_ID)?;

    info!("================================================================================");
    info!("🧪 BẮT ĐẦU FORWARD TEST KIỂM THỬ ĐỘ SUY GIẢM DNA (FIXED TRAINED DNA DECAY TEST)");
    info!("   Symbol: {}, Strategy: {}", SYMBOL, STRATEGY_ID);
    info!("   Total Fetch: {} days | Train Window: {} days | Step Interval: {} mins", 
          TOTAL_FETCH_DAYS, TRAIN_WINDOW_DAYS, STEP_INTERVAL_MINS);
    info!("   Engine Mode: Single Backtest Evaluation (HeliosBridge::diagnostic_backtest)");
    info!("================================================================================");

    // 2. Khởi tạo Database SQLite & Weight Monitor
    let (db_store, monitor) = common::setup_sqlite_and_monitor(STRATEGY_ID, 2400)?;

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

    // 5. GIAI ĐOẠN 1: TỐI ƯU HÓA GA OPTIMIZER 1 LẦN DUY NHẤT TRÊN 30 NGÀY ĐẦU (In-Sample Train)
    let train_start_ts = min_ts;
    let train_end_ts = min_ts + (TRAIN_WINDOW_DAYS * 24 * 60 * 60 * 1000);

    let train_start_str = Utc.timestamp_millis_opt(train_start_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let train_end_str = Utc.timestamp_millis_opt(train_end_ts).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    info!("--------------------------------------------------------------------------------");
    info!("🧬 [PHASE 1 - IN-SAMPLE TRAIN] Khởi chạy Prospector GA Optimizer 1 LẦN DUY NHẤT...");
    info!("   📌 TẬP DỮ LIỆU HUẤN LUYỆN ĐÃ ĐÁNH DẤU: [{}] -> [{}]", train_start_str, train_end_str);

    // Reset DB Prospector tạm để dọn dẹp job cũ nếu có
    let prospector_db = format!("examples/storage/{}_prospector.db", STRATEGY_ID);
    if Path::new(&prospector_db).exists() {
        let _ = std::fs::remove_file(&prospector_db);
        info!("🧹 Đã dọn dẹp file Prospector DB cũ: {}", prospector_db);
    }

    let config_path = common::resolve_strategy_config_path("helios");

    if let Err(e) = ProspectorRunner::run_optimization_cycle_range(
        SYMBOL,
        STRATEGY_ID,
        db_store.clone(),
        &config_path,
        &prospector_db,
        None,
        train_start_ts,
        train_end_ts
    ).await {
        error!("❌ Lỗi khi chạy Optimizer: {}", e);
        return Ok(());
    }

    // Trích xuất Champion DNA từ Database
    let champion_dna_opt = db_store.get_best_dna(SYMBOL, STRATEGY_ID).await?;
    let champion_dna = match champion_dna_opt {
        Some(opt_dna) => opt_dna,
        None => {
            error!("❌ Không tìm thấy Champion DNA trong Database sau khi chạy Optimizer!");
            return Ok(());
        }
    };

    info!("================================================================================");
    info!("🏆 [CHAMPION DNA ACQUIRED] Đã thu được bộ DNA Champion duy nhất!");
    info!("   📌 Dữ liệu Huấn Luyện (In-Sample): [{}] -> [{}]", train_start_str, train_end_str);
    info!("   Fitness (In-Sample Return): {:.2}%", champion_dna.fitness);
    info!("   Updated At: {}", Utc.timestamp_millis_opt(champion_dna.updated_at).unwrap().format("%Y-%m-%d %H:%M:%S UTC"));
    info!("================================================================================");

    // 6. GIAI ĐOẠN 2: FORWARD BACKTEST (OUT-OF-SAMPLE ROLLING SLIDING WINDOW)
    // Tái sử dụng trực tiếp hàm `HeliosBridge::diagnostic_backtest` đã có sẵn trong `bot_engine`
    let oos_start_base = train_end_ts;
    let eval_span_ms = EVAL_WINDOW_DAYS * 24 * 60 * 60 * 1000;

    let initial_oos_end = oos_start_base + eval_span_ms;
    let initial_oos_end_str = Utc.timestamp_millis_opt(initial_oos_end).unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    info!("--------------------------------------------------------------------------------");
    info!("🔄 [PHASE 2 - OUT-OF-SAMPLE FORWARD TEST] Khởi chạy Rolling Evaluation...");
    info!("   📌 Cửa sổ Out-of-Sample đầu tiên: [{}] -> [{}]", train_end_str, initial_oos_end_str);
    info!("   Bước tịnh tiến: {} phút mỗi lượt cho đến hết dữ liệu.", STEP_INTERVAL_MINS);
    info!("--------------------------------------------------------------------------------");

    let step_ms = STEP_INTERVAL_MINS * 60 * 1000;
    let mut current_win_start = oos_start_base;
    let mut step_count = 0usize;

    // Header bảng Log
    info!("{:<6} | {:<20} | {:<20} | {:<12} | {:<10} | {:<8} | {:<10}", 
          "Step", "Window Start (OOS)", "Window End (OOS)", "Return %", "Max DD %", "Trades", "Win Rate %");
    info!("---------------------------------------------------------------------------------------------------");

    while current_win_start + eval_span_ms <= max_ts {
        step_count += 1;
        let current_win_end = current_win_start + eval_span_ms;

        let win_start_str = Utc.timestamp_millis_opt(current_win_start).unwrap().format("%Y-%m-%d %H:%M").to_string();
        let win_end_str = Utc.timestamp_millis_opt(current_win_end).unwrap().format("%Y-%m-%d %H:%M").to_string();

        // Lấy dữ liệu nến 30 ngày Out-Of-Sample trong cửa sổ tịnh tiến
        let window_candles = db_store.get_candles_by_range(SYMBOL, current_win_start, current_win_end).await?;
        if window_candles.len() < 1000 {
            current_win_start += step_ms;
            continue;
        }

        // Tái sử dụng cơ chế Backtest & Precompute MTF có sẵn từ HeliosBridge
        match HeliosBridge::diagnostic_backtest(&window_candles, &champion_dna.dna) {
            Ok(result) => {
                let ret = result.stats.return_pct;
                let dd = result.stats.max_drawdown_pct;
                let trades = result.stats.total_trades;
                let win_rate = result.stats.win_rate_pct;

                info!("#{:04}  | {:<20} | {:<20} | {:>11.2}% | {:>9.2}% | {:>8} | {:>9.2}%", 
                      step_count, win_start_str, win_end_str, ret, dd, trades, win_rate);
            },
            Err(e) => {
                warn!("⚠️ Lỗi chạy Backtest tại step {}: {}", step_count, e);
            }
        }

        // Tịnh tiến thêm 15 phút
        current_win_start += step_ms;
    }

    info!("================================================================================");
    info!("✅ BÁO CÁO THÍ NGHIỆM THÀNH CÔNG: Đã hoàn tất {} bước tịnh tiến Rolling Backtest.", step_count);
    info!("   📌 Dữ liệu Huấn Luyện (In-Sample): [{}] -> [{}]", train_start_str, train_end_str);
    info!("   Log chi tiết đã được ghi đầy đủ vào file: examples/logs/{}.log", STRATEGY_ID);
    info!("================================================================================");

    Ok(())
}
