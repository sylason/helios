# 🚨 Báo Cáo Kiểm Toán Tầng 3: Sensors, Chỉ Báo & Bộ Nhớ Trượt (Sensors & Indicators)

**Dự án:** `ferro_trade` & `helios` & `trading_bot`  
**Ngày kiểm toán:** 2026-09-04  
**Trạng thái:** ⚠️ **PHÁT HIỆN 10 LỖ HỔNG TOÁN HỌC & LOGIC (4 CRITICAL, 4 MAJOR/HIGH, 2 MEDIUM)**

---

## 📌 Bảng Tổng Hợp Lỗ Hổng Phát Hiện Tại Tầng 3

| STT | Mức Độ | Tên Lỗ Hổng | Vị Trí Mã Nguồn | Hậu Quả Thực Tế |
| :---: | :---: | :--- | :--- | :--- |
| **1** | 🔴 **CRITICAL** | **Đóng Băng Cooldown Vĩnh Viễn Trong Live/Paper Trading** | [`core_v3/mod.rs:L190, L222`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L190)<br>[`helios_bridge.rs:L211`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L211) | Trong Live/Paper, `rolling_buffer` cố định 43,100 nến nên `i = last_idx = 43099` ở mọi phút. Phép so sánh `i < locked_until_bar` (ví dụ `43099 < 43109`) **LUÔN ĐÚNG MÃI MÃI**, bot bị **khóa Cooldown vĩnh viễn không thể mở lệnh mới**! |
| **2** | 🔴 **CRITICAL** | **`volume_force` Lấy Nhầm Z-Score GIÁ Thay Vì KHỐI LƯỢNG** | [`matrix.rs:L94-L101`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L94-L101) | `tanh_scale(&sensors.z_score, 0.5)` truyền vào Z-Score của GIÁ. Biến `volume_lookback` bị bỏ hoang. Lực volume thực chất là biến động giá + MFI, làm sai lệch toán học của `Absorption` và `MomentumIgnition`. |
| **3** | 🔴 **CRITICAL** | **Sập Simplex Normalization (Tổng Trọng Số Vọt Lên 200%) Làm Tê Liệt Arbiter** | [`bulk.rs:L1647`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1647)<br>[`matrix.rs:L143-L151`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L143-L151) | `chaos_ratio` sinh ra `NaN`, đi vào ma trận làm `sum` thành `NaN`. Biểu thức `NaN.max(1e-5)` ép mẫu số về `1e-5`, khiến $w_{trend} = 1.0$, $w_{range} = 1.0$ (tổng $= 2.0$). Ngưỡng tự tin Arbiter vọt lên $> 1.20$, **tê liệt hoàn toàn không duyệt được lệnh nào**! |
| **4** | 🔴 **CRITICAL** | **Cache Collision Trong `IndicatorType::KamaSlope`** | [`identifier.rs:L119`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L119)<br>[`bulk.rs:L74-L87`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs#L74-L87) | `KamaSlope` **hoàn toàn thiếu trường `atr_period`**. Khi GA tối ưu hoặc 2 scanner gọi với các `atr_period` khác nhau (ví dụ 14 vs 21), lần gọi 2 bị **hit cache sai của lần 1**, nhận kết quả sai toán học 100%! |
| **5** | 🟠 **MAJOR** | **`CacheKey` Thiếu Symbol/Cặp Tiền** | [`identifier.rs:L152-L157`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L152-L157)<br>[`global_cache.rs:L77-L91`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/data/global_cache.rs#L77-L91) | `CacheKey` không có trường `symbol`. Nếu dùng chung `GlobalCache` cho nhiều cặp tiền (ví dụ BTC và ETH), ETHUSDT sẽ **lấy nhầm toàn bộ nến resampled và chỉ báo của BTCUSDT**! |
| **6** | 🟠 **MAJOR** | **`add_raw_cached` Gắn Cứng Khung 1m Đè Cache Đa Khung Thời Gian** | [`manager.rs:L70-L76`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/manager.rs#L70-L76) | Khóa cache bị gán cứng `Timeframe::T1m`. Hai chỉ báo cùng tên khác khung (ví dụ EMA 1h và EMA 4h) sẽ **đè cache lẫn nhau trực tiếp**. |
| **7** | 🟠 **MAJOR** | **Bypass Bộ Lọc Volume Khi Gặp `NaN` (IEEE-754)** | [`range_osc.rs:L23-L33`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L23-L33) | Theo chuẩn IEEE-754, `NaN > threshold` trả về `false`. Bộ lọc bùng nổ volume trong `range_osc` bị **vô hiệu hóa hoàn toàn**, bot mở lệnh sideway ngay giữa cơn bão biến động! |
| **8** | 🟡 **HIGH** | **KAMA Memory Drift Gây Lật Tín Hiệu Long/Short Khi Sideway** | [`bulk.rs:L1561`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1561)<br>[`trend_pb.rs:L37`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L37) | KAMA khung 4h lưu giữ tới $47\% - 89\%$ ký ức khởi tạo sau 179 nến. Sai số $0.1\% - 0.5\%$ làm đổi dấu độ dốc `kama_slope_adj`, khiến Backtest và Live ra quyết định ngược chiều nhau! |
| **9** | 🔵 **MEDIUM** | **Float Epsilon Dedup Mismatch Trong `rolling_percentile`** | [`normalization.rs:L77, L88`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/utils/normalization.rs#L77) | `dedup_by` dùng epsilon nhưng `binary_search` dùng `total_cmp`. Các số thực lệch nhau $< \text{EPSILON}$ bị mất dấu, làm `binary_search` fail và **biến nến hợp lệ thành `NaN`**! |
| **10** | 🔵 **MEDIUM** | **Bão Hòa Nhị Phân Sớm Của `trend_strength` Trong `tanh_scale`** | [`matrix.rs:L81`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L81) | Dùng nhầm `z_score_sensitivity` ($1 \to 10$) cho ADX ($0 \to 100$). Chỉ cần ADX $\ge 30$, hàm nén đã ép chặt về $0.997$ (bão hòa nhị phân thay vì thang đo mượt). |

---

## 🛡️ Điểm Sáng Toán Học Đã Được Chứng Minh
1. **Thuật toán Fenwick Tree trong `rolling_percentile`:** Đạt độ phức tạp tối ưu $O(N \log N)$ thay vì $O(N \cdot W)$ thông thường, xử lý Hazen Mid-Rank khi thị trường đóng băng (tất cả giá trị bằng nhau) cho kết quả chuẩn xác $0.50$ và không bao giờ bị chia cho 0.
2. **FIR Indicators Tuyệt Đối Miễn Nhiễm Với Buffer Trượt:** Các chỉ báo như Z-Score, Bollinger Bands, Efficiency Ratio có độ lệch số học bằng **$0.00000000$ (trùng khớp $100\%$)** giữa Full Backtest 90 ngày và Live Rolling Buffer 30 ngày.

---

## 🛠️ Kế Hoạch Khắc Phục Khuyến Nghị (Action Plan Cho Tầng 3)

1. **Khắc phục Khóa Cooldown (Chuyển sang Timestamp):**
   - Đổi `locked_until_bar: usize` thành `locked_until_timestamp: i64`.
   - So sánh: `tick.datetimes()[i] < self.cooldown_tracker.locked_until_timestamp`.
2. **Khắc phục `volume_force`:**
   - Bổ sung `vol_z_score` vào `HeliosSensors`, tính Z-Score trên khối lượng và truyền đúng vào `matrix.rs`.
3. **Phòng vệ `NaN` và Simplex Normalization:**
   - Sửa `chaos_ratio` trong `ferro_trade`: kiểm tra `if v.is_nan() || sma_val.is_nan() || sma_val < 1e-9 { 1.0 }`.
   - Trong `matrix.rs`: kẹp `w_volatile_raw = 0.0` nếu là `NaN` trước khi cộng tổng `sum`.
4. **Sửa Khóa Cache `KamaSlope` & `CacheKey`:**
   - Thêm `atr_period` vào `IndicatorType::KamaSlope`.
   - Thêm `symbol` vào `CacheKey` để `GlobalCache` dùng chung an toàn.
5. **Sửa `dedup_by` trong `normalization.rs`:**
   - Dùng `a.total_cmp(b).is_eq()` để khớp 100% với `binary_search`.
