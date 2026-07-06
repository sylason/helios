# 🔍 Vấn đề Cốt lõi: Phễu 4 Tầng Tắc Nghẽn (The Funnel Problem)

## Kiến trúc Hiện tại: "Regime-First" (Phân loại Trước, Đánh Sau)

```
Tick Data (43,200 nến/tháng)
    │
    ▼
┌─────────────────┐
│  Environment    │  ← Cầu chì: Chặn ~40-60% nến
│  Fuse           │     (stability_index * persistence)
└────────┬────────┘
         │ (~20,000 nến sống sót)
         ▼
┌─────────────────┐
│  Regime Scoring │  ← Bầu chọn 1 winner duy nhất
│  + Arbitration  │     final_score < threshold → Wait
└────────┬────────┘
         │ (~5,000 nến sống sót)
         ▼
┌─────────────────┐
│  Playbook       │  ← Lọc phụ: Neutral → Wait
│  Selection      │     is_parabolic → Wait
└────────┬────────┘
         │ (~3,000 nến sống sót)
         ▼
┌─────────────────┐
│  Tactical       │  ← 4-5 điều kiện AND cứng
│  Trigger        │     ĐỒNG THỜI phải đúng
└────────┬────────┘
         │ (~50-100 nến kích hoạt/tháng)
         ▼
     📍 VÀO LỆNH
```

**Vấn đề:** Mỗi tầng loại bỏ 50-80% tín hiệu, nhưng các tầng **không biết nhau tồn tại**. Tầng Tactical không biết rằng Fuse đã lọc sạch 60% nhiễu, nên nó lọc lại từ đầu.

---

## So sánh: "Regime-First" vs. "Signal-First"

### Cách hiện tại (Regime-First):
1. **HỎI:** "Thị trường đang ở trạng thái gì?" → Phân loại
2. **HỎI:** "Chiến thuật nào phù hợp với trạng thái này?" → Chọn playbook
3. **HỎI:** "Có đủ điều kiện bóp cò không?" → Kiểm tra triggers

**Nhược điểm:**
- Bước 1 rất khó chính xác. Thị trường không phải lúc nào cũng nằm gọn trong 1 trạng thái.
- Nhiều khi thị trường đang **chuyển đổi** (transition) giữa 2 trạng thái → Regime scoring dao động liên tục → Không squad nào đạt ngưỡng.
- Forcing "1 winner duy nhất" khiến 5/6 squad BỊ GẠT ra ngoài mỗi tick.

### Cách đề xuất (Signal-First):
1. **HỎI:** "Có tín hiệu kỹ thuật nào đang xảy ra?" → Quét tất cả triggers song song
2. **HỎI:** "Tín hiệu này có đáng tin cậy không?" → Chấm điểm confidence
3. **HỎI:** "Rủi ro có chấp nhận được không?" → Kiểm tra sizing + SL

**Ưu điểm:**
- Mọi squad đều được **scan song song** mỗi tick.
- Không cần regime classification (loại bỏ 1 tầng lọc hoàn toàn).
- Tín hiệu tốt nhất sẽ tự nhiên nổi lên qua confidence scoring.

---

## Tại sao "Signal-First" tốt hơn cho Day Trading?

### 1. Thị trường thay đổi liên tục, không chờ đợi
Trong 1 phút, thị trường có thể đang sideway rồi đột ngột breakout. Với Regime-First:
- Nến 99: Regime = Ranging → Chọn RangeOscillator → Check trigger: Chưa chạm BB Lower → SKIP
- Nến 100: Giá breakout → Regime vẫn = Ranging (vì persistence/hysteresis) → Không kịp switch sang Squeeze → BỎ LỠ

Với Signal-First:
- Nến 100: Quét tất cả triggers → SqueezeBreakout phát hiện BB đang nở + Volume tăng → VÀO LỆNH ngay lập tức. Không cần đợi regime chuyển đổi.

### 2. Các trạng thái CHỒNG LẤN nhau
Bạn đã nhận ra đúng: "tôi biết là các trạng thái có thể cùng lúc xảy ra nhiều cái chồng lấn nhau". Nhưng hệ thống hiện tại **chọn 1 winner duy nhất**. Trong kiến trúc Signal-First, ta cho phép nhiều tín hiệu cùng xuất hiện, rồi chọn cái có confidence cao nhất.

### 3. Giảm số lượng genes cho Optimizer
Loại bỏ 6 bộ ngưỡng Regime (squeeze.threshold, trend.adx_strong, trend.angle_healthy, ...) giúp giảm ~20 genes. GA sẽ hội tụ nhanh hơn 10-100 lần.

---

## Bảng So sánh Chi tiết

| Tiêu chí | Regime-First (V2) | Signal-First (V3) |
|:---------|:-------------------|:-------------------|
| Số tầng lọc | 4 tầng nối tiếp | 2 tầng (Signal + Risk) |
| Xác suất trigger | ~0.1% | ~5-15% |
| Số genes | ~100 | ~50-60 |
| Phản ứng với chuyển đổi | Trễ 5-10 nến (persistence) | Tức thì |
| Range Trading | Gần như không hoạt động | Hoạt động bình thường |
| Code complexity | 7 files logic | 3-4 files logic |
| DRY violation | 14 lần copy SL/TP | 1 hàm duy nhất |
