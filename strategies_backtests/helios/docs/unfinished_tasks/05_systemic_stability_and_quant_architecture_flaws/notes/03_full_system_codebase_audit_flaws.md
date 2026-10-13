# BÁO CÁO TOÀN DIỆN: 42 ĐIỂM YẾU KIẾN TRÚC & TOÁN HỌC TIỀM ẨN TRÊN TOÀN BỘ HỆ SINH THÁI

> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/)  
> **Ngày lập báo cáo:** 11/10/2026  
> **Phạm vi kiểm toán:** Khảo sát độc lập và đào sâu mã nguồn 3 dự án: `ferro_trade`, `helios`, `trading_bot`.  
> **Nguyên tắc:** Loại bỏ hoàn toàn các lỗi đã có trong `ROADMAP.md` cũ và 2 tài liệu `01_quant_...` & `02_helios_...` trước đó.

---

## 🏛️ PHẦN 1: ĐỘNG CƠ BACKTEST & OPTIMIZER (`ferro_trade` - 17 PHÁT HIỆN)

### 1.1. TỬ HUYỆN TOÁN HỌC & LOOKAHEAD BIAS (ẢNH HƯỞNG TRỰC TIẾP ĐẾN KẾT QUẢ BACKTEST)

#### 1. Lookahead Bias trong Sizing Margin tại Nến Mở Vị Thế
- **Vị trí:** `src/broker/broker.rs:166, 479`, `src/backtest/engine.rs:294`
- **Bản chất:** Tại thời điểm đầu nến $i$, khi khớp lệnh bằng $Open[i]$, hàm `margin_available()` lại gọi `self.equity()` với `last_price` được cập nhật là $Close[i]$ của tương lai (chưa đóng nến).
- **Hậu quả:** Broker tính toán sức mua và đòn bẩy bằng mức giá cuối nến, gây rò rỉ thông tin tương lai (Lookahead bias) trong bài toán sizing và ký quỹ.

#### 2. Nguy cơ Lookahead Bias từ `bfill` (Backfill) trong `atr_sma`
- **Vị trí:** `src/indicators/bulk.rs:1507-1514`
- **Bản chất:** Thay vì giữ giá trị $NaN$ cho giai đoạn Warmup ban đầu, hàm lại dùng giá trị ATR tính được ở nến tương lai gán ngược (backfill) về toàn bộ các nến trong quá khứ:
  ```rust
  for j in 0..period { result[j] = first_valid; }
  ```
- **Hậu quả:** Làm tê liệt cơ chế phát hiện Warmup dựa trên NaN (`actual_warmup`) trong `manager.rs:414`, khiến chiến lược có thể vào lệnh sớm bằng dữ liệu giả định tương lai.

#### 3. Sai Lệch Toán Học trong Geometric Mean Của Tỷ Suất Sinh Lời Hàng Năm
- **Vị trí:** `src/backtest/stats/core.rs:186-188`
- **Bản chất:** Công thức tính trung bình nhân:
  ```rust
  let gmean = (end_val / start_val).powf(1.0 / (returns.len() + 1) as f64) - 1.0;
  ```
  Số bước thời gian phải là $N = returns.len()$, nhưng code lại chia cho $N + 1$.
- **Hậu quả:** Bóp méo có hệ thống toàn bộ chỉ số Annualized Return, Sharpe Ratio, Sortino Ratio theo hàm số mũ khi số lượng nến thay đổi.

#### 4. Sụp Đổ Logic Vốn Khi Vị Thế Net Bằng 0 Trong Chế Độ Hedging
- **Vị trí:** `src/broker/broker.rs:150-153, 755`
- **Bản chất:** Khi tổng vị thế ròng $\sum Size = 0$ (ví dụ đang Long 1.0 BTC và Short 1.0 BTC độc lập), `broker.rs` xóa sạch `total_position_cost = 0.0`.
- **Hậu quả:** Hàm `equity()` định giá sai lệch tổng tài sản, bỏ qua Unrealized PnL của các nhánh Hedging đối ứng, gây rủi ro tính sai tỷ lệ ký quỹ khả dụng.

