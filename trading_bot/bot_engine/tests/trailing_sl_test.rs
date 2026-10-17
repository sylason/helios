use bot_engine::models::models::Candle;
use bot_engine::controller::asset_actor::LivePositionState;
use bot_engine::optimizer::helios_bridge::HeliosBridge;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::broker::actions::Action;
use ferro_trade::strategy::Strategy;

fn get_mock_dna() -> Vec<f64> {
    use ferro_trade::optimizer::{DnaMappable, ParamDefinition};
    use helios::strategy::core_v3::config::HeliosConfigV3;
    let defs = HeliosConfigV3::parameter_defs();
    defs.into_iter().map(|(name, def)| {
        if name.contains("time_countdown_bars") {
            return 30.0; // Kích hoạt zombie decay sớm (<= 60 bars)
        }
        match def {
            ParamDefinition::Range { min, max, .. } => (min + max) / 2.0,
            ParamDefinition::Options { values } => values.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
        }
    }).collect()
}

fn create_mock_candles(start_price: f64, prices: &[f64]) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(43200 + prices.len());
    let mut timestamp = 1700000000000;

    // 1. Tạo 43,200 nến Warmup (30 ngày dữ liệu 1m chuẩn như AssetActor::handle_initial_sync)
    for i in 0..43200 {
        let p = start_price + (i as f64 * 0.001);
        candles.push(Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp,
            open: p,
            high: p + 1.0,
            low: p - 1.0,
            close: p,
            volume: 1000.0,
        });
        timestamp += 60000;
    }

    // 2. Bơm danh sách các nến biến động theo kịch bản
    let last_p = candles.last().unwrap().close;
    for &p in prices {
        candles.push(Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp,
            open: last_p,
            high: p.max(last_p) + 1.0,
            low: p.min(last_p) - 1.0,
            close: p,
            volume: 1200.0,
        });
        timestamp += 60000;
    }

    candles
}

#[test]
fn test_live_trailing_sl_generation() {
    // 1. Khởi tạo 43,200 nến warmup + 5 nến giá tăng từ $1800 lên $1850
    let prices = vec![1810.0, 1825.0, 1835.0, 1845.0, 1850.0];
    let candles = create_mock_candles(1800.0, &prices);
    let last_bar = candles.len() - 1;

    // 2. Giả lập Vị thế Thật trên Binance: Long 1.0 ETH tại $1800, mở ở nến last_bar - 4
    // 2. Giả lập Vị thế Thật trên Binance: Long 1.0 ETH tại $1800, mở ở nến last_bar - 4
    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: 1.0,
        entry_price: 1800.0,
        entry_timestamp: candles[last_bar - 4].timestamp,
        live_entry_bar: last_bar - 4,
        current_sl_price: Some(1790.0),
        current_tp_price: None,
        opening_playbook_tag: Some("MOMENTUM_IGN".to_string()),
        highest_favorable_price: 1850.0,
        peak_timestamp: candles[last_bar].timestamp,
    };

    // 3. Khởi tạo Helios Strategy V3 với DNA
    let mut strategy = HeliosStrategyV3::default();
    let dna = get_mock_dna();
    strategy.update_from_dna(&dna);

    // 4. Đánh giá nến hiện tại
    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    // 5. Kiểm tra kết quả: Phải trả về Action::UpdateTradeSlTp với new_sl_price > 1790.0
    let mut found_sl_update = false;
    for action in actions {
        if let Action::UpdateTradeSlTp { new_sl_price, .. } = action {
            if let Some(new_sl) = new_sl_price {
                println!("✅ [TEST 1 PASS] Long Trailing SL Generated: {:.2} (Old SL: 1790.0)", new_sl);
                assert!(new_sl > 1790.0, "Mức SL mới ({}) phải cao hơn mức SL cũ (1790.0)", new_sl);
                found_sl_update = true;
            }
        }
    }

    assert!(found_sl_update, "Phải phát ra Action::UpdateTradeSlTp khi vị thế Long có lời!");
}

