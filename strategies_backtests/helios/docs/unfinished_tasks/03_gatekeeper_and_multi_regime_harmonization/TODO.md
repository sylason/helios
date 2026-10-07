# 📋 DANH MỤC CÔNG VIỆC CẢI TỔ CẦU DAO & HỢP TÁC ĐA CHẾ ĐỘ (UNFINISHED TASK #03)

> **Mô-đun:** `RegimeGatekeeper` & `Multi-Regime Scanners` (Helios V3.3)  
> **Thư mục:** `strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/`  
> **Tài liệu thiết kế:** [`01_gatekeeper_and_multi_regime_harmonization_blueprint.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/01_gatekeeper_and_multi_regime_harmonization_blueprint.md)  
> **Trạng thái:** 🟡 Dự trữ nâng cấp dài hạn (Future Sprint)

---

## 🎯 MỤC TIÊU NÂNG CẤP DÀI HẠN
Nâng cấp `RegimeGatekeeper` từ cơ chế dập cầu dao phẳng (Flat Gating) lên cơ chế Cầu Dao Thích Ứng Đa Tầng (Multi-Regime Adaptive Gating), cho phép các Scanners tần số cao/sóng ngắn (`Range_Osc`, `Absorption`) giao dịch linh hoạt trong vùng biên độ hẹp mà không bị các ngưỡng biến động vĩ mô của `Trend_PB` cản trở.

---

## 📌 CHECKLIST NHIỆM VỤ DÀI HẠN (FUTURE ACTION ITEMS)

- [ ] **TASK-FUT-GK-01: Triển khai Per-Playbook Adaptive Gating trong Arbiter**
  - Tách ngưỡng `min_natr_threshold` thành 2 lớp:
    1. `hard_survival_floor_pct`: Áp dụng toàn cục (chỉ dập khi $\text{NATR} < 2 \times \text{fee} + \text{slip}$).
    2. `tactical_min_natr_pct`: Gắn riêng cho từng Scanner (ví dụ: TrendPB đòi hỏi $\ge 0.40\%$, trong khi RangeOsc chỉ cần $\ge 0.15\%$).
  - **Tệp liên quan:** `arbiter.rs`, `config.rs`.

- [ ] **TASK-FUT-GK-02: Chuẩn Hóa Căn Bậc Hai Thời Gian Theo Timeframe ($\sqrt{T / 15\text{m}}$)**
  - Tự động co giãn ngưỡng Flatline khi mở rộng `ind.natr.timeframe` cho GA Optimizer:
    $$\text{effective\_floor} = \text{base\_floor} \times \sqrt{\frac{T}{15\text{m}}}$$
  - **Tệp liên quan:** `gatekeeper.rs`, `config.rs`.

- [ ] **TASK-FUT-GK-03: Tách Biệt Bộ Cảm Biến Chaos vs Cảm Biến Flatline**
  - Tạo riêng struct `ChaosSensorParams` (chuyên đo bão vĩ mô với cửa sổ dài) độc lập với `MarketNatrParams` (chuyên đo nhịp thở thanh khoản trong phiên).
  - Triệt tiêu hoàn toàn sự lệ thuộc chéo giữa `chaos_ratio` và `natr`.
  - **Tệp liên quan:** `config.rs`, `sensors.rs`, `core_v3/mod.rs`.

- [ ] **TASK-FUT-GK-04: Bộ Test Bench Giả Lập Đa Trạng Thái Thị Trường (Multi-Regime Synthetic Test)**
  - Viết test suite mô phỏng 3 chuỗi nến nhân tạo:
    1. Chuỗi nến Flatline chết hoàn toàn ($ATR = 0.05\%$) $\to$ Xác nhận VETO 100%.
    2. Chuỗi nến Range hẹp ($ATR = 0.35\%$, đi ngang) $\to$ Xác nhận Range_Osc PASS, Trend_PB VETO.
    3. Chuỗi nến Trend bão ($ATR = 1.20\%$, dốc mạnh) $\to$ Xác nhận Trend_PB PASS, Range_Osc VETO.
