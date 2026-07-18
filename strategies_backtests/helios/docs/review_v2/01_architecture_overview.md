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

## 4. Đề Xuất Debug & Giải Quyết Vấn Đề

Để khắc phục triệt để tình trạng "Phễu Cạn" (Funnel Drain) nơi các cổng lọc chà đạp lên nhau và loại bỏ gần như toàn bộ tín hiệu giao dịch, chúng ta cần tái cấu trúc phương thức đánh giá rủi ro và tín hiệu. Dưới đây là các đề xuất giải pháp kiến trúc và các bước debug cụ thể:

### 4.1. Chuyển đổi từ Bộ lọc Nhân (Multiplicative) sang Điểm số Cộng dồn (Additive)
- **Vấn đề:** Các tầng hiện tại đang hoạt động theo dạng cổng (gate/AND condition) nối tiếp. Chỉ một điều kiện sai lệch nhỏ (ví dụ điểm `stability_index` thấp do volume spike) sẽ nhân với điểm của Regime làm cho `final_score` sụp đổ, khiến Playbook không được kích hoạt.
- **Giải pháp kiến trúc:** Thay vì dùng logic bộ lọc nhân ($P = P1 \times P2 \times P3$), hãy sử dụng hệ thống **Điểm số Tích lũy Trọng số (Weighted Additive Scoring)**. 
  - Tầng Environment, Matrix, Regime thay vì loại trừ nhau, sẽ cùng cộng/trừ điểm vào một Điểm Tín Hiệu Tổng (Total Signal Score).
  - Tầng Playbook Arbitrator sẽ xem xét Tổng Điểm này. Nếu Tổng Điểm $> Threshold$, chiến lược sẽ được kích hoạt.

### 4.2. Nới lỏng các điều kiện AND ở tầng Tactical
- **Vấn đề:** Tầng Tactical Execution tiếp tục đưa ra 2-4 điều kiện AND khắt khe cho mỗi Playbook, hoạt động như một cái "phễu" cuối cùng.
- **Giải pháp:** 
  - Chuyển các điều kiện phụ trợ thành "Điểm cộng" thay vì "Điều kiện bắt buộc" (Requirement).
  - Sử dụng cơ chế: 1 Điều kiện Lõi (Core Trigger) kết hợp với các điều kiện OR. (Ví dụ: Có thể mua nếu *RSI cắt lên 30* **HOẶC** *MACD phân kỳ* khi đang ở Regime phù hợp).

### 4.3. Các bước Debug và Thử nghiệm thực tế
1. **Đo lường "Tỷ lệ rụng" (Drop-off Rate) ở từng tầng:** Bổ sung logging hoặc telemetry để ghi nhận xem trên tổng số 100% tín hiệu khởi tạo, bao nhiêu % bị chặn tại Tầng 3A, Tầng 3B, Tầng 4, Tầng 5. Tìm ra cổng nào đang khắt khe nhất để điều chỉnh.
2. **Review lại DNA Bounds cho Optimizer:** Mở rộng (relax) giới hạn của các biến DNA liên quan đến ngưỡng (thresholds) trong các file DNA. Tránh việc Optimizer chỉ loanh quanh ở các giá trị dẫn đến tắc nghẽn toàn bộ lệnh.
3. **Mô phỏng (Mocking) Tín hiệu Mềm:** Tạo một phiên bản bypass tạm thời các tầng 3A và 4 để xem nếu chỉ dựa vào Regime (3B) và Tactical (5) thì số lượng lệnh và lợi nhuận thay đổi ra sao. Từ đó dần dần thêm lại các hệ số phạt/thưởng thay vì chặn tuyệt đối.
