# Báo Cáo Nghiên Cứu Khoa Học: Phân Tích Chi Tiết 29 Giao Dịch Paper Forward & Chuyển Đổi Trạng Thái Thị Trường (Market Regime Shift) ETHUSDT

- **Tác giả:** Hệ thống Phân tích & Tối ưu hóa Chiến lược Bot (Helios Engine)  
- **Ngày lập:** 12/08/2026  
- **Đối tượng phân tích:** File Log `trading_bot/bot_engine/examples/logs/paper_forward_v3.log`  
- **Cặp giao dịch:** ETHUSDT (Khung thời gian nến: 1-phút, Tái tối ưu hóa GA: 24-giờ sliding window)  
- **Khoảng thời gian thử nghiệm:**
  - **In-Sample (Train Window):** 30 ngày (03/07/2026 09:30 UTC -> 02/08/2026 09:30 UTC)
  - **Out-Of-Sample (Forward Test Execution):** 10 ngày (02/08/2026 09:31 UTC -> 12/08/2026 09:30 UTC, tổng 14,386 nến 1m)

---

## 🎯 Executive Summary (Tóm Tắt Tổng Quan)

Nghiên cứu này tiến hành phân tích sâu định lượng và định tính 100% dữ liệu thực thi của 29 giao dịch paper trade đã đóng trong thử nghiệm Out-Of-Sample (OOS) thuộc bộ não Helios v3 (`paper_forward_v3`).

**Các thông số tài chính & hiệu suất cốt lõi:**
- **Vốn khởi điểm (Initial Capital):** $60.00
- **Vốn ròng kết thúc (Final Equity):** $56.58
- **Tỷ suất lợi nhuận tích lũy (Cumulative PnL):** -$3.42 (-5.69%)
- **Tổng số giao dịch đã đóng:** 29 (21 Long / 8 Short)
- **Tỷ lệ thắng (Win Rate):** **31.03%** (9 lệnh Thắng / 20 lệnh Thua)
- **Mức lời trung bình (Average Win):** **+$1.06 / lệnh** (**+12.77% / lệnh**)
- **Mức lỗ trung bình (Average Loss):** **-$0.65 / lệnh** (**-7.54% / lệnh**)
- **Tỷ lệ Thưởng/Rủi ro (Payoff Ratio):** **1.64 : 1** (về giá trị $) / **1.69 : 1** (về %)
- **Hệ số Lợi nhuận (Profit Factor):** **0.74** ($9.54 Tổng Lãi / $12.96 Tổng Lỗ)

---

## 1. 📊 Thống Kê Chi Tiết Hiệu Suất PnL & Lịch Sử 29 Giao Dịch

### 1.1 Phân Tích So Sánh Mức Lợi Nhuận & Lỗ Trung Bình (Payoff Analysis)

- **Tổng lợi nhuận từ 9 lệnh thắng (Gross Profit):** **+$9.54** (+114.90%)
- **Tổng thua lỗ từ 20 lệnh thua (Gross Loss):** **-$12.96** (-150.73%)
- **Mức lời trung bình (Average Win):** **+$1.06 / lệnh** (+12.77% / lệnh)
- **Mức lỗ trung bình (Average Loss):** **-$0.65 / lệnh** (-7.54% / lệnh)
- **Tỷ lệ Thưởng / Rủi ro thực tế (Payoff Ratio):** $1.06 / $0.65 = **1.64 : 1**

---

## 2. 🚀 Phân Tích Các Lệnh Thắng Đột Biến & Hiệu Quả Theo Playbook

### 2.1 Chi Tiết Các Lệnh Thắng Đột Biến (Outlier Big Wins)

1. **TRADE #023: +$3.84 (+46.98%)**
   - **Vị thế:** LONG @ 1872.51 -> Exit @ 1918.01 (Tag: `RANGE_OSC`)
   - **Bối cảnh & Logic:** Nhận diện điểm đảo chiều và bứt phá vùng giá tích lũy 1872. Khi ETHUSDT tăng vọt lên 1925, cơ chế Trailing SL đã bám sát và khóa thành công lợi nhuận khổng lồ +46.98% trên margin vị thế.

2. **TRADE #003: +$2.15 (+24.89%)**
   - **Vị thế:** LONG @ 1855.37 -> Exit @ 1879.95 (Tag: `ABSORPTION`)
   - **Bối cảnh:** Bắt trọn cú rút chân đảo chiều từ 1850 lên 1880 nhờ tín hiệu Hấp thụ Thanh khoản (`ABSORPTION`).

3. **TRADE #011: +$1.70 (+20.80%)**
   - **Vị thế:** LONG @ 1836.85 -> Exit @ 1857.43 (Tag: `ABSORPTION`)

---

### 2.2 So Sánh Hiệu Quả Các Loại Playbook (`RANGE_OSC` vs `ABSORPTION`)

| Loại Playbook | Tổng Lệnh | Lệnh Thắng | Lệnh Thua | Win Rate (%) | Net PnL ($) | Net PnL (%) | Nhận Xét |
|---|---|---|---|---|---|---|---|
| **`ABSORPTION`** | 12 | 6 | 6 | **50.00%** | **+$0.68** | **+1.13%** | **Hiệu quả nhất.** Bắt đảo chiều thanh khoản rất tốt. |
| **`RANGE_OSC`** | 17 | 3 | 14 | **17.65%** | **-$4.10** | **-6.83%** | **Bị dính Trailing SL nhiều nhất.** Dính nhiễu nến (whipsaw) khi giá đi ngang lướt biên. |

---

## 3. 📉 Phân Tích Sự Chuyển Đổi Trạng Thái Thị Trường (Market Regime Shift)

1. **IS (30 ngày trước):** ETHUSDT hình thành các bước sóng xu hướng (trending waves) mượt mà.
2. **OOS (10 ngày sau):** Thị trường chuyển dịch đột ngột sang trạng thái **Choppy Side-way Range** trong vùng 1830 - 1930 USD với râu nến quét 2 đầu liên tục.

---

## 4. 💡 Khuyến Nghị Cải Tiến Hướng Tới Thử Nghiệm Tiếp Theo

1. **Động hóa khoảng đệm Trailing SL (Dynamic ATR-based Trailing SL):**
   - Thay thế khoảng giá Trailing SL cố định bằng bội số ATR (e.g., 2.0x ATR 14m).
2. **Bổ trợ Bộ Phân Loại Trạng Thái Thị Trường (Regime Classification Module):**
   - Tích hợp Choppiness Index / ADX. Khóa lệnh `RANGE_OSC` khi Choppiness Index > 61.8.
