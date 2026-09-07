use bot_engine::models::models::Candle;
use bot_engine::controller::asset_actor::LivePositionState;
use bot_engine::optimizer::helios_bridge::HeliosBridge;
use helios::strategy::core_v3::HeliosStrategyV3;
use ferro_trade::broker::actions::Action;
use ferro_trade::strategy::Strategy;

fn get_mock_dna() -> Vec<f64> {
    let mut dna = vec![
        16.94981430500036, 89.99185660556986, 1.6474193378132775, 22.4136564984559, 1.979772843296138, 
        21.851126983081716, 1.0072089922569456, 40.27267798225382, 4.556300552836553, 1.4255024962373073, 
        18.653609114169797, 1.6926764200255664, 3.263443292758067, 4.350059176742498, 27.929938258727386, 
        57.176459051500714, 9.762082927143151, 2.259529557173139, 46.49764694364555, 9.137396536075496, 
        15.248650810893569, 11.245502920272878, 8.545712331306813, 0.4405899692094199, 2.7774418711944535, 
        25.3959070970525, 2.1209151137756943, 34.96178602021011, 2.4762165476243303, 33.18666272418943, 
        3.2295631506731466, 108.9749104010719, 2.9993064178463693, 38.01796250712025, 3.5477547742730637, 
        95.84582696064469, 444.82688588363345, 194.62409039297287, 208.13244436181702, 116.55074712350252, 
        2.9951198440438707, 0.10458831460061023, 0.059612760617711626, 0.2773795586995403, 40.80944386712355, 
        49.54198811450006, 253.30732295223876, 0.16194340282535413, 0.3300761394257865, 0.3047169657589851, 
        26.256821563736167, 63.08207530572044, 0.6612262066411435, 0.10719375196381833, 0.8631587329045474, 
        8.49278637462941, 0.6313309881176387, 0.00845470187083286, 0.8490611471159928, 0.7056679209608798, 
        1.285608805681743, 66.98446231325136, 1.261813212987032, 0.35012827651680223, 3.982629902880836, 
        2.034202630696551, 0.23546606614313692, 2.081978983308039, 1.1109100877384077, 0.8609302974401756, 
        0.38604798874315316, 0.017747556670386997, 0.22433728089805755, 12.931034913411454, 0.673152388208074, 
        0.9529349727684482, 0.8503218815822065, 2.4713181709152536, 1.9437453730870462, 0.10820422256888475, 
        0.8693500634248618, 0.04307000111085305, 0.20970231089932895, 1.8965023482313932, 0.9091006172946998, 
        0.15199776702407208, 8.793038449431222, 0.8245992406573691, 0.8772090195450687, 0.12270400109753574, 
        14.180831893328376, 0.8272381705945112, 0.9893645681919943, 9.982738071059606, 0.10586240950713378, 
        0.008016710224480732, 0.13449062461962405, 2.7271336926079695, 0.5565766241418625, 1.9581888828635465, 
        11.444362629560356, 1.3593660758151418, 2.171886001646615, 0.8572034014150929, 2.989007664083326, 
        1.4986795177417993, 1.6103779147552029, 1.0413232179387046, 1.1288190630322874, 3.6106382749474952
    ];
    if dna.len() < 250 {
        dna.resize(250, 1.0);
    }
    dna
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
    };

    // 3. Khởi tạo Helios Strategy V3 với DNA
    let mut strategy = HeliosStrategyV3::default();
    let dna = get_mock_dna();
    strategy.update_from_dna(&dna);

    // 4. Đánh giá nến hiện tại
    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0);

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
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0);

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
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0);

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
    };

    let mut strategy = HeliosStrategyV3::default();
    strategy.update_from_dna(&get_mock_dna());

    let actions = HeliosBridge::evaluate_live_tick("ETHUSDT", &candles, 0, Some(&live_pos), &mut strategy, 1000.0);

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
