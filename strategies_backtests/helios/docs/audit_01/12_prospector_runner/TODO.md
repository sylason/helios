# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (ACTIONABLE TODO): PROSPECTOR RUNNER & WALK-FORWARD SMART SEEDING

**Module phụ trách:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs`  
**Tài liệu kiểm toán gốc:** [technical_audit.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/12_prospector_runner/technical_audit.md)  
**Ngày lập kế hoạch:** 19/09/2026  
**Chuyên gia kiểm toán:** Quantitative & Systems Auditor  

---

## 🎯 BẢNG TỔNG QUAN ƯU TIÊN THỰC THI

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-PROS-01` | Thiết kế cơ chế Fallback Baseline Seed chống bẫy khởi động lạnh (Cold-Start Starvation) | **CRITICAL** | 🔴 Chưa xử lý | `prospector_runner.rs:341–350`, `walk_forward_smart_seed.rs:139–147` |
| **2** | `TASK-PROS-02` | Khắc phục nhiễm bẩn dữ liệu huấn luyện vào tập kiểm định WFE (Warmup Overlap Contamination) | **HIGH** | 🔴 Chưa xử lý | `prospector_runner.rs:198–207`, `helios_bridge.rs:328–380` |
| **3** | `TASK-PROS-03` | Nâng cấp tiêu chí phê duyệt WFE Gate: Ràng buộc số lệnh tối thiểu $\ge 2$ và tỷ lệ WFE chuẩn Pardo | **HIGH** | 🔴 Chưa xử lý | `prospector_runner.rs:338–350` |
| **4** | `TASK-PROS-04` | Chống bẫy kẹt cực trị địa phương (Local Minima) cho Smart Seeding: Điều chỉnh `max_stale_generations` & Shake Mutation | **MEDIUM** | 🔴 Chưa xử lý | `ga_worker.rs:441–443`, `helios_optimizer_config.json:22` |
| **5** | `TASK-PROS-05` | Làm mượt hàm phạt Early Termination trong GA để bảo vệ phương sai cho Pruning Strategy | **MEDIUM** | 🔴 Chưa xử lý | `prospector_runner.rs:260–273` |
| **6** | `TASK-PROS-06` | Cấu hình hóa các hằng số WFE Gate vào `FullOptimizerConfig` (Loại bỏ Hardcoded Constants) | **LOW** | 🔴 Chưa xử lý | `prospector_runner.rs:194–196, 339`, `helios_optimizer_config.json` |
| **7** | `TASK-PROS-07` | Bổ sung kiểm tra tính toàn vẹn và tương thích số chiều DNA trước khi tiêm hạt giống mồi (Seed Guard) | **LOW** | 🔴 Chưa xử lý | `prospector_runner.rs:283–292` |
| **8** | `TASK-PROS-08` | Bộ đệm ứng viên Top-K Champions qua WFE Gate chống bẫy Overfitting Rank 1 | **CRITICAL** | 🟢 Đã hoàn thành | `prospector_runner.rs:360–480` |
| **9** | `TASK-PROS-09` | Cơ chế Tẩy Não Hạt Giống (Seed Retirement) khi thất bại liên tiếp $\ge 3$ chu kỳ | **HIGH** | 🔴 Chưa xử lý | `prospector_runner.rs`, `storage.rs` |

---

## 📋 CHI TIẾT CÁC ĐẦU VIỆC CẦN THỰC HIỆN

### 🔴 TASK-PROS-01: Thiết kế cơ chế Fallback Baseline Seed chống bẫy khởi động lạnh (Cold-Start Starvation)
- **Mã lỗi liên đới:** `VULN-PROS-01` (Critical).
- **Hiện trạng:**
  - Tại `prospector_runner.rs:348`, khi chu kỳ Initial Optimize bị WFE Gate từ chối (`val_ret <= 0.0`), hàm thoát ngay bằng `return Ok(())` mà không lưu bất kỳ bản ghi nào vào bảng `optimized_params` của SQLite.
  - Trong `walk_forward_smart_seed.rs:139–146` và `asset_actor.rs:423`, do DB rỗng, `AssetActor` buộc phải chạy bằng bộ tham số rỗng mặc định (`HeliosConfigV3::default()`).
  - Thực nghiệm tại `WFSS_37.log` ghi nhận: 4 chu kỳ Re-Opt đầu tiên liên tiếp bị REJECT, khiến Bot hoàn toàn bị tê liệt trong suốt 20 tiếng đồng hồ không vào được bất kỳ lệnh nào (`Trades: 0, Balance: $60.00`).
  - Đồng thời, `current_dna_vec` bị rỗng khiến cơ chế Smart Seeding 0.5% cũng bị vô hiệu hóa hoàn toàn trong các chu kỳ này.
