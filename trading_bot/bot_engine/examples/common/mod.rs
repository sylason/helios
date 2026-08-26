// trading_bot/bot_engine/examples/common/mod.rs
//
// 🛠️ SHARED UTILITIES & LOGGING CONFIGURATION FOR BOT ENGINE EXAMPLES

use std::error::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
