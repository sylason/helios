# 📋 TODO — PHÁT HIỆN MỚI TỪ CROSS-AUDIT

**Module:** Toàn hệ thống (phát hiện bổ sung sau đợt đối chiếu code)  
**Tài liệu kiểm toán:** [`technical_audit.md`](./technical_audit.md)  
**Ngày cập nhật:** 2026-09-19

---

## 🔴 P0 — CRITICAL

- [ ] **TREND-PB-ISOLATION:** TrendPB bị cô lập vì Arbiter loại nó do SL an toàn hơn — **tự động giải quyết khi sửa VULN-ARB-01** (Giai đoạn 2)

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
