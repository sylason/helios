# 🔬 BÁO CÁO PHÂN TÍCH CHUYÊN SÂU: CÁC LỖ HỔNG LOGIC TRAILING STOPLOSS & INITIAL STOPLOSS TRONG HELIOS V3

**Thư mục lưu trữ**: `strategies_backtests/helios/docs/review_v3_2/01_Trailing_SL_And_Initial_SL_Flaws.md`  
**Ngày lập**: 2026-07-29  
**Đối tượng phân tích**: `strategies_backtests/helios/src/strategy/core_v3/risk.rs` & `mod.rs`

---

## 🏛️ 1. GIẢI THÍCH Ý ĐỒ THIẾT KẾ & LỖ HỔNG CỦA CƠ CHẾ "TẠM DỪNG ĐẾM NGÂM LỆNH KHI SQUEEZE"

### ❓ Ý đồ ban đầu của nhà phát triển:
Trong [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L116-L122), tác giả viết đoạn code:
```rust
let mut effective_stagnant_bars = 0;
for j in peak_bar..=i {
    if matrix.squeeze_multiplier[j] <= squeeze_thresh {
        effective_stagnant_bars += 1;
    }
}
```
* **Lý do thiết kế**: Khi thị trường đi vào trạng thái nén biến động (Squeeze / Lò xo nén), giá thường đi ngang tích lũy hẹp trước khi bùng nổ xu hướng lớn. Tác giả muốn **tạm dừng bộ đếm thời gian ngâm lệnh** (`effective_stagnant_bars`) để tránh việc siết SL quá sớm khiến lệnh bị văng ra ngay trước thềm sóng lớn.

### 💥 Lỗ hổng thực tế (The Structural Flaw):
* **Bắt bệnh**: Trong thực tế, chỉ báo Squeeze Intensity trên khung thời gian lớn (ví dụ `1h` hoặc `4h`) có thể giữ mức điểm cao `squeeze_multiplier > squeeze_thresh` kéo dài liên tục hàng nghìn cây nến 1m!
* **Hậu quả**: Vòng lặp đếm liên tục bỏ qua `effective_stagnant_bars += 1`. Bộ đếm `decay_cycles` bị **đóng băng vĩnh viễn ở giá trị 0**. 
* **Kết chứng thực tế trong Log `v3_15.md`**:
  - Lệnh 0 ngâm **1.058 nến** (+1.06%).
  - Lệnh 8 ngâm **1.624 nến** (+6.20%).
  - Lệnh 16 ngâm **2.839 nến** (+10.19%).
  - Lệnh 40 ngâm **1.767 nến**.
  Lệnh bị giam cầm trong bộ đếm đóng băng, làm giam vốn và triệt tiêu tần suất quay vòng vốn của bot.

---

## ⚡ 2. BẢN NGHỊCH LÝ VỀ GIẢ THUYẾT "VỪA VÀO LỆNH ĐÃ LỖ NẶNG 1-2 TICKS" (INITIAL STOP-LOSS BUG)

### ❓ Giả thuyết của người dùng:
> *"Các lệnh buy/sell trong core_v3 helios version 3 chưa có stoploss khi tạo lệnh, do đó khi vừa vào lệnh chưa được bao lâu thì ta đã lỗ nặng, ví dụ như vừa vào được 1 hay 2 ticks thì đã lỗ mạnh rồi."*

### 🔬 Kết quả đối chiếu mã nguồn Engine & Strategy:

1. **Về mặt kỹ thuật khởi tạo lệnh (`mod.rs` dòng 188-210)**:
   Khi `buy()` hoặc `sell()` được gọi, lệnh ĐÃ ĐƯỢC TRUYỀN `RiskPolicy::FixedPrice { sl_price: Some(initial_sl), ... }`. Engine `ferro_trade` có mở một lệnh Stop Order tương ứng.

2. **TUYÊN BỐ NGUYÊN NHÂN THỰC TẾ (Root Cause of Instant Heavy Losses)**:
   Mặc dù lệnh CÓ Stop Loss ban đầu, nhưng **cách tính `initial_sl` đang bị hổng 3 điểm chết người**:

#### ❌ Lỗi A: Không có Mức Cắt Lỗ Cố Định Tối Đa (Uncapped Initial SL)
- Trong [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L44-L46):
  $$\text{initial\_sl} = \text{Entry Price} \pm (\text{ATR} \times \text{Multiplier})$$