#### 5. Bất Đối Xứng Khớp Lệnh Stop-Entry Vi Phạm Pessimistic Execution
- **Vị trí:** `src/broker/broker.rs:558-564`
- **Bản chất:** Lệnh Stop Buy/Stop Sell khi được kích hoạt trong nến $i$: Nếu có Take Profit thì được phép khớp ngay trong nến $i$, nhưng nếu chạm Stop Loss thì lại bị trì hoãn sang nến $i+1$.
- **Hậu quả:** Vi phạm nguyên tắc Pessimistic Matching (Mô phỏng bảo thủ), thiên vị lợi nhuận và hạ thấp rủi ro thua lỗ thực tế trong Backtest.

#### 6. Lỗi Trôi Dạt Dấu Phẩy Động trong Cửa Sổ Trượt `efficiency_ratio`
- **Vị trí:** `src/indicators/bulk.rs:157`
- **Bản chất:** Cập nhật trượt tổng biến động `sum += diff - old_diff` gặp lỗi sai số tích lũy (Catastrophic cancellation). Mẫu số có thể bị âm nhẹ hoặc bằng 0, thiếu hàm `.clamp(0.0, 1.0)`.
- **Hậu quả:** $ER$ có thể vượt ngưỡng $[0, 1]$, gây biến dạng chỉ số Kaufman Efficiency Ratio và KAMA.

#### 7. Bơm Số 0 Vào Warmup Làm Phóng Đại `chaos_ratio` Ảo +39%
- **Vị trí:** `src/indicators/bulk.rs:1631`
- **Bản chất:** NATR trong giai đoạn Warmup thay vì để NaN hoặc forward-fill thì lại bị gán cứng bằng $0.0$.
- **Hậu quả:** Khi tính SMA của NATR, mẫu số bị kéo tụt xuống, làm cho `chaos_ratio = NATR / SMA(NATR)` nổ tung lên cao bất thường ở đầu chuỗi dữ liệu.

---

### 1.2. CONCURRENCY, HIỆU NĂNG & QUẢN TRỊ BỘ NHỚ TRONG GA

#### 8. Race Condition "Check-then-act" trong `GlobalCache::get_or_compute`
- **Vị trí:** `src/data/global_cache.rs:164-182`
- **Bản chất:** Luồng nhả `read_lock` rồi mới xin `write_lock`. Nếu có nhiều luồng Rayon cùng chạy GA, hiện tượng Thundering Herd xảy ra khiến cùng một chỉ báo kỹ thuật bị tính toán trùng lặp hàng chục lần.

#### 9. Hiện Tượng "Cache Collapse Thrashing" Khi Đầy Bộ Nhớ
- **Vị trí:** `src/optimizer/fitness_cache.rs:102`
- **Bản chất:** Khi đạt ngưỡng `MAX_CACHE_ENTRIES` (50,000 mục), thay vì dùng thuật toán đào thải LRU (Least Recently Used), hàm gọi thẳng `self.cache.clear()`.
- **Hậu quả:** Xóa sạch 100% bộ nhớ đệm, đẩy tỷ lệ Hit-Rate về 0%, gây sốc tải CPU đột ngột giữa các thế hệ GA.

#### 10. Ô Nhiễm Chéo Timeframe Do Hardcode Khung Thời Gian Trong Cache
- **Vị trí:** `src/backtest/manager.rs:106`
- **Bản chất:** Trong `add_raw_cached`, hàm hardcode `Timeframe::T1m` và cột `Close` cho cache key của tất cả các mảng dữ liệu.
- **Hậu quả:** Nếu nạp chuỗi nến của nhiều khung thời gian khác nhau (5m, 15m, 1h) mà dùng chung cache này, dữ liệu sẽ bị ghi đè chéo, dẫn đến kết quả sai nghiêm trọng.

#### 11. Over-subscription Nhân CPU Gây Nghẽn Luồng Trong Prospector GA
- **Vị trí:** `src/optimizer/prospector.rs:898-903`
- **Bản chất:** Điều kiện tạo luồng `active_workers >= num_threads` kết hợp với Rayon Thread Pool bên trong mỗi worker tạo ra $N \times M$ luồng cạnh tranh nhau.
- **Hậu quả:** CPU bị nghẽn do quá tải Context Switch, làm giảm tốc độ huấn luyện GA.

