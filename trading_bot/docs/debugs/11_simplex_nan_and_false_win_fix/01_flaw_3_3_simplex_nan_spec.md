# 🏛️ ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-3.3 (SẬP SIMPLEX NORMALIZATION KHI GẶP NaN)

> **Mã lỗi kiểm toán:** `FLAW-3.3`  
> **Mức độ rủi ro:** **CRITICAL** (Làm tổng trọng số thị trường vọt lên 200%, kích hoạt sai lệch đồng thời cả Trend và Range 100%).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/11_simplex_nan_and_false_win_fix/01_flaw_3_3_simplex_nan_spec.md`  

---

## 1. Bản Chất Vấn Đề & Bằng Chứng Mã Nguồn

### 1.1. Hiện trạng trong `ferro_trade/src/indicators/calculate/bulk.rs`
Tại dòng 1636-1652:
```rust
    let sma_period = 50; 
    if clean_natr.len() < sma_period {
        return Ok(vec![f64::NAN; clean_natr.len()]); // [LỖ HỔNG 1] Trả về toàn bộ NaN khi nến < 50
    }
    
    let natr_sma = sma(&clean_natr, sma_period)?;
    
    let ratio: Vec<f64> = natr_values.iter().enumerate().map(|(i, &v)| {
        let sma_val = natr_sma[i];
        // [LỖ HỔNG 2]: sma_val là NaN trong 49 nến đầu tiên!
        // Trong chuẩn IEEE 754: (NaN < 1e-9) LUÔN TRẢ VỀ FALSE!
        if v.is_nan() || sma_val < 1e-9 { 
            1.0 
        } else { 
            v / sma_val // -> v / NaN = NaN!
        }
    }).collect();
```

### 1.2. Hiện trạng sập đổ trong `strategies_backtests/helios/src/strategy/core_v3/matrix.rs`
Tại dòng 144-151:
```rust
    let w_volatile_raw = chaos_ratio[i] * volume_force[i]; // = NaN
    let max_trend_vol = w_trend_raw.max(w_volatile_raw);  // NaN > w_trend là false -> max trả về NaN!
    let w_range_raw = (1.0 - max_trend_vol).max(0.05).min(1.0); // 1.0 - NaN = NaN -> .max(0.05) = 0.05
    let sum = (w_trend_raw + w_range_raw + w_volatile_raw).max(1e-5); // w_t + 0.05 + NaN = NaN.max(1e-5) = 1e-5!

    w_trend_norm.push((w_trend_raw / sum).max(0.0).min(1.0));    // w_trend / 0.00001 -> 1.0 (100%)
    w_range_norm.push((w_range_raw / sum).max(0.0).min(1.0));    // 0.05 / 0.00001    -> 1.0 (100%)
    w_volatile_norm.push((w_volatile_raw / sum).max(0.0).min(1.0)); // NaN              -> 0.0 (0%)
```

### 1.3. Hậu quả toán học
- Mẫu số `sum` bị ép về $10^{-5}$ ($0.00001$).
- Các trọng số bị chia cho số siêu nhỏ nên phóng đại $40.000$ lần, chạm trần `.min(1.0)`.
- **Tổng trọng số Simplex:** $w_{trend} + w_{range} + w_{volatile} = 1.0 + 1.0 + 0.0 = \mathbf{2.0 \ (200\%!)}$.
- Arbiter bị tê liệt vì không thể phân định thị trường đang có xu hướng hay đang tích lũy, các scanner của cả 2 đội đều phát tín hiệu xung đột nhau.

---

## 2. Thiết Kế Kỹ Thuật: Phòng Thủ 2 Lớp (2-Layer Defense Protocol)

### 2.1. Lớp 1 — Triệt tiêu tận gốc tại `calculate/bulk.rs`
1. Khi `clean_natr.len() < sma_period`: Trả về mảng toàn bộ `1.0` (biến động bình thường/trung tính) thay vì `NaN`.
2. Trong vòng lặp map: Bổ sung tường minh kiểm tra `sma_val.is_nan()`:
```rust
    let sma_period = 50; 
    if clean_natr.len() < sma_period {
        return Ok(vec![1.0; clean_natr.len()]);
    }
    
    let natr_sma = sma(&clean_natr, sma_period)?;
    
    let ratio: Vec<f64> = natr_values.iter().enumerate().map(|(i, &v)| {
        let sma_val = natr_sma[i];
        if v.is_nan() || sma_val.is_nan() || sma_val < 1e-9 { 
            1.0 
        } else { 
            v / sma_val 
        }
    }).collect();
```

### 2.2. Lớp 2 — Phòng thủ tuyệt đối tại `matrix.rs`
Áp dụng nguyên tắc lập trình phòng thủ (Defensive Programming), bọc Zero & NaN Guard cho phép tính Simplex:
```rust
    let w_t = if w_trend_raw.is_nan() { 0.0 } else { w_trend_raw.max(0.0) };
    let w_r = if w_range_raw.is_nan() { 0.0 } else { w_range_raw.max(0.0) };
    let w_v = if w_volatile_raw.is_nan() { 0.0 } else { w_volatile_raw.max(0.0) };

    let sum = w_t + w_r + w_v;
    if sum <= 1e-6 || sum.is_nan() {
        // Fallback cân bằng tuyệt đối: Chia đều 1/3 cho cả 3 chế độ (Simplex 100%)
        w_trend_norm.push(1.0 / 3.0);
        w_range_norm.push(1.0 / 3.0);
        w_volatile_norm.push(1.0 / 3.0);
    } else {
        w_trend_norm.push((w_t / sum).max(0.0).min(1.0));
        w_range_norm.push((w_r / sum).max(0.0).min(1.0));
        w_volatile_norm.push((w_v / sum).max(0.0).min(1.0));
    }
```

---

## 3. Khảo Sát Tính Toàn Vẹn & Edge Cases

1. **Chuỗi dữ liệu nến ngắn ($< 50$ nến):** `chaos_ratio` trả về toàn bộ `1.0`, không có `NaN`.
2. **Dữ liệu nến đi ngang (Biến động = 0):** `sma_val < 1e-9` trả về `1.0`, tránh chia cho 0.
3. **Mọi đầu vào của Simplex đồng thời bằng 0:** `sum <= 1e-6` kích hoạt chia đều $1/3$, bảo đảm $w_t + w_r + w_v \equiv 1.00$.
4. **Miễn nhiễm hoàn toàn với lỗi số học IEEE 754.**

---

## 4. Kế Hoạch Kiểm Thử
- [ ] Unit Test `test_chaos_ratio_short_series_no_nan`: Kiểm tra chuỗi $<50$ nến không có phần tử `NaN`.
- [ ] Unit Test `test_chaos_ratio_warmup_period_no_nan`: Kiểm tra 50 nến đầu tiên đều có giá trị $\ge 0.0$ và không `NaN`.
- [ ] Unit Test `test_simplex_normalization_nan_resilience`: Kiểm tra ma trận khi nạp dữ liệu rác vẫn bảo đảm tổng bằng $1.000$.
