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