- **Hành động cần làm:**
  1. Xây dựng tệp cấu hình hạt giống chuẩn mực: `strategies_backtests/helios/examples/helios_baseline_dna.json` chứa một bộ DNA 95 chiều đã được tối ưu và kiểm định vững chắc trên dữ liệu 6 tháng.
  2. Tại `prospector_runner.rs`, khi chạy Initial Optimize hoặc khi `db.get_best_dna` trả về `None`:
     - Nếu WFE Gate bị REJECT ở chu kỳ đầu tiên, phát cảnh báo `warn!` nhưng tự động nạp `helios_baseline_dna.json` vào SQLite làm Champion tạm thời và phát sóng qua `dna_tx`.
  3. Bổ sung cờ cấu hình `enable_baseline_fallback: bool` trong `FullOptimizerConfig`.
- **Tiêu chuẩn nghiệm thu (Acceptance Criteria):**
  - Khi xóa sạch file DB và chạy lại `walk_forward_smart_seed.rs`, nếu Initial Optimize bị REJECT, `AssetActor` không còn cảnh báo *"No saved DNA found. Using default parameters"*, mà nạp thành công Baseline DNA và có thể mở lệnh bình thường ngay từ những cây nến đầu tiên.

---

### 🟠 TASK-PROS-02: Khắc phục nhiễm bẩn dữ liệu huấn luyện vào tập kiểm định WFE (Warmup Overlap Contamination)
- **Mã lỗi liên đới:** `VULN-PROS-02` (High).
- **Hiện trạng:**
  - Tại `prospector_runner.rs:198–207`, `validation_slice` được tạo ra bằng cách lấy $5,760\text{ nến}$ Warmup gối đầu cộng với $2,880\text{ nến}$ OOS (tổng cộng $8,640\text{ nến}$).
  - Tại `helios_bridge.rs:328`, `HeliosBridge::diagnostic_backtest` chạy backtest trên toàn bộ $8,640\text{ nến}$ này. Nếu các chỉ báo của chiến lược chỉ cần $1,500$ đến $3,000\text{ nến}$ để hết `NaN`, engine sẽ bắt đầu giao dịch từ nến thứ $3,001$.
  - Khoảng nến từ $3,001$ đến $5,760$ thuộc về tập In-Sample cũ nhưng lại bị tính gộp vào kết quả thẩm định ngoài mẫu (`val_ret`, `val_trades`, `val_dd`).
- **Hành động cần làm:**
  1. Trong `ferro_trade::backtest::BacktestBuilder` hoặc `helios_bridge.rs`, bổ sung phương thức hỗ trợ `diagnostic_backtest_sliced`:
     ```rust
     pub fn diagnostic_backtest_isolated_oos(
         candles: &[Candle],
         dna: &[f64],
         oos_start_idx: usize, // Ví dụ: 5760
     ) -> Result<BacktestResult, String>
     ```
  2. Khi tính toán số liệu thống kê (`compute_stats`), chỉ lọc các giao dịch có thời gian mở lệnh (`trade.entry_bar >= oos_start_idx`), hoặc chỉ tính toán PnL trên đường cong vốn từ `oos_start_idx` trở đi.
  3. Trả về kết quả WFE thuần khiết $100\%$ đại diện cho $48\text{ giờ}$ ngoài mẫu.
- **Tiêu chuẩn nghiệm thu:**
  - Viết unit test xác nhận các lệnh mở trước mốc `5760` không được tính vào `val_trades` và `val_ret` của WFE Gate.

---

