# TODO CẢI TỔ CHIẾN THUẬT TREND PULLBACK (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **Ngày lập:** 03/10/2026  
> **Nguyên tắc:** Baby Steps, Phân tích trước - Code sau, Viết testcase đi kèm từng task.

---

## 🔥 NHÓM 1: CÁC CHỐT CHẶN SINH TỬ TRONG CODE `trend_pb.rs` (CRITICAL)

- [x] **TODO-PB-01: Khử Bẫy KAMA Suction Trap (Neo Stop Loss Động Tối Giản Vi Mô)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`
  - **Bản chất:** Bỏ hoàn toàn việc kéo SL theo KAMA 4h (`low.min(kama_mid)`). Neo SL tối giản hình học theo đáy/đỉnh nến vi mô hiện tại + đệm ATR vi mô: `low - buffer` cho Long và `high + buffer` cho Short kèm chốt chặn chống Inverted SL (`sl >= close` hoặc `sl <= close`).
  - **Test case:** 
    - `test_trend_pb_sl_geometry_no_kama_suction`: Đảm bảo SL không bị kéo tụt xuống KAMA 4h cách xa 3.0 ATR.
    - `test_trend_pb_single_bar_sl_anchoring`: Xác nhận SL neo chuẩn xác tại `low - buffer` của nến hiện tại mà không nhìn ngược đa nến.

- [x] **TODO-PB-02: Giải Phóng Nến Pinbar / Hammer Rút Chân & Bổ Sung Gen DNA `min_clv`**
  - **File:** `trend_pb.rs` & `config.rs:481-484, 1093-1095` & `mod.rs:1325`
  - **Bản chất:** Xóa bỏ hoàn toàn định kiến màu nến bán lẻ `close < open` và `close > open`. Bổ sung gen DNA `min_clv` vào `TacticalPullbackParams` (`#[dna(min = 0.05, max = 0.50, step = 0.05)]`) đồng bộ chuẩn 4 scanners. Tăng kích thước chuỗi DNA hệ thống từ `102` lên `103` chiều.
  - **Test case:** 
    - `test_trend_pb_allows_red_hammer`: Nến Hammer thân đỏ rút chân mạnh ($CLV \ge min\_clv$) được duyệt mở Long.
    - `test_trend_pb_blocks_marubozu`: Nến đỏ Marubozu xả đặc ruột ($CLV = -1.0$) bị chặn đứng.
    - `test_dna_width_and_parameter_defs_integrity`: Kiểm tra chuỗi DNA chính xác 103 chiều và chứa trường `pullback.min_clv`.

- [x] **TODO-PB-03: Sửa Lỗi Mất Tính Định Hướng Trong `f_dist`**
  - **File:** `trend_pb.rs:116, 163`
  - **Bản chất:** Bỏ phép lấy `.abs()` trong `sat_proximity(delta_kama.abs(), max_dist)`. Khi giá xuyên thủng KAMA ($\text{delta\_kama} < 0$ cho Long), phải áp dụng hệ số phạt (penalty) thay vì thưởng điểm cao.
  - **Test case:** Giá thủng KAMA -0.4 ATR phải có điểm khoảng cách thấp hơn rõ rệt so với giá giữ vững hỗ trợ +0.1 ATR.

- [x] **TODO-PB-04: Bổ Sung Tín Hiệu Stochastic Ngóc Đầu Đảo Chiều (Stoch Hook Guard)**
  - **File:** `trend_pb.rs:101-103, 140-142`
  - **Bản chất:** Cấm Long khi $\text{stoch\_k} < 10.0$ và đang cắm đầu rơi tự do ($\text{stoch\_k} < \text{prev\_stoch\_k}$); Cấm Short khi $\text{stoch\_k} > 90.0$ và đang dựng đứng ($\text{stoch\_k} > \text{prev\_stoch\_k}$). Chỉ cho phép mở vị thế khi Stochastic đã uốn cong tạo điểm uốn (Hook inflection point).
  - **Test case:** `test_trend_pb_soft_hook_guard_blocks_falling_knife`: Chặn đứng dao rơi Stoch cắm đầu, phê duyệt khi Stoch đã ngóc đầu tạo đáy.

- [x] **TODO-PB-05: Đưa Lực Nến `clv` Vào Hàm Chấm Điểm Lồi**
  - **File:** `trend_pb.rs:110, 149`
  - **Bản chất:** Đưa thành phần $f_{\text{clv}} = \frac{CLV - min\_clv}{1 - min\_clv}$ vào tổ hợp lồi `convex_combine(&[f_slope, f_stoch, f_clv], &[0.40, 0.30, 0.30])`. Phản ánh trung thực chất lượng nến rút chân để tín hiệu Pullback đủ sức cạnh tranh công bằng với Momentum trong đấu thầu Arbiter.
  - **Test case:** Đã xác minh qua test suite của `trend_pb.rs`.

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
