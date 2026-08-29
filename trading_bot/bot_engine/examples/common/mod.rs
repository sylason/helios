// trading_bot/bot_engine/examples/common/mod.rs
//
// 🛠️ SHARED UTILITIES & SCAFFOLD FRAMEWORK FOR BOT ENGINE EXAMPLES

use std::error::Error;
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::weight_monitor::WeightMonitor;

/// Khởi tạo Logger tiêu chuẩn (Xuất ra Console + 1 File log tổng hợp)
pub fn init_logging(strategy_id: &str) -> Result<WorkerGuard, Box<dyn Error>> {
    std::fs::create_dir_all("examples/logs")?;
    let log_filename = format!("{}.log", strategy_id);
    let file_appender = tracing_appender::rolling::never("examples/logs", &log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Ok(guard)
}

/// Khởi tạo Dual-Logger (Xuất ra Console + File log tổng hợp)
#[allow(dead_code)]
pub fn init_dual_logging(
    strategy_id: &str,
) -> Result<WorkerGuard, Box<dyn Error>> {
    init_logging(strategy_id)
}

/// [SCAFFOLD] Tự động dò tìm đường dẫn file cấu hình tương đối (Config Resolver)
#[allow(dead_code)]
pub fn resolve_strategy_config_path(strategy_id: &str) -> String {
    bot_engine::models::config::PathResolver::resolve_strategy_config(strategy_id)
}

/// [SCAFFOLD] Khởi tạo Database SQLite và WeightMonitor tiêu chuẩn cho các bài test
#[allow(dead_code)]
pub fn setup_sqlite_and_monitor(
    strategy_id: &str,
    weight_limit: u32,
) -> Result<(Arc<SqliteStore>, Arc<WeightMonitor>), Box<dyn Error>> {
    std::fs::create_dir_all("examples/storage")?;
    let db_path = format!("examples/storage/{}.db", strategy_id);
    let db_store = Arc::new(SqliteStore::new(&db_path)?);
    let monitor = Arc::new(WeightMonitor::new(weight_limit));
    Ok((db_store, monitor))
}