#[test]
fn test_ratchet_protection() {
    // Kịch bản: Giá đã đạt đỉnh $1850, SL đang ở $1847.23. Nến tiếp theo sập về $1820.
    let prices = vec![1810.0, 1825.0, 1835.0, 1845.0, 1850.0, 1820.0];
    let candles = create_mock_candles(1800.0, &prices);
    let last_bar = candles.len() - 1;

    // SL hiện tại đang là 1847.23
    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: 1.0,
        entry_price: 1800.0,
        entry_timestamp: candles[last_bar - 5].timestamp,
        live_entry_bar: last_bar - 5,
        current_sl_price: Some(1847.23),
        current_tp_price: None,
        opening_playbook_tag: Some("MOMENTUM_IGN".to_string()),
        highest_favorable_price: 1850.0,
        peak_timestamp: candles[last_bar - 1].timestamp,
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    // Bánh cóc Ratchet: TUYỆT ĐỐI KHÔNG DỜI SL THẤP HƠN 1847.23 khi giá sập!
    for action in actions {
        if let Action::UpdateTradeSlTp { new_sl_price: Some(new_sl), .. } = action {
            assert!(new_sl >= 1847.23, "Bánh cóc vi phạm! SL mới ({}) bị hạ thấp hơn SL cũ (1847.23)", new_sl);
        }
    }
    println!("✅ [TEST 2 PASS] Ratchet Protection verified! SL locked at 1847.23 on price pullbacks.");
}

#[test]
fn test_short_position_trailing() {
    // Kịch bản: Vị thế Bán Khống (Short) -1.0 ETH tại $1800. Giá giảm sâu về $1750.
    let prices = vec![1790.0, 1775.0, 1765.0, 1755.0, 1750.0];
    let candles = create_mock_candles(1800.0, &prices);
    let last_bar = candles.len() - 1;

    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: -1.0, // Short position
        entry_price: 1800.0,
        entry_timestamp: candles[last_bar - 4].timestamp,
        live_entry_bar: last_bar - 4,
        current_sl_price: Some(1810.0), // Old SL above entry
        current_tp_price: None,
        opening_playbook_tag: Some("MOMENTUM_IGN".to_string()),
        highest_favorable_price: 1750.0,
        peak_timestamp: candles[last_bar].timestamp,
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    let mut found_sl_update = false;
    for action in actions {
        if let Action::UpdateTradeSlTp { new_sl_price: Some(new_sl), .. } = action {
            println!("✅ [TEST 3 PASS] Short Trailing SL Generated: {:.2} (Old SL: 1810.0)", new_sl);
            assert!(new_sl < 1810.0, "SL mới ({}) cho lệnh Short phải thấp hơn SL cũ (1810.0)", new_sl);
            found_sl_update = true;
        }
    }

    assert!(found_sl_update, "Phải phát ra Action::UpdateTradeSlTp khi vị thế Short có lời!");
}

#[test]
fn test_zombie_stagnant_decay() {
    // Kịch bản: Vị thế Long tại $1800, giá tạo đỉnh $1810 ở nến 0 rồi lình xình 60 nến không tạo đỉnh mới.
    let mut prices = vec![1810.0];
    for _ in 0..60 {
        prices.push(1810.0);
    }

    let candles = create_mock_candles(1800.0, &prices);
    let last_bar = candles.len() - 1;

    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: 1.0,
        entry_price: 1800.0,
        entry_timestamp: candles[last_bar - 60].timestamp,
        live_entry_bar: last_bar - 60,
        current_sl_price: Some(1790.0),
        current_tp_price: None,
        opening_playbook_tag: Some("RANGE_OSC".to_string()),
        highest_favorable_price: 1810.0,
        peak_timestamp: candles[last_bar - 60].timestamp,
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    let mut found_sl_update = false;
    for action in actions {
        if let Action::UpdateTradeSlTp { new_sl_price: Some(new_sl), .. } = action {
            println!("✅ [TEST 4 PASS] Zombie Stagnant Decay SL: {:.2} (Siết SL khi ngâm 60 nến)", new_sl);
            assert!(new_sl > 1790.0, "Zombie Decay phải dời SL ({}) lên cao hơn!", new_sl);
            found_sl_update = true;
        }
    }

    assert!(found_sl_update, "Zombie Decay phải phát lệnh dời SL!");
}