### 🟠 TASK-PROS-03: Nâng cấp tiêu chí phê duyệt WFE Gate
- **Mã lỗi liên đới:** `VULN-PROS-03` (High).
- **Hiện trạng:**
  - Tại `prospector_runner.rs:339`, điều kiện duyệt OOS hiện tại là `val_trades == 0 || val_ret <= 0.0 || val_dd > 15.0`.
  - Một bộ DNA chỉ có đúng $1\text{ lệnh}$ may mắn ăn được một cây râu nến trong 48h ($N_{\text{OOS}} = 1$) vẫn được duyệt vượt qua WFE Gate. Mẫu $N = 1$ không có ý nghĩa thống kê và tiềm ẩn rủi ro sụp đổ cao.
  - Ngưỡng Max Drawdown $15\%$ trong $48\text{ giờ}$ là quá lỏng lẻo đối với một tài khoản đòn bẩy cao (20x).
- **Hành động cần làm:**
  1. Nâng ngưỡng số lệnh OOS tối thiểu: `val_trades < 2` (hoặc $\ge 3$ nếu thị trường biến động mạnh).
  2. Siết chặt ngưỡng sụt giảm vốn tối đa trong 48h: Giảm từ `val_dd > 15.0` xuống `val_dd > 10.0%`.
  3. Bổ sung tỷ số hiệu quả tịnh tiến Pardo:
     $$\text{WFE Ratio} = \frac{R_{\text{OOS}} / 2\text{ ngày}}{R_{\text{IS}} / 12\text{ ngày}}$$
     Yêu cầu $\text{WFE Ratio} \ge 0.15$ (tức hiệu suất OOS phải đạt ít nhất 15% tốc độ sinh lời In-Sample).
- **Tiêu chuẩn nghiệm thu:**
  - Ứng viên có $1\text{ lệnh}$ lãi $+2\%$ trong 48h nhưng In-Sample $+60\%$ phải bị cảnh báo hoặc yêu cầu thêm lệnh xác nhận trước khi cho phép Hot-Swap.

---

### 🟡 TASK-PROS-04: Chống bẫy kẹt cực trị địa phương (Local Minima) cho Smart Seeding
- **Mã lỗi liên đới:** `VULN-PROS-04` (Medium).
- **Hiện trạng:**
  - Trong `helios_optimizer_config.json:22`, `max_stale_generations: 4`.
  - Khi tiêm Rank 1 Champion vào Generation 0, cá thể này sở hữu Fitness vượt trội làm vô hiệu hóa các cá thể ngẫu nhiên xung quanh trong 2–3 thế hệ đầu. GA không tìm thấy đột biến tốt hơn nên dừng sớm ngay tại thế hệ thứ 4.
  - Thuật toán không có đủ thời gian khám phá các vùng không gian mới khi xảy ra Regime Shift.
- **Hành động cần làm:**
  1. Trong `ga_worker.rs`, khi phát hiện có `seed_dna`, tự động nới lỏng `max_stale_generations` từ `4` lên `6` thế hệ.
  2. Kích hoạt cơ chế **Rung Lắc Quần Thể Khi Bế Tắc (Stagnation Shaking)**: Nếu thế hệ stale $\ge 2$, tự động nâng tỷ lệ đột biến `hyper_mutpb` lên $0.70$ và `mut_gene_rate` lên $0.50$ để ép quần thể thoát khỏi thung lũng nghiệm cũ của Seed.
  3. Duy trì tỷ lệ tiêm hạt giống nghiêm ngặt: Tối đa $0.5\% - 1.0\%$ (duy nhất 1 cá thể), tuyệt đối không tăng số lượng hạt giống mồi.
- **Tiêu chuẩn nghiệm thu:**
  - GA có tiêm Seed không bị ngắt sớm trước thế hệ thứ 5; phương sai quần thể sau thế hệ 3 vẫn duy trì $> 0.0005$.

---

### 🟡 TASK-PROS-05: Làm mượt hàm phạt Early Termination trong GA
- **Mã lỗi liên đới:** `VULN-PROS-05` (Medium).
- **Hiện trạng:**
  - Tại `prospector_runner.rs:260–272`, hàm phạt ngắt sớm gán điểm cứng:
    `-5000.0 - current_pct` hoặc `-2000.0 * (1.0 - trades_per_day)`.
  - Điểm số phạt âm quá lớn khiến phương sai của cả quần thể bị bẹp dúm ($Variance \approx 0$).
  - Bộ cắt tỉa `PruningStrategy::QuantileRank` hiểu nhầm rằng quần thể bị vô sinh (Sterile) và cắt tỉa nhầm các Scout Jobs tiềm năng.