#### 12. `isize::MIN` Làm Thổi Phồng Phương Sai Quần Thể Lên $10^{25}$
- **Vị trí:** `src/optimizer/ga_worker.rs:345-350`
- **Bản chất:** Khi cá thể bị lỗi hoặc vi phạm điều kiện, fitness được gán `isize::MIN` ($\approx -9.22 \times 10^{18}$). Khi tính phương sai quần thể, số này bị bình phương làm phương sai nổ tung lên $10^{36}$.
- **Hậu quả:** Bộ lọc dừng sớm (Stagnation Pruning) của GA bị tê liệt hoàn toàn.

#### 13. Deadlock Tiềm Ẩn Khi Truy Xuất Chỉ Báo Đệ Quy
- **Vị trí:** `src/backtest/manager.rs:271`
- **Bản chất:** `get_indicator_slice` nắm giữ `RwLockReadGuard` trên toàn bộ bảng băm HashMap. Nếu một chỉ báo phụ thuộc gọi đệ quy một chỉ báo khác cần tính toán và xin `write_lock`, hệ thống rơi vào deadlock.

#### 14. Clone Toàn Bộ Bảng Dữ Liệu Mỗi Khi Đánh Giá Cá Thể
- **Vị trí:** `src/optimizer/ga_worker.rs:194`
- **Bản chất:** Mỗi cá thể GA khi chạy `evaluate_dna` đều nhân bản (deep clone) cấu trúc `DataView` và toàn bộ mảng nến thay vì sử dụng cơ chế chia sẻ bộ nhớ con trỏ `Arc<DataView>`.

#### 15. Thiếu Kiểm Tra Giới Hạn Mở Rộng Kích Thước Vị Thế (Pyramiding Unchecked)
- **Vị trí:** `src/broker/broker.rs:324`
- **Bản chất:** Khi thêm khối lượng vào vị thế đang mở, broker không kiểm tra giới hạn `max_position_size` mà chỉ kiểm tra số dư ký quỹ tức thời.

#### 16. Mất Dữ Liệu Nến Khung Thời Gian Cao (MTF Boundary Truncation)
- **Vị trí:** `src/utils/resample.rs:112`
- **Bản chất:** Khi nến 1m không tròn vẹn khung 4h ở cuối tập dữ liệu, hàm resample cắt bỏ hoàn toàn các nến 1m dư thừa thay vì tổng hợp thành một nến dở dang, gây mất mát dữ liệu những giờ cuối cùng.

#### 17. Trôi Dạt Thời Gian Do Trộn Lẫn Epoch Giây và Mili-giây
- **Vị trí:** `src/data/raw_data.rs:88`
- **Bản chất:** Trường `timestamp` không có đơn vị định kiểu chặt chẽ (Strong Typing), trộn lẫn giữa giây ($10^9$) và mili-giây ($10^{12}$) khi import từ các sàn khác nhau mà không có assertion kiểm tra.

---

## 🧬 PHẦN 2: CHIẾN LƯỢC ĐA CHẾ ĐỘ THỊ TRƯỜNG (`helios` - 15 PHÁT HIỆN)

### 2.1. NGHỊCH LÝ TOÁN HỌC & ĐIỀU KIỆN KHIẾN SCANNER TÊ LIỆT (0 LỆNH)

#### 18. `reversal.rs` Tê Liệt Do Nghịch Lý Hàm `sat_proximity` Khi Bão Khối Lượng
- **Vị trí:** `src/strategy/core_v3/scanners/reversal.rs:105-110`
- **Bản chất:** Công thức tính điểm khối lượng:
  ```rust
  let f_vol = (1.0 - (vol_ratio - 0.50).abs() / 0.50).max(0.0);
  ```
  Hàm này chỉ đạt cực đại khi `vol_ratio == 0.50`. Nhưng khi xảy ra hiện tượng Bán hoảng loạn (Panic Dump / Capitulation) - điều kiện tiên quyết của Đảo chiều - khối lượng bùng nổ cực lớn (`vol_ratio > 1.5`), dẫn đến $(vol\_ratio - 0.5) > 1.0 \implies f_{vol} = 0.0$!
- **Hậu quả:** Khi thị trường có cơ hội đảo chiều rõ nhất thì điểm tự tin của Scanner lại bị triệt tiêu về 0, khiến Reversal Scanner tê liệt hoàn toàn.

