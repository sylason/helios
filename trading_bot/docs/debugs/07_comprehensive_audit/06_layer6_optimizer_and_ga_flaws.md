# BÁO CÁO KIỂM TOÁN TẦNG 6: ĐỘNG CƠ TỐI ƯU GA, LHS & SMART SEEDING (OPTIMIZER & PROSPECTOR)

> **Mục tiêu:** Rà soát toàn diện giải thuật di truyền (`ferro_trade::Prospector`, `ga_worker`), hàm mục tiêu fitness, bẫy overfitting, thuật toán lấy mẫu không gian Latin Hypercube Sampling (LHS), cơ chế tiêm hạt giống Smart Seeding, và tính an toàn đa luồng Rayon / Fitness Cache.

---

## 📌 TỔNG HỢP DANH SÁCH LỖ HỔNG PHÁT HIỆN TẠI TẦNG 6

| Mã Lỗi | Thành phần | Vị trí (File & Dòng) | Mức độ | Bản chất Lỗ hổng |
| :--- | :--- | :--- | :---: | :--- |
| **FLAW-6.1** | `ferro_trade` | `prospector.rs:360, 644` | 🚨 **CRITICAL** | **Initial Job #0 Không Hề Chạy LHS:** Job khởi tạo bị gán nhãn `Hotspot` thay vì `New`, khiến Job #0 chạy thẳng vào GA với phân bố ngẫu nhiên Monte Carlo; LHS bị vô hiệu hóa hoàn toàn ở đợt khởi động đầu tiên! |
| **FLAW-6.2** | `ferro_trade` | `ga_worker.rs:469, 476, 509` | 🚨 **CRITICAL** | **Lệch Chuẩn Thang Đo Fallback (`-1000` vs `-100M`):** Khi backtest lỗi (`fitness_score = None`), fallback là `-1000 / 1M = -0.001` (gần hòa vốn), cao hơn hàng nghìn lần so với cá thể thua lỗ (-100.0) hay ngắt sớm (-5000.0), làm sai lệch hoàn toàn phương sai và bộ lọc cắt tỉa `PruningStrategy`! |
| **FLAW-6.3** | `ferro_trade` | `ga_worker.rs:192, 257, 426` | 🚨 **CRITICAL** | **"Cấu hình chết" `min_trades` trong GA Engine:** Cấu hình `min_trades` (mặc định 10) được truyền vào `SimpleBacktestFitness` nhưng trong hàm đánh giá `calculate_for_chromosome` hoàn toàn không được đọc đến! |
| **FLAW-6.4** | `trading_bot` | `bot_engine/src/main.rs:104` | 🚨 **CRITICAL** | **Thiếu Out-of-Sample (OOS) Validation Gate Trong Live:** Champion sau khi tối ưu 30 ngày In-Sample được tiêm thẳng vào `AssetActor` qua watch channel mà không có trạm gác OOS kiểm chứng, làm tăng rủi ro giao dịch thật với DNA overfitting. |
| **FLAW-6.5** | `trading_bot` | `prospector_runner.rs:360` vs `core.rs:482` | 🚨 **MAJOR** | **Nghịch Lý Phạt Oan Chiến Lược 100% Winrate:** Khi không có lệnh lỗ, `core.rs` gán `profit_factor = NaN`. Hàm fitness thấy `pf.is_nan()` liền phạt mức nặng nhất `pf_mult = 0.20` (giảm 80% điểm số của cá thể hoàn hảo)! |
| **FLAW-6.6** | `trading_bot` | `prospector_runner.rs:230` & `ga_worker.rs:394` | 🚨 **MAJOR** | **Silent Drop Hạt Giống Smart Seed:** Runner không validate độ dài trước khi tiêm; GA worker lọc `retain` âm thầm loại bỏ nếu lệch chiều mà không có bất kỳ log cảnh báo nào. |
| **FLAW-6.7** | `ferro_trade` | `fitness_cache.rs:100-103` | 🚨 **MAJOR** | **"Flush-All" Cache Stampede Khi Bộ Nhớ Đầy:** Khi đạt 50,000 entries, cache gọi `self.entries.clear()` xóa trắng 100% dữ liệu, gây 100% Cache Miss và đóng băng luồng để acquire write lock toàn bộ shards. |
| **FLAW-6.8** | `ferro_trade` | `fitness_cache.rs:96-105` | 🚨 **MAJOR** | **Lock Contention của `DashMap::len()` Trên Mỗi Lần Insert:** `.len()` phải lấy Read Lock tuần tự trên toàn bộ 64 shards. Việc gọi `.len()` ở mỗi lệnh chèn trên hot-path Rayon gây nghẽn đa luồng CPU. |
| **FLAW-6.9** | `trading_bot` | `prospector_runner.rs:356` | 🟠 **HIGH** | **Bẫy Overfitting Do Drawdown Diminishing Sensitivity:** $base\_score = ret / (dd + 3.0)$ có độ dốc phạt giảm dần khi DD tăng cao, thiếu hình phạt phi tuyến tăng nặng, khiến GA thiên vị các chiến lược gồng lỗ đòn bẩy cao. |
| **FLAW-6.10** | `ferro_trade` | `prospector.rs:874` | 🟡 **MEDIUM** | **Rayon Thread Oversubscription:** `active_workers >= num_threads` cho phép nhiều GA jobs lớn chạy đồng thời, tranh chấp worker threads trong Rayon global pool và phân mảnh CPU cache. |
| **FLAW-6.11** | `trading_bot` | `prospector_runner.rs:232-237` | 🟡 **MEDIUM** | **Tiêm Hạt Giống Dummy `fitness: 0.0` Vào Cuối Champions:** Tiêm candidate rỗng vào cuối mảng với điểm 0.0 khiến cơ chế ưu tiên `champions.first()` có thể chọn sai hoặc bị đè bởi kết quả cũ. |

