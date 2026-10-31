# 01. TECHNICAL DESCRIPTION: KHẮC PHỤC LỖI PANIC MTF 30M VÀ DEADLOCK WFE GATE

## 1. Mục Tiêu Tinh Gọn (Scope & Objectives)
Kế hoạch kỹ thuật này tập trung giải quyết dứt điểm các lỗi logic và crash runtime đang cản trở bài test `walk_forward_smart_seed`:
1. **Hỗ trợ đầy đủ Timeframe 30m (cùng 10m, 2h, 3h)**: Đồng bộ hóa toàn diện chuỗi tiền tính toán MTF Cache, bảo toàn 100% năng lực tính toán đa khung thời gian của hệ thống.
2. **Bảo vệ fail-safe tại `context.rs`**: Loại bỏ panic sập luồng, chuyển sang đào thải êm đẹp qua `mark_infeasible`.
3. **Bẫy ngắt sớm tại Bar 0 do luật trần $40\%$ warmup (`is_warmup_overflow`)**: Thêm `eval_start_bar` để cô lập 100% cửa sổ OOS, trả lại các lệnh giao dịch thực tế ($84 - 82 = \mathbf{2\text{ trades}}$).
4. **Chuẩn hóa WFE Gate**: Bỏ trừ khống `slippage_drag_roe` và bảo vệ vốn khi Top-K fail.

---

## 2. Chi Tiết Lỗi & Giải Pháp Sửa Lỗi

### 2.1. Hỗ Trợ Chuẩn Mực Timeframe 30m & Đồng Bộ Hóa MTF Cache
- **Phản biện đúng đắn:** Không được loại bỏ `30m` khỏi hệ gen! Nến 30m là khung thời gian trung gian cốt tử giữa 15m và 1h, giúp nhận diện sóng đẩy (`ImpulseWave`) và dòng tiền (`CmfTrend`) với độ nhiễu thấp hơn 15m nhưng độ trễ nhỏ hơn 1h. Loại bỏ `30m` là làm nghèo nàn không gian tìm kiếm của GA.
- **Nguyên nhân gốc rễ:** Lỗi không phải do `30m` sai, mà do **lệch pha (Desync) giữa DNA Parameter Space và MTF Precompute Cache**:
  * Các cảm biến trong `config.rs` cho phép gen chọn `30m`.
  * Nhưng danh sách `available_timeframes` và `HELIOS_REQUIRED_TIMEFRAMES` trước đây chưa nạp `30m` vào `GlobalCache` trước khi GA chạy.
- **Giải pháp dứt điểm:**
  1. **Đồng bộ hóa 100% danh sách Precompute MTF:** Đảm bảo `HELIOS_REQUIRED_TIMEFRAMES` (trong `helios_bridge.rs`), `ProspectorConfig::default()`, và `helios_optimizer_config.json` đều chứa đầy đủ các khung thời gian: `["3m", "5m", "10m", "15m", "30m", "1h", "2h", "4h"]`.
  2. **Bảo vệ fail-safe tại `ferro_trade/src/backtest/context.rs:136`:** Thay vì `expect("[FATAL]...")` làm crash sập tiến trình, nếu một chỉ báo đòi hỏi khung thời gian chưa có trong cache, hệ thống sẽ gọi `self.indicator_manager.mark_infeasible("MTF_Missing")` để loại bỏ cá thể đó trong êm đẹp.

### 2.2. Sửa Bẫy Ngắt Sớm `is_warmup_overflow` Tại `ferro_trade::engine`
- **Hiện tượng:** Candidate #1 có 82 trades ở In-Sample và thực tế có 84 trades trên full buffer ($84 - 82 = \mathbf{2\text{ trades}}$ trong 3 ngày OOS), nhưng WFE chẩn đoán trên lát cắt $10,080$ nến lại trả về $0$ trades.
- **Nguyên nhân:** $40\%$ của $10,080$ nến là $4,032$ nến. Các chỉ báo khung $4h$ có warmup $> 4,800$ nến khiến `is_warmup_overflow = true` và bị ngắt tại Bar 0.
- **Giải pháp:**
  1. Thêm `eval_start_bar: Option<usize>` vào `BacktestBuilder` và `Backtest`.
  2. Khi `eval_start_bar.is_some()`, vô hiệu hóa quy tắc $40\%$ và chỉ giao dịch/tính stats trên `eval_start_bar..full_len`.

### 2.3. Chuẩn Hóa WFE Gate
- Xóa bỏ trừ khống `val_trades * slippage_drag_roe`.
- Áp dụng tiêu chuẩn duyệt: `val_trades >= 1`, `val_ret >= 0.0%`, `val_dd <= 15.0%`.
- Khi toàn bộ Top-K fail: Giữ nguyên Champion cũ trong DB, không ghi đè cá thể 0 lệnh.
