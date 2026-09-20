# 🆕 BÁO CÁO PHÁT HIỆN MỚI TỪ CROSS-AUDIT (Chưa có trong audit_01 ban đầu)

**Ngày phát hiện:** 2026-09-19  
**Nguồn gốc:** Cross-Audit đối chiếu source code thực tế vs tài liệu audit_01  
**Trạng thái:** Toàn bộ đều chưa được vá, chưa có trong bất kỳ TODO.md nào trước đây.

> [!IMPORTANT]
> Đây là các lỗ hổng BỔ SUNG, không thay thế các lỗi đã có trong audit_01.
> Kết hợp với audit_01, tổng số lỗ hổng chưa vá là **28 lỗi (18 P0/P1 + 10 P2/P3)**.

---

## 1. VULN-AA-02 — Live Cooldown Bypass trên Live Engine

- **File:** `trading_bot/bot_engine/src/asset_actor.rs` (khoảng dòng 276-284)
- **Mức độ:** 🟠 P1 (HIGH)
- **Mô tả:**
  Khi sàn Binance đóng lệnh ngoài ý muốn (TP/SL chạm), `AssetActor` nhận tín hiệu từ WebSocket nhưng **không cập nhật `closed_intra_candle = true`**. Kết quả: Strategy không biết lệnh đã đóng, bỏ qua việc kích hoạt Cooldown, và có thể mở lệnh mới ngay lập tức ở nến tiếp theo — vi phạm hoàn toàn cơ chế bảo vệ thua lỗ liên tiếp.
- **Tác động:** Trên Live Engine, Cooldown bị bypass mỗi khi lệnh bị đóng bởi sàn (không phải bởi Strategy).

---

## 2. VULN-AA-03 — MTF 4h Warmup Starvation & Resampling Shift

- **File:** `trading_bot/bot_engine/src/asset_actor.rs` và `prospector_runner.rs`
- **Mức độ:** 🟠 P1 (HIGH)
- **Mô tả:**
  Trong `prospector_runner.rs`, nếu số nến vừa đủ 8,640 (tối thiểu), việc cắt ra OOS slice khiến tập In-Sample (Train) bị thu hẹp. Với khung 4h cần ~144,000 nến 1m để warmup đủ, phần Train thực chất có thể **bị ăn hết bởi warmup** → `return <= 0.0` → WFE Gate luôn rớt mà không có thông báo rõ ràng.
  
  **Công thức minh họa:**
  - Train slice = 8,640 × 0.75 = 6,480 nến
  - Warmup cần cho chỉ báo 4h với z_score_lookback=600: `600 × 240 = 144,000` nến → **NHIỀU HƠN TOÀN BỘ DỮ LIỆU**
  - Kết quả: backtest train chạy trên 0 nến hữu ích → return = 0 → Gate fail
- **Tác động:** Cold-Start Starvation kép — vừa do WFE Gate, vừa do cấu hình z_score_lookback quá lớn.

---

## 3. ARB-HARDCODE — Ngưỡng Macro Veto Bị Hardcode Cứng (Không DNA-Mappable)

