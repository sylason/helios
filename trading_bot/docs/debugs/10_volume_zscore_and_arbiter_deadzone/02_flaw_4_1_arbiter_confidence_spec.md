# 🏛️ ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-4.1 (ARBITER CONFIDENCE DEAD ZONE & KÍCH HOẠT GEN MA SCANNER_WEIGHT)

> **Mã lỗi kiểm toán:** `FLAW-4.1` (kèm kích hoạt Gen Ma `FLAW-2.1`)  
> **Mức độ rủi ro:** **CRITICAL** (Tê liệt >55% không gian tìm kiếm GA; triệt tiêu các cơ hội giao dịch hợp lệ).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/10_volume_zscore_and_arbiter_deadzone/02_flaw_4_1_arbiter_confidence_spec.md`  

---

## 1. Bản Chất Vấn Đề & Bằng Chứng Mã Nguồn

### 1.1. Hiện trạng công thức trong Code
Trong [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:140`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L140):
```rust
let confidence = proposal.raw_score * regime_weight * multiplier;
```
Và trong [`core_v3/config.rs:241-255`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L241-L255):
```rust
#[dna(min = 0.01, max = 0.99, step = 0.03)]
pub scanner_weight: f64, // GEN MA: Hoàn toàn không được sử dụng trong arbiter.rs!

#[dna(min = 0.01, max = 0.99, step = 0.03)]
pub min_confidence_trending: f64,
```

### 1.2. Mâu thuẫn toán học cốt lõi
1. **Nghịch lý nhân phân số (Cubic Decay):**  
   - `raw_score` $\in [0.0, 1.0]$ (thường $0.60 \dots 0.85$).  
   - `multiplier` $\in [0.0, 1.0]$ (thường $0.50 \dots 0.80$).  
   - `regime_weight` thuộc Simplex $w_t + w_r + w_v = 1.00$ (kỳ vọng $0.33$, tối đa chỉ đạt $\approx 0.65$).  
   - Tích 3 phân số: $0.80 \times 0.50 \times 0.70 = \mathbf{0.28}$. Kể cả kịch bản hoàn hảo nhất chỉ đạt $\approx \mathbf{0.45}$.
2. **"Vùng đất chết" (Dead Zone 54.5%):**  
   Dải DNA `min_confidence` có 33 nấc từ $0.01 \dots 0.99$. Khi GA chọn bất kỳ giá trị nào $\ge 0.50$ (18/33 nấc), không có bất kỳ lệnh nào có thể kích hoạt $\rightarrow$ **0 Trades suốt 30 ngày backtest**.
3. **Gen Ma `scanner_weight` (FLAW-2.1):** Nằm trong DNA nhưng bị bỏ hoang, làm lãng phí 1 chiều tìm kiếm của GA.

---

## 2. Thiết Kế Kỹ Thuật: Weighted Geometric Mean Có Guard Tuyệt Đối

### 2.1. Đánh giá và Loại bỏ các phương án lỗi
- ❌ **Loại bỏ Phương án Thu hẹp dải DNA $[0.05 \dots 0.45]$:** Vi phạm nguyên lý thiết kế phổ quát (Universal Parameterization), áp đặt cảm tính và bóp nghẹt tính dynamic.
- ❌ **Loại bỏ Phương án Tuyến tính ($W \cdot S + (1-W) \cdot C$):** Dính "Bẫy Bù Trừ" (Compensatory Trap) nguy hiểm: Một tín hiệu Scanner $0.95$ sẽ ghi đè lên Bối cảnh cấm kỵ $C = 0.0$, khiến bot tự sát khi thị trường bão giá.
- ❌ **Loại bỏ Phương án Căn bậc 3 thuần túy $\sqrt[3]{S \cdot w \cdot M}$:** Bỏ quên gen ma `scanner_weight` và cào bằng trọng số $1/3 - 1/3 - 1/3$.

### 2.2. Công thức Chuẩn Quant: Weighted Geometric Mean + Zero Guard
1. **Tổng hợp Bối cảnh thị trường (Context Score):**
   $$C = \sqrt{w_{regime} \times \text{multiplier}} \quad \in [0.0, 1.0]$$
2. **Cơ chế Guard chặn 0 (Zero Guard):**
   Nếu Scanner từ chối ($S \le 0.0$) HOẶC Bối cảnh phủ quyết ($C \le 0.0$), điểm số bằng $0.0$ lập tức.
3. **Tổng hợp Tự tin qua Gen `scanner_weight` ($W \in [0.01, 0.99]$):**
   $$\text{confidence} = S^W \times C^{(1.0 - W)}$$

### 2.3. Mã nguồn triển khai trong `arbiter.rs`
```rust
// 1. Tính toán Context Score (Căn bậc 2 đưa bối cảnh về thang 0.0 .. 1.0)
let context_score = (regime_weight * multiplier).sqrt();

// 2. ZERO GUARD: Bảo vệ 100% khỏi lỗi 0^0 = 1.0 và ngăn chặn Bẫy Bù Trừ
let confidence = if proposal.raw_score <= 0.0 || context_score <= 0.0 {
    0.0
} else {
    let w = config.arbiter.scanner_weight.max(0.01).min(0.99);
    proposal.raw_score.powf(w) * context_score.powf(1.0 - w)
};
```

---

## 3. Khảo Sát Tính Toàn Vẹn & Edge Cases
1. **Tính đơn điệu (Monotonicity):** $S$ tăng $\rightarrow$ `confidence` tăng; $C$ tăng $\rightarrow$ `confidence` tăng.
2. **Không mất mát dữ liệu:** Không có phép chia, không có hằng số `2.0`, không có `min(1.0)` cắt ngọn.
3. **An toàn số học:** Triệt tiêu hoàn toàn lỗi $0^0 = 1.0$, `NaN`, hay chia cho 0.
4. **Độ phủ DNA:** `confidence` trải dài trọn vẹn từ $0.0 \dots 1.0$, làm sống lại toàn bộ dải DNA $1\% \dots 99\%$.

---

## 4. Kế Hoạch Kiểm Thử (Verification Plan)
- [ ] **Unit Test 1 (`test_zero_guard`):** Xác nhận khi `raw_score = 0.0` hoặc `context_score = 0.0`, `confidence` luôn luôn trả về đúng `0.0`.
- [ ] **Unit Test 2 (`test_full_dynamic_range`):** Xác nhận với các bộ tham số thực tế, `confidence` đạt được các mốc $0.30, 0.50, 0.70, 0.85$, vượt qua được các ngưỡng `min_confidence` cao của GA.
- [ ] **Unit Test 3 (`test_scanner_weight_impact`):** Xác nhận khi thay đổi `scanner_weight` từ $0.1 \rightarrow 0.9$, tỷ trọng điểm nghiêng chính xác về Scanner.
- [ ] **Biên dịch:** `cargo test --package helios --lib`.
