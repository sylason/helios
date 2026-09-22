# 📋 TODO — PHÁT HIỆN MỚI TỪ CROSS-AUDIT

**Module:** Toàn hệ thống (phát hiện bổ sung sau đợt đối chiếu code)  
**Tài liệu kiểm toán:** [`technical_audit.md`](./technical_audit.md)  
**Ngày cập nhật:** 2026-09-22

---

## 🔴 P0 — CRITICAL

- [ ] **TREND-PB-ISOLATION:** TrendPB bị cô lập vì Arbiter loại nó do SL an toàn hơn — **tự động giải quyết khi sửa VULN-ARB-01** (Giai đoạn 2)

- [ ] **VULN-AA-09: Same-Side Price Guard Deadlock** — `bot_engine/src/controller/asset_actor.rs:73-85`
  - `Should_veto` Khối 1 không giới hạn trong cooldown → chặn vĩnh viễn sau hết cooldown → Deadlock.
  - **Bằng chứng log:** `WFSS_arbiter_weakpoint2.log` — ENGINE COOLDOWN VETO kéo dài 2.64 ngày / 280 lần.
  - **Giải pháp:** Đưa Khối 1 vào trong `if current_bar_idx < locked_until_bar`. Thêm `enum CooldownVetoReason`.
  - Chi tiết: [`11_asset_actor/TODO.md#P0-3`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/11_asset_actor/TODO.md)

---

## 🟠 P1 — HIGH

- [ ] **VULN-AA-02:** Live Cooldown Bypass — sàn đóng lệnh không cập nhật `closed_intra_candle`
  - File: `asset_actor.rs:276-284`
  - Giai đoạn: 1

- [ ] **VULN-AA-03:** MTF 4h Warmup Starvation — WFE Gate rớt do `z_score_lookback` quá lớn
  - File: `prospector_runner.rs:325-349`
  - Cần: Giới hạn `z_score_lookback` tối đa theo kích thước window (ví dụ max 100 thay vì 600)
  - Giai đoạn: 4

- [ ] **ARB-HARDCODE:** `macro_thresh = 0.10` và `trend_axis >= 0.40` bị hardcode trong Arbiter
  - File: `arbiter.rs:128, 116`
  - Cần: Chuyển vào `ArbiterConfig` với `#[dna(min=0.05, max=0.30, step=0.01)]`
  - Giai đoạn: 2

- [ ] **VULN-AA-10: ATR Mismatch Engine vs Helios** — `asset_actor.rs:655`
  - Engine dùng `rolling_atr(14 nến 1m) ≈ 2–5 USDT`. Helios dùng `sensors.atr_sl` (4h ATR ≈ 50–100 USDT).
  - Ngưỡng Early Release Engine thấp hơn ~10–20x → hai tầng không đồng bộ ngưỡng bứt phá.
  - **Giải pháp:** Truyền `current_atr` từ HeliosBridge xuống EngineCooldownGuard.
  - Chi tiết: [`11_asset_actor/TODO.md#P1-4`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/11_asset_actor/TODO.md)

- [ ] **VULN-AA-11: ClosePosition Không Dùng Exponential Cooldown** — `asset_actor.rs:729-736`
  - Nhánh `Action::ClosePosition` gán thẳng `base_bars` mà không dùng exponential, không tăng streak.
  - **Giải pháp:** Hàm `trigger_engine_cooldown(pnl, side, exit_price)` thống nhất cho cả 2 nhánh.
  - Chi tiết: [`11_asset_actor/TODO.md#P1-5`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/11_asset_actor/TODO.md)

- [ ] **ABS-SL-DOUBLE-PADDING: SL Double Padding trong Absorption Scanner** — `absorption.rs:122-133`
  - `sl = min(low, bb_lower) - (struct_sl_buffer_atr * ATR)` → SL cách entry 4–5 ATR.
  - Kết hợp `max_initial_sl_pct` trần 40% ROE → ROE thua trung bình -26% → R:R âm ~1:0.43.
  - **Giải pháp:** Loại bỏ 1 tầng đệm: hoặc neo SL theo `low - buffer` (bỏ bb_lower), hoặc nếu neo theo `bb_lower` thì buffer tối đa 0.2 ATR. Giảm trần `max_initial_sl_pct` xuống 15–20% ROE.
  - Chi tiết: [`01_absorption/TODO.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_absorption/TODO.md)

---

## 🟡 P2 — MEDIUM

- [ ] **SENSOR-DEAD-DATA:** `kama_slope` raw được tính nhưng không bao giờ được đọc
  - File: `mod.rs:166`, `sensors.rs`
  - Cần: Xóa trường `kama_slope` khỏi `HeliosSensors`
  - Giai đoạn: 3

- [ ] **FLAW-RISK-4:** Khi size_usd < Min Notional ($21) Binance reject im lặng
  - File: `risk.rs:18-25`
  - Cần: Thêm guard `if size_usd < MIN_NOTIONAL_USD { return 0.0; }`
  - Giai đoạn: 4

- [ ] **CONFIG-SERDE-DEFAULT:** Thiếu `#[serde(default)]` → panic khi JSON thiếu trường
  - File: `config.rs`
  - Cần: Thêm `#[serde(default)]` cho tất cả struct con của `HeliosConfigV3`
  - Giai đoạn: 3

- [ ] **ACTOR-PENDING-DNA-LOCK:** DNA vá khẩn cấp bị chặn khi đang giữ vị thế
  - File: `asset_actor.rs:276-284`
  - Cần: Thêm nút force-apply DNA trong UI với cảnh báo rủi ro
  - Giai đoạn: 4

- [ ] **ABS-DNA-FLOOR-TOO-LOOSE: DNA Absorption quá lỏng gây Playbook Bias** — `config.rs`
  - `min_clv = 0.00`, `vol_z = 0.20`, `min_activation_score = 0.01` → Absorption spam tín hiệu mọi lúc.
  - GA bị kẹt cực trị cục bộ → ABSORPTION chiếm 90% lệnh, 5 playbook còn lại tê liệt hoàn toàn.
  - **Giải pháp:** Siết cận dưới DNA: `min_activation_score >= 0.20`, `min_clv >= 0.15`, `vol_z >= 1.0`.
  - **Bằng chứng log:** `WFSS_arbiter_weakpoint2.log` — 9/10 lệnh là ABSORPTION.
  - Chi tiết: [`01_absorption/TODO.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_absorption/TODO.md)
