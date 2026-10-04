# KIỂM TOÁN DÒNG DỮ LIỆU CẢM BIẾN (DATA LINEAGE VERIFICATION) — HELIOS CORE V3

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `core_v3/mod.rs`, `core_v3/config.rs`, `ferro_trade/src/indicators/factory/bulk.rs`, `core_v3/telemetry/cards.rs`.  
> **Mục tiêu:** Xác minh tính toàn vẹn của dữ liệu cảm biến đa khung thời gian (MTF), giải mã độ lệch giữa các chỉ báo biến động và hiện tượng ô nhiễm dữ liệu.

---

## 1. GIẢI MÃ ĐỘ LỆCH `NATR 0.89%` VS `ATR/Price 0.41%` (LỆCH ~2.2 LẦN)

Trong quá trình kiểm toán log giao dịch (Run 2, Bar 20484), phát hiện telemetry ghi nhận:
```text
Price: 2687.66 | NATR: 0.89% | ATR: 11.0907
```
Thực tế tính toán: $11.0907 / 2687.66 = 0.4126\%$. So với NATR $0.89\%$, hai đại lượng cùng phản ánh biến động giá lại lệch nhau tới **2.16 lần**.

### 🔍 Kết quả truy vết dòng code (Root Cause)
1. **Hai cấu hình gen DNA hoàn toàn độc lập:**
   Tại [`core_v3/config.rs:188-204`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L188-L204):
   ```rust
   pub struct AtrParams {
       #[dna(min = 8.0, max = 80.0, step = 3.0)]
       pub period: usize,
       #[dna(subset = "5m, 15m, 1h, 4h")]
       pub timeframe: Timeframe,
   }

   pub struct NAtrParams {
       #[dna(min = 8.0, max = 80.0, step = 3.0)]
       pub period: usize,
       #[dna(subset = "5m, 15m, 1h, 4h")]
       pub timeframe: Timeframe,
   }
   ```
2. **Khởi tạo độc lập tại [`core_v3/mod.rs:175, 251`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L175):**
   ```rust
   s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
   ...
   s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);
   ```
3. **Kết luận khoa học:**
   - Thuật toán di truyền GA đã tối ưu và ghép **hai khung thời gian / chu kỳ khác nhau**:
     - `atr_sl` (dùng làm đệm tính SL trong các Scanner) chạy trên khung nhỏ (ví dụ 5m).
     - `natr` (dùng trong Gatekeeper né bão/né flatline) chạy trên khung lớn (ví dụ 1h).
   - Thư viện `kand` tính toán đúng công thức $NATR = (ATR / Close) \times 100$, nhưng do khác nến cơ sở nên tỷ lệ phần trăm lệch nhau.

---

## 2. HIỆN TƯỢNG Ô NHIỄM DỮ LIỆU MTF & MAPPING LOOKUP TABLE

### 2.1. Cảm biến Volume Z-Score bị neo cứng theo KAMA
- **Mã nguồn:** [`core_v3/mod.rs:250`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L250)
  ```rust
  s.vol_z_score = ctx.z_score_volume(ind.kama.timeframe, "VolZScore", ind.z_score_lookback);
  ```
- **Vấn đề First-Principles:** 
  - `ind.kama.timeframe` bị khóa cận dưới trong tập `["15m", "1h", "4h"]` (Macro Anchor).
  - Do đó, `vol_z_score` được tính trên chuỗi nến 15m/1h/4h, sau đó ánh xạ (mapping lookup table) trải đều về các nến 1m.
  - **Hệ quả vi cấu trúc:** Nếu một nến 15m có volume tăng đột biến, **suốt 15 cây nến 1m tiếp theo đều ghi nhận giá trị `vol_z_score` cao chót vót như nhau**. Điều này làm cho các Scanner (đặc biệt là Momentum và Reversal) bị đánh lừa rằng nến 1m hiện tại đang có khối lượng bùng nổ, trong khi thực tế thanh khoản nến 1m có thể đã cạn kiệt.

### 2.2. Dải Bollinger Bands MTF làm méo mó hình học Stop Loss
- **Mã nguồn:** [`core_v3/mod.rs:195`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L195)
  ```rust
  let (bb_u, bb_m, bb_l, ..) = ctx.bbands(ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
  ```
- **Hệ quả trong Absorption Scanner (`absorption.rs:127, 135`):**
  - Scanner lấy `base_support = low.min(bb_lower)` và `base_resistance = high.max(bb_upper)`.
  - Khi `bbands.timeframe` rơi vào 1h hoặc 4h, biên BB Upper/Lower cách giá hiện tại 1m hàng chục USD (2%–3% giá).
  - Đòn bẩy 20x nhân lên khiến khoảng cách SL vọt lên **25% – 45% ROE**, lập tức bị Sniper Entry Guard VETO (`Structural SL exceeds ROE cap: 34.5% > 11.0%`).

---

## 3. CHECKLIST KIẾN NGHỊ KHẮC PHỤC

- [ ] **Tách bạch rõ ranh giới Local (1m) vs Macro (MTF):**
  - Các chỉ báo dùng đo vi cấu trúc nến vào lệnh và Stop Loss (`vol_z_score`, nến breakout, đệm SL) phải neo theo nến cơ sở hoặc khung ngắn cố định (1m - 5m).
  - Các chỉ báo MTF (15m, 1h, 4h) chỉ đóng vai trò lọc xu hướng vĩ mô (Macro Trend Filter) và chế độ thị trường (Regime Filter).
- [ ] **Đồng bộ hóa hoặc quy chuẩn ATR:** Đảm bảo tỷ lệ chuyển đổi giữa ATR và NATR có sự nhất quán về khung nến quan sát để tránh làm sai lệch các phép chuẩn hóa phi thứ nguyên.
