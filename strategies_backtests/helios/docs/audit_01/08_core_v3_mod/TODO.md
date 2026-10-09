# 📋 DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO) - TRÁI TIM CHIẾN LƯỢC (CORE_V3/MOD.RS)

**Module:** `HeliosStrategyV3 Core Engine & Cooldown Tracker`  
**Đường dẫn tài liệu kiểm toán:** [`strategies_backtests/helios/docs/audit_01/08_core_v3_mod/technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/08_core_v3_mod/technical_audit.md)  
**Ngày cập nhật:** 2026-09-19  

---

## 🎯 TỔNG QUAN PHÂN BỔ CÔNG VIỆC

| Mức Độ Ưu Tiên | Số Lượng Task | Trạng Thái | Mô Tả Trọng Tâm |
| :---: | :---: | :---: | :--- |
| **P0 (Critical)** | 2 | 🔴 Chưa xử lý | Hóa giải bẫy khóa vĩnh viễn (Deadlock) trong `CooldownTracker` và triệt tiêu nguy cơ Panic của `RangeOsc`. |
| **P1 (Major)** | 2 | 🟠 Chưa xử lý | Sửa lỗi chuẩn IEEE 754 nuốt NaN trong `bb_slope` và xóa bỏ lãng phí cấp phát Heap của Arbiter mỗi tick. |
| **P2 (Medium)** | 3 | 🟡 Chưa xử lý | Khắc phục fallback PnL nến rút chân, thêm sàn giá cho Breakout Threshold, và bắt lỗi Period = 0 trong `init()`. |
| **P3 (Minor)** | 2 | 🔵 Chưa xử lý | Bổ sung Post-Exit Cooloff sau lệnh thắng và siết chặt kiểm tra độ dài DNA chống Genome Corruption. |

---

## 🔴 ƯU TIÊN P0: LỖ HỔNG CHÍ TỬ & BÃY CRASH HỆ THỐNG

- [x] **Task 01: Hóa giải bẫy khóa vĩnh viễn (Permanent Deadlock) trong `Same-Side Price Guard` (COR-01)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - **Vấn đề:**  
    `Same-Side Price Guard` cấm vào lệnh cùng chiều khi giá tồi tệ hơn giá cắt lỗ gần nhất (`current_price < last_losing_price` với Long). Tuy nhiên logic này không có thời hạn hết hạn (TTL) và chỉ được reset khi có một lệnh thắng. Khi thị trường downtrend sâu rồi tạo đáy tích lũy ở vùng giá thấp hơn điểm cắt lỗ cũ, chiến lược sẽ bị kẹt cấm mở Long vĩnh viễn (suốt hàng nghìn bars).
  - **Giải pháp triển khai thực tế (Kiến trúc phân định độc quyền):**  
    Theo nguyên tắc phân định trách nhiệm cốt lõi (Separation of Concerns), chức năng Cầu Dao ngắt mạch (Circuit Breaker / Cooldown) sau chuỗi thua thuộc thẩm quyền độc quyền của Bot Engine (`AssetActor::cooldown_guard`) dựa trên PnL tài khoản thực tế. Helios Strategy chuyên tâm 100% vào việc sinh tín hiệu Alpha.
    + Đã loại bỏ hoàn toàn struct `CooldownTracker`, trường `cooldown_tracker` và dead-code `should_block` khỏi `HeliosStrategyV3`.
    + Loại bỏ vĩnh viễn nguy cơ Deadlock khóa lệnh và xung đột 2 lần Cooldown (Double Cooldown).
    + Dọn dẹp toàn bộ unit tests liên quan của Strategy, bảo toàn trọn vẹn 103 gen DNA và các tests của toàn bộ 4 Scanners, Matrix, Arbiter, Risk.
  - **Tiêu chí nghiệm thu (Acceptance Criteria):**  
    + Toàn bộ 96/96 unit tests của `helios` passed 100%.
    + 50/50 tests `ferro_trade` và 46/46 tests `bot_engine` cùng 4/4 integration tests `trailing_sl_test` passed 100%. Không có bất kỳ regression nào.

---

- [ ] **Task 02: Loại bỏ triệt để nguy cơ Panic Index Out-of-Bounds trong `range_osc.rs` (COR-02)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:L70-L71`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L70-L71)  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L203-L213`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L203-L213)
  - **Vấn đề:**  
    Trong `init()`, chỉ báo `stoch_range` đã bị tắt (mảng `sensors.stoch_range_k` có độ dài 0). Trong khi đó file `range_osc.rs` vẫn truy cập `sensors.stoch_range_k[i]`. Nếu scanner này được kích hoạt lại trong Arbiter, bot sẽ crash tức thì.
  - **Giải pháp triển khai:**  
    Cập nhật `range_osc.rs` chuyển sang sử dụng kênh Stochastic an toàn hoặc bọc kiểm tra phòng thủ:
    ```rust
    // File: strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs
    let stoch_k = sensors.stoch_range_k.get(i).copied().unwrap_or(50.0);
    let stoch_d = sensors.stoch_range_d.get(i).copied().unwrap_or(50.0);
    ```
  - **Tiêu chí nghiệm thu:**  
    + Thêm `RangeOscScanner` vào Arbiter và chạy backtest không phát sinh panic `out of bounds`.

---

## 🟠 ƯU TIÊN P1: LÃNG PHÍ TÀI NGUYÊN & LỖ HỔNG XỬ LÝ SỐ LIỆU

- [ ] **Task 03: Sửa lỗi chuẩn IEEE 754 nuốt NaN trong tính toán độ dốc `bb_slope` (COR-03)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L220-L239`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L220-L239)
  - **Vấn đề:**  
    Biểu thức `s.atr_sl[idx].max(1e-5)` biến `f64::NAN` thành `1e-5`, khiến điều kiện `atr.is_nan()` trả về `false`. Dẫn đến độ dốc `raw_slope` bị chia cho `5 * 1e-5`, khuếch đại sai lệch lên hàng nghìn đơn vị trong giai đoạn khởi động Warmup của ATR.
  - **Giải pháp triển khai:**  
    Kiểm tra tính hợp lệ của `s.atr_sl[idx]` trước khi clamping:
    ```rust
    // File: strategies_backtests/helios/src/strategy/core_v3/mod.rs
    let k_bars = 5;
    let bb_m_len = s.bb_middle.len();
    let mut bb_slope = Vec::with_capacity(bb_m_len);
    for idx in 0..bb_m_len {
        if idx < k_bars {
            bb_slope.push(0.0);
        } else {
            let m_curr = s.bb_middle[idx];
            let m_prev = s.bb_middle[idx - k_bars];
            let atr_raw = if idx < s.atr_sl.len() { s.atr_sl[idx] } else { 1.0 };
            if m_curr.is_nan() || m_prev.is_nan() || atr_raw.is_nan() {
                bb_slope.push(0.0);
            } else {
                let atr = atr_raw.max(1e-5);
                let raw_slope = (m_curr - m_prev) / (k_bars as f64 * atr);
                bb_slope.push(raw_slope);
            }
        }
    }
    s.bb_middle_slope_adj = std::sync::Arc::new(bb_slope);
    ```
  - **Tiêu chí nghiệm thu:**  
    + Unit test với `atr_sl` chứa NaN tại các nến đầu đảm bảo `bb_slope` trả về 0.0 tuyệt đối, không có giá trị đột biến > 10.0.

