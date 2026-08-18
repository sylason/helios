# Báo Cáo Khoa Học: Phân Tích Hiện Tượng Overfitting & Chiến Lược Tối Ưu Hóa DNA Trong Chiến Thuật Helios V3

- **Ngày thực hiện:** 12/08/2026
- **Dự án:** Trading Bot Core Architecture (`ferro_trade` & `trading_bot`)
- **File báo cáo chỉ định:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\tests\paper_forward_test\10_optimizer_overfitting_and_dna_analysis.md`

---

## 🎯 Executive Summary (Tóm tắt thực thi)

Qua quá trình nghiên cứu chuyên sâu file cấu hình bộ tối ưu hóa `strategies_backtests/trading_bot_demo/examples/helios_optimizer_config.json`, cấu trúc mã nguồn chiến thuật Helios V3 trong `strategies_backtests/trading_bot_demo/src`, và nhật ký vận hành thực tế `trading_bot/bot_engine/examples/logs/paper_forward_v3.log`, chúng tôi đã xác định được nguyên nhân gốc rễ gây ra sự lệch pha nghiêm trọng giữa kết quả **Huấn luyện (In-Sample)** và **Thử nghiệm Thực tế (Out-Of-Sample Paper Trades)**.

### Kết quả Tương phản Đặc biệt:
- **In-Sample Train Return (30 ngày):** Đạt **+85.66%** (Max DD: -15.49%, 45 trades) ở chu kỳ khởi tạo và bùng nổ lên **+138.86%** (Max DD: -15.00%, 50 trades) ở chu kỳ Re-Opt #0005.
- **Out-Of-Sample Paper Trades Return (Tích lũy 6 ngày):** Kết quả thực tế đạt **-5.69%** (Vốn ban đầu $60.00 giảm còn $56.58), Win Rate tụt thảm hại xuống **31.03%** (9 thắng / 20 thua trên 29 lệnh đã đóng).

---

## 1. Phân Tích Nguyên Nhân Sự Lệch Pha In-Sample vs Out-Of-Sample

### 1.1. Khiếm Khuyết Cấu Trúc Của Hàm Fitness (Objective Function Defect)
Trực tiếp kiểm tra file `trading_bot/bot_engine/src/optimizer/prospector_runner.rs` (dòng 189–204), hàm đánh giá Fitness của GA Prospector được định nghĩa như sau:

```rust
let objective = |stats: &ferro_trade::stats::Stats| {
    let total_days = 30.0;
    let trades_per_day = stats.total_trades as f64 / total_days;
    if trades_per_day < 0.5 {
        return -2000.0 * (1.0 - trades_per_day); 
    }
    let ret = stats.return_pct;
    let dd = stats.max_drawdown_pct.abs();
    if dd > 40.0 {
        return -5000.0 - dd; 
    }
    if ret.is_nan() || ret <= 0.0 {
        return -50.0;
    }
    ret + (trades_per_day * 0.1)
};
```

#### Phân tích sai lệch thuật toán:
1. **Bỏ qua hoàn toàn Drawdown khi $\text{Drawdown} \le 40\%$:** Hàm Fitness không phạt bất kỳ % sụt giảm tài khoản nào miễn là DD không vượt quá 40%. Một cá thể DNA đạt Profit +138.86% với DD -39.9% nhận điểm Fitness $138.86$, đè bẹp hoàn toàn một DNA bền vững đạt Profit +35% với DD chỉ -3%.
2. **Thiếu các chỉ số điều chỉnh rủi ro (Risk-Adjusted Metrics):** Hàm không tính Sharpe Ratio, Sortino Ratio, Calmar Ratio hay Profit Factor. Thuật toán di truyền bị "kích thích" săn tìm các bộ tham số gánh rủi ro cực đại (High Leverage/Tight SL) ăn may trong quá khứ.
3. **Mẫu thống kê quá nhỏ (Small Sample Overfitting):** Chỉ yêu cầu $\ge 0.5$ lệnh/ngày ($15$ lệnh/30 ngày). Với $45$–$50$ lệnh trên $43.159$ nến 1 phút, tỷ lệ nấc vào lệnh chưa tới $0.1\%$. Kết quả +138.86% bị chi phối bởi $1$–$2$ lệnh may mắn trúng xu hướng lớn trong quá khứ.

### 1.2. Sự Chuyển Đổi Chế Độ Thị Trường (Market Regime Shift)
- Cửa sổ In-Sample (03/07 – 02/08/2026) mang tính chất biến động đặc thù (Trends/Swings rộng).
- Khi bước sang cửa sổ Out-Of-Sample (02/08 – 08/08/2026), ETHUSDT đi vào giai đoạn tích lũy hẹp (Sideways) với độ nhiễu cao. Bộ tham số DNA tối ưu hóa theo xu hướng mạnh lập tức bị "quét 2 đầu" (Whipsaws).

### 1.3. Cơ Chế Smart Seeding Phát Tán Vòng Lặp Overfitting
- `prospector_runner.rs` (dòng 173–182): Thuật toán tiêm cá thể Rank 1 Champion từ chu kỳ trước vào Generation 0 của chu kỳ Re-Opt tiếp theo.
- Do Champion cũ đã bị học tủ (Curve-fitted) với dữ liệu tháng trước, việc tiêm cá thể này làm GA bị mất tính đa dạng sinh học và liên tục hội tụ quanh các đỉnh cực trị địa phương (Local Optima) bị lỗi.

---

## 2. Đánh Giá Nguy Cơ Overfitting Từ Không Gian Tham Số

### 2.1. Thảm Họa "Bùng Nổ Chiều Không Gian" (Curse of Dimensionality)
Đối chiếu giữa `HeliosConfigV3` (`strategies_backtests/trading_bot_demo/src/strategy/core_v3/config.rs`) và `helios_optimizer_config.json`:

- **Chiều không gian tham số của Helios V3:**
  - **12 Chỉ báo kỹ thuật:** KAMA, ADX, RSI, BBands, Keltner, MACD, Stoch, ATR, NATR, MFI, VolMA, LinReg.
  - **Market Matrix & Weights:** 9 tham số lookback và trọng số z-score.
  - **6 Tactical Scanners/Playbooks:** Squeeze (3), Pullback (7), Momentum (8), Range (6), Reversal (8), Anomaly (8), Absorption (5) $\rightarrow$ 45 tham số ngưỡng.
  - **Risk Management & Stops:** 16 tham số ATR, Time decay, Tighten steps, Cooldowns.
  - **Tổng số chiều của NST DNA:** **$> 75 - 80$ chiều liên tục và rời rạc**.

- **Quy mô tìm kiếm trong `helios_optimizer_config.json`:**
  - Worker: `population_size = 100`, `n_generations = 7` $\rightarrow$ **700 đánh giá**.
  - Scout: `population_size = 25`, `n_generations = 3` $\rightarrow$ **75 đánh giá**.

#### Đánh giá:
Quét một không gian nghiệm 80 chiều với chỉ 700 mẫu thử tương đương với việc tìm một hạt cát trong sa mạc. Thuật toán di truyền rơi ngay vào các "đỉnh nhọn nhiễu" (Spiky noise peaks) thay vì tìm được vùng "cao nguyên tham số" (Flat parameter plateaus) có khả năng tổng quát hóa cao.

### 2.2. Xung Đột Tranh Chấp Giữa 6 Playbooks ("Playbook Hopping")
- Việc duy trì 6 Playbooks chạy song song qua `HeliosArbiter` làm chiến thuật bị hiện tượng "nhảy Playbook".
- Nhật ký log minh chứng: Lệnh #017 (LONG) cắt lỗ $\rightarrow$ Lệnh #018 (SHORT) mở ngay lập tức $\rightarrow$ Cắt lỗ -14.18% $\rightarrow$ Lệnh #019 (SHORT) lại mở tiếp.

---

## 3. Đề Xuất Định Hướng Tinh Chỉnh

### 3.1. Thiết Kế Lại Hàm Fitness (Multi-Objective Risk-Adjusted Fitness)
Thay thế hàm Fitness thô bằng công thức kết hợp rủi ro và hiệu suất:

$$\text{Fitness} = \text{Sharpe Ratio} \times \text{Calmar Ratio} \times \min\left(1.0, \frac{N_{\text{trades}}}{30}\right) \times (1 - \text{Penalty}_{\text{DD}})$$

1. **Phạt Drawdown nghiêm khắc khi Max DD > 15%.**
2. **Ngưỡng mẫu giao dịch tối thiểu bắt buộc $N_{\text{trades}} \ge 30$.**
3. **Yêu cầu Profit Factor $\ge 1.5$.**

### 3.2. Thu Gọn Chiều Không Gian Tham Số (Search Space Pruning)
1. **Đóng băng các chỉ báo tiêu chuẩn:** RSI = 14, MACD = 12/26/9, BBands = 20/2.0.
2. **Thống nhất Tham số Risk/Stop cho toàn hệ thống.**
3. **Cắt giảm Playbooks:** Thu gọn từ 6 xuống **2-3 Playbooks cốt lõi**.
