# 🚪 TÀI LIỆU KỸ THUẬT: CỔNG THẨM ĐỊNH WFE (WALK-FORWARD EFFICIENCY GATE)
> **Mã lỗi xử lý:** FLAW-6.4 (Thiếu Out-Of-Sample Validation Gate trong Live/Walk-Forward Trading)  
> **Vị trí áp dụng:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs`  
> **Trạng thái:** Thiết kế kỹ thuật & Chuẩn bị triển khai.

---

## 1. Bối cảnh & Vấn đề Cốt lõi
- **Hiện trạng:** Thuật toán Di truyền (GA) của `Prospector` tối ưu hóa trên toàn bộ cửa sổ nến In-Sample (14-30 ngày). Với 110+ chiều tự do trong DNA, GA luôn tìm được một bộ DNA khớp rất tốt với dữ liệu quá khứ (Best Return +60% đến +100%).
- **Lỗ hổng:** Ngay khi tìm được Champion DNA, `ProspectorRunner` lập tức lưu vào SQLite và phát qua kênh `dna_tx` sang `AssetActor` mà **không hề qua bất kỳ bài kiểm tra dữ liệu chưa thấy (Out-Of-Sample Validation) nào**. Nếu bộ DNA bị Overfit (chỉ học thuộc nến quá khứ nhưng thất bại ở tương lai), `AssetActor` sẽ gánh chịu thua lỗ ngay lập tức trên Paper/Live.

---

## 2. Thiết kế Cổng Thẩm Định WFE (Split-Window Validation)

### 2.1. Phân chia Cửa Sổ Huấn Luyện
Tại mỗi chu kỳ tối ưu hóa `run_optimization_cycle_range_smart_seed`:
- Tổng khoảng thời gian nến lấy từ database: `[opt_start_ts -> current_ts]`.
- Thời lượng kiểm định Validation: `validation_span_ms = 48 * 3600 * 1000` (48 giờ gần nhất, tương đương 2.880 nến 1m).
- Điểm cắt (Split Point): `train_end_ts = current_ts - validation_span_ms`.

```
[opt_start_ts ───────────────────────► train_end_ts ────────► current_ts]
│◄────── IN-SAMPLE TRAINING (GA) ─────────►│◄── OOS VALIDATION GATE ──►│
```

### 2.2. Quy trình Thẩm định & Tiêu chuẩn Phê duyệt
1. GA Optimizer chỉ chạy tối ưu trên khoảng In-Sample: `[opt_start_ts -> train_end_ts]`.
2. Khi GA tìm được `best_dna`, trước khi lưu SQLite và broadcast:
   - Trích xuất danh sách nến Validation: `[train_end_ts -> current_ts]`.
   - Chạy `diagnostic_backtest(&validation_candles, &best_dna.dna)`.
3. **Tiêu chuẩn Vượt Cổng (WFE Gate Criteria):**
   - $\text{Return OOS} > 0.0\%$ (Lợi nhuận Validation phải dương).
   - $\text{Max Drawdown OOS} \le 12.0\%$ (Kiểm soát rủi ro sụt giảm vốn).
4. **Phân nhánh Quyết định & Logging Debug Chi Tiết:**
   - ✅ **PASS:** Bắn log INFO:
     `✨ [WFE GATE: PASS] Champion DNA Deployed! IS Return: {:.2}% | OOS Val Return: {:+.2}% | OOS Max DD: {:.2}% | OOS Trades: {} (Win Rate: {:.1}%)`  
     Lưu SQLite và gửi qua `dna_tx`.
   - ❌ **REJECT (OVERFIT DETECTED):** Bắn log WARN chi tiết:
     `🚨 [WFE GATE: REJECTED - OVERFIT DETECTED] Symbol: {} | Candidate DNA Overfit! IS Return: {:.2}% BUT OOS Val Return: {:+.2}% (Max DD: {:.2}%, Trades: {}) -> REJECTED! Preserving active Champion DNA to protect capital!`  
     **Không lưu SQLite và không broadcast cho Actor**, giữ nguyên bộ DNA cũ đang an toàn.
