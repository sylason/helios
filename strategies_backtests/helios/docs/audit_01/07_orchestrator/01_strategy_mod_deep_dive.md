# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU: BỘ ĐIỀU PHỐI CHIẾN LƯỢC TRUNG TÂM (`mod.rs`)

> **File kiểm toán:** [`strategies_backtests/helios/src/strategy/core_v3/mod.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)  
> **Cấu trúc:** `HeliosStrategyV3` (Triển khai Trait `Strategy` của `ferro_trade`), `CooldownTracker`, `telemetry`  
> **Định vị kiến trúc:** Strategy Core tập trung 100% vào việc sinh tín hiệu Alpha. Quản trị khớp lệnh và vị thế thực tế thuộc về Bot Engine.

---

## 🧭 1. ĐỊNH VỊ VAI TRÒ KIẾN TRÚC & RANH GIỚI TRÁCH NHIỆM

Theo nguyên tắc phân định trách nhiệm hệ thống:
1. **`HeliosStrategyV3` (Alpha & Signal Engine):**
   - Tiền tính toán cảm biến MTF trong `init()`.
   - Điều phối luồng nến trong `next()`: Cập nhật Trailing Stop/TP qua `RiskManager`, chạy Gatekeeper & Arbiter, tính toán kích thước vị thế và kẹp SL.
   - **Ranh giới:** Không can thiệp vào quản lý tài khoản, số dư thực tế hay Cầu Dao cấp sàn.
2. **`Bot Engine` / `AssetActor` (Execution & Account Defense):**
   - Độc quyền sở hữu logic Cầu Dao Cooldown (`EngineCooldownGuard`) dựa trên Realized PnL thực tế của sàn Binance.
   - Tiếp quản vị thế (Adopt & Protect), đồng bộ SL đám mây, kiểm soát API Weight và bảo vệ vốn sàn (Base Floor).

---

## 🔬 2. MỔ XẺ CÁC LỖ HỔNG & ĐIỂM CẦN KHẮC PHỤC TRONG `mod.rs`

### 🔍 VẤN ĐỀ 1: LÀM RÕ KIẾN TRÚC COOLDOWN & DỌN SẠCH CODE THỪA
- **Hiện trạng:**
  - Khối code kiểm tra `cooldown_tracker.should_block(...)` tại dòng 404–426 đã được comment out có chủ đích vì **Cầu Dao Cooldown thuộc về trách nhiệm của Bot Engine (`AssetActor`)**.
  - Tuy nhiên, trong `mod.rs` vẫn tồn tại struct `CooldownTracker` (dòng 31–100) và logic đếm nến/chuỗi thua (dòng 296–361).
- **Hệ quả & Đề xuất:**
  - Code bị thừa (dead code một phần), gây hiểu nhầm về luồng vận hành.
  - Cần dọn sạch các biến và logic Cooldown thừa thãi trong `HeliosStrategyV3` để Strategy thuần khiết 100% làm nhiệm vụ sinh tín hiệu Alpha, nhường toàn quyền Cooldown cho Bot Engine.

---

### ⚠️ LỖ HỔNG 2: SỰ LÃNG PHÍ TÍNH TOÁN CỦA CHỈ BÁO ZOMBIE TRONG `init()`
- **Vị trí code:** [`mod.rs` dòng 253–272](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L253-L272).
- **Hiện trạng code:**
  - Hàm `init()` chạy vòng lặp duyệt qua toàn bộ lịch sử nến để tính độ dốc trục giữa Bollinger Bands:
    ```rust
    let raw_slope = (m_curr - m_prev) / (k_bars as f64 * atr);
    bb_slope.push(raw_slope.clamp(-2.0, 2.0));
    ```
    và lưu vào `s.bb_middle_slope_adj`.
- **Hệ quả:**
  - Biến này vốn chỉ dùng cho `RangeOscillator` cũ (đã bị tắt). Hiện tại 4 scanners chủ lực không hề đọc tới!
  - Cấp phát `Arc<Vec<f64>>` vô ích làm lãng phí RAM và CPU trong mỗi chu kỳ chạy của GA Optimizer.

---

### ⚠️ LỖ HỔNG 3: MAGIC NUMBER `0.15` & GỌI KÉP KẸP STOP LOSS (LIÊN KẾT `risk.rs` & `config.rs`)
- **Vị trí code:** [`mod.rs` dòng 469–484](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L469-L484).
- **Hiện trạng code:**
  ```rust
  let hard_cap_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, 0.15, min_initial_sl_pct, leverage);
  let is_hard_clamped = (raw_sl - hard_cap_sl).abs() > 1e-4;
  let is_playbook_clamped = (raw_sl - initial_sl).abs() > 1e-4;
  ```
- **Hệ quả & Đối chiếu Single Source of Truth (SSOT):**
  - Số `0.15` (15% ROE) bị gán cứng trực tiếp trong câu lệnh, vi phạm nguyên tắc đưa mọi hằng số/DNA vào `config.rs`.
  - Phép tính `clamp_sl_price` bị gọi lặp lại 2 lần chỉ để phân biệt log cho Sổ cái (`is_hard_clamped` vs `is_playbook_clamped`).
  - **Liên kết chéo:**
    - [Báo cáo Risk Manager (`05_risk/01_risk_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md): `GLOBAL_HARD_MAX_ROE` cần được cấu hình hóa.
    - [Báo cáo Config (`06_config/01_config_and_hardcode_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md): Trần ROE phải nằm trong struct `RiskStopParams`.

---

### ⚠️ LỖ HỔNG 4: TYPE-SAFETY CHO POSITION SIZING TRONG `ferro_trade` & `helios`
- **Vị trí code:** [`mod.rs` dòng 429–435 & dòng 494–496](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L429-L496).
- **Hiện trạng code:**
  - `RiskManager::calculate_size` trả về `f64` trần trụi (tỷ lệ phần trăm vốn, ví dụ `0.20`).
  - Truyền trực tiếp vào `tick.buy(Some(size), ...)` mà không có enum phân định ngữ nghĩa.
- **Hệ quả:**
  - Phía engine `ferro_trade` hiện chưa hỗ trợ struct/enum tường minh cho kiểu lệnh (% vốn vs số coin vs notional USD).
  - Khi mở rộng sang các sàn ngoài hoặc Forex, việc dùng `f64` không có Type Wrapper dễ dẫn đến lỗi đặt nhầm khối lượng.
  - Cần ghi nhận để nâng cấp đồng bộ ở cả tầng `ferro_trade` và `helios`.

---

## 📊 3. BẢNG TỔNG HỢP GIẢI PHÁP ĐỀ XUẤT

| Thành phần | Hiện trạng | Giải pháp Đề xuất | Thư mục Liên quan |
| :--- | :--- | :--- | :--- |
| **Cooldown trong Strategy** | Code comment out + struct thừa | Dọn sạch `CooldownTracker` trong `mod.rs`. Khẳng định Cooldown thuộc quyền Bot Engine | `07_orchestrator/` |
| **Chỉ báo Zombie** | Tính `bb_middle_slope_adj` thừa | Xóa bỏ vòng lặp tính toán trong `init()`, giải phóng tài nguyên | `04_matrix/`, `07_orchestrator/` |
| **Magic Number 0.15 & Gọi kép** | Hardcode `0.15` và gọi 2 lần `clamp_sl_price` | Chuyển trần ROE thành SSOT trong `config.rs`. Hàm clamp trả về struct `ClampedSlResult` | `05_risk/`, `06_config/`, `07_orchestrator/` |
| **Position Sizing Type** | `f64` không type-safe | Bổ sung Type-Safe Enum `OrderSize::CapitalFraction` bên `ferro_trade` và đồng bộ sang `helios` | `ferro_trade`, `07_orchestrator/` |