- **Hành động cần làm:**
  1. Chuẩn hóa hàm phạt theo hàm suy giảm trượt mượt mà (Smooth Decay Function):
     $$\text{Penalty} = -50.0 \times \left(1.0 + \frac{\text{total\_bars} - \text{bar\_index}}{\text{total\_bars}}\right) - \text{drawdown\_pct}$$
  2. Giữ điểm phạt nằm trong biên độ $[-200.0, -50.0]$ thay vì rớt xuống $-5000.0$.
- **Tiêu chuẩn nghiệm thu:**
  - Phương sai của quần thể chứa các cá thể bị dừng sớm vẫn phản ánh đúng độ phân tán, không kích hoạt lỗi `Sterile Population` sai trong `prospector.rs:95`.

---

### 🟢 TASK-PROS-06: Cấu hình hóa các hằng số WFE Gate vào `FullOptimizerConfig`
- **Mã lỗi liên đới:** `VULN-PROS-06` (Low).
- **Hiện trạng:**
  - `VALIDATION_SPAN_BARS = 2880` (48h) và `WARMUP_SAFE_BARS = 5760` bị gán cứng `const` trong code tại dòng 194–195.
  - Ngưỡng `val_dd > 15.0` bị hardcode tại dòng 339.
  - Không thể thay đổi các ngưỡng thẩm định WFE cho các cặp coin khác nhau (ví dụ: BTC biến động thấp cần OOS DD $\le 8\%$, trong khi MEME coin cần $\le 20\%$) mà không phải biên dịch lại mã nguồn.
- **Hành động cần làm:**
  1. Thêm struct `WfeGateConfig` vào `FullOptimizerConfig` (`prospector_runner.rs:73–82`):
     ```rust
     #[derive(Debug, Clone, Deserialize)]
     pub struct WfeGateConfig {
         #[serde(default = "default_validation_span_bars")]
         pub validation_span_bars: usize,
         #[serde(default = "default_warmup_safe_bars")]
         pub warmup_safe_bars: usize,
         #[serde(default = "default_max_oos_drawdown")]
         pub max_oos_drawdown_pct: f64,
         #[serde(default = "default_min_oos_trades")]
         pub min_oos_trades: usize,
     }
     ```
  2. Cập nhật `strategies_backtests/helios/examples/helios_optimizer_config.json`.
- **Tiêu chuẩn nghiệm thu:**
  - Có thể thay đổi cửa sổ thẩm định từ 48h về 24h hoặc 72h chỉ bằng cách sửa file JSON cấu hình.

---

### 🟢 TASK-PROS-07: Bổ sung kiểm tra tính toàn vẹn và tương thích số chiều DNA trước khi tiêm hạt giống mồi (Seed Guard)
- **Mã lỗi liên đới:** `VULN-PROS-07` (Low).
- **Hiện trạng:**
  - Tại `prospector_runner.rs:283–290`, `seed_dna` được bọc vào `Candidate` và đẩy thẳng vào `prospector.champions` mà không kiểm tra độ dài `dna.len()`.
  - Nếu cấu trúc gen của `HeliosConfigV3` thay đổi (ví dụ: tăng từ 95 lên 100 gen do thêm Playbook mới), việc tiêm DNA cũ 95 chiều có thể gây panic hoặc undefined behavior trong toán tử lai ghép.
- **Hành động cần làm:**
  1. Kiểm tra điều kiện:
     ```rust
     if let Some(dna) = seed_dna {
         if dna.len() == helios::strategy::core_v3::config::HeliosConfigV3::DNA_WIDTH {
             // Tiêm hạt giống an toàn
         } else {
             warn!("⚠️ [Seed Guard] Độ dài Seed DNA ({}) không khớp DNA_WIDTH ({}) -> Bỏ qua tiêm mồi!", dna.len(), HeliosConfigV3::DNA_WIDTH);
         }
     }
     ```