---

- [ ] **Task 04: Triệt tiêu lãng phí cấp phát Heap của Arbiter mỗi tick (COR-04)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L338-L346`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L338-L346)  
    + [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:L74-L93`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L74-L93)
  - **Vấn đề:**  
    Hàm `next()` mỗi tick đều khởi tạo `HeliosArbiter::new()` và gọi 4 lần `Box::new` cho các Unit Structs (ZSTs), gây ra 5 triệu lần cấp phát/giải phóng heap trong một lần backtest 1 triệu nến, làm chậm hệ thống 25%.
  - **Giải pháp triển khai:**  
    Lưu trữ thể hiện của `HeliosArbiter` trực tiếp trong struct `HeliosStrategyV3` và khởi tạo một lần duy nhất tại `Strategy::new`:
    ```rust
    // Trong struct HeliosStrategyV3:
    pub struct HeliosStrategyV3 {
        ...
        pub arbiter: HeliosArbiter,
    }

    // Trong Strategy::new():
    let mut arbiter = HeliosArbiter::new();
    arbiter.add_scanner(Box::new(scanners::trend_pb::TrendPBScanner));
    arbiter.add_scanner(Box::new(scanners::momentum_ignition::MomentumIgnitionScanner));
    arbiter.add_scanner(Box::new(scanners::reversal::ReversalScanner));
    arbiter.add_scanner(Box::new(scanners::absorption::AbsorptionScanner));

    // Trong next():
    let decision = self.arbiter.process_tick(tick, &self.sensors, &self.matrix, &self.config, self.debug_mode);
    ```
  - **Tiêu chí nghiệm thu:**  
    + Vòng lặp `next()` đạt chuẩn Zero-Allocation (0 allocations trên Heap khi duyệt nến không có tín hiệu khớp lệnh).
    + Tốc độ thực thi backtest 1 triệu nến tăng ít nhất 20%.