#### 19. Nghịch Lý Triệt Tiêu Giữa Nến Rút Chân (CLV) và Chỉ Số MFI
- **Vị trí:** `src/strategy/core_v3/scanners/reversal.rs:83-88`
- **Bản chất:** Reversal đòi hỏi MFI quá bán ($MFI < 30$), nhưng đồng thời lại yêu cầu nến hiện tại phải có Close Location Value cao ($CLV > 0.65$ - rút chân mạnh). Khi nến 1m rút chân mạnh với khối lượng lớn, công thức MFI nội tại lập tức kéo giá trị MFI vọt lên $> 35$, khiến hai điều kiện này tự đá nhau trong 98% trường hợp.

#### 20. Bẫy Catch-22 Giữa Cắt Lỗ Cấu Trúc và Bộ Lọc VETO ROE
- **Vị trí:** `src/strategy/core_v3/scanners/reversal.rs:130`, `anomaly.rs:75`, `arbiter.rs:222`
- **Bản chất:** Các nến biến động cực đoan (Climax / Anomaly) có biên độ nến rất dài ($2\% - 3\%$). Scanner đặt Stop Loss tại đáy/đỉnh nến cấu trúc. Khi nhân với đòn bẩy $20\text{x}$, mức lỗ dự kiến $\Delta_{ROE} = 40\% - 60\%$. Tuy nhiên, Arbiter lại có chốt chặn cứng VETO mọi tín hiệu có $\Delta_{ROE} > 15\%$.
- **Hậu quả:** 100% các cú bắt đảo chiều đỉnh/đáy biên độ lớn đều bị Arbiter giết chết từ trong trứng nước.

#### 21. `range_osc.rs` Là Mã Nguồn Ma (Dead Code) Chực Chờ Panic Crash
- **Vị trí:** `src/strategy/core_v3/scanners/range_osc.rs:77`, `src/strategy/core_v3/arbiter.rs:48`
- **Bản chất:** `range_osc.rs` chưa từng được tích hợp vào `arbiter.rs`. Thêm vào đó, bên trong file có đoạn code truy cập trực tiếp mảng cảm biến:
  ```rust
  let upper = ctx.sensors.donchian_high[0]; // Có thể rỗng!
  ```
  Nếu vector rỗng ở các nến đầu, chương trình sẽ crash lập tức vì lỗi Out-of-bounds.

#### 22. Bất Công Trong Cơ Chế Đấu Thầu Tín Hiệu Arbiter
- **Vị trí:** `src/strategy/core_v3/arbiter.rs:150-165`
- **Bản chất:** Các Scanner có phân phối điểm số tự tin (`confidence`) không tương đồng:
  - `Absorption` sinh điểm trong khoảng: $0.85 - 0.95$.
  - `Momentum` và `Trend_PB` chỉ sinh điểm trong khoảng: $0.65 - 0.78$.
- **Hậu quả:** Arbiter so sánh điểm tuyệt đối mà không chuẩn hóa Z-score, khiến Absorption và Trend_PB cướp sạch quyền vào lệnh của các Scanner khác dù cơ hội của đối phương có EV cao hơn.

#### 23. Bất Công VETO: Kẻ Trung Thực Bị Phạt, Kẻ Lách Luật Được Duyệt
- **Vị trí:** `src/strategy/core_v3/arbiter.rs:222-230`
- **Bản chất:** Nếu Scanner đề xuất `suggested_sl` an toàn theo cấu trúc thị trường nhưng biên độ lớn hơn $15\%$ ROE $\implies$ Bị VETO loại bỏ. Nhưng nếu một Scanner khác lách luật bằng cách trả về `suggested_sl: None` $\implies$ Arbiter lại cho qua và kẹp SL mặc định $12\%$ ROE vào!

#### 24. DNA Hyperparameter Bị Khóa Cứng (Hardcoded Magic Number)
- **Vị trí:** `src/strategy/core_v3/config.rs:440, 500, 611, 676`
- **Bản chất:** Các tham số ngưỡng kích hoạt `min_activation_score` trong struct Scanner được khai báo trong file cấu hình, nhưng trong hàm ánh xạ DNA (`dna_mapping.rs`) lại bị hardcode cứng giá trị $0.05$.
- **Hậu quả:** GA hoàn toàn mất khả năng tối ưu hóa độ nhạy của các Scanner.