- **Tiêu chuẩn nghiệm thu:**
  - Khi truyền một vector DNA sai số chiều vào `run_optimization_cycle_range_smart_seed`, hệ thống phát cảnh báo và tự động fallback về khởi tạo ngẫu nhiên, không gây crash luồng.

---

### 🟢 TASK-PROS-08: Bộ đệm ứng viên Top-K Champions qua WFE Gate chống bẫy Overfitting Rank 1 (ĐÃ HOÀN THÀNH)
- **Trạng thái:** `[x]` Đã hoàn thành 100%. Đã triển khai Top-K Candidates trích xuất từ `Prospector`, khử trùng lặp qua `DnaKey`, duyệt vòng lặp Waterfall First-Pass Greedy và chọn ứng viên đầu tiên vượt qua OOS Do-No-Harm Gate. Đã thêm 2 unit test xác minh hành vi.
- **Mã lỗi liên đới:** `VULN-PROS-08` (Critical).
- **Tài liệu thiết kế chi tiết:** [`top_k_champions_and_seed_retirement.md`](./top_k_champions_and_seed_retirement.md)
- **Hiện trạng:**
  - GA chỉ trả về duy nhất 1 cá thể Rank 1 Champion. Nếu Rank 1 trượt OOS ($R_{\text{OOS}} \le 0$), toàn bộ chu kỳ bị REJECT.
  - Rank 1 In-Sample thường là cá thể học vẹt quá khớp râu nến (Overfitted). Trong khi Rank 2, 3 có tính tổng quát hóa cao hơn lại bị bỏ rơi.
- **Hành động cần làm:**
  1. Trích xuất nhóm tinh hoa `Top-K Champions` ($K = 3 \sim 5$) từ `Prospector`.
  2. Khử trùng lặp Euclidean Genetic Distance giữa các cá thể.
  3. Duyệt tuần tự qua Cổng WFE Gate: Nếu Champ 1 PASS $\implies$ Deploy; nếu REJECT $\implies$ tiếp tục thẩm định Champ 2, Champ 3.
  4. Duy trì tiêu chuẩn OOS: $R_{\text{OOS}} > 0\%$, $\text{MaxDD} \le 12\%$, $N_{\text{trades}} \ge 1$. Không ép chỉ tiêu Win Rate trên mẫu 3 ngày.
- **Tiêu chuẩn nghiệm thu:**
  - Khi Rank 1 bị REJECT nhưng Rank 2 đạt OOS dương, hệ thống tự động chọn Rank 2 làm Champion mới, tránh việc hủy chu kỳ oan uổng.

---

### 🟠 TASK-PROS-09: Cơ chế Tẩy Não Hạt Giống (Seed Retirement) khi thất bại liên tiếp $\ge 3$ chu kỳ
- **Mã lỗi liên đới:** `VULN-PROS-09` (High).
- **Tài liệu thiết kế chi tiết:** [`top_k_champions_and_seed_retirement.md`](./top_k_champions_and_seed_retirement.md)
- **Hiện trạng:**
  - Khi thị trường đổi pha, hạt giống cũ tiếp tục thống trị In-Sample nhưng thua lỗ liên tục ngoài OOS, dẫn đến kẹt 11 chu kỳ liên tiếp (44 tiếng) như trong log `20261001_180444`.
- **Hành động cần làm:**
  1. Thêm bộ đếm `consecutive_rejections: usize` trong `ProspectorRunner`.
  2. Nếu chu kỳ có cá thể PASS $\implies$ đặt lại `consecutive_rejections = 0`.
  3. Nếu toàn bộ Top-K đều REJECT $\implies$ tăng `consecutive_rejections += 1`.
  4. Khi `consecutive_rejections >= 3`:
     - Xóa sạch hạt giống cũ khỏi SQLite DB.
     - Kích hoạt chu kỳ kế tiếp chạy 100% Latin Hypercube Sampling (LHS) thuần khiết không hạt giống để giải phóng quần thể tìm miền đất mới.
- **Tiêu chuẩn nghiệm thu:**
  - Hệ thống không bao giờ bị kẹt hạt giống cũ quá 3 chu kỳ liên tiếp. Sau 3 chu kỳ thất bại, tự động reset về 100% LHS ngẫu nhiên.