---

## 🟡 ƯU TIÊN P2: ĐỘ CHÍNH XÁC TOÁN HỌC & BẢO VỆ GIAO DỊCH

- [ ] **Task 05: Khắc phục kịch bản Fallback nhận diện PnL sai khi nến rút chân (COR-05)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L269-L284`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L269-L284)
  - **Vấn đề:**  
    Khi thiếu `last_closed_trade` từ broker, code dùng fallback `tick.closes()[i]` so sánh với `entry_price`. Nếu nến có râu quét cắt lỗ rồi rút chân xanh, fallback tính PnL > 0, ngộ nhận là lệnh thắng và xóa sạch Cooldown, dẫn đến mở lệnh ngay lập tức ở tick sau.
  - **Giải pháp triển khai:**  
    Nếu không có `last_closed_trade` và `last_trade.sl_price` là `None`, dùng giá cực trị bất lợi nhất của nến (nến Long dùng `tick.lows()[i]`, nến Short dùng `tick.highs()[i]`) để phòng thủ thận trọng:
    ```rust
    let (is_long, trade_pnl, exit_price) = if let Some(closed) = tick.last_closed_trade {
        let exit_p = closed.exit_price.unwrap_or_else(|| tick.closes()[i]);
        (closed.is_long(), closed.pl(None), exit_p)
    } else {
        let is_l = last_trade.is_long();
        // Thận trọng: Dùng SL price nếu có, nếu không có dùng đáy nến (Long) hoặc đỉnh nến (Short)
        let fallback_price = if is_l { tick.lows()[i] } else { tick.highs()[i] };
        let exit_p = last_trade.sl_price.unwrap_or(fallback_price);
        let pnl = if is_l { exit_p - last_trade.entry_price } else { last_trade.entry_price - exit_p };
        (is_l, pnl, exit_p)
    };
    ```
  - **Tiêu chí nghiệm thu:**  
    + Nến rút chân quét SL nhưng đóng cửa xanh vẫn được nhận diện chính xác là lệnh thua, kích hoạt Cooldown phạt đúng quy định.

---

- [ ] **Task 06: Bổ sung sàn giá tối thiểu cho Universal Breakout Threshold (COR-07)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L79-L81`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L79-L81)
  - **Vấn đề:**  
    Khi ATR bị nén trong vùng Bollinger Squeeze, `threshold = 1.5 * ATR` trở nên cực nhỏ ($< 0.02\%$), khiến dao động vi cấu trúc (Micro-noise) cũng kích hoạt mở khóa Cooldown sớm.
  - **Giải pháp triển khai:**  
    Thêm sàn bảo vệ tỷ lệ phần trăm giá tối thiểu (ví dụ 0.15% giá hiện tại):
    ```rust
    let mult = if breakout_atr_mult <= 0.0 { 1.5 } else { breakout_atr_mult };
    let min_price_threshold = current_price * 0.0015; // Sàn tối thiểu 0.15%
    let threshold = (mult * atr.max(1e-5)).max(min_price_threshold);
    ```
  - **Tiêu chí nghiệm thu:**  
    + Khi ATR = 0.01 USD trên giá 2,000 USD, ngưỡng bứt phá tối thiểu vẫn giữ mức 3.0 USD, không bị mở khóa non bởi micro-noise.