---

## 🔍 CHI TIẾT CÁC LỖ HỔNG NGHIÊM TRỌNG

### 1. FLAW-6.1 [CRITICAL]: Initial Job #0 Bị Vô Hiệu Hóa Thuật Toán LHS
- **Hiện trạng:**
  Tại `ferro_trade/src/optimizer/prospector.rs` (dòng 360–361):
  ```rust
  let mut job = Job::new(id, params_def);
  job.status = JobStatus::Hotspot; // ⚠️ Gán cứng trạng thái là Hotspot!
  ```
  Và tại vòng lặp thực thi `Prospector::run` (dòng 644–648):
  ```rust
  let is_scout_mode = matches!(current_job.status, JobStatus::New);
  let result = if is_scout_mode && effective_sampling != SamplingMethod::Genetic {
      // 🚀 CHỈ CHẠY LHS KHI LÀ SCOUT JOB (JobStatus::New)
  } else {
      // ⚙️ CHẠY GA WORKER TRUYỀN THỐNG
  }
  ```
- **Hậu quả:**
  Job khởi tạo ban đầu (Job #0) mang trạng thái `Hotspot`, do đó **Job #0 hoàn toàn không bao giờ chạy qua thuật toán LHS (Latin Hypercube Sampling)**! Nó chạy thẳng vào `SimpleOptimizer` với cơ chế sinh ngẫu nhiên Monte Carlo mặc định của `MultiRangeGenotype`. Tính năng Maximin LHS chỉ có tác dụng ở các Scout Jobs sinh ra từ `SubspaceSplit` hoặc `RandomRestart` sau đó.

---

### 2. FLAW-6.2 [CRITICAL]: Lệch Chuẩn Thang Đo Fallback Biến Lỗi Runtime Thành Lãi Danh Nghĩa
- **Hiện trạng:**
  Tại `ferro_trade/src/optimizer/ga_worker.rs` (dòng 469, 476, 509):
  ```rust
  // Tính tổng fitness quần thể:
  let sum_fitness: f64 = chromosomes.iter()
      .map(|c| c.fitness_score.unwrap_or(-1000) as f64 / 1_000_000.0)
      .sum();
  // Tính phương sai quần thể:
  let f = c.fitness_score.unwrap_or(-1000) as f64 / 1_000_000.0;
  // Trích xuất Top Candidates:
  let fitness_f64 = chromo.fitness_score.unwrap_or(-100000000) as f64 / 1_000_000.0;
  ```
- **Hậu quả:**
  Khi một cá thể bị lỗi backtest (`run_with_dna` trả về `Err`), `fitness_score` mang giá trị `None`.
  Giá trị fallback `-1000` sau khi chia `1_000_000.0` trở thành **`-0.001`**!
  Trong khi một chiến lược thua lỗ có fitness là `-100.0`, và cá thể dính Early Termination có điểm là `-5000.0`. Một cá thể bị crash/lỗi runtime lại nhận điểm số gần như hòa vốn (`-0.001`), làm kéo lệch giả tạo `population_avg_fitness` và `population_variance`, phá hủy hoàn toàn logic cắt tỉa nhánh (`PruningStrategy`) của Prospector.

---

### 3. FLAW-6.3 [CRITICAL]: "Cấu Hình Chết" `min_trades` Trong GA Engine
- **Hiện trạng:**
  Tại `ga_worker.rs` (dòng 192, 257, 426):
  ```rust
  min_trades: self.config.min_trades, // Cấu hình mặc định 10 lệnh
  ```
  Tuy nhiên, trong toàn bộ hàm đánh giá cá thể `calculate_for_chromosome` (`ga_worker.rs:302-365`), trường `self.min_trades` **hoàn toàn không được sử dụng ở bất kỳ dòng code nào**!
- **Hậu quả:**
  Engine phó mặc 100% trách nhiệm kiểm soát số lệnh cho hàm `objective_func`. Nếu người dùng sử dụng hàm objective bên ngoài mà quên kiểm tra số lệnh, một cá thể chỉ có 1-2 lệnh ăn may sẽ không bị GA Engine phạt, dễ dàng vượt mặt các cá thể có tần suất giao dịch ổn định.

---

### 4. FLAW-6.4 [CRITICAL]: Thiếu Out-Of-Sample (OOS) Validation Gate Trong Live Trading
- **Hiện trạng:**
  Tại `trading_bot/bot_engine/src/main.rs` (dòng 104–126):
  Bộ tối ưu JIT chạy định kỳ mỗi 5–15 phút trên cửa sổ trượt 30 ngày (In-Sample). Ngay khi GA hoàn thành, cá thể Champion Rank 1 được gửi qua `dna_tx` (watch channel) và tiêm trực tiếp vào `AssetActor` đang chạy.
- **Hậu quả:**
  Hoàn toàn **không có bất kỳ trạm kiểm định Out-Of-Sample (OOS Validation Gate)** nào trước khi áp dụng DNA vào thị trường thật. Bất kỳ cá thể nào bị curve-fitting/overfitting trên 30 ngày quá khứ sẽ được áp dụng ngay lập tức cho các lệnh giao dịch tiền thật tiếp theo.

---

### 5. FLAW-6.5 [MAJOR]: Nghịch Lý Phạt Oan Chiến Lược 100% Winrate ($PF = \text{NaN}$)
- **Hiện trạng:**
  Trong `ferro_trade/src/stats/core.rs` (dòng 482):
  ```rust
  stats.profit_factor = if gross_losses > 0.0 { gross_profits / gross_losses } else { f64::NAN };
  ```
  Trong `trading_bot/bot_engine/src/optimizer/prospector_runner.rs` (dòng 360–361):
  ```rust
  let pf = stats.profit_factor;
  let pf_mult = if pf.is_nan() || pf <= 0.0 {
      0.20 // ⚠️ Bị phạt nặng nhất!
  ```
- **Hậu quả:**
  Khi chiến lược đạt 100% Winrate (không có lệnh thua), về mặt toán học $PF = +\infty$. Nhưng do `core.rs` gán `NaN`, hàm fitness coi đây là lỗi và phạt tụt điểm **80%** (`pf_mult = 0.20`). Cá thể hoàn hảo bị trừng phạt và đào thải oan uổng khỏi quần thể.

---

### 6. FLAW-6.7 & FLAW-6.8 [MAJOR]: "Flush-All" Stampede & Lock Contention Trong `FitnessCache`
- **Hiện trạng:**
  Tại `ferro_trade/src/optimizer/fitness_cache.rs` (dòng 96–105):
  ```rust
  if self.entries.len() >= self.capacity_limit {
      self.entries.clear(); // ⚠️ Xóa sạch toàn bộ cache!
  }
  self.entries.insert(key, entry);
  ```
- **Hậu quả:**
  1. **Lock Contention:** Trong `DashMap`, gọi `.len()` không phải $O(1)$ mà phải lấy Read Lock trên toàn bộ 64 shards. Gọi `.len()` ở mỗi lệnh chèn làm nghẽn CPU nghiêm trọng khi có 8–16 luồng Rayon cùng chạy.
  2. **Cache Stampede:** Khi đạt 50,000 cá thể, lệnh `clear()` giữ Exclusive Write Lock trên toàn bộ shards và xóa sạch 100% cache. Thế hệ tiếp theo bị 100% Cache Miss, buộc hệ thống tính toán lại từ đầu.

---

## 🛡️ ĐIỂM SÁNG XÁC NHẬN TRONG TẦNG 6

- **An Toàn Đa Luồng Tuyệt Đối:** `DataView` chia sẻ zero-copy qua `Arc<SharedData>` với `AtomicUsize` cục bộ cho từng luồng; `IndicatorManager` cấp phát 100% trên stack, không có data race hay deadlock.
- **SipHash 1-3 & Lượng Tử Hóa $10^{-5}$:** Băm DNA an toàn trên 64 shards; lượng tử hóa số thực với $\epsilon = 0.00001$ loại bỏ triệt để hiện tượng sai số trôi dấu phẩy động IEEE-754.
- **5 Tầng Clamping Biên Gen:** Đảm bảo 100% các giá trị gen sau đột biến và lai ghép không bao giờ bị văng ra ngoài khoảng `[min, max]`.
- **Cơ Chế Thoát Bẫy Cực Trị:** Hyper-Mutation kích nổ khi trì trệ $\ge 2$ thế hệ (đột biến 39/110 gene trên 75% quần thể) kết hợp cắt tỉa quần thể vô sinh (`population_variance < min_variance`).
