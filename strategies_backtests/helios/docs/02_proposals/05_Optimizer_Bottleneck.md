# 🧬 Tắc nghẽn Optimizer (The Optimizer Bottleneck)

## Tình trạng Thực tế
Hiện tại, `HeliosConfig` chứa khoảng **100 tham số (genes)** cần tối ưu hóa. 
Thuật toán Genetic Algorithm (GA - Prospector) đang được cấu hình:
- `population_size`: 35
- `n_generations`: 15
- Tổng số lần đánh giá (evaluations) tối đa mỗi worker: `35 × 15 = 525`.

## 1. Bài toán Lời nguyền Chiều dữ liệu (Curse of Dimensionality)
Với 100 tham số, không gian tìm kiếm (Search Space) lớn khủng khiếp (ước tính > $10^{80}$ tổ hợp).
Theo nguyên tắc chung của máy học và GA, để tối ưu hiệu quả N tham số, population size tối thiểu nên là `10 × N`.
Với 100 tham số, population cần ít nhất **1000** và chạy hàng trăm thế hệ.
Việc chạy 525 đánh giá trên không gian $10^{80}$ giống như **mò kim đáy biển**, GA chỉ đang tìm kiếm ngẫu nhiên (Random Walk) chứ không hề hội tụ.

## 2. Ký sinh Gene (Parasitic Genes)
Trong số 100 tham số, có tới **40-50 tham số thuộc về các Playbook không bao giờ được kích hoạt** (Range, Momentum, Reversal, Anomaly).
Vì các Playbook này bị "chặn" từ vòng ngoài (Regime Scoring), việc thay đổi các tham số này (ví dụ: `range_sl_factor`, `anomaly_z_score_threshold`) **không tạo ra bất kỳ sự thay đổi nào về kết quả (PnL)**.
- Khi Optimizer thay đổi một "ký sinh gene", PnL giữ nguyên.
- Optimizer hiểu lầm rằng: "A, thay đổi này không làm giảm lợi nhuận, vậy gene này tốt".
- Kết quả: Không gian tìm kiếm bị loãng, thuật toán tốn tài nguyên tối ưu những thứ không tồn tại.

## 3. Lỗi Hàm Mục Tiêu (Objective Function)
```rust
let objective = |stats: &ferro_trade::stats::Stats| {
    if trades_per_day < 1.0 { return -2000.0 * (1.0 - trades_per_day); }
    if dd > 40.0 { return -1000.0 - dd; }
    if ret.is_nan() || ret <= 0.0 { return -50.0; }
    ret
};
```
- **Penalize < 1 trade/ngày:** Điều này ép bot phải giao dịch bừa bãi. Khi bot không tìm được điểm vào lệnh tốt (do các bộ lọc quá khắt khe), nó sẽ chọn các thông số làm giảm ngưỡng lọc (ví dụ: Squeeze threshold) chỉ để đủ số lượng trade, bất chấp chất lượng.
- **Tối ưu duy nhất Return (Lợi nhuận):** Mặc dù hiển thị Sharpe/Sortino, nhưng hàm mục tiêu chỉ trả về `ret`. Bot sẽ chọn các setup rủi ro cao nhất miễn là không dính DD > 40%. Điều này giải thích tại sao tỷ lệ thắng (Win Rate) của Top 10 chỉ ở mức 30-35%.

## 4. Các Chỉ số Bất thường
- **Sortino Ratio = 5477+**: Đây là một dấu hiệu bất thường về mặt toán học. Nguyên nhân là do tập dữ liệu backtest (30 ngày) quá ngắn, hoặc chiến lược có quá ít trade thua lỗ lớn (downside deviation ≈ 0), dẫn đến phép chia cho số gần 0. Điều này làm mất đi tính tin cậy của chỉ số rủi ro.

---

## 🚀 Đề xuất Cải tiến Optimizer cho V3

1. **Giảm số lượng Genes (Dimensionality Reduction):**
   - Chuyển sang kiến trúc V3 (Signal-First) sẽ loại bỏ toàn bộ các tham số của tầng Regime Scoring (~20 genes).
   - "Đóng băng" (Hardcode) các tham số Indicator chuẩn như KAMA (10, 100) và MACD (12, 26, 9) thay vì cho phép GA tối ưu loạn xạ.
   - Chỉ đưa vào GA các tham số **ngưỡng kích hoạt (thresholds)** và **rủi ro (SL factors)**. Mục tiêu: Giảm xuống còn **dưới 50 genes**.

2. **Áp dụng Penalty hợp lý hơn:**
   - Thay vì phạt nặng nếu số trade < 1/ngày, hãy tối ưu hóa theo **Sharpe Ratio** hoặc **Profit Factor** để GA tự tìm ra điểm cân bằng giữa số lượng và chất lượng.
   - Ví dụ hàm mục tiêu mới: `(Profit Factor * 100.0) + Return - (Max Drawdown * 2.0)`.

3. **Huấn luyện theo từng Chuyên đề (Domain-Specific Training):**
   - Đừng tối ưu cả 6 Playbook cùng một lúc.
   - Bật chế độ "Đóng băng Playbook": Ví dụ, tắt tất cả trừ `RangeScanner`, chạy GA để tìm ra bộ gen tối ưu riêng cho Range. Sau đó làm tương tự với Squeeze, Pullback...
   - Cuối cùng, kết hợp các gen tốt nhất lại để chạy Arbiter.