---

- [ ] **Task 07: Bắt lỗi Period = 0 toàn diện trong `init()` chống "Chết Lâm Sàng" (COR-06)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L135-L141`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L135-L141)
  - **Vấn đề:**  
    Bỏ sót kiểm tra period = 0 cho Keltner, VWMA, ZScore, RSI các phân đội và Stochastic. Lệnh `return;` âm thầm làm chiến lược chạy rỗng không báo lỗi.
  - **Giải pháp triển khai:**  
    Kiểm tra đầy đủ tất cả chỉ báo hoạt động và thay `println!` bằng cảnh báo chuẩn `tracing::error!` hoặc gán fallback an toàn:
    ```rust
    if ind.natr.period == 0 || ind.mfi.period == 0 || 
       ind.kama.short_period == 0 || ind.kama.long_period == 0 ||
       ind.adx.period == 0 || ind.bbands.period == 0 || ind.atr.period == 0 ||
       ind.keltner.period == 0 || ind.vwma.period == 0 || ind.z_score_lookback == 0 ||
       ind.rsi_trend.period == 0 || ind.stoch_trend.period == 0 {
        tracing::error!("🚨 [CRITICAL CONFIG ERROR] Một hoặc nhiều tham số chỉ báo có period = 0!");
        panic!("HeliosStrategyV3: Không thể khởi tạo chiến lược với chỉ báo có period = 0");
    }
    ```
  - **Tiêu chí nghiệm thu:**  
    + Không cho phép chiến lược chạy ở trạng thái "rỗng/chết lâm sàng".

---

## 🔵 ƯU TIÊN P3: TỐI ƯU HÓA HOÀN THIỆN & KHÁNG LỖI HỆ THỐNG

- [ ] **Task 08: Bổ sung "Post-Exit Cooloff" sau khi chốt lời lệnh thắng (COR-08)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L312-L325`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L312-L325)
  - **Vấn đề:**  
    Khi lệnh chốt lời có lãi, cooldown bị reset về 0 hoàn toàn. Bot có thể mở lệnh mới ngay nến tiếp theo và dính bẫy Bull Trap ở đỉnh sóng cao trào.
  - **Giải pháp triển khai:**  
    Thiết lập thời gian làm nguội tối thiểu 2-3 nến sau khi chốt lời:
    ```rust
    // Thay vì locked_until_bar = 0:
    self.cooldown_tracker.consecutive_losses = 0;
    self.cooldown_tracker.locked_until_bar = i + 2; // Khoảng đệm 2 nến làm nguội thị trường
    self.cooldown_tracker.locked_side = None;
    self.cooldown_tracker.last_losing_side = None;
    ```
  - **Tiêu chí nghiệm thu:**  
    + Sau khi đóng lệnh TP, bot không mở lệnh mới trong 2 nến liên tiếp ngay sau đó.

---

- [ ] **Task 09: Siết chặt kiểm tra tính toàn vẹn độ dài DNA (COR-09)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L120-L127`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L120-L127)
  - **Vấn đề:**  
    `dna.len() < HeliosConfigV3::DNA_WIDTH` cho phép nạp vector dài hơn (ví dụ 118 gen cũ) vào cấu hình 95 gen mới, gây xáo trộn bản đồ genome âm thầm.
  - **Giải pháp triển khai:**  
    Đổi điều kiện kiểm tra thành đẳng thức chặt chẽ:
    ```rust
    if dna.len() != HeliosConfigV3::DNA_WIDTH {
        warn!(
            "⚠️ [DNA LENGTH MISMATCH] Yêu cầu chính xác {} gen nhưng DNA cung cấp {} gen -> Từ chối nạp!",
            HeliosConfigV3::DNA_WIDTH, dna.len()
        );
        return;
    }
    ```
  - **Tiêu chí nghiệm thu:**  
    + Nạp vector 118 gen cũ bị từ chối an toàn, không làm biến dạng cấu hình hiện tại.
