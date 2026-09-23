// trading_bot/bot_engine/examples/common/mod.rs
//
// 🛠️ SHARED UTILITIES & SCAFFOLD FRAMEWORK FOR BOT ENGINE EXAMPLES

use std::error::Error;
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::weight_monitor::WeightMonitor;

/// Bộ bảo vệ WorkerGuard kép cho 2 luồng log không đồng bộ
pub struct LoggingGuards {
    pub _system: WorkerGuard,
    pub _trades: WorkerGuard,
}

/// Khởi tạo Logger kép độc lập (Dual-Appender: System Log + Trades OOS Telemetry Log + Lean Console)
pub fn init_logging(strategy_id: &str) -> Result<LoggingGuards, Box<dyn Error>> {
    std::fs::create_dir_all("examples/logs")?;

    // 1. File Log Hệ Thống ({strategy_id}_system.log)
    let system_log_filename = format!("{}_system.log", strategy_id);
    let system_appender = tracing_appender::rolling::never("examples/logs", &system_log_filename);
    let (system_non_blocking, system_guard) = tracing_appender::non_blocking(system_appender);

    let system_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("warn")
            .add_directive("bot_engine=info".parse().unwrap())
            .add_directive("walk_forward_smart_seed=info".parse().unwrap())
            .add_directive("helios=info".parse().unwrap())
    });

    let system_layer = fmt::layer()
        .with_writer(system_non_blocking)
        .with_ansi(false)
        .with_filter(system_filter);

    // 2. File Log Giao Dịch OOS Chuyên Biệt ({strategy_id}_trades.log)
    let trades_log_filename = format!("{}_trades.log", strategy_id);
    let trades_appender = tracing_appender::rolling::never("examples/logs", &trades_log_filename);
    let (trades_non_blocking, trades_guard) = tracing_appender::non_blocking(trades_appender);

    let trades_filter = EnvFilter::new("off")
        .add_directive("bot_engine::execution::paper_trader=info".parse().unwrap())
        .add_directive("bot_engine::controller::asset_actor=info".parse().unwrap())
        .add_directive("helios::strategy::core_v3::arbiter=info".parse().unwrap())
        .add_directive("helios::strategy::core_v3=info".parse().unwrap())
        .add_directive("walk_forward_smart_seed=info".parse().unwrap());

    let trades_layer = fmt::layer()
        .with_writer(trades_non_blocking)
        .with_ansi(false)
        .with_filter(trades_filter);

    // 3. Console Output (Chỉ in các mốc quan trọng, không spam terminal)
    let stdout_filter = EnvFilter::new("warn")
        .add_directive("walk_forward_smart_seed=info".parse().unwrap())
        .add_directive("bot_engine::execution::paper_trader=info".parse().unwrap());

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(stdout_filter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(system_layer)
        .with(trades_layer)
        .init();

    Ok(LoggingGuards {
        _system: system_guard,
        _trades: trades_guard,
    })
}

/// Khởi tạo Dual-Logger (Alias tiện dụng)
#[allow(dead_code)]
pub fn init_dual_logging(
    strategy_id: &str,
) -> Result<LoggingGuards, Box<dyn Error>> {
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