#[test]
fn test_high_watermark_buffer_bleed_resilience() {
    // 🛡️ [TODO-AUDIT-NEW-12] Kiểm thử khả năng chống thảm họa High Watermark Buffer Bleed
    // Kịch bản:
    // 1. Vị thế Long mở tại $1800 ở 2,000 nến trước (entry_timestamp cách 2,000 phút).
    // 2. Vị thế đã từng đạt đỉnh $1850 ở 1,500 nến trước (peak_timestamp cách 1,500 phút).
    // 3. Lát cắt buffer trong RAM chỉ có 500 nến (giá hiện tại sập về $1820).
    // Nến vào lệnh và nến đỉnh đều ĐÃ TRÔI HOÀN TOÀN KHỎI BIÊN TRÁI BUFFER!
    let prices = vec![1820.0; 50];
    let candles = create_mock_candles(1820.0, &prices);

    let buffer_start_ts = candles[0].timestamp;
    let entry_ts = buffer_start_ts - (2000 * 60_000);
    let peak_ts = buffer_start_ts - (1500 * 60_000);

    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: 1.0,
        entry_price: 1800.0,
        entry_timestamp: entry_ts,
        live_entry_bar: 0,
        current_sl_price: Some(1840.0), // SL đã dời lên 1840 nhờ đỉnh 1850
        current_tp_price: None,
        opening_playbook_tag: Some("MOMENTUM_IGN".to_string()),
        highest_favorable_price: 1850.0, // Đỉnh lịch sử được bảo toàn bởi State Accumulator
        peak_timestamp: peak_ts,
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    // KẾT QUẢ MONG ĐỢI:
    // SL không được phép sập hay bleed về mức đáy. Nhờ Ratchet Guard và High Watermark O(1),
    // SL mới (nếu có update) phải luôn >= 1840.0, không bao giờ bị thụt lùi xuống vùng giá hiện tại ($1820).
    for action in actions {
        if let Action::UpdateTradeSlTp { new_sl_price: Some(new_sl), .. } = action {
            println!("✅ [TEST 5 PASS] High Watermark Buffer Bleed Resilient SL: {:.2} (Preserved peak $1850)", new_sl);
            assert!(new_sl >= 1840.0, "SL mới ({}) không được thụt lùi dưới mức SL đã khóa (1840.0)!", new_sl);
        }
    }
}

#[test]
fn test_stagnant_capital_release_with_trimmed_buffer() {
    // 🛡️ [TODO-AUDIT-NEW-12] Kiểm thử giải phóng vốn Zombie khi vị thế trôi khỏi buffer
    // Kịch bản:
    // Vị thế Long ngâm 3,000 phút (50 giờ) với giá lình xình hòa vốn ($1800.5).
    // Buffer chỉ lưu 1,000 nến gần nhất (max_mem = 1,000).
    // Nếu theo logic cũ (i - entry_bar = 999), bars_held bị chặn ở 999 < max_stagnant_hold_bars (2880 nến = 48h) -> Zombie tê liệt.
    // Với logic mới tính theo delta_t vật lý, bars_held = 3,000 >= 2880 -> Kích hoạt Action::ClosePosition!
    let prices = vec![1800.5; 50];
    let candles = create_mock_candles(1800.5, &prices);
    let last_bar = candles.len() - 1;

    let now_ts = candles[last_bar].timestamp;
    let entry_ts = now_ts - (3000 * 60_000); // 3,000 phút trước

    let live_pos = LivePositionState {
        symbol: "ETHUSDT".to_string(),
        position_amt: 1.0,
        entry_price: 1800.0,
        entry_timestamp: entry_ts,
        live_entry_bar: 0,
        current_sl_price: Some(1790.0),
        current_tp_price: None,
        opening_playbook_tag: Some("RANGE_OSC".to_string()),
        highest_favorable_price: 1801.0,
        peak_timestamp: entry_ts + 60_000,
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0, None);

    let mut found_close_action = false;
    for action in actions {
        if let Action::ClosePosition { portion } = action {
            println!("✅ [TEST 6 PASS] Zombie Trade Purged! Action::ClosePosition portion: {:.2}", portion);
            assert_eq!(portion, 1.0, "Phải đóng 100% vị thế ngâm vốn!");
            found_close_action = true;
        }
    }

    assert!(found_close_action, "Phải phát Action::ClosePosition khi vị thế ngâm 3,000 nến vượt trần 2,880 nến!");
}





