# 🔬 BÁO CÁO KIỂM TOÁN CHUYÊN SÂU: ĐỘNG CƠ CHỈ BÁO FERRO_TRADE
> **Thư mục kiểm toán:** `ferro_trade/src/indicators/`  
> **Các tập tin trọng tâm:**  
> - [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs) (58.9 KB — 34 hàm toán học chỉ báo)  
> - [`ferro_trade/src/indicators/factory/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs) (17.4 KB — Tầng Factory & MTF Dispatcher)  
> - [`ferro_trade/src/backtest/context.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs) (Quản trị bộ nhớ đệm Cache & Warmup)  
> **Thời điểm lập:** 2026-09-28  
> **Tài liệu liên quan:** [`TODO.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/14_ferro_trade_indicators/TODO.md)

---

## BẢNG TỔNG HỢP 6 LỖI TIỀM TÀNG TRONG FERRO_TRADE

| Mã Lỗi | Vị Trí Mã Nguồn | Mức Độ | Bản Chất Lỗ Hổng & Hậu Quả Thực Tế | Giải Pháp Xử Lý Triệt Để |
| :---: | :--- | :---: | :--- | :--- |
| **`DEFECT-FT-01`** | [`context.rs:167-176`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs#L167-L176)<br>[`factory/bulk.rs:98`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs#L98) | **CRITICAL** | **Bẫy Nuốt Chửng Toàn Bộ Nến Khi Lỗi:**<br>Khi tính toán trả về `Err` (vd chu kỳ lớn hơn data), Factory nuốt lỗi bằng `vec![f64::NAN; len]`. `context.rs` đếm thấy toàn bộ là NaN nên gán `warmup = len - 1`. Toàn bộ dữ liệu bị warmup nuốt chửng, backtest ra 0 lệnh mà không có cảnh báo! | Nếu `actual_warmup >= len`, log `warn!`, chặn không cho warmup vượt quá trần an toàn (`len / 2` hoặc max lookback). |
| **`DEFECT-FT-02`** | [`calculate/bulk.rs:125-130`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L125-L130) | **HIGH** | **Tê Liệt Z-Score Khi Thị Trường Đi Ngang Phẳng Lì:**<br>Dòng 128: `if s > 0.0 { (val - m) / s } else { f64::NAN }`. Khi giá đi ngang, độ lệch chuẩn $s = 0.0$ và giá bằng trung bình ($val = m$), đáng lẽ $Z = 0.0$ (tâm phân phối), nhưng code biến thành **`NaN`**! | Nếu $s.is\_finite()$ và $s < 1e-9$: nếu $\|val - m\| < 1e-9 \implies 0.0$, ngược lại mới là `NaN`. |
| **`DEFECT-FT-03`** | [`calculate/bulk.rs:201`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L201) | **HIGH** | **Đầu Độc Trạng Thái Yata SMA Khi Nến Đầu Là NaN:**<br>`SMA::new(period, &source[0])`. Nếu `source` là kết quả của một chỉ báo khác (nến 0 là NaN), yata SMA khởi tạo trạng thái với NaN và lây nhiễm NaN sang $100\%$ các nến sau! | Quét tìm phần tử `first_valid` đầu tiên không phải NaN để khởi tạo `SMA::new`. |
| **`DEFECT-FT-04`** | [`calculate/bulk.rs:162`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L162) | **MEDIUM** | **Sai Số Trôi Nổi (Catastrophic Cancellation) Trong ER:**<br>`volatility_sum = volatility_sum - old_diff + new_diff`. Qua hàng chục nghìn nến, sai số làm tròn khiến tổng âm siêu nhỏ ($-10^{-16}$), khiến $ER$ bị ép về $0.0$ giả mạo. | Thêm kẹp sàn bảo vệ: `(volatility_sum - old_diff + new_diff).max(0.0)`. |
| **`DEFECT-FT-05`** | [`calculate/bulk.rs:28-30`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L28-L30) | **MEDIUM** | **Bẫy Kẹp Ép Kiểu `u8::MAX` Trong `to_yata_ma`:**<br>`let p = period.min(u8::MAX as usize) as u8`. Nếu `period == 0`, $p = 0 \implies$ yata panic hoặc lỗi. Nếu `period > 255`, bị kẹp trần ngầm mà không có thông báo. | Kẹp an toàn tối thiểu: `period.clamp(1, 255) as PeriodType`. |
| **`DEFECT-FT-06`** | [`calculate/bulk.rs:1390-1405`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1390-L1405) | **LOW** | **Thiếu Guard Input Trong `keltner_channel`:**<br>Không kiểm tra `period == 0` hoặc `multiplier <= 0.0`. Nếu GA gửi tham số dị tật, yata trả về Err và Factory nuốt lỗi trả về toàn bộ NaN. | Bổ sung kiểm tra đầu vào trước khi gọi `KeltnerChannel::init`. |

---

## MỔ XẺ CHI TIẾT 3 LỖI TỬ HUYỆT NGUY HIỂM NHẤT

### 💣 Tử Huyệt 1: `DEFECT-FT-01` (Bẫy Nuốt Chửng Nến Warmup)
- **Trích dẫn mã nguồn:** [`ferro_trade/src/backtest/context.rs:167-176`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs#L167-L176)
  ```rust
  let mut actual_warmup = 0;
  for &val in arc_result.iter() {
      if !val.is_nan() { break; }
      actual_warmup += 1;
  }
  let safe_actual_warmup = actual_warmup.min(self.raw_data.close.len().saturating_sub(1));
  self.indicator_manager.warmup_period = self.indicator_manager.warmup_period.max(nbars).max(safe_actual_warmup);
  ```
- **Bản chất kỹ thuật:**  
  Khi một chỉ báo kỹ thuật bị lỗi tham số (ví dụ chu kỳ `lookback = 600` trên khung 4h chỉ có 263 nến), hàm `calculate` trả về `Err`. Tầng Factory tại `factory/bulk.rs` bắt lỗi và thay thế bằng `vec![f64::NAN; len]`.  
  Vòng lặp trên duyệt qua toàn bộ mảng và thấy 100% phần tử đều là `NaN`. Do đó, `actual_warmup = len = 63,200`. Biến `safe_actual_warmup = 63,199`.  
- **Hậu quả thực tế:**  
  `warmup_period` của toàn bộ Backtest bị ép lên $63,199$ nến. Toàn bộ lịch sử dữ liệu bị coi là giai đoạn khởi động. Backtest hoàn thành với **0 trades**, đánh lừa thuật toán GA Optimizer rằng cá thể đó có Fitness âm và vứt bỏ cấu hình tốt một cách oan uổng.

---

### 💣 Tử Huyệt 2: `DEFECT-FT-02` (Z-Score Tự Sát Khi Đi Ngang Phẳng Lì)
- **Trích dẫn mã nguồn:** [`ferro_trade/src/indicators/calculate/bulk.rs:125-130`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L125-L130)
  ```rust
  let z_values: Vec<f64> = source.iter().enumerate().map(|(i, &val)| {
      let m = mean_vec[i];
      let s = std_vec[i];
      if s > 0.0 { (val - m) / s } else { f64::NAN }
  }).collect();
  ```
- **Bản chất toán học:**  
  Công thức Z-Score:
  $$Z = \frac{X - \mu}{\sigma}$$
  Khi thị trường rơi vào trạng thái nén biên độ kiệt cùng (ví dụ các nến Doji liên tiếp trước thềm tin tức), độ lệch chuẩn $\sigma = 0.0$. Lúc này giá đóng cửa đúng bằng giá trị trung bình ($X = \mu$).  
  Trong lý thuyết xác suất thống kê, một điểm trùng khít với tâm phân phối có độ lệch chuẩn bằng **$0.0$ ($Z = 0.0$)**.  
  Tuy nhiên, điều kiện kiểm tra `if s > 0.0` đã loại bỏ trường hợp $s = 0.0$ và ép toàn bộ chuỗi giá trị này thành **`f64::NAN`**!
- **Hậu quả thực tế:**  
  Ngay tại thời điểm thị trường tích lũy nén chặt nhất — thời điểm vàng để `Absorption` phát hiện gom hàng và `MomentumIgnition` chuẩn bị bùng nổ — Z-Score lại bị biến thành `NaN`. Toàn bộ Scanner bị tê liệt chính vào lúc cần thiết nhất!

---

### 💣 Tử Huyệt 3: `DEFECT-FT-03` (Lây Nhiễm Trạng Thái Nội Bộ Yata SMA Khi Nến Đầu Là NaN)
- **Trích dẫn mã nguồn:** [`ferro_trade/src/indicators/calculate/bulk.rs:201-207`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L201-L207)
  ```rust
  let mut method = SMA::new(period as yata::core::PeriodType, &source[0])
      .map_err(|e| IndicatorError::Calculation(e.to_string()))?;

  for i in 0..len {
      let val = method.next(&source[i]);
      if i >= period - 1 {
          output[i] = val;
      }
  }
  ```
- **Bản chất kỹ thuật:**  
  Thư viện `yata::methods::SMA` sử dụng thuật toán tích lũy vòng lặp:
  $$\text{sum} = \text{sum} - \text{old} + \text{new}$$
  Nó yêu cầu truyền giá trị khởi tạo `&source[0]` để nạp vào bộ đệm nội bộ.  
  Nếu chuỗi `source` là kết quả của một chỉ báo phái sinh (ví dụ tính SMA của Volume hay SMA của NATR), các nến đầu tiên trong giai đoạn warmup của chỉ báo trước đó chắc chắn mang giá trị `f64::NAN`.  
  Khi `SMA::new` nhận `source[0] = NaN`, biến tích lũy nội bộ `sum` bị gán `NaN`. Kể từ đó, mọi phép tính:
  $$\text{sum} = \mathbf{NaN} - \text{old} + \text{new} \equiv \mathbf{NaN}$$
- **Hậu quả thực tế:**  
  Toàn bộ $100\%$ các nến sau của SMA đều trả về `NaN` vĩnh viễn, làm sập hoàn toàn bất kỳ chỉ báo nào dùng SMA trên dữ liệu phái sinh (như `chaos_ratio` hay `volume_sma`).
