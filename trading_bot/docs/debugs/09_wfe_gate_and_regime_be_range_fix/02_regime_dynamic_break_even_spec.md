# 🛡️ TÀI LIỆU KỸ THUẬT: PHÂN TẦNG DYNAMIC BREAK-EVEN THEO 3 NHÓM REGIME SIMPLEX
> **Mục tiêu:** Xóa bỏ hoàn toàn con số hardcode $0.0008$ (0.08%) bóp chết lệnh thắng ở hòa vốn, thay bằng khoảng đệm ATR động phân theo 3 nhóm trạng thái thị trường.  
> **Vị trí áp dụng:**  
> - `strategies_backtests/helios/src/strategy/core_v3/config.rs`  
> - `strategies_backtests/helios/src/strategy/core_v3/risk.rs`

---

## 1. Bối cảnh & Điểm nghẽn Toán học
- **Hiện trạng:** Trong `risk.rs:240-253`, hệ thống dùng `be_price_buffer_pct = 0.0008` (0.08%).
- **Lỗ hổng:**
  - Với ETH ($2500$), $0.08\%$ tương đương đúng **$2.00** trên giá vào.
  - Nhưng độ rung lắc tự nhiên của 1 cây nến 1m ETH là **$2.5 – $4.0** (ATR 1m).
  - Với đòn bẩy 20x, giá chỉ cần tăng nhẹ $0.4\%$ ($10 USD), ROE đã đạt $8\%$ $\rightarrow$ SL bị giật vội vàng lên cách Entry đúng $2.00.
  - Ngay cây nến sau, râu nến quét văng lệnh ra ngoài ở mức **+$0.00 hoặc -$0.00** (Log 24 có 10/18 lệnh bị giết theo cách này).
  - Đồng thời, áp dụng cùng một luật cho cả lệnh Trend (cần thở) và lệnh Range (sóng ngắn) là phi lý.

---

## 2. Thiết kế Giải pháp: Phân Tầng Theo 3 Nhóm Regime Simplex (Phương Án 1)

### 2.1. Cấu hình DNA trong `StopManagementConfig` (`config.rs`)
Bổ sung 6 gen DNA phổ quát:
```rust
pub struct StopManagementConfig {
    // 1. NHÓM XU HƯỚNG (TREND REGIME: TREND_PB, ABSORPTION)
    #[dna(min = 1.5, max = 4.0, step = 0.25)]
    pub be_atr_trigger_trend: f64,
    #[dna(min = 1.0, max = 2.5, step = 0.25)]
    pub be_buffer_atr_trend: f64,

    // 2. NHÓM ĐI NGANG (RANGE REGIME: RANGE_OSC, REVERSAL)
    #[dna(min = 1.0, max = 2.5, step = 0.20)]
    pub be_atr_trigger_range: f64,
    #[dna(min = 0.5, max = 1.5, step = 0.20)]
    pub be_buffer_atr_range: f64,

    // 3. NHÓM BIẾN ĐỘNG / XUNG LỰC (MOMENTUM_IGN, ANOMALY)
    #[dna(min = 1.0, max = 3.0, step = 0.25)]
    pub be_atr_trigger_volatile: f64,
    #[dna(min = 0.8, max = 2.0, step = 0.20)]
    pub be_buffer_atr_volatile: f64,
}
```

### 2.2. Logic Tính Toán Khớp Lệnh trong `risk.rs`
1. Nhận diện bộ tham số theo Playbook Tag của lệnh:
   - `TREND_PB` / `ABSORPTION` $\rightarrow$ `(be_atr_trigger_trend, be_buffer_atr_trend)`.
   - `RANGE_OSC` / `REVERSAL` $\rightarrow$ `(be_atr_trigger_range, be_buffer_atr_range)`.
   - `MOMENTUM_IGN` / `ANOMALY` $\rightarrow$ `(be_atr_trigger_volatile, be_buffer_atr_volatile)`.
2. Kích hoạt khi:
   $$\text{Current PnL} \ge \text{current\_atr} \times \text{be\_atr\_trigger} \times \text{size}$$
3. Mức Stop Loss mới:
   - Lệnh Long:
     $$\text{Target BE SL} = \max\Big(\text{Entry} \times (1 + \text{fee\_buffer}),\ \text{High Watermark} - \text{current\_atr} \times \text{be\_buffer\_atr}\Big)$$
     $$\text{Calculated SL} = \max(\text{Calculated SL},\ \text{Target BE SL})$$
   - Lệnh Short:
     $$\text{Target BE SL} = \min\Big(\text{Entry} \times (1 - \text{fee\_buffer}),\ \text{Low Watermark} + \text{current\_atr} \times \text{be\_buffer\_atr}\Big)$$
     $$\text{Calculated SL} = \min(\text{Calculated SL},\ \text{Target BE SL})$$
