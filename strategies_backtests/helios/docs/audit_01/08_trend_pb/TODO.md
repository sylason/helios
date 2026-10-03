# TODO CẢI TỔ CHIẾN THUẬT TREND PULLBACK (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **Ngày lập:** 03/10/2026  
> **Nguyên tắc:** Baby Steps, Phân tích trước - Code sau, Viết testcase đi kèm từng task.

---

## 🔥 NHÓM 1: CÁC CHỐT CHẶN SINH TỬ TRONG CODE `trend_pb.rs` (CRITICAL)

- [ ] **TODO-PB-01: Khử Bẫy KAMA Suction Trap (Sửa Neo Stop Loss Động)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:122-126, 170-174`
  - **Bản chất:** Bỏ phép toán `.min(kama_mid)` cho Long và `.max(kama_mid)` cho Short. Khi nến hồi nông giữ được nhịp trên KAMA (`low > kama_mid`), chỉ neo SL theo `low - buffer` của nến hiện tại/gần nhất để giữ SL gọn trong trần 15% ROE.
  - **Test case:** Unit test với nhịp hồi nông cách KAMA 1.5 ATR $\to$ SL không bị dãn quá 1.0 ATR $\to$ không bị Arbiter VETO.

- [ ] **TODO-PB-02: Giải Phóng Nến Pinbar / Hammer Rút Chân (Bỏ Bẫy Retail Candle Color Dogma)**
  - **File:** `trend_pb.rs:112-114, 159-161`
  - **Bản chất:** Cho phép nến thân đỏ mỏng vào lệnh Long nếu có lực hấp thụ rút chân mạnh ($CLV \ge 0.50$). Chỉ cấm khi $CLV < 0.15$ HOẶC ($Close < Open$ VÀ $CLV < 0.50$). Tương tự cho Short.
  - **Test case:** Nến Hammer đỏ rút râu với $CLV = 0.85 \to$ Được phép Long thành công.

- [ ] **TODO-PB-03: Sửa Lỗi Mất Tính Định Hướng Trong `f_dist`**
  - **File:** `trend_pb.rs:116, 163`
  - **Bản chất:** Bỏ phép lấy `.abs()` trong `sat_proximity(delta_kama.abs(), max_dist)`. Khi giá xuyên thủng KAMA ($\text{delta\_kama} < 0$ cho Long), phải áp dụng hệ số phạt (penalty) thay vì thưởng điểm cao.
  - **Test case:** Giá thủng KAMA -0.4 ATR phải có điểm khoảng cách thấp hơn rõ rệt so với giá giữ vững hỗ trợ +0.1 ATR.

- [ ] **TODO-PB-04: Bổ Sung Tín Hiệu Stochastic Ngóc Đầu Đảo Chiều (Stoch Hook Guard)**
  - **File:** `trend_pb.rs:106, 153`
  - **Bản chất:** Cấm Long khi $\text{stoch\_k} < 5.0$ (đang cắm đầu rơi tự do / thác lũ thanh lý) hoặc yêu cầu xác nhận ngóc đầu $\text{stoch\_k} \ge \text{stoch\_k}[i-1]$ (hoặc $\text{stoch\_k} \ge \text{stoch\_d}$).
  - **Test case:** Nến đang rơi tự do Stoch = 0 bị từ chối; nến tạo đáy cong ngóc đầu được phê duyệt.

- [ ] **TODO-PB-05: Đưa Lực Nến `clv` Vào Hàm Chấm Điểm Lồi**
  - **File:** `trend_pb.rs:118, 166`
  - **Bản chất:** Bổ sung thành phần $f_{\text{clv}}$ vào `convex_combine(&[f_slope, f_dist, f_stoch, f_clv], &[0.35, 0.25, 0.20, 0.20])` để nâng phổ điểm tự tin của Trend_PB lên dải $0.75 \sim 0.88$, đủ sức cạnh tranh công bằng với Momentum.
  - **Test case:** Cú rút chân mạnh $CLV = 0.95$ nhận điểm tự tin cao hơn cú rút chân yếu $CLV = 0.20$.

---

## ⚡ NHÓM 2: TINH CHỈNH DẢI DNA TRONG `config.rs` (HIGH)

- [ ] **TODO-PB-06: Thu Hẹp Dải Cận Trên `struct_sl_buffer_atr` từ 3.00 về 1.20 ATR**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:448`
  - **Bản chất:** Đổi macro `#[dna(min = 0.20, max = 1.20, step = 0.05)]` để dập tắt 100% bẫy GA chọn SL quá rộng gây VETO toàn bộ hệ thống.

- [ ] **TODO-PB-07: Nâng Sàn `kama_slope_threshold` từ 0.01 lên 0.04**
  - **File:** `config.rs:434`
  - **Bản chất:** Đổi macro `#[dna(min = 0.04, max = 0.35, step = 0.01)]` để triệt tiêu việc nhận diện nhầm sideway thành xu hướng.

- [ ] **TODO-PB-08: Nới Rộng Dải `max_boundary_z` từ [0.20, 0.80] lên [0.50, 2.00]**
  - **File:** `config.rs:451`
  - **Bản chất:** Đồng bộ dải Z-score với đặc tính toán học của nến MTF trong siêu sóng tăng trưởng.

- [ ] **TODO-PB-09: Thu Hẹp Cận Trên `max_pullback_dist_atr` (từ 3.50 về 2.40) & `invalidation_buffer_atr` (từ 1.50 về 0.80)**
  - **File:** `config.rs:438, 442`
  - **Bản chất:** Ngăn chặn việc mua đu đỉnh khi giá bay quá xa KAMA và cắt lỗ cấu trúc sớm khi trend đã gãy.

---

## 🛡️ NHÓM 3: TƯƠNG TÁC THOÁT LỆNH & ARBITER (MEDIUM)

- [ ] **TODO-PB-10: Bổ Sung Tham Số `ttp_callback_atr_trend` Cho Các Playbook Trend**
  - **File:** `config.rs:914` & `risk.rs:305-325`
  - **Bản chất:** Tách riêng khoảng cách Callback TTP cho Trend ($1.50 \sim 3.00$ ATR) thay vì dùng chung mức hẹp $0.80$ ATR, tạo không gian thở để nuôi trọn các con sóng Mega Trend.
  - **Test case:** Lệnh Trend_PB không bị đá văng khi giá chỉ thoái lui $1.0$ ATR trong xu hướng lớn.