- Optimizer tự do tìm kiếm `multiplier` có thể lên tới `5.0` hoặc `7.0`.
- Khi thị trường biến động mạnh (ATR vọt lên 30 USD), khoảng cách `initial_sl` bị đẩy xa tới **$5 \times 30 = 150 \text{ USD}$ (tương đương $10\% - 15\%$ khoảng cách giá)**!
- **Khi nến 1-2 ticks sau đó giật sập (Flash Crash)**, lệnh bị cắn Stop Loss ở tận đáy râu nến, gây ra khoản **lỗ nặng -13.47% chỉ trong đúng 1 cây nến** (Lệnh 32 trong `v3_15.md`)!

#### ❌ Lỗi B: Bẫy Râu Nến Ngay Nến Vào Lệnh (Entry Wick Bug)
- Trong [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L95-L103), code tìm giá đỉnh cao nhất `peak_val` bắt đầu quét từ `entry_bar`.
- Ngay tại nến vừa vào lệnh (`i == entry_bar`), nến đó có thể có một râu nến trên rất dài (`high = 3100`, `entry_price = 3000`).
- Trailing SL lập tức gán `peak_val = 3100`, kéo `new_sl` vọt lên mức $3050$ (cao hơn cả giá vào lệnh $3000$).
- Nến tiếp theo nhích nhẹ xuống liền **bị văng SL oan và chịu mức lỗ bất ngờ**.

---

## 📊 3. TỔNG HỢP TOÀN BỘ 5 LỖ HỔNG LOGIC CỦA HỆ THỐNG STOP-LOSS V3

| STT | Tên Lỗi Logic | Dòng Code Gây Lỗi (`risk.rs`) | Mô Tả Tác Động Thực TẾ |
|:---:|---|---|---|
| **1** | **Uncapped Initial SL** (SL Ban Đầu Quá Xa) | Dòng 44 - 46 | Khi ATR bùng nổ, SL ban đầu cách xa 10-15%, khiến lệnh dính râu flash crash lỗ nặng -13.47% chỉ trong 1-2 nến. |
| **2** | **Squeeze Counter Freeze** (Đóng Băng Đếm Ngâm) | Dòng 118 - 122 | Lệnh bị giam cầm ngâm hàng ngàn nến (1.058 đến 2.839 nến) vì chỉ báo Squeeze luôn giữ điểm cao. |
| **3** | **ATR Short Explosion** (Đẩy SL Short Lên Trời) | Dòng 174 | Phép cộng $\text{peak\_val} + (\text{ATR} \times \text{mult})$ khi ATR bùng nổ làm SL lệnh Short vọt lên đỉnh, chịu lỗ rất nặng. |
| **4** | **Entry Bar Wick Trap** (Bẫy Râu Nến Vào Lệnh) | Dòng 95 - 114 | Lấy `highs[entry_bar]` làm đỉnh ngay nến vừa mở lệnh làm SL bị kéo sai lệch ngay từ giây đầu tiên. |
| **5** | **Tag Name Mismatch** (Sai Tên Thẻ Scanner) | Dòng 82 | Tag `"ANOMALY"` trong `mod.rs` không khớp với `"ANOMALY_BREAKOUT"` trong `risk.rs`, khiến Anomaly bị tước quyền dùng `tsl_atr_loose`. |

---

## 💡 4. ĐỀ XUẤT ĐIỀU CHỈNH KIẾN TRÚC CHO HỆ THỐNG V3.2

1. **Khống chế Cắt Lỗ Ban Đầu Cố Định (Hard Max Cap)**:
   Bổ sung quy định: `initial_sl` không được phép cách xa quá `max_initial_sl_pct` (ví dụ tối đa $2.0\%$ tài sản/giá).
2. **Khôi phục Đếm Thời Gian Thực Tế (Real Stagnant Decay)**:
   Loại bỏ điều kiện bỏ qua Squeeze để bộ đếm `effective_stagnant_bars` hoạt động liên tục, siết chặt SL sau tối đa 30-50 nến.
3. **Tính Peak Bar từ `entry_bar + 1`**:
   Chỉ bắt đầu theo dõi đỉnh lợi nhuận Trailing SL từ cây nến kế tiếp sau nến vào lệnh để tránh râu nến nhiễu của entry bar.
