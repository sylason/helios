# Tài Liệu Kỹ Thuật: Cấu Hình Siêu Tốc Cho Walk-Forward & Tham Số Hóa Early Termination

## 1. Bối Cảnh & Vấn Đề (Problem Statement)
Trong các lần chạy thử nghiệm Walk-Forward Out-Of-Sample (OOS) trên cặp tiền `ETHUSDT`:
1. **Thời gian chạy quá lâu:** Một bài test 4 ngày OOS mất từ **80 đến 100 phút**, gây tắc nghẽn chu kỳ kiểm thử và nghiên cứu.
2. **Nguyên nhân thời gian lớn:**
   - Cấu hình GA `Prospector` trong `helios_optimizer_config.json` đặt `max_depth: 2` (Scout + 2 tầng Worker), dẫn đến $\approx 1.560$ lần backtest đầy đủ cho mỗi chu kỳ Re-Opt.
   - Cửa sổ dữ liệu In-Sample dài tới 30 ngày (43.158 nến 1m) khiến mỗi lần tính ma trận và backtest tốn nhiều CPU cycle.
   - Ngưỡng kiểm tra dừng sớm (`early_termination`) đang bị hardcode trong `prospector_runner.rs` với `after_bars: 14400` (chạy hết 10 ngày mới loại bỏ cá thể không ra lệnh), lãng phí tài nguyên CPU cho các cá thể rác.
3. **Thiếu sót Telemetry:**
   - Dòng log `[STAGNANT TELEMETRY]` tính số nến giữ lệnh (`bars_held`) bằng chỉ số nến trong bộ đệm trượt (`bar_index.saturating_sub(self.entry_bar_index)`). Do bộ đệm có trần 43.200 nến và dùng `pop_front`, chỉ số này luôn bằng nhau (43.199 - 43.199 = 0), dẫn đến in ra `Held: 0 bars` dù thực tế giữ 581 nến.

---

## 2. Giải Pháp Kỹ Thuật (Architecture & Solutions)

### A. Tham Số Hóa `early_termination` Sang File JSON (`FullOptimizerConfig`)
- Bổ sung struct `EarlyTerminationConfig`:
  ```rust
  #[derive(Debug, Clone, Deserialize)]
  pub struct EarlyTerminationConfig {
      #[serde(default = "default_max_drawdown")]
      pub max_drawdown_pct: f64,
      #[serde(default = "default_warmup_bars")]
      pub warmup_bars: usize,
      #[serde(default = "default_min_trades")]
      pub min_trades: usize,
  }
  ```
- Tích hợp vào `FullOptimizerConfig` với thuộc tính `#[serde(default)]` để tương thích ngược hoàn toàn nếu file JSON không khai báo.
- Khai báo trong `helios_optimizer_config.json`:
  ```json
  "early_termination": {
    "max_drawdown_pct": 35.0,
    "warmup_bars": 5760,
    "min_trades": 2
  }
  ```

### B. Tối Ưu Tốc Độ GA (Rút Ngắn 80% Thời Gian Chạy)
- Chuyển `max_depth` từ `2` xuống `1` trong `helios_optimizer_config.json` (giảm 50% số backtests từ 1.560 xuống ~840).
- Tinh chỉnh `population_size: 40` và `n_generations: 8` cho Worker (giảm thêm 55% khối lượng đánh giá).
- Giảm `TRAIN_WINDOW_DAYS` trong `walk_forward_smart_seed.rs` từ `30` xuống `14` (20.160 nến 1m, vẫn đảm bảo trọn vẹn 50+ nến 4h warmup = 12.000 nến, nhưng tốc độ backtest tăng gấp 2 lần).
- Tổng thời gian ước tính toàn bài test 4 ngày OOS: **10 – 15 phút** (thay vì 90 phút).

### C. Khắc Phục Telemetry `STAGNANT TELEMETRY` & Bổ Sung `TRAILING SL`
- Sửa công thức `bars_held` trong `paper_trader.rs` dựa trên độ lệch timestamp:
  ```rust
  let duration = timestamp.saturating_sub(self.entry_timestamp);
  let bars_held = if duration >= 60_000 {
      (duration / 60_000) as usize
  } else {
      bar_index.saturating_sub(self.entry_bar_index)
  };
  ```
- Nâng cấp log `[TRAILING SL UPDATE]` lên mức `info!` khi mức Stop Loss thực sự được nâng lên.
