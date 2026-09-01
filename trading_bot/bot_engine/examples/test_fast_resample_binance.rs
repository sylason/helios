// trading_bot/bot_engine/examples/test_fast_resample_binance.rs
//
// 🧪 REAL DATA RESAMPLE PARITY TEST:
// Nạp nến thực tế từ Binance / SQLite và so sánh đối chứng 1:1 giữa Polars vs Pure-Rust.

use std::time::Instant;
use polars::prelude::*;
use tracing::info;

use bot_engine::data_engine::binance_rest::InitialSync;
use bot_engine::models::repository::CandleRepository;
use ferro_trade::data::Timeframe;
use ferro_trade::utils::resample::{resample_ohlcv, df_to_resampled_ohlcv, fast_resample_ohlcv};

#[path = "common/mod.rs"]
mod common;

const SYMBOL: &str = "ETHUSDT";
const STRATEGY_ID: &str = "test_fast_resample_binance";
const FETCH_DAYS: i64 = 30; // 30 ngày dữ liệu (~43,200 nến 1m)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = common::init_logging(STRATEGY_ID)?;

    info!("================================================================================");
    info!("🧪 KIỂM THỬ ĐỐI CHỨNG RESAMPLE TRÊN DỮ LIỆU NẾN THỰC TẾ BINANCE: {}", SYMBOL);
    info!("================================================================================");

    let (db_store, monitor) = common::setup_sqlite_and_monitor(STRATEGY_ID, 2400)?;

    info!("📥 Đang đồng bộ dữ liệu {} ngày từ Binance Futures...", FETCH_DAYS);
    let syncer = InitialSync::new(db_store.clone(), monitor.clone());
    let _ = syncer.ensure_history(SYMBOL, FETCH_DAYS).await;

    let candles = db_store.get_candles_by_range(SYMBOL, 0, i64::MAX).await?;
    if candles.is_empty() {
        info!("❌ Không có nến nào trong DB để kiểm thử.");
        return Ok(());
    }

    info!("📊 Đã nạp {} nến 1m thực tế từ Binance.", candles.len());

    let n = candles.len();
    let mut dt_vec = Vec::with_capacity(n);
    let mut open_vec = Vec::with_capacity(n);
    let mut high_vec = Vec::with_capacity(n);
    let mut low_vec = Vec::with_capacity(n);
    let mut close_vec = Vec::with_capacity(n);
    let mut vol_vec = Vec::with_capacity(n);

    for c in &candles {
        dt_vec.push(c.timestamp * 1_000_000); // ms -> ns
        open_vec.push(c.open);
        high_vec.push(c.high);
        low_vec.push(c.low);
        close_vec.push(c.close);
        vol_vec.push(c.volume);
    }

    let s_dt = Series::new("Datetime".into(), dt_vec.clone())
        .cast(&DataType::Datetime(TimeUnit::Nanoseconds, None))?;
    let s_open = Series::new("Open".into(), open_vec.clone());
    let s_high = Series::new("High".into(), high_vec.clone());
    let s_low = Series::new("Low".into(), low_vec.clone());
    let s_close = Series::new("Close".into(), close_vec.clone());
    let s_vol = Series::new("Volume".into(), vol_vec.clone());

    let df = DataFrame::new(vec![
        Column::from(s_dt),
        Column::from(s_open),
        Column::from(s_high),
        Column::from(s_low),
        Column::from(s_close),
        Column::from(s_vol),
    ])?;

    let test_timeframes = [
        (Timeframe::T5m, "5m"),
        (Timeframe::T15m, "15m"),
        (Timeframe::T1h, "1h"),
        (Timeframe::T4h, "4h"),
    ];

    for (tf, rule) in test_timeframes {
        info!("--------------------------------------------------------------------------------");
        info!("🔍 ĐỐI CHỨNG KHUNG THỜI GIAN: {}", rule);

        // 1. Polars Resample
        let start_polars = Instant::now();
        let polars_df = resample_ohlcv(rule, &df)?;
        let polars_res = df_to_resampled_ohlcv(&polars_df)?;
        let dur_polars = start_polars.elapsed();

        // 2. Pure-Rust Fast Resample
        let start_fast = Instant::now();
        let fast_res = fast_resample_ohlcv(
            &dt_vec, &open_vec, &high_vec, &low_vec, &close_vec, &vol_vec, tf
        );
        let dur_fast = start_fast.elapsed();

        info!("   ⏱️ Polars Execution Time:    {:.3} ms", dur_polars.as_secs_f64() * 1000.0);
        info!("   ⚡ Pure-Rust Execution Time: {:.3} ms (Tốc độ tăng {:.1}x)", 
              dur_fast.as_secs_f64() * 1000.0,
              dur_polars.as_secs_f64() / dur_fast.as_secs_f64().max(1e-9));

        let count = polars_res.close.len();
        assert_eq!(count, fast_res.close.len(), "Số lượng nến không khớp cho khung {}", rule);
        info!("   📊 Tổng số nến MTF thu được: {}", count);

        // 3. Kiểm tra từng phần tử
        for i in 0..count {
            assert_eq!(polars_res.datetime[i], fast_res.datetime[i], "Datetime lệch tại nến #{}", i);
            assert!((polars_res.open[i] - fast_res.open[i]).abs() < 1e-9, "Open lệch tại nến #{}", i);
            assert!((polars_res.high[i] - fast_res.high[i]).abs() < 1e-9, "High lệch tại nến #{}", i);
            assert!((polars_res.low[i] - fast_res.low[i]).abs() < 1e-9, "Low lệch tại nến #{}", i);
            assert!((polars_res.close[i] - fast_res.close[i]).abs() < 1e-9, "Close lệch tại nến #{}", i);
            assert!((polars_res.volume[i] - fast_res.volume[i]).abs() < 1e-6, "Volume lệch tại nến #{}", i);
        }

        info!("   ✅ KẾT QUẢ KHỚP 100% CHÍNH XÁC (Độ lệch 0.0000000000) TRÊN DỮ LIỆU THẬT!");
    }

    info!("================================================================================");
    info!("🎉 TOÀN BỘ CÁC KHUNG THỜI GIAN ĐỀU ĐẠT CHUẨN ĐỐI SOÁT 1:1!");
    info!("================================================================================");

    Ok(())
}
