# ⚖️ 03 — Phân Tích Arbiter & Matrix Multiplier

> Files: `src/strategy/core_v3/arbiter.rs`, `src/strategy/core_v3/matrix.rs`

---

## I. ARBITER — Cơ Chế Lựa Chọn Tín Hiệu

### Code Logic Hiện Tại

```rust
// arbiter.rs
let confidence = if multiplier < min_veto {
    0.0 // Veto hoàn toàn
} else {
    (proposal.raw_score * s_weight) + (multiplier * m_weight)
};

// Chọn scanner có confidence cao nhất
if confidence > max_confidence {
    max_confidence = confidence;
    best_proposal = Some((proposal, confidence));
}

// Phê duyệt nếu confidence >= min_approval_confidence
if confidence >= self.min_approval_confidence { ... }
```

### Lỗi / Quan Sát

#### A-01: Arbiter Chỉ Chọn 1 Scanner — Không Có Cơ Chế Multi-Confirmation

Mỗi tick, Arbiter thu thập tất cả Proposals rồi **chỉ chọn 1 scanner có confidence cao nhất**.
Không có cơ chế xét thêm:
- Bao nhiêu scanner đồng thuận cùng hướng?
- Scanner nào có confidence cao nhưng contradicting direction?

**Hệ quả:** ABSORPTION luôn thắng vì `absorption_multiplier` thường cao (volume force + squeeze luôn có giá trị) và `raw_score > 0` dễ đạt.

#### A-02: `min_multiplier_veto` Không Cân Bằng Giữa Các Scanner

```rust
let min_veto = config.arbiter.min_multiplier_veto;
```

Một ngưỡng duy nhất `min_veto` áp dụng cho tất cả scanner. Nhưng mỗi scanner có multiplier từ các nguồn khác nhau:
- ABSORPTION: `absorption_multiplier = squeeze * abs_sqz_w + volume * (1-abs_sqz_w)` → thường 0.3–0.7
- REVERSAL: `reversal_multiplier = (1-t_mul) * (momentum_extremity + z_abs)` → rất thấp khi market trending

Nếu `min_veto = 0.3`, ABSORPTION dễ vượt qua trong khi REVERSAL luôn bị veto trong trend.

#### A-03: Scanner Weight (`s_weight`) Ảnh Hưởng Không Đồng Đều

```rust
let confidence = (proposal.raw_score * s_weight) + (multiplier * m_weight)
```

- Nếu `s_weight = 0.3`, `m_weight = 0.7`
- ABSORPTION với `raw_score = 0.1` và `multiplier = 0.6`:
  `confidence = 0.1*0.3 + 0.6*0.7 = 0.03 + 0.42 = 0.45`
- REVERSAL với `raw_score = 0.9` và `multiplier = 0.2`:
  `confidence = 0.9*0.3 + 0.2*0.7 = 0.27 + 0.14 = 0.41`

ABSORPTION thắng dù REVERSAL có raw_score tốt hơn nhiều — **Multiplier chiếm ưu thế**.

---

## II. MATRIX MULTIPLIER — Công Thức Chưa Tối Ưu

### Code Logic Hiện Tại (matrix.rs)

```rust
// Trend
let t_mul = trend_strength[i];
trend_multiplier.push(t_mul);

// Range = nghịch đảo của Trend
range_multiplier.push(1.0 - t_mul);

// Anomaly = Z-Score + Chaos
let anomaly_score = (z_abs * ano_z_w) + (chaos_ratio[i] * (1.0 - ano_z_w));

// Reversal = (1 - Trend) * (Momentum Extremity + Z-Score)
let rev_score = (1.0 - t_mul) * ((momentum_extremity * rev_m_w) + (z_abs * (1.0 - rev_m_w)));

// Absorption = Squeeze + Volume
let abs_score = (squeeze_intensity[i] * abs_sqz_w) + (volume_force[i] * (1.0 - abs_sqz_w));

// Volatility Expansion = Chaos + Volume
let vol_exp_score = (chaos_ratio[i] * vol_chaos_w) + (volume_force[i] * (1.0 - vol_chaos_w));
```

### Lỗi / Quan Sát

#### M-01: Range = 1 - Trend — Quá Đơn Giản

