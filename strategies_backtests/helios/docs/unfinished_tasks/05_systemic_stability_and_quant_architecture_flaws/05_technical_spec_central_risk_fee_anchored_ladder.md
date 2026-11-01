# TÀI LIỆU THIẾT KẾ KỸ THUẬT: TÁI CẤU TRÚC CENTRAL RISK MANAGER & BẬC THANG KHÓA LÃI ĐỘNG HỌC

> **Mã số:** `05_technical_spec_central_risk_fee_anchored_ladder.md`  
> **Thư mục:** `strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/`  
> **Module liên quan:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `config.rs`  
> **Tiêu chuẩn tuân thủ:** `GEMINI.md` (Mục 6.1: Khử Price Action, Mục 6.2: Ranh giới Bất biến vs Biến thiên, Mục 6.4: Bảo toàn vốn & Monotonic Ratchet, Mục 6.5: Strict Separation of Concerns).

---

## 1. BỐI CẢNH VẬN HÀNH & NGUYÊN NHÂN GỐC RỄ (ROOT CAUSE ANALYSIS)

### 1.1. Vụ Án Thất Thoát Thực Nghiệm (Trade #008 & #009)
Trong đợt chạy thực nghiệm `run_walk_forward_smart_seed_20261031_121845`, hệ thống ghi nhận sự cố rò rỉ lợi nhuận nghiêm trọng:
- **Lệnh #008:** Long ETH, lãi đỉnh MFE đạt **$+12.4\%$ ROE**, nhưng sau đó giá đảo chiều và hệ thống để mặc lệnh cắn Hard Stop Loss ở mức âm **$-8.6\%$ ROE**.
- **Lệnh #009:** Long ETH, lãi đỉnh MFE đạt **$+10.1\%$ ROE**, dội ngược cắn Hard Stop Loss âm **$-8.6\%$ ROE**.
- Hai lệnh này gây thất thoát $-\$2.55$ (chiếm **$91.7\%$ tổng lỗ** của toàn bộ chu kỳ thử nghiệm).

### 1.2. Ba Tử Huyệt Kiến Trúc Trong `risk.rs`
Qua giải phẫu thực nghiệm (`/helios-autopsy`) và kiểm chứng chéo (`/helios-verify`), ba nguyên nhân gốc rễ được xác định:
1. **Bẫy "Cổng Chờ Kép" (The Double-Gating Trap):**
   - Khối `Tiered Profit Ratchet` (`risk.rs:780`) bị nhốt sau điều kiện `if is_be_triggered || is_mega_trend || is_momentum_post_trigger_be`.
   - Cờ `is_be_triggered` đòi hỏi giá phải vượt qua cả `min_required_gain` từ DNA VÀ `atr_required_gain` (nhân với `be_trigger_mult` có thể lên tới 2.5 - 3.5 ATR). Khi biến động thấp, điều kiện này đánh giá `false`, khiến **toàn bộ logic khóa lãi bậc thang phía sau bị vô hiệu hóa hoàn toàn**.
2. **Khu Vực Chân Không Của Trailing Take Profit (TTP Vacuum Zone):**
   - Khối TTP (`risk.rs:691-736`) mặc định kích hoạt tại `ttp_act_roe = 0.15` (hoặc 0.20).
   - TTP dẫm chân lên Trailing Stop Loss (TSL), đóng lệnh bằng Market Order chịu trễ 1 nến và trượt giá kép.
   - Khoảng lợi nhuận từ $+10.0\% \to +14.9\%$ ROE rơi vào vùng "chân không" không ai bảo vệ.
3. **Sự Lạc Hướng Của Tư Duy Retail / Price Action:**
   - Các logic đếm nến vi mô (`bars_held == 1`, `close < prev_open`) vi phạm nguyên tắc Quant loại bỏ Price Action (Mục 6.1 `GEMINI.md`).

---

## 2. BẢN CHẤT VẬT LÝ & TOÁN HỌC NỀN TẢNG (MATHEMATICAL FORMULATION)

### 2.1. Ma Sát 2 Chiều Chuẩn Hóa Theo Đòn Bẩy (Leverage-Adjusted Friction)
Mỗi giao dịch chịu ma sát giá gồm 2 lần phí Taker (mở lệnh và Stop Market thoát lệnh) cộng với trượt giá:
$$\text{fee\_buffer\_pct} = 2 \times \text{taker\_fee\_rate} + \text{slippage\_allowance}$$
$$\text{Friction\_ROE} = \text{fee\_buffer\_pct} \times \text{Leverage}$$

*Ví dụ:* Với $\text{taker\_fee} = 0.05\%$, $\text{slippage} = 0.03\%$, đòn bẩy $20\text{x}$:
$$\text{fee\_buffer\_pct} = 0.10\% + 0.03\% = 0.13\% \implies \text{Friction\_ROE} = 0.13\% \times 20 = 2.6\% \text{ ROE}$$

### 2.2. Điểm Hòa Vốn Kinh Tế Tuyệt Đối (True Economic Break-Even Price)
Mức giá dừng lỗ hòa vốn bảo hiểm vốn ($P_{\text{BE}}$) phải đảm bảo tài khoản có lãi ròng dương ($\Delta PnL > 0$) khi bị quét:
- **Lệnh Long:** $P_{\text{BE}} = P_{\text{entry}} \times \left(1 + \text{fee\_buffer\_pct} + 0.0003\right)$
- **Lệnh Short:** $P_{\text{BE}} = P_{\text{entry}} \times \left(1 - \text{fee\_buffer\_pct} - 0.0003\right)$
*(với $0.0003$ tương đương $0.03\%$ giá làm đệm an toàn dự phòng).*

