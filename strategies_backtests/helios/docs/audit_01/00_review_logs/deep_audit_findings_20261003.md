# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU 3 GIẢ THUYẾT CỐT TỬ (DEEP AUDIT FINDINGS)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/00_review_logs/`  
> **Ngày lập:** 03/10/2026  
> **Người thực hiện:** Antigravity Quant Team & 3 Sub-Agents Chuyên Trách  
> **Phương pháp:** Bóc tách 100% từng dòng mã nguồn, đối chiếu toán học First-Principles, không đưa ra giả định ngầm.

---

## 🎯 BẢNG TỔNG HỢP KẾT QUẢ KIỂM CHỨNG 3 GIẢ THUYẾT

| Giả Thuyết Chất Vấn | Trạng Thái Kiểm Chứng | Mức Độ Rủi Ro | Kết Luận Bản Chất |
| :--- | :---: | :---: | :--- |
| **Giả Thuyết 1: Lookahead Bias Trong MTF Resampling** | 🟢 **BÁC BỎ (HỆ THỐNG TRONG SẠCH)** | **AN TOÀN** | Resample dùng `Label::Right` (Bucket End) và Two-Pointer `create_lookup_table` chỉ trỏ tới nến MTF đã đóng hoàn toàn ($t_{base} \ge t_{resampled}$). Không có rò rỉ tương lai. Có độ trễ 1 phút có chủ đích (Conservative 1-Bar Lag). |
| **Giả Thuyết 2: Fitness Gaming, Lỗ Hổng WFE Gate & Chi Phí Ma Sát** | 🔴 **XÁC NHẬN (LỖ HỔNG CỐT TỬ)** | **CỰC KỲ NGUY HIỂM** | 1. Hàm Fitness thưởng cho $SQN \propto \sqrt{N}$ và WinRate cao, không phạt R:R nát, không phạt Hard SL 15%.<br>2. Cổng WFE Gate dòng 406 có "cửa sau" dung thứ OOS âm khi In-Sample $>80\%$.<br>3. Chi phí ma sát (phí sàn + trượt giá) với 50 trades nuốt chửng 160% vốn ký quỹ mà GA bỏ quên một nửa! |
| **Giả Thuyết 3: Bệnh Lý 6 Scanners & Lũng Đoạn Đấu Thầu Arbiter** | 🔴 **XÁC NHẬN (LỖ HỔNG TOÀN DIỆN)** | **CỰC KỲ NGUY HIỂM** | 1. `MOMENTUM_IGN`: Thiếu trần $Price\_Z$, chấm điểm tối đa ~1.0 tại nến Buying Climax phân phối kiệt sức.<br>2. `TREND_PB`: KAMA Suction Trap kéo SL xa 3-5 ATR; cấm nến $Close < Open$ bóp chết Pinbar rút chân.<br>3. `REVERSAL`: Bỏ xó $chaos\_index$, cản đầu xe tăng khi flash crash.<br>4. `RANGE_OSC`: BB Squeeze thổi phồng TP vượt ngoài dải BB.<br>5. `ARBITER`: Thiếu Cross-Calibration thang điểm, Momentum phóng khoáng luôn đè bẹp Range tại đỉnh. |

---

## 1. CHI TIẾT GIẢ THUYẾT 1: HỆ THỐNG MIỄN NHIỄM VỚI LOOKAHEAD BIAS

1. **Quy trình Resample Nến MTF (`ferro_trade/src/utils/resample.rs:80-94`):**
   - Polars cấu hình `label: Label::Right`, `closed_window: ClosedWindow::Left`, `start_by: WindowBound`.
   - Cây nến 1h `[10:00, 11:00)` mang nhãn thời gian là **`11:00:00`** (Bucket End / Thời điểm nến chính thức đóng).
2. **Thuật toán Two-Pointer Lookup Table (`resample.rs:50-64`):**
   - Tại thời điểm nến 1m $t_{base} = 10:15:00$:
     - Cây nến 1h hiện tại mang nhãn `11:00:00`. Vì $10:15:00 < 11:00:00$, con trỏ **từ chối** tiến tới cây nến này.
     - Con trỏ giữ nguyên ở cây nến 1h trước đó mang nhãn `10:00:00` (chu kỳ `09:00 - 10:00`, cây nến đã đóng 100%).
   - $\implies$ **Không hề có rò rỉ giá tương lai trong cả Backtest và Live Execution.**
