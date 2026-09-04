# Kiến Trúc Bộ Chẩn Đoán Phân Kỳ: Walk-Forward Backtest vs Paper Trading

Tài liệu này xác lập cơ sở lý thuyết, các giả thuyết cốt lõi và kiến trúc 3 đầu dò chẩn đoán (Probes 1, 3, 4) nhằm giải mã triệt để nguyên nhân khiến Paper Trading bị thua lỗ (-22.26%) trong khi Single Backtest 4h vẫn sinh lời tốt (+500% đến +900%).

---

## 🔬 1. Giả Thuyết Vòng Lặp Chết: "Stagnant Decay & Cooldown Freeze" (Death Loop)

Qua phân tích log 16 và mã nguồn, ta phát hiện một chuỗi phản ứng dây chuyền tiêu cực (Chain Reaction) có thể là thủ phạm chính:

```mermaid
sequenceDiagram
    autonumber
    actor Market as Thị Trường
    participant Actor as AssetActor (RollingBuffer)
    participant Strategy as HeliosStrategyV3
    participant Risk as RiskManager
    participant Paper as PaperTrader

    Market->>Actor: Nến mới (Bar i)
    Actor->>Strategy: evaluate_live_tick (i = 43,099 cố định)
    Strategy->>Paper: Mở vị thế Long (ví dụ giá 60,000)
    Note over Market,Paper: Sau 3-5 cây nến, giá rung lắc nhẹ (59,990, âm -0.01%)
    Risk->>Strategy: decay_cycles >= 1 && current_pnl <= 0.0 -> tick.close(None)!
    Strategy->>Paper: Cắt lỗ non (STAGNANT_DECAY) -> Lỗ giá + phí 0.08%
    Strategy->>Strategy: Kích hoạt Cooldown: locked_until_bar = 43,099 + 15 = 43,114
    Note over Actor,Strategy: BẪY ĐÓNG BĂNG: Nến sau trượt nhưng len vẫn là 43,100 -> i vẫn là 43,099!
    Note over Actor,Strategy: i (43,099) < 43,114 VĨNH VIỄN -> BỊ CHẶN LỆNH MUA HỒI PHỤC!
    Market->>Strategy: Giá bứt phá mạnh > 1.5 ATR (Phá Cooldown)
    Strategy->>Paper: Mua đu đỉnh (FOMO Re-entry)!
    Note over Market,Paper: Rung lắc tiếp 3-5 nến -> LẶP LẠI VÒNG LẶP CHẾT (19 lần liên tiếp)!
```

### Các mắt xích nghi vấn:
1. **Mắt xích A (`Stagnant Decay` quá nhạy):** `effective_stagnant_bars = i - peak_bar`. Nếu `time_countdown_bars = 3..5`, một vị thế vừa mở gặp nhịp thở tự nhiên của nến 1m đã bị coi là "ngâm vốn" và bị cắt cụt.
2. **Mắt xích B (Lệch hệ quy chiếu Cooldown):** `i` trong `evaluate_live_tick` là chỉ số cục bộ trong `rolling_buffer` ($43,099$), trong khi nến liên tục trượt làm $i$ không bao giờ tiến lên $43,114$, khiến Cooldown bị khoá cứng bất thường.
3. **Mắt xích C (Đứt gãy khi Hot-Swap):** Đổi DNA làm thay đổi đột ngột `time_countdown_bars` hoặc các ngưỡng bảo vệ.

---

## 🛠️ 2. Chi Tiết Kiến Trúc 3 Đầu Dò Chẩn Đoán (Telemetry Probes)

### **Đầu Dò 4 (Hot-Swap & DNA Parameters Telemetry)**
* **Vị trí:** `AssetActor::check_and_update_dna` (hoặc `update_strategy_dna`).
* **Định dạng Log:**
  `🧬 [DNA HOT-SWAP] Symbol: {} | Fitness: {:.4} | TimeCountdown: {} | TSL_Loose_ATR: {:.2} | TSL_Tight_ATR: {:.2} | Base_SL_ATR: {:.2} | Leverage: {:.1}x`
* **Mục tiêu:** Kiểm tra mỗi 4h tham số nào thay đổi và có gây đột biến khiến vị thế bị giật SL không.

### **Đầu Dò 3 (Stagnant Decay & Cooldown Blackbox Telemetry)**
* **Vị trí:** 
  1. `RiskManager::manage_trailing_stop` (ngay trước khi gọi `tick.close(None)`).
  2. `HeliosStrategyV3::next` (khi kiểm tra bộ lọc Cooldown).
  3. `PaperTrader::close_position` (khi nhận `ExitReason::StagnantDecay` hoặc `HardStopLoss`).
* **Định dạng Log:**
  * Khi Cooldown kích hoạt:
    `🔒 [COOLDOWN SET] Bar: {} (Global: {}) | LockedUntil: {} | Side: {:?} | LastExitPrice: {:.2}`
  * Khi Cooldown chặn lệnh:
    `⛔ [COOLDOWN BLOCKED] Bar: {} | LockedUntil: {} | Side: {:?} | PriceDiff: {:.2} < ReqBreakout: {:.2}`
  * Khi Stagnant Decay kích hoạt:
    `🚨 [STAGNANT TELEMETRY] Bar: {} | EntryBar: {} | PeakBar: {} | StagnantBars: {} | CountdownBars: {} | DecayCycles: {} | PnL: ${:.2} (ROE: {:.2}%)`

### **Đầu Dò 1 (Đối Chiếu Khớp Lệnh 4h Giữa Backtest vs PaperTrader)**
* **Vị trí:** Vòng lặp chính trong `walk_forward_smart_seed.rs` tại mốc `current_ts - last_reopt_ts >= reopt_span_ms` (mỗi 4h).
* **Định dạng Log:**
  ```text
  ╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
  ║ ⚖️ [4H CHECKPOINT COMPARISON] Window: 4 Hours | Time: {}                                        ║
  ╠══════════════════════════════════════════════════════════════════════════════════════════════════╣
  ║ 🔬 Single Backtest: Trades: {:>3} | WinRate: {:>5.1}% | PnL: ${:>+7.2} ({:>+6.2}%) | MaxDD: {:>5.2}%  ║
  ║ 📝 Paper Trader:    Trades: {:>3} | WinRate: {:>5.1}% | PnL: ${:>+7.2} ({:>+6.2}%) | Balance: ${:>7.2}║
  ║ ⚠️ Divergence Delta: ΔTrades: {:>+3} | ΔPnL: ${:>+7.2}                                          ║
  ╚══════════════════════════════════════════════════════════════════════════════════════════════════╝
  ```
* **Mục tiêu:** Đo lường chính xác độ lệch sau mỗi 4h để biết ngay chu kỳ 4h nào bắt đầu phân kỳ.

---

## 🔇 3. Quy Tắc Giữ Sạch Log (Filter Constraint)

Tuyệt đối không in log từ bên trong GA Optimizer (`ferro_trade::optimizer::prospector` và `ga_worker`):
* Thiết lập `EnvFilter` đảm bảo chỉ in `info` cho `walk_forward_smart_seed`, `paper_trader`, `asset_actor`, `helios_bridge`.
* Các crate tính toán GA giữ nguyên mức `warn` hoặc `error`.
