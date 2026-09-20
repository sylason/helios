# 🚨 TÀI LIỆU KIỂM TOÁN: 3 SAI LẦM CHÍ TỬ VỀ GIẢ ĐỊNH RISK:REWARD & KHOẢNG CÁCH STOP LOSS TRONG HELIOS ARBITER

- **Ngày ghi nhận:** 2026-09-19
- **Vị trí lỗi:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:L173-L183`
- **Tác động:** Dẫn đến việc Arbiter liên tục cướp thầu sai, loại bỏ các Scanner có điểm tự tin cao và cấu trúc an toàn, lựa chọn các lệnh có Stop Loss quá sát giá vào lệnh khiến tài khoản dính **HARD STOP LOSS** hàng loạt chỉ sau 2 đến 3 cây nến 1 phút.

---

## 🧭 1. HIỆN TRẠNG MÃ NGUỒN GÂY LỖI

Tại `arbiter.rs:L173-L183`:

```rust
if proposal.signal_type == current_prop.signal_type {
    // CÙNG CHIỀU: Ưu tiên lệnh có khoảng cách Stop Loss gần hơn (R:R tốt hơn)
    let dist_new = proposal.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);
    let dist_curr = current_prop.suggested_sl.map(|sl| (current_close - sl).abs()).unwrap_or(f64::MAX);

    if dist_new < dist_curr {
        best = Some((proposal, conf));
    } else if (dist_new - dist_curr).abs() < 1e-6 && conf > *current_conf {
        best = Some((proposal, conf));
    }
}
```

---

## 🔬 2. MỔ XẺ 3 SAI LẦM CỐT LÕI MANG TÍNH GIẢ ĐỊNH & PHẢN THỰC TẾ

### ❌ Sai lầm 1: Ảo tưởng "Khoảng cách Stop Loss gần = Tỷ lệ Risk:Reward tốt hơn" (The Tight Stop Loss Fallacy)
- **Bản chất toán học:** Công thức chuẩn của tỷ lệ Lời:Lỗ (Risk:Reward Ratio) là:
  $$\text{R:R} = \frac{\text{Potential Reward}}{\text{Risk}} = \frac{|\text{Target TP} - \text{Entry}|}{|\text{Entry} - \text{Stop Loss}|}$$
- **Lỗ hổng trong mã nguồn:** 
  + Đa số các Scanners (`trend_pb`, `momentum_ignition`, `absorption`, `anomaly`) đều đặt `suggested_tp = None` vì ủy thác hoàn toàn cho cơ chế Trailing Stop Loss thả diều theo sóng trong `RiskManager`.
  + Do đó, tại thời điểm Arbiter duyệt lệnh, vế **Potential Reward hoàn toàn chưa xác định** (bằng ẩn số hoặc vô cực).
  + Thuật toán tự động giả định rằng mẫu số càng nhỏ ($|\text{Entry} - \text{SL}| \to 0$) thì phân số R:R mặc định càng lớn. Đây là một suy luận giáo điều, thuần túy dựa trên giả định phi thực tế.

---

### ❌ Sai lầm 2: Bẫy nhiễu nến vi mô và quét thanh khoản cực nhanh trên khung 1m Crypto
- **Đặc thù thị trường:** Trên đồ thị nến 1 phút thị trường phái sinh Crypto (ETH/BTC/SOL), độ rung lắc ngẫu nhiên (micro-noise) và râu nến quét 2 đầu thường dao động từ $0.3\%$ đến $0.8\%$.
- **Sự phân kỳ thiết lập Stop Loss giữa các Scanner:**
  + `momentum_ignition.rs`: Đặt SL dính sát vào râu cây nến bùng nổ xung lực (`High[i] + buffer` hoặc `Low[i] - buffer`). Khoảng cách SL chỉ khoảng **$10 trên ETH (~0.4%)**.
  + `trend_pb.rs`: Đặt SL an toàn dựa trên cấu trúc hỗ trợ KAMA Midline hoặc đáy nhịp hồi (`Low.min(kama_mid) - buffer`). Khoảng cách SL khoảng **$20 trên ETH (~0.8%)**, tạo không gian cho giá rung lắc lành mạnh.
- **Hệ quả chết người:**
  + Thuật toán so sánh: `dist_new ($10) < dist_curr ($20)` => Arbiter luôn **chọn `momentum_ignition` và vứt bỏ `trend_pb`**!
  + Giá sau cú bùng nổ chỉ cần giật nhẹ 1-2 cây nến là chạm ngay mức SL $10. Trong `WFSS_37.log`, **100% các lệnh của Momentum Ignition đều bị Hard SL tiêu diệt chỉ sau 2 đến 3 cây nến**!

---

### ❌ Sai lầm 3: Triệt tiêu hoàn toàn Điểm Tự Tin (Confidence) & Độ chuẩn xác của Mẫu hình
- **Sự bất công trong cơ chế chấm thầu:**
  + Giả sử `trend_pb` nhận diện một xu hướng hoàn hảo: KAMA dốc đẹp, nến hồi quy chuẩn mực, Stochastic phân kỳ thuận chiều => Điểm tự tin đạt **0.90**.
  + Trong khi đó, `momentum_ignition` chỉ vừa chớm bứt phá yếu ớt, thị trường hỗn loạn => Điểm tự tin chỉ đạt **0.50**.
  + Nhưng vì khoảng cách SL của `momentum_ignition` gần hơn chỉ $1, Arbiter **thản nhiên gạt bỏ lệnh có Confidence 0.90 để chọn lệnh 0.50**!
- **Hệ quả kiến trúc:** Arbiter bị biến tướng từ một "Trọng tài thẩm định chất lượng tín hiệu" thành một "Cỗ máy săn lùng các lệnh có Stop Loss hẹp nhất để dính cắt lỗ nhanh nhất"!

---

## 🛠️ 3. ĐỀ XUẤT HƯỚNG KHẮC PHỤC CHUẨN MỰC (ENGINEERING ACTION PLAN)

1. **Khôi phục vai trò tối thượng của Điểm Tự Tin (Confidence First):**
   - Khi có nhiều Scanner cùng phát tín hiệu cùng chiều (Long - Long hoặc Short - Short), Scanner nào có **Confidence cao nhất** PHẢI được chọn làm người chiến thắng (`Winner = argmax(conf)`).
2. **Bảo tồn tính toàn vẹn của Stop Loss cấu trúc:**
   - Giữ nguyên `suggested_sl` của Scanner chiến thắng, không can thiệp ép dùng Stop Loss hẹp nhân tạo.
3. **Cơ chế hòa thầu (Tie-breaker):**
   - Chỉ khi nào 2 Scanner có điểm tự tin chênh lệch không đáng kể ($|\text{conf}_1 - \text{conf}_2| < 10^{-4}$), mới sử dụng tiêu chí phụ (như độ dốc xu hướng KAMA hoặc khoảng cách SL) để phân xử.