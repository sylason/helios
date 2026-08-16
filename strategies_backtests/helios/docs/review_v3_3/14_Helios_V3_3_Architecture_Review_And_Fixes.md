# HELIOS V3.3 ARCHITECTURE REVIEW: PART 14 - DETAILED DIAGNOSTIC REPORT & ARCHITECTURAL SOLUTIONS

Tài liệu này tổng hợp kết quả rà soát sâu rộng toàn bộ mã nguồn `src/` của dự án **Helios V3.3** (`strategies_backtests/helios/src/`), đối chiếu trực tiếp với kết quả kiểm thử Walk-Forward 365 ngày của **V3 Cũ (`helios_legacy`)**, xác định chính xác các nguyên nhân kỹ thuật khiến V3.3 bị tịt ngòi tín hiệu và đề xuất giải pháp kiến trúc khắc phục triệt để.

---

## 🏛️ 1. TỔNG QUAN BỐI CẢNH & ĐỐI CHIẾU HIỆU SUẤT

| Chỉ Số Đánh Giá | Mô Hình V3 Cũ (`helios_legacy`) | Mô Hình V3.3 Vừa Refactor (`helios`) |
|---|---|---|
| **Tổng Lợi Nhuận OOS (30-60 Ngày)** | **$+166.98\% \rightarrow +431.25\%$** (Duy trì xanh liên tục suốt 60 ngày) | $+138.74\%$ |
| **Tổng Số Lượng Lệnh/Tháng** | $30 \rightarrow 118$ lệnh (Linh hoạt theo bối cảnh) | 32 lệnh (Bị khóa cứng) |
| **Phân Bổ Playbooks Phát Tín Hiệu** | Đa dạng 6 Playbooks | **$100\%$ Trades từ `ABSORPTION`** (5 Playbooks còn lại $0\%$) |
| **Tần Suất Đi Lệnh** | Nâng đều suốt 30 ngày | Ngày đầu tiên trade là **Nến #6.878 (Ngày thứ 5)** |
| **Thời Gian Giữ Lệnh Trung Bình** | $120 \rightarrow 400$ nến ($2 \rightarrow 6.5$ giờ) | $1.000 \rightarrow 2.694$ nến ($16 \rightarrow 45$ giờ) |

---

## 🔬 2. XÁC ĐỊNH CHÍNH XÁC 4 VẤN ĐỀ CỐT LÕI VÀ NGUYÊN NHÂN SÂU XA TRONG `helios/src/`

### 🚨 Vấn Đề 1: Hiện Tượng "Phễu Cạn Tín Hiệu" (Funnel Drain Effect) Tại Thượng Tầng Scanners

- **Mô tả hiện tượng**: 5 Playbooks (`TREND_PB`, `MOMENTUM_IGN`, `RANGE_OSC`, `REVERSAL`, `ANOMALY`) hoàn toàn không phát ra bất kỳ lệnh nào trong suốt 30 ngày backtest.
- **Nguyên nhân dòng code (Root Cause)**:
  - Cả 5 Scanners đều đang lồng ghép từ **3 đến 5 câu lệnh kiểm tra cứng `AND`** (`if A && B && C && D`). Nếu bất kỳ tiêu chí nhỏ nào bị lệch $0.01\%$, Scanner lập tức `return None` và ngắt tín hiệu.
  - **Dẫn chứng 1 (`range_osc.rs:50`)**:
    ```rust
    if close <= bb_l * 1.005 && stoch_k <= stoch_os && stoch_k > stoch_d
    ```
    *Phân tích Toán học*: Trên nến 1 phút, khi giá chạm dải dưới Bollinger Bands (`close <= bb_lower * 1.005`), chỉ số Stochastic %K thường đang cắm xuống (%K < %D). Ngay giây phút %K bắt đầu móc lên (%K > %D) thì giá nến đã nảy lên cao hơn mốc `bb_lower * 1.005`. **Do đó 2 điều kiện này gần như KHÔNG BAO GIỜ XẢY RA ĐỒNG THỜI TRÊN CÙNG 1 MINUTE BAR!**
  - **Dẫn chứng 2 (`trend_pb.rs:40`)**:
    ```rust
    if slope_adj > slope_thresh && stoch_k < stoch_oversold && stoch_k > stoch_d && rsi <= rsi_buy_max
    ```
    Xếp chồng 5 điều kiện `AND` cứng (`ADX > 0.60`, `KAMA Slope > 0.10`, `Stoch < 30`, `Stoch %K > %D`, `RSI <= 50`). Trong một xu hướng tăng rất mạnh, Stochastic hiếm khi rơi sâu xuống dưới 30 đồng thời móc lên và RSI $\le 50$, làm triệt tiêu $100\%$ tín hiệu Pullback.

---

### 🚨 Vấn Đề 2: Khóa Lãnh Thổ Cứng Trong Regime Router (`arbiter.rs`)

