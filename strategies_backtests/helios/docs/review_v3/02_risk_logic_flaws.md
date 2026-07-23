# Helios V3 Core Review: Risk Management

## Review: `risk.rs` & `config.rs` (Lỗ hổng Logic Risk Management)

### Quan sát & Phân tích
1. **Dead Code trong Risk Parameters:**
   Trong `RiskStopParams` của `config.rs`, có đến 7 thông số được mở cho Genetic Algorithm tối ưu hóa (từ dòng 321 đến 334):
   - `squeeze_sl_factor`, `trend_sl_factor`, `momentum_sl_factor`, `range_sl_factor`, `reversal_sl_factor`, `anomaly_sl_factor`, `absorption_sl_factor`.
   Tuy nhiên, khi kiểm tra trong `mod.rs` (dòng 168), hàm `get_initial_sl` luôn luôn nhận đầu vào mặc định là `tsl_atr_normal`. Những thông số trên hoàn toàn là **DEAD CODE**.
   - **Hậu quả:** Việc mở ra 7 biến số vô dụng trong DNA khiến không gian tìm kiếm của Optimizer bị phình to vô ích (Dimensionality Curse). Optimizer sẽ lãng phí hàng ngàn phép thử để thay đổi các biến này mà không mang lại bất kỳ sự thay đổi nào về Fitness.
   - **Đề xuất:** Xóa bỏ 7 thông số này khỏi `config.rs` hoặc sửa logic trong `mod.rs` để truyền đúng hệ số SL dựa trên `decision.winning_scanner`.

2. **Cơ chế Break-Even chưa hoàn chỉnh:**
   `config.rs` có khai báo `use_breakeven` và `be_profit_target_rr`. Tuy nhiên, trong `risk.rs`, không hề có đoạn code nào dời SL về Entry Price (Break Even) khi giá đạt đến RR mục tiêu.
   - **Hậu quả:** Lệnh chỉ phụ thuộc hoàn toàn vào Trailing Stop. Trailing Stop đôi khi chạy khá chậm (ví dụ `tsl_atr_normal = 2.98 ATR`). Nếu giá chạy được 2.5 ATR rồi đảo chiều cực mạnh quét SL ban đầu, lệnh sẽ bị lỗ toàn bộ (full SL) thay vì hòa vốn.
   - **Đề xuất:** Cần thêm logic Break Even (chặn lỗ bằng giá Entry) vào `manage_trailing_stop`.
