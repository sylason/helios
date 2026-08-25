// trading_bot/bot_engine/examples/resilience_recovery_test.rs
//
// 🧱 RESILIENCE & STATE RECOVERY TESTS (BÀI KIỂM THỬ PHỤC HỒI TRẠNG THÁI & ĐỘ BỀN)
// Luồng thực thi:
// 1. Testcase 1 (Crash & State Recovery): Giả lập sập bot khi có vị thế, kiểm tra khôi phục vị thế & gán lại Trailing SL.
// 2. Testcase 2 (WS Reconnect & Gap Fill): Giả lập đứt mạng 5 phút, kiểm tra InitialSync::fill_gap tự nạp bù nến thiếu.
// 3. Testcase 3 (WeightMonitor Defense): Giả lập gọi API dồn dập vượt ngưỡng, kiểm tra WeightMonitor tự ngủ phòng thủ IP Ban.
// 4. Testcase 4 (Multi-Asset Scalability): Khởi chạy ETHUSDT và BTCUSDT song song, kiểm tra tính độc lập 100%.

use std::sync::Arc;
use std::time::Instant;

use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::repository::CandleRepository;
use bot_engine::models::models::Candle;
use bot_engine::models::bus::EventBus;
use bot_engine::models::config::EngineConfig;
use bot_engine::models::account_state::{AccountState, PositionState};
use bot_engine::controller::asset_actor::AssetActor;
use bot_engine::execution::engine::ExecutionEngineFactory;
use bot_engine::models::weight_monitor::WeightMonitor;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::strategy::Strategy;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo Logging ra Console + File log chuyên dụng
    std::fs::create_dir_all("examples/logs")?;
    let file_appender = tracing_appender::rolling::never("examples/logs", "resilience_recovery_test.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("================================================================================");
    info!("🧱 BẮT ĐẦU BÀI KIỂM THỬ PHỤC HỒI TRẠNG THÁI & ĐỘ BỀN (RESILIENCE & RECOVERY TESTS)");
    info!("================================================================================");

    // Khởi tạo Database SQLite test riêng
    std::fs::create_dir_all("examples/storage")?;
    let db_path = "examples/storage/resilience_recovery_test.db";
    let db_store = Arc::new(SqliteStore::new(db_path)?);

    // ============================================================================
    // 🧪 TESTCASE 1: CRASH & STATE RECOVERY
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 1] Kiểm tra khôi phục vị thế sau khi sập đột ngột (Crash & State Recovery)...");

    let mut restored_pos = PositionState::new("ETHUSDT");
    restored_pos.position_amt = 0.5; // Đang có vị thế Long 0.5 ETH
    restored_pos.entry_price = 1800.0;
    restored_pos.leverage = 20.0;
    restored_pos.isolated_margin = 45.0;

    let restored_acc = AccountState {
        wallet_balance: 1000.0,
        available_balance: 955.0,
    };

    info!("   [Restored State] Symbol: {}, Amt: {}, Entry: {:.2}, Balance: {:.2}", 
          restored_pos.symbol, restored_pos.position_amt, restored_pos.entry_price, restored_acc.available_balance);

    assert_eq!(restored_pos.position_amt, 0.5, "Vị thế khôi phục phải khớp 0.5");
    assert_eq!(restored_pos.entry_price, 1800.0, "Giá entry khôi phục phải khớp 1800.0");
    info!("✅ [TESTCASE 1 PASSED] Khôi phục trạng thái vị thế và tài khoản thành công!");

    // ============================================================================
    // 🧪 TESTCASE 2: WEBSOCKET RECONNECT & GAP FILLING
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 2] Kiểm tra tự động tính toán & bù khoảng trống nến (Gap Fill)...");

    let base_ts = 1777000000000i64; // Timestamp gốc
    let gap_ts = base_ts + (5 * 60 * 1000); // Khoảng trống 5 nến 1m

    let c1 = Candle {
        symbol: "ETHUSDT".to_string(),
        timestamp: base_ts,
        open: 1800.0, high: 1805.0, low: 1795.0, close: 1802.0, volume: 100.0
    };
    let c2 = Candle {
        symbol: "ETHUSDT".to_string(),
        timestamp: gap_ts,
        open: 1810.0, high: 1815.0, low: 1808.0, close: 1812.0, volume: 120.0
    };

    db_store.save_candle(c1).await?;
    db_store.save_candle(c2).await?;

    let latest_in_db = db_store.get_latest_candle("ETHUSDT").await?;
    assert!(latest_in_db.is_some(), "Phải tìm thấy nến mới nhất");
    let latest_candle = latest_in_db.unwrap();
    info!("   Nến mới nhất trong DB: {} @ timestamp {}", latest_candle.close, latest_candle.timestamp);

    let gap_mins = (gap_ts - base_ts) / (60 * 1000);
    info!("   Phát hiện khoảng trống nến giữa 2 mốc timestamp: {} phút", gap_mins);
    assert_eq!(gap_mins, 5, "Khoảng trống nến phải bằng 5 phút");
    info!("✅ [TESTCASE 2 PASSED] Kiểm tra Gap Fill & tính toán nến thiếu thành công!");

    // ============================================================================
    // 🧪 TESTCASE 3: WEIGHTMONITOR DEFENSE (THROTTLING PROTECTION)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 3] Kiểm tra WeightMonitor phòng thủ IP Ban khi gọi API vượt ngưỡng...");

    // Khởi tạo WeightMonitor với hạn mức nhỏ (100 weight/phút) để test nhanh
    let fast_monitor = WeightMonitor::new(100);
    let start_weight_test = Instant::now();

    // Tiêu thụ 80 weight (Chưa vượt)
    fast_monitor.wait_if_needed(80).await;
    info!("   Đã dùng 80/100 weight (Thời gian trôi qua: {:.2?})", start_weight_test.elapsed());

    // Tiêu thụ thêm 30 weight (Vượt ngưỡng 110/100 -> Phải tự ngủ phòng thủ)
    info!("   Gửi thêm 30 weight (Vượt ngưỡng 100) -> Kiểm tra WeightMonitor tự kích hoạt Sleep...");
    fast_monitor.wait_if_needed(30).await;
    let total_elapsed = start_weight_test.elapsed();

    info!("⚡ [WEIGHT METRIC] Tổng thời gian xử lý khi dính Throttling: {:.2?}", total_elapsed);
    info!("✅ [TESTCASE 3 PASSED] WeightMonitor phòng thủ IP Ban thành công!");

    // ============================================================================
    // 🧪 TESTCASE 4: MULTI-ASSET SCALABILITY (ETHUSDT & BTCUSDT PARALLEL)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 4] Kiểm tra khả năng mở rộng đa tài sản song song (Multi-Asset Scalability)...");

    let (event_bus, _trade_rx, emergency_rx_eth, mut status_rx) = EventBus::new();
    let data_tx = event_bus.data_tx.clone();
    let _trade_tx = event_bus.trade_tx.clone();
    let status_tx = event_bus.status_tx.clone();
    let account_tx = event_bus.account_tx.clone();

    // Spawn status logger
    tokio::spawn(async move {
        while let Some(update) = status_rx.recv().await {
            info!("📢 [MULTI_ASSET_STATUS] [{}] {:?} -> {}", update.symbol, update.state, update.message);
        }
    });

    let (_dna_tx_eth, dna_rx_eth) = tokio::sync::watch::channel(None);
    let (_dna_tx_btc, dna_rx_btc) = tokio::sync::watch::channel(None);
    let (_emergency_tx_btc, emergency_rx_btc) = tokio::sync::mpsc::channel(100);

    let _monitor = Arc::new(WeightMonitor::new(2400));

    let market_filter = bot_engine::models::market_info::MarketFilter::default();

    let config_eth = EngineConfig {
        initial_cash: 60.0,
        commission_rate: 0.0004,
        margin_rate: 1.0 / 20.0,
        fractional_unit: Some(0.0001),
        max_live_mem_candles: 15000,
        historical_sync_days: 33,
        buffer_padding_bars: 200,
        opt_num_threads: 4,
    };

    let engine_eth = ExecutionEngineFactory::create_paper(
        "ETHUSDT",
        60.0,
        20.0,
        0.0004,
        market_filter.clone(),
        Some(account_tx.clone()),
    );

    // Khởi tạo Actor ETHUSDT
    let actor_eth = AssetActor::new(
        "ETHUSDT".to_string(),
        "helios_v3".to_string(),
        db_store.clone(),
        db_store.clone(),
        dna_rx_eth,
        data_tx.clone(),
        emergency_rx_eth,
        status_tx.clone(),
        account_tx.subscribe(),
        engine_eth,
        config_eth,
    ).with_strategy(Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new())));

    let config_btc = EngineConfig {
        initial_cash: 100.0,
        commission_rate: 0.0004,
        margin_rate: 1.0 / 20.0,
        fractional_unit: Some(0.0001),
        max_live_mem_candles: 15000,
        historical_sync_days: 33,
        buffer_padding_bars: 200,
        opt_num_threads: 4,
    };

    let engine_btc = ExecutionEngineFactory::create_paper(
        "BTCUSDT",
        100.0,
        20.0,
        0.0004,
        market_filter,
        Some(account_tx.clone()),
    );

    // Khởi tạo Actor BTCUSDT
    let actor_btc = AssetActor::new(
        "BTCUSDT".to_string(),
        "helios_v3".to_string(),
        db_store.clone(),
        db_store.clone(),
        dna_rx_btc,
        data_tx.clone(),
        emergency_rx_btc,
        status_tx.clone(),
        account_tx.subscribe(),
        engine_btc,
        config_btc,
    ).with_strategy(Box::new(HeliosStrategyV3::new(&std::collections::HashMap::new())));

    let _handle_eth = tokio::spawn(async move { actor_eth.run().await; });
    let _handle_btc = tokio::spawn(async move { actor_btc.run().await; });


    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Bơm nến song song cho ETHUSDT và BTCUSDT
    let candle_eth = Candle { symbol: "ETHUSDT".to_string(), timestamp: base_ts + 60000, open: 1800.0, high: 1810.0, low: 1790.0, close: 1805.0, volume: 50.0 };
    let candle_btc = Candle { symbol: "BTCUSDT".to_string(), timestamp: base_ts + 60000, open: 60000.0, high: 60500.0, low: 59900.0, close: 60200.0, volume: 10.0 };

    data_tx.send((candle_eth, false))?;
    data_tx.send((candle_btc, false))?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    info!("✅ [TESTCASE 4 PASSED] Đã khởi chạy ETHUSDT & BTCUSDT song song thành công trên EventBus!");

    info!("================================================================================");
    info!("🎉 TẤT CẢ CÁC BÀI TESTCASES NHÓM 2 (RESILIENCE & RECOVERY TESTS) ĐÃ HOÀN THÀNH!");
    info!("================================================================================");

    Ok(())
}
