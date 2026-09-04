# 🚨 Báo Cáo Kiểm Toán Tầng 4: Playbook Scanners, Arbiter & Vòng Đời Chiến Lược

**Dự án:** `helios` & `trading_bot`  
**Ngày kiểm toán:** 2026-09-04  
**Trạng thái:** ⚠️ **PHÁT HIỆN 8 LỖ HỔNG LOGIC & TOÁN HỌC (5 CRITICAL, 2 HIGH, 1 MAJOR)**

---

## 📌 Bảng Tổng Hợp Lỗ Hổng Phát Hiện Tại Tầng 4

| STT | Mức Độ | Tên Lỗ Hổng | Vị Trí Mã Nguồn | Hậu Quả Thực Tế |
| :---: | :---: | :--- | :--- | :--- |
| **1** | 🔴 **CRITICAL** | **"DNA Dead Zone" Trong Arbiter: >50% Quần Thể GA Bị Bất Khả Thi Toán Học** | [`arbiter.rs:L102-140`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L102-L140)<br>[`config.rs:L245`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L245) | `confidence = score * regime_weight * multiplier` nhân 3 phân số nên hầu như không bao giờ vượt quá **0.45**. Khi GA chọn `min_confidence >= 0.50` (chiếm >50% không gian DNA), điều kiện duyệt lệnh **bất khả thi về mặt toán học** $\implies$ Hơn 50% cá thể GA ra 0 trades! |
| **2** | 🔴 **CRITICAL** | **Take Profit Không Được Validate Dẫn Đến Đóng Nhầm Chiều (Wrong-Side TP)** | [`mod.rs:L275-L278`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L275-L278)<br>[`range_osc.rs:L67, L75`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L67) | `decision.final_tp` chuyển nguyên vẹn sang Broker không qua kiểm tra. Trong `range_osc`, nếu nến bứt phá ra ngoài dải (`close > bb_u`), TP của Long lại **thấp hơn giá Entry**, làm vị thế vừa mở đã bị đóng ngay ở nến sau với giá lỗ! |
| **3** | 🔴 **CRITICAL** | **Lỗi Trị Tuyệt Đối `.abs()` Làm Bot Bắt Dao Rơi Khi Giá Sập Hầm** | [`mod.rs:L228-L233`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L228-L233) | `price_diff = (current_price - last_exit).abs()`. Khi lệnh Long vừa cắt lỗ mà giá **tiếp tục sập hầm $\ge 1.5 \text{ATR}$**, hàm `.abs()` vẫn mở khóa Cooldown, khiến bot nhảy vào **Long bắt dao rơi ngay giữa đà lao dốc**! |
| **4** | 🔴 **CRITICAL** | **Nhận Diện Sai Lệnh Thắng / Thua (False Win) Khi Nến Rút Chân** | [`mod.rs:L180-L189`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L180-L189) | Tính PnL bằng `current_close - entry_price` thay vì giá khớp SL thực tế. Nếu Long bị cắn SL ở đáy nhưng cuối nến rút chân tăng, code nhận diện SAI là lệnh thắng $\rightarrow$ Không kích hoạt Cooldown và **đu đỉnh mua lại ngay đỉnh nến rút chân**! |
| **5** | 🔴 **CRITICAL** | **Lỗi "Double Tanh" Dìm Điểm Tê Liệt Reversal & Anomaly Scanners** | [`reversal.rs:L35`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L35)<br>[`anomaly.rs:L39`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs#L39) | `matrix.price_z_score` đã bị nén qua `tanh` về $(-1.0, 1.0)$. Scanner lại lấy nó chia cho `z_thresh` ($2.5 \sim 3.0$) rồi `tanh` lần 2. Điểm $f_z$ tối đa chỉ đạt **0.32** (kể cả khi giá xả 10-sigma), dìm điểm `raw_score` luôn $< 0.55$, **làm liệt 2 scanner này**! |
| **6** | 🟡 **HIGH** | **Lệch Thang Đo Volume Dìm Điểm Absorption & Momentum** | [`absorption.rs:L34`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L34)<br>[`config.rs:L418`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L418) | `matrix.volume_force` có dải $[0.0, 1.0]$, nhưng `vol_z_threshold` lại cấu hình $[0.20, 5.00]$. Khi GA chọn $\ge 2.0$, điểm volume luôn $< 0.32$, khiến Absorption Scanner **bị liệt 0 tín hiệu trên 75% không gian GA**! |
| **7** | 🟡 **HIGH** | **Đấu Thầu "Naive Winner-Takes-All" Mù Quáng Khi Xung Đột Tín Hiệu** | [`arbiter.rs:L142-L146`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L142-L146) | Nếu Scanner A báo Long 0.75 và Scanner B báo Short 0.80, bên 0.80 thắng tuyệt đối với 0 điểm phạt. Thiếu cơ chế chiết khấu rủi ro tranh chấp (Conflict Penalty), rất dễ bị dính quét 2 đầu (Whipsaw). |
| **8** | 🟠 **MAJOR** | **Lệnh Mở Và Cắn SL Trong Cùng 1 Nến Bị "Tàng Hình" Khỏi Cooldown** | [`mod.rs:L176-L197`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L176-L197) | Nếu lệnh mở ở Open và cắn SL ở Low trong cùng nến `i`, `prev_open_trade` chưa kịp lưu. Khối Cooldown bị bỏ qua hoàn toàn, bot có thể mở tiếp 1 lệnh mới ngay trên nến vừa bị quét SL. |

---

## 🛡️ Điểm Sáng Đã Kiểm Chứng
1. **Tính đối xứng Long/Short hoàn hảo:** Cả 6 Scanners đều phản chiếu toán học sạch sẽ giữa 2 chiều Mua/Bán qua các mốc trung vị KAMA, dải Bollinger và Stochastic.
2. **Không bị chia cho 0 khi nến phẳng lì:** Nhờ kẹp mẫu số `wick_thresh * current_atr` với $\text{ATR} > 0$ và `wick_thresh >= 0.1`, các hình thái nến râu/thân không bao giờ sinh ra `Inf` hay `NaN`.

---

## 🛠️ Kế Hoạch Khắc Phục Khuyến Nghị (Action Plan Cho Tầng 4)

1. **Chuẩn hóa lại Thang Đo Arbiter (Phá vỡ DNA Dead Zone):**
   - Đưa `confidence` về dải $[0.0, 1.0]$ chuẩn bằng cách cộng trọng số hoặc bù trừ hệ số Simplex:
     `confidence = proposal.raw_score * multiplier * (regime_weight / max_expected_w)`.
2. **Bổ sung `RiskManager::clamp_tp_price`:**
   - Kiểm tra `decision.final_tp`: Nếu Long thì bắt buộc `tp > entry * (1.0 + min_tp_dist)`, nếu Short thì `tp < entry * (1.0 - min_tp_dist)`. Nếu không thỏa mãn thì hủy bỏ TP (để Trailing SL tự quản lý).
3. **Sửa Định Hướng Volatility Breakout Override:**
   - Bỏ `.abs()`, kiểm tra nghiêm ngặt theo hướng vị thế:
     - Long: `current_price - last_exit >= 1.5 * current_atr`.
     - Short: `last_exit - current_price >= 1.5 * current_atr`.
4. **Sửa lỗi Double Tanh:**
   - Trong `reversal.rs` và `anomaly.rs`, đọc trực tiếp `sensors.z_score[i]` (Z-score thô) thay vì `matrix.price_z_score[i]`.
5. **Sửa dải `vol_z_threshold`:**
   - Đưa về dải $[0.05, 0.95]$ để tương thích với `matrix.volume_force`.