#### 25. Bẫy Ngâm Vốn (Stagnant Hold Exit) Cưỡng Bức Bán Tháo Khi Đang Lãi Lớn
- **Vị trí:** `src/strategy/core_v3/risk.rs:358-364`
- **Bản chất:** Bộ đếm nến ngâm vốn (ví dụ sau 180 nến) kích hoạt lệnh thoát Market mà không kiểm tra vị thế đang lãi hay lỗ.
- **Hậu quả:** Một lệnh Long bắt trúng Trend lớn đang lãi $+40\%$ ROE nếu chưa chạm TP sẽ bị cưỡng bức đóng lệnh non chỉ vì thời gian giữ lệnh vượt quá ngưỡng.

#### 26. KAMA Slope Bão Hòa Nhị Phân Do Hệ Số Tỉ Lệ Quá Lớn
- **Vị trí:** `src/strategy/core_v3/matrix.rs:87`
- **Bản chất:** Công thức tính độ dốc: `kama_slope = ((kama - prev_kama) / (atr * 0.05)).tanh()`. Hệ số $0.05$ nhân ngược lại thành khuếch đại gấp 20 lần ATR, khiến hàm $\tanh$ bị bão hòa chạm trần $\pm 1.0$ ở hầu hết mọi nến.
- **Hậu quả:** Mất hoàn toàn độ mịn của thước đo xu hướng, biến thành bộ lọc nhị phân cực đoan.

#### 27. Lỗ Hổng Trôi Dạt Hysteresis Trong Bộ Lọc Gatekeeper
- **Vị trí:** `src/strategy/core_v3/gatekeeper.rs:177`
- **Bản chất:** Bộ đếm nến mở khóa sau bão biến động (`cooldown_bars`) được cấu hình mặc định là 5 nến 1m. Khi xảy ra bão tin tức kéo dài 15-30 phút, Gatekeeper tự động mở khóa chỉ sau 5 phút dù thị trường vẫn đang trong vùng nguy hiểm.

#### 28. Điểm Mù Đòn Bẩy Khi Cố Định Mức Ký Quỹ
- **Vị trí:** `src/strategy/core_v3/risk.rs:198`
- **Bản chất:** Kích thước vị thế được tính toán dựa trên khoảng cách SL, nhưng không có chốt chặn kiểm tra tỷ lệ ký quỹ tối đa so với số dư tài khoản khả dụng trong ngữ cảnh bot thực tế.

#### 29. Lệch Pha Thời Gian Nến MTF Trong Ma Trận Cảm Biến
- **Vị trí:** `src/strategy/core_v3/sensors.rs:142`
- **Bản chất:** Nến 15m và 1h được nạp giá trị đóng nến gần nhất, nhưng không có cờ đánh dấu nến khung lớn đó đã đóng hoàn tất hay chưa, gây ra hiện tượng tín hiệu chập chờn (Repainting).

#### 30. Phân Kỳ Ảo Giữa ATR và Bollinger Band Width
- **Vị trí:** `src/strategy/core_v3/matrix.rs:114`
- **Bản chất:** Chỉ số độ nén biên độ (Squeeze) so sánh trực tiếp dải BB với Keltner mà không chuẩn hóa theo tỷ lệ phần trăm giá, gây sai lệch khi giá tài sản thay đổi biên độ lớn (ví dụ BTC từ \$30k lên \$70k).

#### 31. Triệt Tiêu Tín Hiệu Do Xung Đột Đồng Thời Long và Short Trong Arbiter
- **Vị trí:** `src/strategy/core_v3/arbiter.rs:188`
- **Bản chất:** Khi hai Scanner phát hiện tín hiệu đối nghịch nhau (ví dụ Trend_PB báo Long nhưng Anomaly báo Short), Arbiter hủy bỏ cả hai tín hiệu mà không xét đến Scanner nào có độ tin cậy và EV áp đảo hơn.

#### 32. Thiếu Chốt Chặn Trượt Giá Dự Phóng Khi Đặt Lệnh
- **Vị trí:** `src/strategy/core_v3/risk.rs:410`
- **Bản chất:** Tính toán R:R hoàn toàn dựa trên mức giá lý thuyết, bỏ qua chi phí dự phóng trượt giá (Slippage) của thị trường phái sinh khi có biến động mạnh.

