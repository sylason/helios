# 🛡️ BÁO CÁO KIỂM TOÁN CHIẾN THUẬT & NGƯỠNG KÍCH HOẠT 6 SCANNERS (V3.3)

> **Phân hệ:** Six Regime Scanners (`trend_pb.rs`, `momentum_ignition.rs`, `reversal.rs`, `absorption.rs`, `range_osc.rs`, `anomaly.rs`)  
> **Chuyên đề:** Kiểm toán Lệch Ngữ Nghĩa (Semantic Mismatch Trap) & Lỗ Hổng Kích Hoạt  
> **Tài liệu liên quan:** [`config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs), [`sensors.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs), [`matrix.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs), [`core_v3/mod.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)

---

## 1. Executive Summary (Tóm Tắt Phát Hiện Cốt Tử)

Qua quá trình kiểm toán chi tiết từng dòng code trong thư mục `strategies_backtests/helios/src/strategy/core_v3/scanners/` đối chiếu với cấu hình DNA và dòng dữ liệu nến, kiểm toán phát hiện **5 LỖ HỔNG CẤP ĐỘ CRITICAL / HIGH** đang trực tiếp gây tê liệt hoặc làm sai lệch hành vi chiến thuật của các Scanners:

1. **`trend_pb.rs` (CRITICAL - Bẫy Tê Liệt Kích Hoạt & KAMA Suction Trap):**
   - `delta_kama = (close - kama_mid) / current_atr`: Khoảng cách KAMA vĩ mô ($15\text{m}, 1\text{h}, 4\text{h}$) bị chia cho ATR vi mô ($1\text{m}, 3\text{m}, 5\text{m}$). Khi KAMA chọn khung 1h hoặc 4h, `delta_kama` vọt lên $15 \to 50$ ATR vi mô, trong khi ngưỡng chặn trên DNA `max_pullback_dist_atr` tối đa chỉ $3.50$ ATR $\to$ **Trend_PB bị tê liệt 100%, không bao giờ kích hoạt trong xu hướng mạnh!**
   - `base_support = low.min(kama_mid)`: **Bẫy KAMA Suction Trap** (tương tự BBands Suction Trap từng xuất hiện ở Absorption). Kéo SL xuống tận KAMA 4h, khiến khoảng cách SL phình to $> 60\%$ ROE $\to$ Chắc chắn bị Sniper Guard VETO từ chối vào lệnh!
2. **`reversal.rs` (CRITICAL - Biến Không Khởi Tạo & Gen DNA Mồ Côi):**
   - `sensors.vol_sma` **CHƯA TỪNG ĐƯỢC KHỞI TẠO TRONG `init()`** (mảng rỗng) $\to$ `vol_ratio = vol_current / vol_sma` luôn fallback về $1.0$! Thành phần khối lượng kiệt sức của Reversal bị đóng băng ở một giá trị hằng số vô nghĩa!
   - `rsi_reversal` là **Gen DNA Mồ Côi (Orphan Sensor)**: Tốn 2 gen DNA và CPU tính toán trong `init()`, nhưng `ReversalScanner` hoàn toàn không đọc!
   - `suggested_tp = Some(kama_mid)`: Bất đối xứng rủi ro cực đoan (Vào lệnh & SL theo nến 1m nhưng TP lại kỳ vọng giá hồi 100% về KAMA 4h cách hàng nghìn USD).
3. **`absorption.rs` (HIGH - Lệch Pha Timeframe VWMA vs BB Middle):**
   - `vwma < bb_middle` (Long) và `vwma > bb_middle` (Short): `vwma.timeframe` và `bbands.timeframe` là 2 gen DNA độc lập không được đồng bộ hóa. Nếu DNA chọn VWMA 4h và BBands 5m, hệ thống đang so sánh giá trị gia quyền 3 ngày với đường trung bình 1.6 giờ!
4. **`range_osc.rs` (CRITICAL nếu tái kích hoạt):**
   - Truy cập trực tiếp `sensors.stoch_range_k[i]` khi mảng này đã bị comment out trong `init()` $\to$ **Panic crash ngay lập tức nếu được kích hoạt lại!**
   - Vẫn còn nguyên bẫy **BBands Suction Trap** (`low.min(bb_l)`).

