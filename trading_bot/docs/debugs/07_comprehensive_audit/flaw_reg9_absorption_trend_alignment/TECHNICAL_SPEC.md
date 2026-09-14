# 📘 THIẾT KẾ KỸ THUẬT: FLAW-REG.9 - ABSORPTION MACRO TREND ALIGNMENT

> **Mã định danh:** `FLAW-REG.9`  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg9_absorption_trend_alignment/`  
> **Trọng tâm:** Khắc phục bẫy bắt dao rơi và cản tàu của `AbsorptionScanner` khi thị trường xả lũ hoặc bơm dốc, chuẩn hóa 100% Dynamic qua DNA.

---

## 1. 🔬 BỐI CẢNH & PHÂN TÍCH NGUYÊN NHÂN GỐC RỄ (ROOT CAUSE)

### 1.1. Lỗ hổng trong mã nguồn hiện tại
Trong [`strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:43-47`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L43-L47):
```rust
// 2. Xác định hướng bùng nổ bằng VWMA vs SMA middle:
// VWMA > BB_Middle: Volume lớn tập trung nến mua -> Accumulation (Long)
// VWMA < BB_Middle: Volume lớn tập trung nến bán -> Distribution (Short)
let signal_type = if vwma >= bb_middle {
    SignalType::Long
} else {
    SignalType::Short
};
```

### 1.2. Sai lệch toán học (Mathematical Lag of VWMA)
$$\text{VWMA}_t = \frac{\sum_{k=0}^{n-1} \text{Close}_{t-k} \cdot \text{Vol}_{t-k}}{\sum_{k=0}^{n-1} \text{Vol}_{t-k}}$$
* Khi giá bắt đầu sụp đổ mạnh từ đỉnh cao (ví dụ ETH từ $2625$ rớt về $2470$), khối lượng bán tháo cực lớn ở các cây nến đầu tiên khiến `VWMA` bị "neo" ở mức giá cao ($2605$).
* Trong khi đó, `BB_Middle` (SMA 20 nến không trọng số volume) rơi nhanh hơn về $2600$.
* Mặc dù giá đóng cửa thực tế (`Close`) đã cắm sâu xuống $2576$ (dưới cả đáy dải Bollinger Bands), biểu thức `vwma >= bb_middle` ($2605 \ge 2600$) vẫn trả về **`TRUE`**.
* `Absorption` ngộ nhận đây là lực gom hàng của Smart Money (Accumulation) và phát lệnh **MUA LONG cản tàu**!

### 1.3. Minh chứng thực tế từ Nhật ký ETHUSDT (`WFSS_ETH_FLAW-REG.1_FLAW-TSL.9.log`)
* **Lệnh #011 (09-11 17:30 UTC):** ETH đang trong cơn bán tháo lớn, Absorption mua Long tại $2576.27$. Sau 35 nến, giá sập thủng đáy cắn thẳng vào Hard SL $2544.07$ (**-$3.07, -26.6% ROE**).
* **Lệnh #012 (09-11 18:15 UTC):** Vừa hết thời gian cách ly Cooldown, nến vẫn đang lao dốc, Absorption tiếp tục mua Long lần 2 tại $2539.08$. Lệnh bị dập chết tại $2507.34$ (**-$2.90, -26.6% ROE**).
* **Tổng thiệt hại:** 4 cú Hard SL của Absorption chiếm tới **-$12.29**, biến danh mục từ lãi ròng +$8.54 thành âm lỗ -$3.75!

---

## 2. 📐 GIẢI PHÁP KIẾN TRÚC: 100% DYNAMIC DUAL-LAYER CONFIRMATION

Để loại bỏ hoàn toàn bẫy bắt dao rơi mà tuyệt đối **không hardcode bất kỳ hằng số nào**, ta triển khai cơ chế xác nhận 2 tầng:

### 2.1. Tầng 1: Xác nhận Vị trí Giá Hành Động (Price Action Alignment)
* Một vùng gom hàng (Accumulation) thực sự chỉ hợp lệ khi giá không bị bẹp dúm dưới đáy mà phải giữ được ít nhất ở nửa trên hộp nén:
  $$\text{Long Condition: } \text{Close} \ge \text{BB\_Middle}$$
* Một vùng phân phối (Distribution) thực sự chỉ hợp lệ khi giá nằm ở nửa dưới hộp nén:
  $$\text{Short Condition: } \text{Close} \le \text{BB\_Middle}$$

### 2.2. Tầng 2: Chốt chặn Xu hướng Vĩ mô Động (Dynamic Macro Trend Veto)
* Bổ sung gene DNA `max_adverse_slope` vào `TacticalAbsorptionParams`:
  ```rust
  #[dna(min = 0.05, max = 0.50, step = 0.05)]
  #[serde(default = "default_absorption_max_adverse_slope")]
  pub max_adverse_slope: f64,
  ```
  *(Dải giá trị: 0.05 đến 0.50, bước nhảy 0.05 -> 10 nấc tối ưu cho GA, mặc định 0.15)*.
* Đọc độ dốc xu hướng KAMA/Trend từ `matrix.trend_dir[i]`:
  * Nếu $\text{signal\_type} == \text{Long}$ mà $\text{trend\_dir} < -\text{max\_adverse\_slope} \implies$ **CẤM LONG (Phủ quyết 100%)**.
  * Nếu $\text{signal\_type} == \text{Short}$ mà $\text{trend\_dir} > +\text{max\_adverse\_slope} \implies$ **CẤM SHORT (Phủ quyết 100%)**.

---

## 3. 🧬 TÁC ĐỘNG LÊN KHÔNG GIAN DNA & HỆ THỐNG

* **Kích thước DNA:** Tăng từ **118 chiều lên 119 chiều**.
* **Đồng bộ hóa:**
  - `TacticalAbsorptionParams`: Thêm `pub max_adverse_slope: f64`.
  - `DNA_WIDTH`: Cập nhật hằng số và test suite trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs`.
* **Zero Magic Numbers:** Không có bất kỳ hằng số cố định nào (`0.10`, `0.20`, v.v.) trong code logic.

---

## 4. 🧪 KẾ HOẠCH KIỂM THỬ (UNIT TESTS)

1. `test_absorption_falling_knife_blocked_by_macro_trend`:
   - Giả lập thị trường đang sập dốc mạnh (`trend_dir = -0.35`, `close < bb_middle`).
   - Mặc dù `vwma >= bb_middle` do volume trễ, `scan()` phải trả về `None` (chặn đứng bắt dao rơi).
2. `test_absorption_valid_accumulation_in_neutral_range`:
   - Giả lập thị trường nén phẳng (`trend_dir = 0.02`, `close >= bb_middle`, `squeeze` cao, `volume` cao).
   - `scan()` phải phát sinh `Some(Proposal)` chiều `Long` bình thường.
3. `test_dna_width_119_integrity`:
   - Kiểm tra vector DNA 119 chiều giải mã chính xác vào `HeliosConfigV3`.