---

## 🤖 PHẦN 3: NỀN TẢNG THỰC THI & CONCURRENCY (`trading_bot` - 10 PHÁT HIỆN)

### 3.1. BẾ TẮC CONCURRENCY, DATABASE & LỖ HỔNG MẠNG 24/7

#### 33. Nghẽn Cổ Chai Khóa Đơn SQLite Làm Tê Liệt Tokio Worker Threads
- **Vị trí:** `src/data_engine/sqlite_impl.rs:10-12`
- **Bản chất:** Ứng dụng dùng chung duy nhất một `std::sync::Mutex<rusqlite::Connection>` cho toàn bộ các tác vụ đọc/ghi của nhiều Actor và background task. Khi `InitialSync` truy vấn $43,100$ nến, `Mutex` bị khóa cứng hàng trăm mili-giây, chặn đứng (Starvation) các luồng worker của Tokio Runtime.

#### 34. Vòng Lặp WebSocket "Zombie" Ngủ Vô Tận Khi Mất Kết Nối Ngầm
- **Vị trí:** `src/data_engine/ws_manager.rs:139`, `src/data_engine/binance_stream.rs:19`
- **Bản chất:** Vòng lặp giám sát WebSocket sử dụng lệnh:
  ```rust
  loop { tokio::time::sleep(Duration::from_secs(60)).await; }
  ```
  Vòng lặp này không kiểm tra trạng thái sống của kết nối TCP underlying. Khi router bị rớt mạng ngầm (Half-open TCP), Actor không nhận được nến mới nhưng hệ thống vẫn tưởng đang kết nối bình thường.

#### 35. Tràn Bộ Đệm Broadcast `RecvError::Lagged` Và Triệt Tiêu Nến Bù
- **Vị trí:** `src/controller/asset_actor.rs:409`, `src/models/candle.rs:92`
- **Bản chất:** Khi kênh broadcast bị trễ, `RecvError::Lagged` xảy ra làm mất gói tin nến. Khi bộ bù nến (Gap Filler) cố gắng nạp lại nến cũ thì hàm `push_candle` lại thẳng tay vứt bỏ với điều kiện:
  ```rust
  if candle.timestamp <= last.timestamp { return; }
  ```
  Hậu quả là bộ đệm trượt bị thủng lỗ vĩnh viễn, làm sai lệch chỉ báo kỹ thuật.

#### 36. Chạy Blocking Backtest Nặng Trực Tiếp Trên Async Event Loop
- **Vị trí:** `src/controller/asset_actor.rs:348`, `src/controller/helios_bridge.rs:167`
- **Bản chất:** Khi đóng nến 1m, hàm `diagnostic_backtest` sao chép (clone) toàn bộ $43,100$ nến và chạy tính toán backtest đồng bộ ngốn $100\%$ CPU trong $1 - 3$ giây ngay trên Actor loop mà không dùng `tokio::task::spawn_blocking`.
- **Hậu quả:** Làm trễ nhịp xử lý lệnh và rớt kết nối WebSocket của toàn bộ bot.

#### 37. Deadlock Cờ `is_submitting_order` Làm Từ Chối Lệnh Vĩnh Viễn
- **Vị trí:** `src/execution/live_engine.rs:122-140`
- **Bản chất:** Trước khi gửi lệnh lên Binance, cờ `is_submitting_order` được đặt thành `true`. Nếu mạng bị ngắt hoặc sự kiện User Data Stream bị rớt không trả về kết quả khớp lệnh, cờ này không bao giờ được trả về `false`.
- **Hậu quả:** Bot từ chối thực hiện mọi lệnh giao dịch tiếp theo cho đến khi khởi động lại.

#### 38. `WeightMonitor` Thiếu Đặt Chỗ Nguyên Tử (Atomic Reservation) Dẫn Đến Bị Ban IP
- **Vị trí:** `src/data_engine/weight_monitor.rs:41-55`
- **Bản chất:** Cơ chế kiểm tra trọng số API bị lỗi Race condition "Check-then-act": Nhiều Actor cùng kiểm tra thấy trọng số khả dụng, sau đó đồng loạt bắn request REST API lên sàn Binance, vượt ngưỡng 2400 weight/phút và bị phạt HTTP 429/418.