- **Mô tả hiện tượng**: Arbiter không cho phép các Playbook được tự do thử sức khi thị trường chuyển tiếp giữa các bối cảnh.
- **Nguyên nhân dòng code (Root Cause)**:
  - Trong `arbiter.rs:88-97`:
    ```rust
    let is_allowed = if is_trending_regime {
        id == "TREND_PB" || id == "MOMENTUM_IGN"
    } else if is_ranging_regime {
        id == "RANGE_OSC" || id == "REVERSAL"
    } else if is_volatile_regime {
        id == "ANOMALY" || id == "ABSORPTION"
    } else {
        true
    };
    ```
  - Phân nhánh `else if` thứ tự cứng tạo ra bẫy khóa: Khi `ADX < 0.35` (xuất hiện phổ biến nhất), Arbiter gán cứng bối cảnh Ranging và **TỪ CHỐI BẢO TRỢ CHO `ANOMALY` VÀ `ABSORPTION`**, ngay cả khi Volume bùng nổ mạnh mẽ!

---

### 🚨 Vấn Đề 3: Sự Áo Nhất Của `ABSORPTION` Thâu Tóm 100% Lệnh

- **Nguyên nhân dòng code (Root Cause)**:
  - Trong `absorption.rs:29-38`, điều kiện kích hoạt của Absorption nới lỏng hơn hẳn 5 Scanners còn lại (`squeeze >= 0.40 && vol_force >= 0.15`).
  - Khi 5 Scanners kia bị ngắt tín hiệu bởi bẫy `AND`, `ABSORPTION` trở thành ứng viên duy nhất gửi `Proposal` về Arbiter và giành chiến thắng $100\%$ các nến.

---

### 🚨 Vấn Đề 4: Tắc Nghẽn Luồng Do Strict Exclusive Position Lock & Hold Duration

- **Nguyên nhân dòng code (Root Cause)**:
  - Khi 1 lệnh `ABSORPTION` khớp lệnh, do `suggested_tp = None` và Trailing SL nới lỏng (`tsl_atr_loose`), vị thế bị ngâm từ $1.000 \rightarrow 2.694$ nến ($16 \rightarrow 45$ tiếng).
  - Do quy tắc **Strict Exclusive Order** (chỉ cho phép 1 vị thế mở duy nhất), trong suốt 45 tiếng đó, toàn bộ bộ máy Helios bị đóng băng không thể quét hay mở bất cứ lệnh nào khác.

---

## 🛠️ 3. ĐỀ XUẤT HƯỚNG GIẢI QUYẾT KIẾN TRÚC VIỆT MỚI (ACTIONABLE SOLUTIONS)

Để Helios V3.3 đạt được sự linh hoạt và lợi nhuận khủng của V3 Cũ nhưng giữ nguyên $100\%$ tính sạch sẽ và chặt chẽ của Clean Quant Architecture, chúng ta cần triển khai 3 cải tiến kiến trúc sau:

### 🎯 Giải Pháp 1: Chuyển Cổng Lọc Hard AND sang Soft Activation Additive Scoring (Chấm Điểm Tích Lũy)
- **Cơ chế**: Thay vì `if A && B && C && D` ngắt lệnh `return None`, mỗi Scanner tính điểm số hoàn hảo `raw_score` theo công thức cộng dồn trọng số:
  $$\text{raw\_score} = w_1 \cdot \text{Price\_Zone} + w_2 \cdot \text{Oscillator\_Score} + w_3 \cdot \text{Volume\_Score}$$
- **Áp dụng cho `RANGE_OSC`**:
  - Giá sát dải biên dưới (`close <= bb_lower * 1.01`) $\implies +0.50$ điểm.
  - Stochastic quá bán (`stoch_k < 35`) $\implies +0.30$ điểm.
  - Stochastic %K móc lên (%K > %D) $\implies +0.20$ điểm.
  - Tổng điểm $\ge 0.50 \implies$ Phát tín hiệu `Proposal`. Điều này giúp nảy tín hiệu mượt mà ngay khi giá chạm dải biên mà không bắt buộc %K phải cắt %D ở đúng nến đó!

### 🎯 Giải Pháp 2: Chuyển Regime Router Sang Soft Context Weighting (Trọng Số Bối Cảnh Mềm)
- **Cơ chế**: Bỏ lệnh khóa cứng `is_allowed`. Cho phép MỌI Scanner tự do quét tín hiệu và gửi `Proposal`.
- Arbiter sẽ nhân điểm tự tin `confidence` với **Hệ số Trọng số Bối cảnh (`regime_weight`)**:
  $$\text{confidence} = \text{proposal.raw\_score} \times \text{regime\_weight}(\text{scanner\_id}, \text{regime}) \times \text{matrix\_multiplier}$$
- **Tác dụng**: Trong Sideway, `RANGE_OSC` được nhân trọng số $1.0$, nhưng nếu có cú nổ volume bất ngờ, `MOMENTUM_IGN` vẫn có cửa phát lệnh nếu điểm số kỹ thuật đủ cao.

### 🎯 Giải Pháp 3: Tối Ưu Thời Gian Ngâm Lệnh & Stagnant Capital Release
- Điều chỉnh tham số `time_countdown_bars` và `tighten_step_multiplier` trong `RiskManager` để chủ động chốt lời/cắt lỗ giải phóng vốn khi vị thế đi ngang ngâm quá 240 nến (4 tiếng), giúp bot nâng tần suất giao dịch lên $30 - 60$ lệnh/tháng.

---

## 📌 KẾT LUẬN & KẾ HOẠCH HÀNH ĐỘNG
Báo cáo trên đã xác định chính xác $100\%$ nguyên nhân gốc rễ. Chúng ta đã có giải pháp kiến trúc rõ ràng để đưa Helios V3.3 trở lại vị thế Siêu Chiến Thuật!
