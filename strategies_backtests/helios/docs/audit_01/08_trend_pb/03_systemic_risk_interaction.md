# KIỂM TOÁN TƯƠNG TÁC HỆ THỐNG, ĐẤU THẦU TÍN HIỆU & QUẢN TRỊ RỦI RO (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **Các module liên đới:** `sensors.rs`, `matrix.rs`, `arbiter.rs`, `risk.rs`, `gatekeeper.rs`, `mod.rs`  
> **Ngày lập:** 03/10/2026

---

## 🧭 1. KIỂM TOÁN DÒNG DỮ LIỆU ĐẦU VÀO (DATA LINEAGE)

1. **Bẫy Tử Địa Z-Score MTF (`core_v3/mod.rs:249`):**
   - `sensors.z_score` được tính trên khung KAMA lớn (15m/1h/4h) với lookback 600 nến.
   - Trong xu hướng tăng mạnh, $Z_{\text{1h}}$ luôn neo cao ở mức $+1.0\sigma \sim +2.5\sigma$.
   - Trong khi đó, `max_boundary_z` bị kẹp trần DNA từ $0.20$ đến $0.80$.
   - **Hệ quả:** 90%+ tín hiệu Trend_PB bị triệt hạ ngay cả khi nến 1m đã pullback hoàn hảo chạm KAMA!
2. **Bẫy Không Gian ATR (Spatial Mismatch):**
   - Nếu DNA chọn `atr.timeframe` là 5m nhưng `kama.timeframe` là 1h: Dùng ATR 5m để chuẩn hóa khoảng cách `delta_kama = (close - kama_mid) / current_atr` khiến `delta_kama` bùng nổ quá lớn ($> 3.5$ ATR), làm bộ lọc khoảng cách bị liệt.

---

## ⚖️ 2. TẦNG ĐẤU THẦU TÍN HIỆU ARBITER (`arbiter.rs`)

1. **Phổ Điểm `raw_score` Bị Dìm Xuống 0.55 - 0.70:**
   - Công thức tổ hợp lồi với hàm Tanh của Stochastic phạt nặng khi Stoch ở mức $20 \sim 30$. Điểm số tự nhiên của Trend_PB chỉ đạt $0.55 \sim 0.70$.
   - Ngược lại, `Momentum_Ignition` với hàm bùng nổ `sat_unbounded(vol_z)` dễ dàng đạt $0.85 \sim 0.98$.
   - **Trong các cuộc đối đầu trực diện:** Momentum luôn đè bẹp và tước quyền vào lệnh của Trend_PB 100%.
2. **Cơ Chế Khắc Phục:**
   - Cần chuẩn hóa lại thang điểm Stochastic trong `trend_pb.rs`: Khi $\text{stoch\_k} \le \text{stoch\_thresh}$, điểm thành phần này phải đạt tối thiểu $\ge 0.75$, nâng phổ điểm tổng thể của Trend_PB lên **$0.75 \sim 0.88$** để cạnh tranh công bằng.

---

## 🛑 3. TẦNG QUẢN TRỊ RỦI RO & THỰC THI (`risk.rs`)

1. **Bẫy KAMA Suction Trap Triệt Hạ Lệnh:**
   - Lệnh Pullback nông đẹp nhất ($\text{Low} > \text{kama\_mid}$) bị phép tính `low.min(kama_mid)` kéo SL lùi sâu về tận KAMA MTF, khiến khoảng cách ROE vọt lên $40\% \sim 80\%$.
   - Arbiter kích hoạt `StructuralRiskVeto` (trần 15% ROE) bắn hạ 100% các lệnh này.
2. **Bóp Nghẹt Siêu Sóng Tại Hybrid TTP:**
   - Trend_PB là chiến lược Trend Following sống nhờ đuôi lãi dày (fat-tail).
   - Tuy nhiên, `ttp_callback_atr` mặc định chỉ là **$0.80$ ATR** (`config.rs:938`).
   - Một con sóng lớn luôn có nhịp rung lắc tự nhiên $1.0 \sim 1.5$ ATR. Khoảng cách Callback $0.80$ ATR quá hẹp sẽ đá văng lệnh ra khỏi vị thế ngay nhịp thở đầu tiên, biến Trend Following thành Scalping chốt non!
   - **Giải pháp:** Tách riêng `ttp_callback_atr_trend` với dải DNA $[1.50, 3.50]$ ATR cho các chiến lược Trend.