`range_multiplier = 1.0 - trend_strength`

Công thức này nghĩa là: **bất cứ khi nào trend yếu là range tốt**. Nhưng market yếu trend có thể đang trong trạng thái chaos, breakout chuẩn bị, hoặc accumulation — không phải range thực sự. Range thực sự cần thêm:
- BB width ở mức trung bình (không quá hẹp, không quá rộng)
- Bounce rõ ràng ở biên BB

#### M-02: Reversal Multiplier Bị Triệt Tiêu Bởi Trend

```rust
rev_score = (1.0 - t_mul) * (...)
```

Khi `t_mul = 0.8` (trend mạnh), toàn bộ reversal_multiplier bị nhân với `0.2` — **giảm 80% giá trị**. Điều này có nghĩa:
- Trong bull run mạnh, reversal signal cực kỳ khó vượt `min_approval_confidence`
- Nhưng thực tế, reversal **chính xác nhất** khi trend đang cực đoan

Đây là **xung đột triết lý thiết kế** — hệ thống ưu tiên theo trend nhưng lại muốn bắt reversal.

**Đề xuất:** Tách `reversal_multiplier` thành 2 thành phần:
- Khi trend **bình thường** → dùng `(1 - t_mul) * ...` (hiện tại)
- Khi trend **cực đoan** (`t_mul > 0.9`) → reversal score tăng, không giảm

#### M-03: Absorption và Volatility Expansion Overlapping

- ABSORPTION: `squeeze * w + volume * (1-w)`
- VOL_EXPANSION: `chaos_ratio * w + volume * (1-w)`

Cả hai đều dùng `volume_force`. Khi volume cao:
- Cả hai multiplier đều cao
- Cả hai scanner (ABSORPTION và MOMENTUM_IGN) đều có khả năng kích hoạt cùng lúc
- Arbiter chọn 1 → tùy thuộc vào raw_score thô

Không có **mutual exclusion logic** giữa hai scanner này.

#### M-04: Squeeze Multiplier Không Cần Thiết Cho Arbiter

```rust
squeeze_multiplier.push(squeeze_intensity[i]);
```

`squeeze_multiplier` được tạo ra nhưng trong Arbiter:
```rust
let multiplier = match scanner.id() {
    "TREND_PB" => matrix.trend_multiplier[i],
    "MOMENTUM_IGN" => matrix.volatility_expansion_multiplier[i],
    "RANGE_OSC" => matrix.range_multiplier[i],
    "ANOMALY" => matrix.anomaly_multiplier[i],
    "REVERSAL" => matrix.reversal_multiplier[i],
    "ABSORPTION" => matrix.absorption_multiplier[i],
    _ => 1.0,
};
```

`squeeze_multiplier` không được gán cho bất kỳ scanner nào trong Arbiter. **Dead field**.

---

## III. MISSING: Không Có Market Context Gate

V3 không có khái niệm **Environment Fuse** như V2:
```rust
// V2: analyze_environment -> StabilityIndex -> chặn giao dịch khi unstable
let env = core_v2::sensor::analyze_environment(...);
```

V3 hoàn toàn phụ thuộc vào `min_approval_confidence` để lọc — nhưng không có cơ chế block hard khi thị trường đang ở trạng thái nhiễu (chaos) cao.

---

## Bảng Tổng Hợp

| ID   | Vấn Đề                                              | Mức Độ    |
|------|-----------------------------------------------------|-----------|
| A-01 | Arbiter không có multi-confirmation                 | 🔴 NGUY   |
| A-02 | `min_multiplier_veto` một ngưỡng cho tất cả scanner | 🟡 QUAN   |
| A-03 | Scanner Weight ưu tiên Multiplier hơn raw_score     | 🟡 QUAN   |
| M-01 | Range = 1 - Trend: quá đơn giản                    | 🟡 QUAN   |
| M-02 | Reversal Multiplier bị triệt tiêu khi Trend mạnh   | 🔴 NGUY   |
| M-03 | ABSORPTION và VOL_EXPANSION overlapping trên Volume | 🟡 QUAN   |
| M-04 | `squeeze_multiplier` là dead field                  | 🟠 DEAD   |
| M-05 | Không có Environment Gate (như V2 Fuse)            | 🟡 QUAN   |
