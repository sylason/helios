# 01 — Kiến Trúc Tổng Thể & Luồng Dữ Liệu

## 1. Pipeline 5 Tầng

```
Tầng 1: SENSORS (sensors.rs)
  ├─ 17 chỉ báo kỹ thuật thô (NATR, ADX, RSI, MACD, BB, KC, Stoch, KAMA, LinReg, MFI, ATR, ER...)
  ├─ Dữ liệu: Arc<Vec<f64>> — giá trị gốc từ factory
  └─ Đặc điểm: KHÔNG chuẩn hóa, đơn vị khác nhau (%, degree, absolute...)

        ↓

Tầng 2: MATRIX (matrix.rs)
  ├─ 12 metrics đã chuẩn hóa thành percentile [0.0, 1.0]
  ├─ Rolling Percentile (cửa sổ trượt) cho: squeeze, momentum, liquidity, chaos, entropy, volume_force, bb_width_norm
  ├─ Volatility-Adjusted Slope → trend_dir [-1, 1], trend_angle_adj [0, 1]
  └─ Price Z-Score → tanh → [-1, 1]

        ↓

Tầng 3A: ENVIRONMENT FUSE (sensor.rs) ← Cổng lọc #1
  ├─ 3 điều kiện: chaos < thresh AND volume_force > thresh AND entropy > thresh
  ├─ Bộ đếm kiên nhẫn (state_duration_counter) — tích lũy dần
  ├─ Output: stability_index [0.0, 1.0]
  └─ ⚠️ VOLUME SPIKE PANIC: volume_force > 0.95 → stability = 0.0 ngay lập tức

        ↓

Tầng 3B: REGIME SCORING (regime.rs) ← Cổng lọc #2
  ├─ 6 scores: trend, squeeze, ranging, absorption, anomaly, parabolic
  ├─ Mỗi score dùng tanh() soft-cap
  ├─ Primary Regime = argmax(scores) nếu > 0.1
  └─ Ranging = 1.0 - max(others) → Mặc định khi không có gì nổi bật

        ↓

Tầng 4: PLAYBOOK ARBITRATOR (playbook.rs) ← Cổng lọc #3
  ├─ final_score = max_raw_score × stability_index
  ├─ Nếu final_score ≤ min_score_threshold → Wait
  ├─ Map Regime → Playbook (thêm các điều kiện con: RSI, angle...)
  └─ Output: 1 trong 8 TacticalPlaybook

        ↓

Tầng 5: TACTICAL EXECUTION (tactical.rs) ← Cổng lọc #4
  ├─ Mỗi Playbook có trigger riêng (2-4 điều kiện AND)
  ├─ PHẢI chưa có vị thế (!tick.position().is_open())
  └─ Output: buy() hoặc sell() với SL dựa trên ATR
```

## 2. Vấn Đề Kiến Trúc Cốt Lõi: "Phễu Cạn" (Funnel Drain)

Mỗi tầng là một bộ lọc nhân (`×`), không phải cộng (`+`):

```
P(vào lệnh) = P(env OK) × P(regime thắng) × P(playbook map OK) × P(tactical trigger)
```

Giả sử mỗi tầng cho qua 50% tín hiệu:
- **P = 0.5 × 0.5 × 0.5 × 0.5 = 6.25%**

Thực tế, một số tầng lọc còn khắt khe hơn 50%, khiến xác suất vào lệnh cực thấp.
Đây là **nguyên nhân gốc rễ** khiến chỉ có 81 lệnh trong 43,000+ nến.

## 3. Nhận Xét

| Ưu điểm | Nhược điểm |
|---------|-----------|
| Kiến trúc sạch, phân tầng rõ ràng | Quá nhiều cổng lọc nối tiếp |
| Percentile chuẩn hóa đồng nhất | Các tầng "chà đạp" nhau (chi tiết ở báo cáo #7) |
| Trailing stop thích nghi theo Playbook | Chỉ cho phép 1 vị thế tại 1 thời điểm |
| DNA/Optimizer tối ưu tự động | DNA bounds sai → Optimizer bị "bịt mắt" |
