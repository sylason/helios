# 📋 TODO: KIỂM SOÁT PLAYBOOK BẮT ĐÁY & EXPONENTIAL COOLDOWN

---

## 🎯 Mục Tiêu Cốt Lõi
- Xử lý dứt điểm hiện tượng `ANOMALY` cướp thầu và bắt dao rơi 12 lệnh liên tiếp.
- Triển khai bộ nhớ chuỗi thua `Losing Streak Memory` và thời gian phạt lũy tiến `Exponential Cooldown`.
- Đảm bảo cơ chế tự động reset về 0 khi có lệnh thắng, chặn trần thời gian tối đa chống khóa vô hạn.
- Rà soát và giải phóng tiềm năng của `MOMENTUM_IGNITION`.

---

## 🛠️ Danh Sách Nhiệm Vụ Baby Steps

- [x] **Bước 1: Khởi Tạo Hồ Sơ Kỹ Thuật & Nghiên Cứu Chuyên Sâu**
  - [x] Tạo folder `docs/08_anomaly_reversal_and_streak_cooldown/`.
  - [x] Viết `01_ANOMALY_VS_REVERSAL_DEEP_DIVE.md` (Phân tích bản chất 2 scanner & mổ xẻ Trade #005).
  - [x] Viết `02_EXPONENTIAL_COOLDOWN_AND_STREAK_SPEC.md` (Đặc tả toán học Exponential Cooldown).
  - [x] Viết `03_DIRECTIONAL_MACRO_VETO_SPEC.md` (Đặc tả bộ lọc vĩ mô đối xứng 2 chiều).
  - [x] Viết `04_GLOBAL_MARKET_REGIME_MATRIX_AND_PLAYBOOK_TOPOLOGY.md` (Ma trận toàn cảnh trạng thái thị trường & định tuyến 4 Playbooks).
  - [x] Viết `TODO.md` (Kế hoạch hành động).
  - [x] Tạo `Errors.md` ghi nhận lỗi logic thiết kế.

- [x] **Bước 2: Thảo Luận & Chốt Quyết Định Kiến Trúc Với Người Dùng**
  - [x] Thống nhất tắt `ANOMALY` và `RANGE_OSC` khỏi Arbiter, tập trung 4 Playbook chủ lực.
  - [x] Thống nhất thông số Exponential Cooldown (Base 15 nến, trần Max Cap 60 nến, reset khi thắng).
  - [x] Thống nhất cơ chế Directional Macro Veto đối xứng 2 chiều.
  - [x] Thống nhất Neo Khung Vĩ Mô (Macro Anchor): Khóa cận dưới KAMA $\ge 15\text{m}$.
  - [x] Thống nhất giữ nguyên quy tắc "Khóa DNA cũ cho vị thế đang mở (Lock DNA on Active Trade)".

- [x] **Bước 3: Tinh Giản Không Gian DNA & Neo Khung Vĩ Mô (`config.rs`)**
  - [x] Neo Khung Vĩ Mô (Macro Anchor): Khóa dải gen `ind.kama.timeframe` tối thiểu $\ge 15\text{m}$ (chỉ chọn giữa 15m, 1h, 4h).
  - [x] Comment out các gen DNA của `ANOMALY` (~10 gen) và `RANGE_OSC` (~8 gen) trong `config.rs`.
  - [x] Khử tắc nghẽn cho `MOMENTUM_IGNITION`: Điều chỉnh dải gen `vol_z_threshold` từ $[0.20, 5.00]$ về $[0.10, 1.00]$.
  - [x] Tính toán và cập nhật lại hằng số `DNA_WIDTH` chính xác cho `HeliosConfigV3` (chính xác 95 chiều).

- [x] **Bước 4: Nâng Cấp `HeliosArbiter` & Bảo Vệ Xu Hướng Vĩ Mô (`arbiter.rs`)**
  - [x] Triển khai **Directional Macro Veto**:
    - Khi $\delta_{\text{KAMA}} < -\text{thresh} \land \text{trend\_axis} \ge 0.40$ (Thác đổ): CẤM MỞ LỆNH LONG (Veto tuyệt đối Reversal Long / Pullback Long); Mở rộng cửa cho Short.
    - Khi $\delta_{\text{KAMA}} > +\text{thresh} \land \text{trend\_axis} \ge 0.40$ (Dựng cột): CẤM MỞ LỆNH SHORT; Mở rộng cửa cho Long.
  - [x] Loại bỏ `AnomalyScanner` và `RangeOscScanner` khỏi danh sách đăng ký trong `core_v3/mod.rs` (chỉ chạy 4 Playbook chủ lực).
  - [x] Chuẩn hóa `MomentumIgnitionScanner` đọc theo trục trực giao `flow_axis`.

- [x] **Bước 5: Triển Khai Exponential Cooldown & Losing Streak Trong `cooldown_tracker`**
  - [x] Mở rộng struct `CooldownTracker` với `consecutive_losses`, `last_losing_side`, `last_losing_price`.
  - [x] Triển khai phạt lũy tiến: $15 \to 30 \to 60$ nến (Chốt cứng trần `max_cooldown_cap = 60` nến ~ 1 giờ).
  - [x] Triển khai cơ chế Reset toàn bộ về 0 ngay khi có lệnh THẮNG thực tế ($\text{PnL} > 0$).
  - [x] Triển khai `Same-Side Price Guard`: Cấm mở lệnh cùng chiều nếu giá hiện tại tồi tệ hơn giá vừa cắt lỗ.

- [x] **Bước 6: Tối Ưu Freshness `pending_dna` Trong `bot_engine` (`asset_actor.rs`)**
  - [x] Đảm bảo khi vị thế đóng (cả intra-candle SL/TP và account_event), `AssetActor` kích hoạt trực tiếp bản Champion DNA mới nhất.
  - [x] Đồng bộ `EngineCooldownGuard` với Exponential Cooldown, trần 60 nến, reset khi thắng và `Same-Side Price Guard`.

- [x] **Bước 7: Kiểm Thử Toàn Workspace & Thông Báo Commit**
  - [x] Viết bộ unit tests cho Directional Macro Veto và Exponential Cooldown.
  - [x] Chạy `cargo test --lib` trên `strategies_backtests/helios` (49/49 PASSED).
  - [x] Chạy `cargo test --lib` trên `trading_bot/bot_engine` (40/40 PASSED).
  - [x] Thông báo câu lệnh commit thủ công cho người dùng.

---

## 🚨 BƯỚC 8: KHẮC PHỤC NGHỊCH LÝ OPTIMIZER 0 TRADES & WFE GATE FALSE PASS

### 📊 Bảng Đề Xuất Giải Pháp Khắc Phục Triệt Để

| STT | Vấn đề phát hiện | Giải pháp đề xuất | Ưu điểm | Nhược điểm & Rủi ro biên (Edge Cases) |
| :--- | :--- | :--- | :--- | :--- |
| **1** | **Chỉ báo thừa có period = 0 làm hỏng Warmup** | Dọn dẹp dứt điểm `rsi_anomaly` & `stoch_range` khỏi `HeliosStrategyV3::init` và `HeliosSensors`. Chỉ tính toán chỉ báo cho 4 Playbook active. | Triệt tiêu 100% mảng toàn NaN. `warmup_period` trở về giá trị thực (~100 - 300 nến). Backtest chạy bình thường. | Cần đảm bảo không còn scanner nào tham chiếu tới 2 biến này trong struct `HeliosSensors`. |
| **2** | **Lỗ hổng WFE Gate lọt lưới DNA 0 trade** | Siết chặt cổng WFE: Bắt buộc `val_trades >= 1` VÀ `val_ret > 0.0` VÀ `ret > 0.0` VÀ `trades >= min_trades`. Nếu vi phạm $\to$ **REJECT NGAY**. | Không bao giờ deploy DNA rác/chết vào hệ thống thực chiến. | Trong giai đoạn thị trường đóng băng (flat chop), nếu không có lệnh nào trong 48h OOS, bot sẽ giữ nguyên DNA cũ thay vì cập nhật mới. |
| **3** | **Bảo vệ chống tràn Warmup trong `ferro_trade`** | Trong `manager.rs`, nếu một chỉ báo trả về `actual_warmup >= full_len`, bắn `warn!` và không cộng dồn vào `warmup_period`. | Phát hiện ngay lập tức chỉ báo lỗi thay vì âm thầm triệt tiêu vòng lặp của engine. | Cần sửa đổi mã nguồn ở crate `ferro_trade`. |
| **4** | **Bảo vệ Lookback Z-Score MTF** | Khống chế `z_score_lookback` tối đa theo khung thời gian (chẳng hạn khung 4h chỉ cho phép lookback $\le 50$ nến) để không vượt quá chiều dài In-Sample. | Chống trường hợp lookback lớn hơn kích thước cửa sổ trượt. | Cần điều chỉnh khoảng giá trị trong `config.rs`. |

---

### 📋 Checklist Nhiệm Vụ Triển Khai (Baby Steps)

- [x] **8.1 (Blocker Cốt Lõi - Ưu Tiên 1): Loại bỏ triệt để các Indicator mồ côi (`period = 0`)**
  - [x] Xóa/comment out khởi tạo `rsi_anomaly` trong `HeliosStrategyV3::init` (`strategies_backtests/helios/src/strategy/core_v3/mod.rs`).
  - [x] Xóa/comment out khởi tạo `stoch_range` trong `HeliosStrategyV3::init` (`strategies_backtests/helios/src/strategy/core_v3/mod.rs`).
  - [x] Dọn dẹp các trường tương ứng trong `HeliosSensors` nếu không còn dùng.
  - [x] Kiểm tra xác nhận `warmup_period` giảm về mức chuẩn (~100-300 nến thay vì 63,199).

- [x] **8.2 (Phòng Ngự Cổng - Ưu Tiên 1.5): Khóa chặn WFE Validation Gate với DNA 0 Trades**
  - [x] Cập nhật điều kiện WFE Gate trong `prospector_runner.rs`: Chặn deploy nếu `val_trades == 0` hoặc `trades == 0` hoặc `ret <= 0.0`.
  - [x] Đảm bảo chỉ có DNA có lợi nhuận thực tế và có giao dịch hợp lệ mới được phong Champion.

- [x] **8.3 (Kiểm Chứng Tức Thì): Chạy Test Xác Minh `diagnostic_backtest`**
  - [x] Chạy test `diagnostic_backtest` trên dữ liệu nến: Xác nhận `Trades > 0` và `Return != 0.0%` (Đạt 18 trades trong test).
  - [x] Đảm bảo toàn bộ testsuite (`cargo test --lib`) trong cả 2 crate `helios` (49/49) và `bot_engine` (40/40) đều pass 100%.

- [ ] **8.4 (Thực Nghiệm Walk-Forward): Chạy Lại Walk-Forward Test**
  - [ ] Chạy lại `walk_forward_smart_seed.rs` trên ETHUSDT.
  - [ ] Kiểm tra log: Xác nhận GA Optimizer qua từng chu kỳ sinh ra Champion với số lệnh $\ge 7$, Return dương, và WFE Gate đánh giá đúng thực chất.

- [ ] **8.5 (Cải Thiện Độ Vững Chắc Hệ Thống - Giai Đoạn Hoàn Thiện):**
  - [ ] Bổ sung cơ chế bảo vệ chống tràn `warmup_period` trong `ferro_trade::IndicatorManager`.
  - [ ] Rà soát giới hạn trần `z_score_lookback` cho khung MTF KAMA.