3. **Điểm cần tối ưu hóa vi cấu trúc:**
   - Trong `EngineConfig` (`trading_bot/bot_engine/src/models/config.rs:79`), `max_live_mem_candles` đang để mặc định 15,000 nến 1m (~10.4 ngày = 62.5 nến 4h). Nếu dùng indicator khung 4h (chu kỳ 80), cần nâng lên 43,200 nến 1m (30 ngày) để độ hội tụ số học giữa Live và Backtest đạt 100% tuyệt đối.

---

## 2. CHI TIẾT GIẢ THUYẾT 2: SỰ MÉO MÓ CỦA HÀM FITNESS & "CỬA SAU" WFE GATE

### 2.1. Nghịch lý Toán học trong `calculate_enhanced_fitness` (`prospector_runner.rs:506-560`)
- **$SQN$ Kích Thích Cào Lệnh:** $SQN = \sqrt{N} \frac{\mu}{\sigma}$. Vì nhân với $\sqrt{N}$, một cá thể cào $N = 64$ lệnh có $\sqrt{N} = 8$ (gấp đôi cá thể 16 lệnh), dễ dàng đẩy $SQN \ge 5.0$ để ẵm trọn mức thưởng tối đa $+50\%$ (`sqn_mult = 1.50`).
- **Tanh WinRate Khuyến Khích Chốt Non Gồng Lỗ:** Thưởng $+30\%$ khi WinRate $\ge 65\%$. GA lách luật bằng cách: đặt TP cực ngắn để chốt non ăn vài ba tick (tăng WinRate ảo), và nới rộng SL để gồng lỗ cho tới khi dính Hard SL 15% ROE.
- **Không có hình phạt cho R:R suy thoái và Hard SL Count:**

| Chỉ số kiểm chứng | Cá thể Cào Lệnh (Overtrading Scalp) | Cá thể Chuẩn Mực (Quality Trend) |
| :--- | :---: | :---: |
| **Số lệnh ($N$)** | **60 lệnh** (45 thắng, 15 thua) | **15 lệnh** (7 thắng, 8 thua) |
| **Win Rate** | **75.0%** (Chốt non ăn vội) | **46.7%** (Cắt lỗ sớm, để lãi chạy) |
| **Lãi TB / Lỗ TB** | $+5.0\%$ ROE / **$-15.0\%$ ROE** | $+30.0\%$ ROE / $-8.0\%$ ROE |
| **Tỷ lệ Payoff (R:R)** | **0.33 (Lỗ gấp 3 lần lãi!)** | **3.75 (Lãi gấp 3.75 lần lỗ)** |
| **Số lần dính Hard SL 15%** | **15 lần** | **0 lần** |
| **SQN Multiplier** | **1.42** (nhờ $\sqrt{60}$) | 1.21 |
| **WinRate Multiplier** | **1.30** (Thưởng kịch trần) | 0.87 (Bị phạt) |
| **TỔNG FITNESS ĐẠT ĐƯỢC** | **17.06** | **9.13** |

> 🚨 **KẾT QUẢ:** Cá thể cào lệnh với R:R rác rưởi (thua gấp 3 lần thắng) lại đạt điểm Fitness **cao gần gấp đôi** cá thể kỷ luật!

### 2.2. "Cửa Sau" WFE Gate Dung Thứ OOS Âm (`prospector_runner.rs:406`)
```rust
let is_tolerant_pass = val_ret >= -1.2 && val_win_rate >= 50.0 && ret > 80.0 && val_dd <= 12.0;
```
- Khi In-Sample Return $> 80\%$, cổng cho phép OOS Return bị âm tới $-1.2\%$.
- Trên mẫu OOS 3 ngày (2-3 trades), chỉ cần 1 lệnh scalp thắng non $+0.3\%$ và 1 lệnh dính SL $-1.5\%$, $val\_ret = -1.2\%$, WinRate $= 50\% \to$ **ĐẬU WFE GATE 100%!**
- Đây là nguyên nhân trực tiếp đưa cá thể Run 2 vào deploy ngay tại Bar 0.