---

## 3. THIẾT KẾ BẬC THANG KHÓA LÃI ĐỘNG HỌC (DYNAMIC FEE-ANCHORED LADDER)

Khối Bậc Thang Khóa Lãi hoạt động **độc lập 100%**, dựa trên Lãi Đỉnh Danh Nghĩa ($\text{MFE}_{\text{ROE}}$ = `peak_roe`).

### 3.1. Bảng Quy Tắc 5 Nấc Thang Bảo Vệ Lợi Nhuận

| Tầng Bậc | Tên Gọi | Ngưỡng Kích Hoạt ($\text{MFE}_{\text{ROE}}$) | Sàn Lợi Nhuận Được Khóa Cứng ($\text{Locked\_ROE}$) | Ý Nghĩa Nghiệp Vụ |
| :---: | :---: | :---: | :---: | :--- |
| **Nấc 0** | **True Break-Even** | $\ge 2.5 \times \text{Friction\_ROE}$ | $\text{Friction\_ROE} + 0.0003 \times L$ | Thoát khỏi rủi ro âm vốn, dời SL lên giá $P_{\text{BE}}$ (Dương tiền sau phí). |
| **Nấc 1** | **Survival Lock** | $\ge \text{Friction\_ROE} + 6.0\%$ | $\text{Friction\_ROE} + 3.0\%$ | **Cứu Lệnh #008, #009**. Đảm bảo lãi ròng tối thiểu $+3.0\%$ ROE khi giá chạm $+8.6\%$ ROE (với 20x). |
| **Nấc 2** | **Half-Back Lock** | $\ge \text{Friction\_ROE} + 10.0\%$ | $\max(\text{Nấc 1},\; 50\% \times \text{MFE}_{\text{ROE}})$ | Khóa tối thiểu $50\%$ lợi nhuận đỉnh cao nhất. |
| **Nấc 3** | **Trend Lock** | $\ge \text{Friction\_ROE} + 18.0\%$ | $\max(\text{Nấc 2},\; 60\% \times \text{MFE}_{\text{ROE}})$ | Bảo vệ sóng tăng trưởng trung bình. |
| **Nấc 4** | **Mega Trend** | $\ge 35.0\%$ ROE (hoặc `is_mega_trend`) | $\max(\text{Nấc 3},\; 70\% \times \text{MFE}_{\text{ROE}})$ | Thả diều siêu sóng với đệm ATR lỏng (`tsl_atr_loose`), khóa tối đa $70\%$ lãi đỉnh. |

### 3.2. Quy Tắc Monotonic Ratchet Guard (Khóa Một Chiều Tuyệt Đối)
- Stop Loss tính ra từ Bậc Thang:
  - Long: $SL_{\text{ladder}} = P_{\text{entry}} \times \left(1 + \frac{\text{Locked\_ROE}}{\text{Leverage}}\right)$
  - Short: $SL_{\text{ladder}} = P_{\text{entry}} \times \left(1 - \frac{\text{Locked\_ROE}}{\text{Leverage}}\right)$
- Mức Dừng Lỗ Tổng Hợp:
  - Long: $SL_{\text{final}} = \max(SL_{\text{ATR\_trailing}},\; SL_{\text{ladder}})$
  - Short: $SL_{\text{final}} = \min(SL_{\text{ATR\_trailing}},\; SL_{\text{ladder}})$
- **Nguyên tắc bảo vệ một chiều:**
  - Long: $SL_{\text{new}} = \max(SL_{\text{current}},\; SL_{\text{final}})$
  - Short: $SL_{\text{new}} = \min(SL_{\text{current}},\; SL_{\text{final}})$
  $\implies$ SL chỉ được phép tịnh tiến theo chiều có lợi cho tài khoản, **tuyệt đối cấm lùi**.

---

## 4. BẢO TỒN DNA & KHÔNG GIAN GA (DNA PRESERVATION)

- **Nguyên tắc bất biến:** Giữ nguyên `DNA_WIDTH = 141`.
- **Xử lý các gene tàn dư của TTP (`ttp_activation_roe`, `ttp_callback_atr`):**
  - Giữ nguyên cấu trúc struct và mảng gene trong `config.rs`.
  - Khai tử toàn bộ khối thực thi TTP trong `risk.rs:691-736`.
  - Các gene TTP trở thành dormant (không thực thi), không gây lỗi hay phân mảnh bộ nhớ, bảo toàn tính tương thích với các subagent/conversation khác.

---

## 5. BỘ KIỂM CHỨNG & KIỂM THỬ (VERIFICATION SUITE)

1. `test_tiered_profit_lock_break_even_at_fee_hurdle`: Xác minh khi MFE chạm $2.5 \times \text{Friction\_ROE}$, SL được kéo lên trên mức giá hòa vốn sau phí.
2. `test_tiered_profit_lock_locks_survival_at_high_watermark`: Xác minh kịch bản Lệnh #008 (#009) với MFE $+12.4\%$ ROE $\implies$ SL được khóa tại tối thiểu $+5.6\%$ ROE (Lãi ròng sau phí $\ge +3.0\%$).
3. `test_tiered_profit_lock_monotonic_ratchet_never_retreats`: Xác minh khi giá đỉnh thoái lui, SL tuyệt đối không bị thụt lùi.
4. `test_ttp_is_fully_decommissioned`: Xác minh không còn bất kỳ lệnh đóng sớm vô căn cứ nào từ TTP.
