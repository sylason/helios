# 🔬 Kiểm Chứng Chuyên Sâu: Lỗi Đóng Băng Dữ Liệu MTF (Frozen MTF Data Bug) trong `HeliosBridge`

**Chủ đề:** Bất đồng bộ Resampling Đa Khung Thời Gian giữa Backtest và Live Tick  
**Vị trí liên quan:**  
- [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L221-L243)  
- [`ferro_trade/src/utils/resample.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/utils/resample.rs)  
- [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)  

---

## 📌 1. Triệu Chứng & Hiện Tượng Thực Tế

1. **Trong Single Backtest Eval (In-Sample / Diagnostic):** Lợi nhuận và hiệu suất giao dịch luôn cao ($+200\% \rightarrow +510\%$), hệ thống phát hiện và bắt sóng nhịp nhàng trên các khung thời gian lớn (5m, 15m, 1h, 4h).
2. **Trong Forward Paper Trading:** Tín hiệu vào lệnh bị chậm trễ hàng giờ, hoặc Arbiter đưa ra quyết định sai lệch nghiêm trọng so với khi chạy Backtest trên cùng một khoảng dữ liệu.

---

## 🔬 2. Bằng Chứng Mã Nguồn & Cơ Chế Gây Lỗi

### 🔬 Bằng chứng 1: Cơ chế Caching Resampled OHLCV chỉ kích hoạt khi Đổi Bucket
Trong [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L221-L236):

```rust
for &tf in &HELIOS_REQUIRED_TIMEFRAMES {
    let current_bucket = tf.get_bucket_index(last_candle_ts);
    let prev_bucket = store.tf_buckets.get(&tf.to_string()).copied().unwrap_or(-1);

    // ❌ CHỈ RESAMPLE KHI ĐỔI BUCKET HOẶC CACHE CHƯA CÓ:
    let maybe_flat = if current_bucket != prev_bucket || cache.get_resampled_ohlcv(tf).is_none() {
        if let Ok(df_res) = ferro_trade::utils::resample::resample_ohlcv(tf.as_polars_rule(), base_df) {
            if let Ok(flat_ohlcv) = ferro_trade::utils::resample::df_to_resampled_ohlcv(&df_res) {
                let flat_arc = Arc::new(flat_ohlcv);
                cache.put_resampled_ohlcv(tf, flat_arc.clone());
                store.tf_buckets.insert(tf.to_string(), current_bucket);
                Some(flat_arc)
            } else { None }
        } else { None }
    } else {
        // ❌ NẾU ĐANG TRONG CÙNG 1 BUCKET: TÁI SỬ DỤNG MẢNG OHLCV CŨ TRONG CACHE!
        cache.get_resampled_ohlcv(tf)
    };

    if let Some(flat_arc) = maybe_flat {
        let table = Arc::new(ferro_trade::utils::resample::create_lookup_table(base_times, &flat_arc.datetime));
        cache.put_lookup(tf, table);
    }
}
```

### 🔬 Bằng chứng 2: Tác động "Mù Thông Tin" (Information Blindness) trong 239 phút
1. Đối với khung thời gian **4h** (`Timeframe::T4h`):
   - Một chu kỳ nến 4h kéo dài $240$ cây nến 1m.
   - Tại phút thứ 0 của chu kỳ, `resample_ohlcv` chạy trên `base_df` tại thời điểm đó và lưu `flat_ohlcv` vào `cache`.
   - Trong $239$ phút tiếp theo ($239$ cây nến 1m mới liên tục xuất hiện), vì `current_bucket == prev_bucket`, nhánh `else` được thực thi và trả về `cache.get_resampled_ohlcv(tf)`.
2. Mảng nến 4h trong `cache` **hoàn toàn không được cập nhật bất kỳ biến động giá, khối lượng hay nến mới nào trong suốt 239 phút đó**.
3. Khi `strategy.init(&mut init_context)` được gọi ở mỗi nến:
   - Các chỉ báo như ADX 4h, KAMA 4h, RSI 4h, Bollinger Bands 4h được tính toán trên **chuỗi dữ liệu bị đông cứng từ nhiều giờ trước**.
   - Arbiter và các Scanners (Trend PB, Momentum Ignition, Reversal) đưa ra quyết định giao dịch dựa trên bức tranh thị trường đã lỗi thời $2-4$ tiếng.
4. **Sự sai lệch so với Backtest:**
   - Trong Backtest tĩnh, toàn bộ mảng nến được resample đầy đủ, các điểm cắt nến được chốt chuẩn xác theo dòng thời gian.
   - Live Tick bị lệch hoàn toàn về mặt tín hiệu so với Backtest.

---

## 🛠️ 3. Đề Xuất Giải Pháp Kiến Trúc

1. **Streaming Resampling Cập Nhật Cây Nến Đang Hình Thành (Forming Candle Update):**
   - Khi có nến 1m mới trong cùng bucket, thay vì bỏ qua, ta cập nhật giá `High`, `Low`, `Close`, `Volume` của cây nến cuối cùng trong mảng `flat_ohlcv` tương ứng với khung lớn mà không cần phải gọi lại toàn bộ hàm `resample_ohlcv` của Polars từ đầu.
2. **Triệt tiêu Cache Polars Overhead:**
   - Sử dụng mô hình `StreamManager` (theo đúng thiết kế TDD gốc trong `STREAMING_INDICATOR_SYNC.md`) để cập nhật gia tăng $O(1)$ cho các chỉ báo đa khung thời gian.
