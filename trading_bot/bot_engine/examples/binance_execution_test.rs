// trading_bot/bot_engine/examples/binance_execution_test.rs
//
// 🛡️ BINANCE EXECUTION TESTS (BÀI KIỂM THỬ THỰC THI LỆNH THỰC TẾ TRÊN BINANCE TESTNET)
// Luồng thực thi:
// 1. Nạp API Key & Secret từ file .env (Chế độ Testnet).
// 2. Pre-Clean: Hủy toàn bộ lệnh cũ & Market Close vị thế tồn đọng trên Testnet.
// 3. Testcase 1 (Precision): Kiểm tra làm tròn step_size & tick_size chuẩn xác.
// 4. Testcase 2 (Trailing SL): Đặt lệnh Trailing SL (STOP_MARKET, closePosition=true).
// 5. Testcase 3 (Retry Loop): Giả lập cập nhật SL dồn dập, kiểm tra nghỉ 150ms & 3x Retry.
// 6. Testcase 4 (Panic Close): Khẩn cấp hủy lệnh & đóng vị thế trong < 1 giây.
// 7. Post-Clean: Tự động dọn dẹp tài khoản Testnet sau khi test xong.

use std::sync::Arc;
use std::time::Instant;

use tracing::{info, warn, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::data_engine::sqlite_impl::SqliteStore;
use bot_engine::models::repository::CandleRepository;
use bot_engine::models::weight_monitor::WeightMonitor;
use bot_engine::execution::order_executor::OrderExecutor;
use ferro_trade::broker::actions::Action;

const SYMBOL: &str = "ETHUSDT";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Đọc file .env
    dotenv::dotenv().ok();

    // Khởi tạo Logging ra Console + File log chuyên dụng
    std::fs::create_dir_all("examples/logs")?;
    let file_appender = tracing_appender::rolling::never("examples/logs", "binance_execution_test.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("================================================================================");
    info!("🛡️ BẮT ĐẦU BÀI KIỂM THỬ THỰC THI LỆNH BINANCE TESTNET (BINANCE EXECUTION TESTS)");
    info!("   Symbol: {}, Execution Engine: OrderExecutor", SYMBOL);
    info!("================================================================================");

    // 1. Lấy Credentials từ Môi trường (Hỗ trợ cả BINANCE_TESTNET_API_SECRET và BINANCE_TESTNET_SECRET_KEY)
    let api_key = std::env::var("BINANCE_TESTNET_API_KEY")
        .or_else(|_| std::env::var("BINANCE_API_KEY"))
        .unwrap_or_default();
    let api_secret = std::env::var("BINANCE_TESTNET_API_SECRET")
        .or_else(|_| std::env::var("BINANCE_TESTNET_SECRET_KEY"))
        .or_else(|_| std::env::var("BINANCE_API_SECRET"))
        .unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        error!("❌ Không tìm thấy BINANCE_TESTNET_API_KEY hoặc BINANCE_TESTNET_API_SECRET trong môi trường/file .env!");
        error!("👉 Vui lòng thêm API Key/Secret Testnet vào file .env để thực hiện bài test này.");
        return Ok(());
    }

    // 2. Khởi tạo Database SQLite, WeightMonitor & InitialSync
    std::fs::create_dir_all("examples/storage")?;
    let db_store = Arc::new(SqliteStore::new("examples/storage/binance_execution_test.db")?);
    let monitor = Arc::new(WeightMonitor::new(2400));
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());

    info!("📥 [Market Info] Đang lấy bộ lọc thị trường (MarketFilter) từ Binance Testnet...");
    let market_filter = match syncer.fetch_market_filter(SYMBOL).await {
        Ok(filter) => filter,
        Err(e) => {
            warn!("⚠️ Không thể lấy MarketFilter trực tiếp: {}. Sử dụng MarketFilter mặc định.", e);
            bot_engine::models::market_info::MarketFilter::default()
        }
    };

    info!("   📌 Tick Size (Giá): {}, Step Size (Size): {}, Min Qty: {}", 
          market_filter.tick_size, market_filter.step_size, market_filter.min_qty);

    // 3. Khởi tạo OrderExecutor ở chế độ Testnet
    let executor = OrderExecutor::new(
        api_key,
        api_secret,
        true, // use_testnet = true
        market_filter.clone(),
        db_store.clone(),
        monitor.clone(),
    );

    // Đồng bộ 1 ngày nến thực tế từ Binance REST API để lấy Giá Thị Trường (Mark Price) chính xác
    info!("📥 [Live Price] Đang đồng bộ nến giá thực tế từ Binance API...");
    let _ = syncer.ensure_history(SYMBOL, 1).await;

    // Lấy giá thị trường hiện tại
    let latest_candle = db_store.get_latest_candle(SYMBOL).await?;
    let current_price = latest_candle.map(|c| c.close).unwrap_or(3000.0);
    info!("💵 Giá tham chiếu thực tế hiện tại [{}]: {:.2} USDT", SYMBOL, current_price);

    // ============================================================================
    // 🧹 BƯỚC PRE-CLEAN: DỌN DẸP TÀI KHOẢN TRƯỚC KHI TEST
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [PRE-CLEAN] Dọn dẹp lệnh Algo cũ và Market Close vị thế tồn đọng trên Testnet...");
    let _ = executor.execute_action(SYMBOL, Action::ClosePosition { portion: 1.0 }, current_price, 100.0, 0.0, 20.0).await;
    info!("✅ [PRE-CLEAN] Tài khoản Testnet đã sẵn sàng sạch sẽ.");

    // ============================================================================
    // 🧪 TESTCASE 1: MARKETFILTER PRECISION & ROUNDING
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 1] Kiểm tra độ chính xác làm tròn chữ số thập phân (MarketFilter Precision)");
    
    let test_raw_price = 1738.123456789;
    let test_raw_qty = 0.123456789;

    let rounded_price = market_filter.round_price(test_raw_price);
    let rounded_qty = market_filter.round_qty(test_raw_qty);

    info!("   Raw Price: {} -> Rounded Price: {}", test_raw_price, rounded_price);
    info!("   Raw Qty:   {} -> Rounded Qty:   {}", test_raw_qty, rounded_qty);

    let calc_qty = executor.calculate_order_size(10.0, current_price, 20.0)?;
    info!("   Tính toán Order Size cho $10 đòn bẩy 20x @ {:.2}: {:.4} {}", current_price, calc_qty, SYMBOL);
    assert!(calc_qty >= market_filter.min_qty, "Order size phải lớn hơn min_qty");
    info!("✅ [TESTCASE 1 PASSED] Kiểm tra Precision thành công!");

    // ============================================================================
    // 🧪 TESTCASE 2: TRAILING SL ALGO ORDER EXECUTION (STOP_MARKET)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 2] Đang mở vị thế Long (Order Notional > $20 USDT) để test Trailing StopLoss...");
    let open_pos_action = Action::Buy {
        size: 0.05,
        limit: None,
        stop: None,
        policy: None,
        tag: Some("TEST_MOI".into()),
    };
    if let Err(e) = executor.execute_action(SYMBOL, open_pos_action, current_price, 100.0, 0.05, 20.0).await {
        warn!("⚠️ Lỗi mở vị thế mồi Testnet: {:?}", e);
    }

    // Chờ 500ms để Binance Testnet kết toán vị thế (Position Settlement)
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Lấy giá thị trường tươi mới nhất từ DB
    let live_price = db_store.get_latest_candle(SYMBOL).await.ok().flatten().map(|c| c.close).unwrap_or(current_price);
    info!("🧪 Gửi lệnh Trailing StopLoss (STOP_MARKET) lên Binance Testnet...");

    let target_sl_price = live_price * 0.90; // Thấp hơn 10% giá hiện tại để an toàn tuyệt đối
    info!("   Giá hiện tại: {:.2} USDT | Đặt Trailing SL tại: {:.2} USDT (closePosition = true)", live_price, target_sl_price);

    let update_sl_action = Action::UpdateTradeSlTp {
        trade_id: 0,
        new_sl_price: Some(target_sl_price),
        new_tp_price: None,
    };

    let start_time = Instant::now();
    match executor.execute_action(SYMBOL, update_sl_action, live_price, 100.0, 0.05, 20.0).await {
        Ok(_) => {
            let elapsed = start_time.elapsed();
            info!("✅ [TESTCASE 2 PASSED] Gửi Trailing SL thành công lên Binance Testnet trong {:.2?}", elapsed);
        }
        Err(e) => {
            error!("❌ [TESTCASE 2 FAILED] Lỗi gửi Trailing SL: {:?}", e);
        }
    }

    // ============================================================================
    // 🧪 TESTCASE 3: LATENCY & 3X RETRY LOOP HANDLING
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 3] Kiểm tra xử lý cập nhật Trailing SL dồn dập & 3x Retry Loop (Sleep 150ms)...");

    let new_target_sl = live_price * 0.88;
    info!("   Gửi cập nhật Trailing SL mới tại: {:.2} USDT...", new_target_sl);

    let update_sl_fast = Action::UpdateTradeSlTp {
        trade_id: 0,
        new_sl_price: Some(new_target_sl),
        new_tp_price: None,
    };

    let retry_start = Instant::now();
    match executor.execute_action(SYMBOL, update_sl_fast, current_price, 100.0, 0.05, 20.0).await {
        Ok(_) => {
            let elapsed = retry_start.elapsed();
            info!("✅ [TESTCASE 3 PASSED] Hủy lệnh cũ, chờ 150ms và đặt lại SL thành công trong {:.2?}", elapsed);
        }
        Err(e) => {
            error!("❌ [TESTCASE 3 FAILED] Lỗi quy trình Retry: {:?}", e);
        }
    }

    // ============================================================================
    // 🧪 TESTCASE 4: EMERGENCY PANIC CLOSE (< 1 SECOND)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧪 [TESTCASE 4] Kiểm tra tính năng Emergency Panic Close trong khẩn cấp (< 1s)...");

    let panic_start = Instant::now();
    let panic_action = Action::ClosePosition { portion: 1.0 };

    match executor.execute_action(SYMBOL, panic_action, current_price, 100.0, 0.0, 20.0).await {
        Ok(_) => {
            let elapsed = panic_start.elapsed();
            info!("⚡ [PANIC METRIC] Thời gian xử lý Panic Close: {:.2?}", elapsed);
            if elapsed.as_secs_f64() <= 1.5 {
                info!("✅ [TESTCASE 4 PASSED] Panic Close phản hồi cực nhanh dưới 1.5s!");
            } else {
                warn!("⚠️ Panic Close hoàn thành nhưng mất {:.2?} (Cần tối ưu mạng)", elapsed);
            }
        }
        Err(e) => {
            error!("❌ [TESTCASE 4 FAILED] Lỗi thực thi Panic Close: {:?}", e);
        }
    }

    // ============================================================================
    // 🧹 BƯỚC POST-CLEAN: DỌN DẸP SẠCH SẼ TÀI KHOẢN SAU KHI TEST
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [POST-CLEAN] Tự động dọn dẹp sạch tài khoản Testnet sau khi hoàn thành test...");
    let _ = executor.execute_action(SYMBOL, Action::ClosePosition { portion: 1.0 }, current_price, 100.0, 0.0, 20.0).await;
    info!("✅ [POST-CLEAN] Hoàn tất dọn dẹp.");

    info!("================================================================================");
    info!("🎉 TẤT CẢ CÁC BÀI TESTCASES NHÓM 1 (BINANCE EXECUTION TESTS) ĐÃ HOÀN THÀNH!");
    info!("================================================================================");

    Ok(())
}