- **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:128,116`
- **Mức độ:** 🟠 P1 (HIGH)
- **Mô tả:**
  ```rust
  // arbiter.rs:128
  let macro_thresh = 0.10;  // HARDCODE — không phải tham số DNA!
  
  // arbiter.rs:116
  if trend_axis >= 0.40 && ... { // HARDCODE — không phải tham số DNA!
  ```
  Hai ngưỡng quyết định Macro Veto (`macro_thresh = 0.10` và `trend_axis >= 0.40`) bị đóng chết trong code. Optimizer **không thể điều chỉnh** các ngưỡng này để thích nghi với từng asset hoặc từng chế độ thị trường.
  
  **Tại sao là lỗi thiết kế?** Theo triết lý Helios: mọi threshold nên là tham số DNA. Với BTC (biến động lớn), `macro_thresh = 0.10` có thể quá nhạy. Với stablecoin, nó có thể quá lờ.
- **Giải pháp:** Chuyển cả hai vào `ArbiterConfig` với `#[dna(...)]`.

---

## 4. SENSOR-DEAD-DATA — `kama_slope` Raw Là Dead Data (Dữ Liệu Tính Ra Nhưng Không Dùng)

- **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:166`  
  `strategies_backtests/helios/src/strategy/sensors.rs`
- **Mức độ:** 🔵 P2 (MEDIUM)
- **Mô tả:**
  ```rust
  // mod.rs:166 — Tính kama_slope raw
  s.kama_slope = k_slope;  // Tính xong gán vào sensors
  ```
  Nhưng trong **toàn bộ codebase**, không có dòng nào đọc `sensors.kama_slope`. Mọi nơi chỉ dùng `sensors.kama_slope_adj` (phiên bản đã chuẩn hóa qua ATR).
  
  **Tác động:** Lãng phí tính toán, ô nhớ, và tạo nhầm lẫn khi đọc code.
- **Giải pháp:** Xóa trường `kama_slope` khỏi `HeliosSensors`, chỉ giữ `kama_slope_adj`.

---

## 5. TREND-PB-ISOLATION — TrendPB Bị Cô Lập Vì SL An Toàn Hơn Bị Arbiter Loại

- **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:175-182`
- **Mức độ:** 🔴 P0 (CRITICAL — là hệ quả của VULN-ARB-01)
- **Mô tả:**
  `TrendPB` là scanner được thiết kế tốt nhất (logic pullback theo KAMA, Stochastic hạ nhiệt, RSI filter). **SL của nó rộng hơn vì an toàn hơn** (ví dụ: `struct_sl_buffer_atr = 1.2 ATR`).
  
  Do `VULN-ARB-01` dùng `dist_new < dist_curr` (SL hẹp nhất thắng), TrendPB luôn thua Momentum (SL `0.05 ATR`) trong đấu thầu dù Confidence của TrendPB cao hơn nhiều.
  
  **Nghịch lý:** Scanner tốt nhất → SL an toàn nhất → bị loại bởi scanner tệ nhất có SL siêu mỏng.
  
  **Tác động trực tiếp:** Mọi lệnh thực tế đều từ Momentum (SL `0.05 ATR`) → bị quét SL sau 2-3 nến → return = 0 trong WFE.
- **Ghi chú:** Được giải quyết hoàn toàn khi sửa `VULN-ARB-01` sang Confidence-First.

---

## 6. FLAW-RISK-4 — Min Notional Không Được Kiểm Tra

- **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:18-25`
- **Mức độ:** 🟡 P2 (MEDIUM)
- **Mô tả:**
  Khi vốn nhỏ (ví dụ `$30/asset`), hàm `get_position_size()` tính size hợp lệ theo tỷ lệ `max_cap_fraction = 25%`. Nhưng kết quả cuối `size_usd` có thể < `$21` (Min Notional của Binance Futures). Lệnh sẽ bị Binance **reject** với lỗi `MIN_NOTIONAL`, nhưng code không xử lý trường hợp này — lệnh bị mất im lặng.
  
  **Công thức vi phạm:**
  - Capital = $30, leverage = 5x, max_cap = 25% → size_usd = $30 × 0.25 = $7.5 × 5 = $37.5 → OK
  - Capital = $30, leverage = 1x, max_cap = 25% → size_usd = $30 × 0.25 × 1 = $7.5 → DƯỚI MIN ($21) → Binance reject!
- **Giải pháp:** Sau khi tính size, kiểm tra `if size_usd < MIN_NOTIONAL_USD { return 0.0; }` để không đặt lệnh.

---

## 7. CONFIG-SERDE-DEFAULT — Thiếu `#[serde(default)]` Trong Config Structs

- **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`
- **Mức độ:** 🔵 P2 (MEDIUM)
- **Mô tả:**
  Các struct cấu hình trong `config.rs` không có attribute `#[serde(default)]` ở cấp độ struct. Khi nạp file JSON cấu hình thiếu một trường mới (ví dụ sau khi thêm tham số DNA mới), Serde sẽ **panic** thay vì dùng giá trị mặc định.
  
  **Tình huống xảy ra:** Deploy phiên bản mới Helios với DNA mới, nhưng vẫn dùng file `.json` cũ từ DB → Crash ngay khi load.
- **Giải pháp:** Thêm `#[serde(default)]` cho mọi struct con trong `HeliosConfigV3`.

---

## 8. ACTOR-PENDING-DNA-LOCK — DNA Khẩn Cấp Bị Chặn Khi Đang Giữ Vị Thế

- **File:** `trading_bot/bot_engine/src/asset_actor.rs:276-284`
- **Mức độ:** 🔵 P2 (MEDIUM)
- **Mô tả:**
  Khi đang giữ vị thế và nhận DNA update mới từ Optimizer, `AssetActor` lock lại và lưu vào `pending_dna` — không apply ngay. Điều này được thiết kế để tránh thay đổi cấu hình giữa chừng của một lệnh đang mở.
  
  **Vấn đề:** Nếu DNA mới là bản **vá khẩn cấp** (ví dụ: TSL quá rộng đang gây lỗ nặng, người dùng muốn thắt lại ngay), nó sẽ bị lờ đi cho đến khi lệnh đóng tự nhiên.
  
  **Tác động:** Không nghiêm trọng trong điều kiện bình thường, nhưng nguy hiểm trong điều kiện thị trường cực đoan.
- **Giải pháp tạm:** Bổ sung lệnh force-apply DNA cho người dùng qua UI (có xác nhận cảnh báo rủi ro).

---

## 📊 Bảng Tổng Hợp Phát Hiện Mới

| Mã lỗi | File | Mức | Được giải quyết khi |
| :--- | :--- | :---: | :--- |
| `VULN-AA-02` | `asset_actor.rs` | P1 | Giai đoạn 1 — Ghi nhận tín hiệu close từ sàn |
| `VULN-AA-03` | `prospector_runner.rs` | P1 | Giai đoạn 4 — Giới hạn `z_score_lookback` tối đa |
| `ARB-HARDCODE` | `arbiter.rs:128,116` | P1 | Giai đoạn 2 — Chuyển sang DNA-mappable |
| `SENSOR-DEAD-DATA` | `mod.rs:166` | P2 | Giai đoạn 3 — Xóa trường zombie |
| `TREND-PB-ISOLATION` | `arbiter.rs:175-182` | P0 | Giai đoạn 2 — Hệ quả khi sửa VULN-ARB-01 |
| `FLAW-RISK-4` | `risk.rs:18-25` | P2 | Giai đoạn 4 — Kiểm tra Min Notional |
| `CONFIG-SERDE-DEFAULT` | `config.rs` | P2 | Giai đoạn 3 — Thêm `#[serde(default)]` |
| `ACTOR-PENDING-DNA-LOCK` | `asset_actor.rs:276-284` | P2 | Giai đoạn 4 — UI force-apply option |