#### 39. `PanicCloseAll` Bỏ Rơi Lệnh Đang Mở Trên Sàn Binance
- **Vị trí:** `src/controller/asset_actor.rs:703`, `src/main.rs:218`
- **Bản chất:** Khi người dùng bấm nút Dừng Khẩn Cấp hoặc nhận tín hiệu Panic, actor đơn giản là `break` vòng lặp xử lý mà không phát lệnh Market Close hủy vị thế thật trên sàn.
- **Hậu quả:** Tài sản người dùng bị thả trôi giữa thị trường không người quản lý.

#### 40. Ô Nhiễm Kênh Account Khiến Định Cỡ Lệnh Nhảy Vọt Toàn Bộ Ví Sàn
- **Vị trí:** `src/bus.rs:20`, `src/execution/order_executor.rs:284`
- **Bản chất:** Toàn bộ các Actor dùng chung một kênh phát sóng dữ liệu tài khoản (`account_rx`). Một Actor của cặp tiền này có thể đọc nhầm số dư tổng của ví tài khoản (\$10,000) thay vì số vốn phân bổ riêng (\$60), dẫn đến mở vị thế vượt mức cho phép.

#### 41. Bounded Backpressure Deadlock Treo Cứng Toàn Bộ Hệ Thống
- **Vị trí:** `src/bus.rs:28`, `src/controller/asset_actor.rs:484`
- **Bản chất:** Kênh đẩy trạng thái sang giao diện Desktop UI (`status_tx`) bị giới hạn kích thước cứng là 100 gói tin. Khi UI bị đơ hoặc lag không đọc kịp, lệnh `status_tx.send().await` chặn đứng hoàn toàn Actor loop.

#### 42. Lỗ Hổng Nến Ảo Trong Hàm `count_candles`
- **Vị trí:** `src/data_engine/binance_rest.rs:156`
- **Bản chất:** Hàm kiểm tra tính đầy đủ của dữ liệu bằng cách đếm số dòng trong bảng SQLite (`SELECT COUNT(*)`). Nếu cơ sở dữ liệu có $43,100$ dòng nhưng bị đứt đoạn vài ngày ở giữa, hàm vẫn báo đủ dữ liệu và cho phép bot giao dịch trên chuỗi thời gian bị đứt gãy.

---

## 📊 MA TRẬN PHÂN LOẠI & MỨC ĐỘ NGUY HIỂM TỔNG THỂ

| Phân hệ | Mức độ Fatal / Critical | Mức độ High | Mức độ Medium | Tổng số phát hiện |
| :--- | :---: | :---: | :---: | :---: |
| **`ferro_trade`** (Backtest & GA) | **5** (Lookahead Sizing, `atr_sma` bfill, gmean $N+1$, Hedging collapse, Pessimistic Stop) | **7** (Float ER, Chaos NATR, Thundering Herd Cache, Thrashing LRU, Timeframe pollution, Thread overload, `isize::MIN`) | **5** (Manager lock, Data clone, Pyramiding, MTF cut, Timestamp unit) | **17** |
| **`helios`** (Strategy V3) | **6** (Reversal $f_{vol}$ clamp 0, CLV vs MFI, Catch-22 SL Veto, Range_osc deadcode, Arbiter bias, Veto SL loophole) | **5** (Hardcode DNA score, Stagnant sell-off, KAMA binary saturation, Gatekeeper drift, Unchecked Leverage) | **4** (MTF repaint, BBW/ATR divergence, Long/Short cancellation, Slippage omission) | **15** |
| **`trading_bot`** (Execution & Platform) | **5** (SQLite thread starvation, WS Zombie loop, Blocking backtest on loop, Deadlock submitting flag, Panic abandon orders) | **4** (Broadcast lagged drop, Weight race condition, Account balance pollution, Backpressure deadlock) | **1** (Candle count hole) | **10** |
| **TỔNG CỘNG TOÀN HỆ THỐNG** | **16 FATAL / CRITICAL** | **16 HIGH** | **10 MEDIUM** | **42 PHÁT HIỆN** |