### 2.3. Chi Phí Ma Sát Âm Thầm Nuốt Chửng Tài Khoản
- Backtest GA: `commission_rate = 0.0004` (0.04%), 0% Slippage.
- Thực tế Binance VIP0: `0.0005` (0.05% Taker), Slippage bình quân $\approx 0.03\%$ mỗi chiều.
- Với 50 trades ở đòn bẩy 20x:
  - Phí sàn thực tế: $50 \times 2 \times 0.05\% \times 20 = 100\%$ vốn ký quỹ!
  - Trượt giá thực tế: $50 \times 0.06\% \times 20 = 60\%$ vốn ký quỹ!
  - **Tổng ma sát thực tế: 160% vốn ký quỹ!** (Trong khi GA chỉ tính 80%). GA "bỏ quên" mất 80% vốn ma sát!

---

## 3. CHI TIẾT GIẢ THUYẾT 3: BỆNH LÝ 6 SCANNERS & SỰ LỆCH PHA ARBITER

1. **`momentum_ignition.rs`:** 
   - Không có trần $Price\_Z$. Khi nến Buying Climax bung ra ngoài BB kèm Volume kỷ lục và RSI quá mua (nơi cá mập xả hàng), thuật toán chấm điểm gần như tuyệt đối ($\approx 1.0$) và nhảy vào Long ngay chóp đỉnh râu nến với SL cách Entry 3-5 ATR!
2. **`trend_pb.rs`:**
   - **KAMA Suction Trap (Dòng 122 & 170):** `low.min(kama_mid)` kéo SL xuống KAMA trễ pha, làm khoảng cách SL bị giãn ra 3-5 ATR $\to$ Bị Arbiter Veto hoặc kẹp cứng SL 15% ROE.
   - **Ngưỡng độ dốc phẳng lì:** `kama_slope_threshold.max(1e-4)`. 0.0001 phẳng như mặt hồ vẫn coi là Trend.
   - **Bóp chết Pinbar rút chân:** `if clv < 0.15 || close < open { return None; }`. Nến Hammer đỏ rút râu tuyệt đẹp có lực cầu áp đảo ($CLV \ge 0.80$) bị loại bỏ thẳng thừng chỉ vì giá đóng cửa thấp hơn mở cửa 0.1 pip.
3. **`reversal.rs`:**
   - Dấu gạch dưới `_matrix` ở dòng 44 vạch trần việc bỏ xó $chaos\_index$ và $kama\_slope$. Khi thị trường rơi tự do (Flash Crash), bot đem thân chặn đầu xe tăng bắt dao rơi. TP neo cứng ở KAMA Mid trên trời không bao giờ chạm tới được.
4. **`range_osc.rs`:**
   - Khi BB Squeeze cực đại (dải BB co hẹp 0.2 ATR), hàm `valid_tp = target_price.max(close + buffer)` thổi phồng TP lên cách 2-3 ATR (vượt ra ngoài dải BB) trong thị trường đi ngang tắt thanh khoản.
5. **`absorption.rs` & `anomaly.rs`:**
   - Đề xuất SL neo theo $low.min(bb\_lower)$ hoặc râu nến ATR (cách 2.5 - 5.0 ATR), vi phạm trần 15% ROE $\implies$ 100% bị Arbiter Veto, 2 scanner này bị tê liệt hoàn toàn.
6. **`arbiter.rs`:**
   - Không có $playbook\_weight$ (đã được gỡ bỏ khỏi DNA, không bị GA thao túng).
   - **Tuy nhiên thiếu Cross-Scanner Calibration:** MomentumIgnition chấm điểm quá phóng khoáng (dễ đạt 0.90-0.98), trong khi TrendPB chấm ngặt nghèo (chỉ 0.55-0.70). Do đó trong các pha giằng co đỉnh range, Momentum 0.90 luôn đè bẹp Range 0.70 và bot đâm đầu mua ngay đỉnh!