---

## 2. BẢNG TỔNG KẾT KIỂM TOÁN TOÀN DIỆN 6 SCANNERS

| Scanner | Dòng Code | Phép Tính / So Sánh | Nguồn Gốc Đại Lượng A (Timeframe) | Nguồn Gốc Đại Lượng B (Timeframe/Threshold) | Phân Tích Lệch Ngữ Nghĩa & Hậu Quả Thực Tế | Mức Độ | Đề Xuất Chuẩn Hóa |
| :--- | :--- | :--- | :--- | :--- | :--- | :---: | :--- |
| **`trend_pb`** | L.79, 101, 148 | `delta_kama = (close - kama_mid) / current_atr` vs `max_dist` (0.5..3.5 ATR) | `close - kama_mid`: Giá 1m so với KAMA trung vị vĩ mô (**15m, 1h, 4h**) | `current_atr`: ATR_SL vi mô (**1m, 3m, 5m**); Ngưỡng DNA: `[0.5, 3.5]` | **Bẫy Tê Liệt Tuyệt Đối:** Khoảng cách KAMA 4h thường từ 500 - 2,000 USD (BTC), chia cho ATR 1m (30 USD) ra `delta_kama` = 15 - 60 ATR. Luôn vượt quá `max_pullback_dist_atr` (max 3.5). Scanner bị tê liệt hoàn toàn khi có trend mạnh 1h/4h! Nếu giá chạm sát KAMA thì chỉ cần nhúng 1.5 ATR 1m (~45 USD) là dính `invalidation_buffer` bị loại tiếp! | 🔴 **CRITICAL** | Chuẩn hóa khoảng cách bằng **ATR của chính Timeframe KAMA** (`mtf_atr` của 15m/1h/4h), hoặc tính `delta_kama = (close - kama_mid) / kama_atr`. |
| **`trend_pb`** | L.122, L.170 | `low.min(kama_mid)` (Long) & `high.max(kama_mid)` (Short) | `low` / `high`: Nến tín hiệu 1m | `kama_mid`: KAMA vĩ mô (**15m, 1h, 4h**) | **KAMA Suction Trap:** Khi giá đang tăng cách xa KAMA 4h, SL bị kéo tuột xuống tận KAMA 4h, làm khoảng cách SL phình to 40% - 80% ROE. Lệnh bị Sniper Guard VETO từ chối lập tức! | 🔴 **CRITICAL** | Sửa triệt để theo bài học `absorption.rs`: SL chỉ neo sau đáy/đỉnh nến vi mô: `let base_support = low;` (Long) và `let base_resistance = high;` (Short). |
| **`trend_pb`** | L.47-48 | `kama_slope` fallback giữa `matrix.kama_slope_atr` vs `sensors.kama_slope_adj` | `matrix.kama_slope_atr`: Đã chuẩn hóa qua `scale_atr` kẹp `[-1.0, 1.0]` | `sensors.kama_slope_adj`: Chưa chia `scale_atr` (lệch ~5 lần) | Không nhất quán thang đo giữa nhánh chính và nhánh fallback. | 🟢 **LOW** | Bỏ nhánh fallback hoặc chia `scale_atr` đồng nhất. |
| **`momentum_ign`** | L.109-114 | `sat_penetration(dist, target_breakout)` với `target_breakout = atr * 0.25` | `dist = (close - bb_u)` với BB Upper vĩ mô (**5m, 15m, 1h, 4h**) | `atr`: ATR vi mô (**1m, 3m, 5m**) | Khi BBands chọn khung 1h/4h, cú bứt phá vượt BB khung lớn tạo ra `dist` rất lớn so với `0.25 * atr_1m`, làm điểm số `f_b` lập tức bão hòa ở 1.0, mất hoàn toàn độ nhạy gradient. | 🟠 **HIGH** | `target_breakout` nên tính theo tỷ lệ độ rộng BB: `0.05 * (bb_u - bb_l)` hoặc theo ATR của khung BBands. |
| **`momentum_ign`** | L.60, 115-124 | `(rsi_mom - rsi_thresh)` | `rsi_momentum`: Khung thời gian vĩ mô (**5m, 15m, 1h, 4h**) | `rsi_thresh`: Ngưỡng bùng nổ nến vi mô 1m | Nếu DNA chọn RSI 1h/4h, RSI chạy rất chậm không phản ánh kịp cú nổ vol 1m, khiến `f_r` bị triệt tiêu về 0 dù nến 1m đang nổ vol cực mạnh. | 🟡 **MEDIUM** | Khóa timeframe của `rsi_momentum` chỉ trong phạm vi vi mô/ngắn hạn `[1m, 3m, 5m, 15m]`, loại bỏ 1h/4h. |
| **`reversal`** | L.55, 104-105 | `vol_ratio = vol_current / vol_sma` | `vol_current`: Volume nến 1m | `sensors.vol_sma`: **MẢNG RỖNG CHƯA KHỞI TẠO** trong `init()` | **Dead Logic Bug:** Vì mảng rỗng nên `vol_sma` luôn fallback về `vol_current` $\to$ `vol_ratio` luôn bằng 1.0! Điểm số khối lượng `sat_proximity(1.0, exhaust_drop)` bị đóng băng thành hằng số, mất hoàn toàn chức năng đo volume kiệt sức! | 🔴 **CRITICAL** | Khởi tạo `s.vol_sma = ctx.volume_sma(Timeframe::T1m, "VolSMA", 20)` trong `core_v3/mod.rs` `init()`, hoặc thay bằng `sensors.vol_z_score`. |
| **`reversal`** | Codebase wide | `ind.rsi_reversal` trong `config.rs` & `core_v3/mod.rs` | Khởi tạo RSI_Rev (5m..4h), tốn 2 gen DNA | `reversal.rs`: **HOÀN TOÀN KHÔNG ĐỌC** | **Gen DNA Mồ Côi:** Tốn 2 gen của GA và tốn CPU tính RSI mỗi bar nhưng Scanner không sử dụng, thay vào đó đọc MFI. | 🟠 **HIGH** | Hoặc tích hợp `rsi_reversal` vào `reversal.rs`, hoặc xóa bỏ/comment out 2 gen này trong `config.rs` để tinh giản DNA. |
| **`reversal`** | L.122 | `suggested_tp = Some(kama_mid)` | Điểm vào lệnh & SL: Vi mô nến 1m (SL râu nến 1m) | TP: KAMA vĩ mô (**15m, 1h, 4h**) cách hàng nghìn USD | **Bất đối xứng R:R vi mô vs vĩ mô:** Bắt dao rơi trên nến 1m với SL cực mỏng nhưng đòi chốt lời ở tận KAMA 4h. Giá thường chỉ nảy kỹ thuật 38.2% rồi quay đầu quét SL, khiến tỷ lệ chốt lời thành công cực thấp. | 🟠 **HIGH** | Đặt TP thực tế theo tỷ lệ hồi quy sóng vi mô: ví dụ `close + 2.0 * current_atr` hoặc trục giữa BB Middle vi mô, hoặc để Trailing Stop tự xử lý (`None`). |
| **`reversal`** | L.65-68 | `displacement = (close - kama_mid).abs() >= min_disp * current_atr` | `displacement`: Khoảng cách tới KAMA vĩ mô (**15m..4h**) | `min_disp * current_atr`: Tính bằng ATR vi mô (**1m..5m**) | Khi KAMA là 1h/4h, khoảng cách giá luôn lớn hơn vài ATR 1m, khiến bộ lọc `Remaining Displacement` mất tác dụng sàng lọc (luôn pass). | 🟡 **MEDIUM** | Chuẩn hóa `displacement` qua ATR của chính timeframe KAMA. |
| **`absorption`** | L.95-97 | `vwma < bb_middle` (Long) & `vwma > bb_middle` (Short) | `sensors.vwma`: Khung thời gian độc lập (**5m, 15m, 1h, 4h**) | `sensors.bb_middle`: Khung thời gian độc lập (**5m, 15m, 1h, 4h**) | **Lệch pha ngữ nghĩa Timeframe:** Hai chỉ báo không được đồng bộ timeframe. Nếu DNA chọn VWMA 4h và BBands 5m, điều kiện so sánh giá trị gia quyền 3 ngày với SMA 1.6 giờ, bóp méo hoàn toàn bản chất tích lũy/phân phối vi mô. | 🟠 **HIGH** | Khóa đồng bộ 100%: `self.config.indicators.vwma.timeframe = self.config.indicators.bbands.timeframe;` trong `core_v3/mod.rs` (như đã làm với Keltner Channel). |
| **`range_osc`** | L.77-78 | `sensors.stoch_range_k[i]` & `stoch_range_d[i]` | Truy cập mảng trực tiếp bằng `[i]` | `sensors.stoch_range_k`: **ĐÃ BỊ COMMENT OUT Ở INIT()** | **Crash Hazard:** Nếu kích hoạt lại `RangeOscScanner`, chương trình sẽ lập tức bị **Panic: index out of bounds** vì mảng rỗng! | 🔴 **CRITICAL** (nếu bật lại) | Chuyển sang đọc `sensors.stoch_trend_k` dùng chung hoặc khởi tạo lại cảm biến trước khi kích hoạt. |
| **`range_osc`** | L.149, 184 | `low.min(bb_l)` (Long) & `high.max(bb_u)` (Short) | `low` / `high`: Nến 1m | `bb_l` / `bb_u`: Bollinger Bands (**5m, 15m, 1h, 4h**) | **BBands Suction Trap:** Nếu BBands là khung 1h/4h, SL bị kéo ra ngoài biên BB khung lớn, phình to lỗ ROE >50% $\to$ Vi phạm trần SL. | 🔴 **CRITICAL** (nếu bật lại) | Sửa thành `let base_support = low;` và `let base_resistance = high;`. |
| **`anomaly`** | L.43-44 | `sensors.z_score[i]` & `sensors.atr_sl[i]` | Truy cập chỉ mục trực tiếp `[i]` | N/A | Tiềm ẩn rủi ro Panic nếu độ dài mảng warmup không đồng đều. | 🟢 **LOW** | Đổi sang `.get(i).copied()`. |

