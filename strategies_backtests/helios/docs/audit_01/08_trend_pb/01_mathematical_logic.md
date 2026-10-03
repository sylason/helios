# KIỂM TOÁN TOÁN HỌC & VI CẤU TRÚC: TREND PULLBACK SCANNER (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **File mã nguồn đối chiếu:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`  
> **Ngày lập:** 03/10/2026

---

## 🏛️ 1. BẢN CHẤT NGHIỆP VỤ VI CẤU TRÚC CỦA CÚ PULLBACK LÀNH MẠNH

Một nhịp hồi quy (Pullback) chuẩn mực định lượng trong xu hướng tăng/giảm đòi hỏi hội tụ đủ 5 yếu tố:
1. **Trục xu hướng định hướng (Directional Trend):** Đường giá trị động KAMA dốc rõ rệt, dòng tiền lớn đồng thuận.
2. **Hồi quy về vùng giá trị (Value Area Reversion):** Giá điều chỉnh về sát KAMA, giải tỏa trạng thái căng thẳng biên độ.
3. **Cạn kiệt thanh khoản điều chỉnh (Supply/Demand Dry-Up):** Khối lượng nến hồi phải giảm sút (Low Volume Pullback), thể hiện không có bàn tay xả hàng của tổ chức.
4. **Hạ nhiệt động lượng ngắn hạn (Oscillator Cooldown):** Stochastic và RSI hạ nhiệt về vùng trung hòa nhưng không được gãy đổ.
5. **Phản ứng từ chối giá tại hỗ trợ (Rejection / High CLV):** Xuất hiện lực hấp thụ đẩy giá rút chân đóng nến ở nửa trên của biên độ nến ($CLV \ge 0.50$).

---

## 🔍 2. BẢY LỖ HỔNG TOÁN HỌC & CẠM BẪY VI CẤU TRÚC TRONG `trend_pb.rs`

### 2.1. Lỗ hổng 1: Hoàn Toàn Mù Khối Lượng (Volume Blindness)
- Code hiện tại hoàn toàn không đọc `Volume`, `vol_z_score` hay `MFI`.
- Khi thị trường bán tháo với Volume kỷ lục đâm thủng KAMA, chỉ cần một cây nến xanh hồi kỹ thuật mỏng manh xuất hiện là `trend_pb` vẫn phát tín hiệu Long bắt dao rơi.

### 2.2. Lỗ hổng 2: Hằng Số Sàn Độ Dốc KAMA Quá Phẳng (`1e-4`)
- Dòng 64: `let slope_thresh = pb_cfg.kama_slope_threshold.max(1e-4);`
- Giá trị $10^{-4} = 0.0001$ là một đường thẳng nằm ngang phẳng lì. Khi thị trường đi ngang ngưng đọng (Dead Flat), vi nhiễu $0.00015$ cũng bị ngộ nhận là xu hướng mạnh và kích hoạt cào lệnh.

### 2.3. Lỗ hổng 3: Bẫy Mâu Thuẫn Triệt Hạ Nến Pinbar / Hammer Rút Chân Kinh Điển
- Dòng 112-114 (Long): `if clv < 0.15 || close < open { return None; }`
- Dòng 159-161 (Short): `if clv > -0.15 || close > open { return None; }`
- **Chứng minh số học:** Cây nến Hammer rút chân kinh điển có râu dưới dài 95% thân, lực cầu áp đảo tuyệt đối với $CLV = +0.94$, nhưng chỉ vì giá đóng cửa thấp hơn mở cửa $0.1$ pip (`close < open` - thân nến màu đỏ siêu mỏng), code lập tức loại bỏ thẳng thừng!
- Đây là bẫy giáo điều bán lẻ (Retail Candle Dogma) bóp chết chính những điểm vào lệnh đảo chiều đẹp nhất.

### 2.4. Lỗ hổng 4: Bẫy Hút Khoảng Cách Stop Loss (KAMA Suction Trap)
- Dòng 122 (Long): `let base_support = low.min(kama_mid); let mut sl = base_support - buffer;`
- Dòng 170 (Short): `let base_resistance = high.max(kama_mid); let mut sl = base_resistance + buffer;`
- Khi giá hồi nông chưa chạm tới KAMA (`low > kama_mid`), phép toán `.min(kama_mid)` chọn đường KAMA ở sâu tít bên dưới rồi trừ thêm buffer $ATR$.
- Khoảng cách Stop Loss bị kéo giãn ra $3.0 \sim 4.5$ ATR (tương đương lỗ ROE $-40\% \sim -80\%$).
- Mức SL này lập tức bị chốt chặn `StructuralRiskVeto` trong Arbiter bắn hạ (vì trần tối đa là $15\%$ ROE), khiến các cơ hội Pullback nông đẹp nhất bị **VETO 100% trong im lặng**!

### 2.5. Lỗ hổng 5: Chấm Điểm Khoảng Cách `delta_kama.abs()` Bị Mất Tính Định Hướng
- Dòng 116 & 163: `let f_dist = sat_proximity(delta_kama.abs(), max_dist);`
- Dùng `.abs()` khiến giá đâm thủng KAMA $-0.4$ ATR (sắp gãy trend) vẫn nhận điểm số khoảng cách cao ngang ngửa với giá giữ vững hỗ trợ $+0.1$ ATR!

### 2.6. Lỗ hổng 6: Bắt Dao Rơi Khi Stochastic = 0
- Hàm `sat_proximity(stoch_k, stoch_thresh)` trao điểm tối đa khi $\text{stoch\_k} \to 0$.
- Trong thực tế, $\text{stoch\_k} = 0$ là lúc thị trường đang đổ đèo thanh lý theo chiều dọc. Một nhịp Pullback lành mạnh không bao giờ đè Stoch về 0 mà tạo đáy cong và ngóc đầu lên. Code thiếu điều kiện xác nhận động lượng quay đầu ($\text{stoch\_k} \ge \text{stoch\_k}[i-1]$).

### 2.7. Lỗ hổng 7: Bỏ Rơi Lực Nến `clv` Trong Hàm Chấm Điểm
- Dòng 118: `convex_combine(&[f_slope, f_dist, f_stoch], &[1.0, 1.0, 1.0])`
- Trọng số cào bằng $1:1:1$ và hoàn toàn bỏ rơi biến số phản xạ lực nến `clv`. Một cú rút chân thần thánh $CLV = 0.98$ nhận điểm ngang bằng với cú rút chân yếu ớt $CLV = 0.16$.