---

## 3. ĐỀ XUẤT CÁC BƯỚC KHẮC PHỤC CỤ THỂ (Action Items Cho Parent Agent)

### 3.1. Khắc phục Cấp thiết (Immediate Hotfixes cho 4 Playbooks Đang Chạy):
1. **Sửa `trend_pb.rs`:**
   - **Loại bỏ ngay KAMA Suction Trap:** Sửa `let base_support = low;` (Long) và `let base_resistance = high;` (Short).
   - **Chuẩn hóa thước đo khoảng cách KAMA:** Bổ sung `kama_atr` (ATR tính trên `ind.kama.timeframe`) vào `HeliosSensors`, sau đó tính:
     ```rust
     let delta_kama = (close - kama_mid) / kama_atr;
     ```
     Giải phóng Trend_PB khỏi bẫy tê liệt ở mọi khung thời gian!
2. **Sửa `reversal.rs`:**
   - **Khởi tạo cảm biến Volume SMA:** Gọi `s.vol_sma = ctx.volume_sma(Timeframe::T1m, "VolSMA", 20);` trong `core_v3/mod.rs` `init()` để hồi sinh logic Volume kiệt sức.
   - **Xử lý gen mồ côi `rsi_reversal`:** Hoặc đưa `rsi_reversal` vào chấm điểm kiệt sức, hoặc xóa bỏ khỏi `config.rs` để tiết kiệm 2 gen DNA.
   - **Thực tế hóa Take Profit:** Đổi `suggested_tp` từ `Some(kama_mid)` sang `None` (để Trailing Stop quản trị) hoặc đặt mục tiêu thực tế theo sóng nảy vi mô $2.0 \times \text{ATR}_{1\text{m}}$.
3. **Sửa `absorption.rs`:**
   - **Đồng bộ hóa Timeframe VWMA theo BBands:** Trong `core_v3/mod.rs`:
     ```rust
     self.config.indicators.vwma.timeframe = self.config.indicators.bbands.timeframe;
     ```
