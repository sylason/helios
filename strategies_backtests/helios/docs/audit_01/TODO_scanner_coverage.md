# KẾ HOẠCH HỆ THỐNG & ĐỘ PHỦ HÀNH VI THỊ TRƯỜNG (SYSTEMIC & COVERAGE TODO)

> **Mục tiêu:** Giải quyết các vấn đề tương tác hệ thống giữa Scanner và Arbiter/Risk, đồng bộ hóa dòng dữ liệu cảm biến MTF và xử lý các trạng thái thị trường còn bỏ ngỏ.  
> **Tài liệu tham chiếu gốc:**
> - [scanner_coverage_audit.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/scanner_coverage_audit.md)
> - [01_data_lineage_verification.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md)

---

## 🧬 GIAI ĐOẠN 1: ĐỒNG BỘ HÓA DÒNG DỮ LIỆU CẢM BIẾN (DATA LINEAGE FIXES)

### 1. [ ] `TODO-SYS-01`: Tách Bạch Cảm Biến Vi Cấu Trúc (1m) Khỏi Dải MTF Vĩ Mô
- **Tài liệu chi tiết:** [§2 trong 01_data_lineage_verification.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md#2-hiện-tượng-ô-nhiễm-dữ-liệu-mtf--mapping-lookup-table)
- **Vị trí code:** [`core_v3/mod.rs:249-252`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L249) & [`core_v3/config.rs:188-204`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L188)
- **Nội dung:**
  - [ ] **Bước 1.1:** Tách `vol_z_score` khỏi `ind.kama.timeframe`. Khối lượng bùng nổ nến vào lệnh phải tính trên khung nến cơ sở (1m hoặc 5m), không để nến 15m/1h rải đều giá trị volume cao giả tạo cho 15 cây nến 1m kế tiếp.
  - [ ] **Bước 1.2:** Thống nhất quy chuẩn tỷ lệ giữa `ATR_SL` và `NATR` để Gatekeeper và Risk Manager sử dụng chung một thước đo biến động chuẩn hóa.
- **Tiêu chí nghiệm thu:**
  - [ ] Nến 1m có volume thấp ngay sau nến 15m pump không bị gán nhầm Volume Z-Score cao.

---

## ⚖️ GIAI ĐOẠN 2: TỐI ƯU HÓA TƯƠNG TÁC GIỮA CÁC SCANNER (INTERACTION MATRIX)

### 2. [ ] `TODO-SYS-02`: Cô Lập Cầu Dao Cooldown Theo Chiều Lệnh & Playbook (Interaction I2)
- **Tài liệu chi tiết:** [§4.2 trong scanner_coverage_audit.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/scanner_coverage_audit.md#42-các-hình-thức-tương-tác-thực-sự-đã-xác-minh-bằng-codelog)
- **Vị trí code:** [`core_v3/mod.rs:57-99`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L57) & `bot_engine/src/controller/asset_actor.rs`
- **Nội dung:**
  - [ ] Khắc phục hiện tượng: Một lệnh thua của `Momentum` (Long) kích hoạt Cooldown khóa luôn cơ hội vào lệnh `Absorption` (Short) hoặc `Trend_PB` thuận xu hướng.
  - [ ] Cooldown chỉ phạt lặp lại lệnh cùng chiều hoặc cùng playbook gây ra chuỗi thua, cho phép playbook khác có điều kiện thuận lợi tham gia thị trường.

### 3. [ ] `TODO-SYS-03`: Đồng Bộ Hóa Hình Học SL Của Scanner Với Risk Cap (Interaction I4)
- **Tài liệu chi tiết:** [§4.2 (I4) trong scanner_coverage_audit.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/scanner_coverage_audit.md#42-các-hình-thức-tương-tác-thực-sự-đã-xác-minh-bằng-codelog)
- **Vị trí code:** [`core_v3/arbiter.rs:219-233`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L219)
- **Nội dung:**
  - [ ] Đảm bảo công thức tính SL của mọi Scanner (`Trend_PB`, `Absorption`, `Momentum`) luôn bị giới hạn bởi `max_initial_sl_pct` ngay tại Scanner trước khi gửi Proposal sang Arbiter.
  - [ ] Triệt tiêu 100% việc Proposal có điểm số cao (Raw 0.96) bị Sniper Guard VETO oan uổng.

---

## 🌊 GIAI ĐOẠN 3: LẤP ĐẦY CÁC TRẠNG THÁI THỊ TRƯỜNG CÒN BỎ NGỎ (REGIME COVERAGE)

### 4. [ ] `TODO-SYS-04`: Quy Hoạch Trách Nhiệm Vùng Đi Ngang & Nén Biến Động (Hành vi B6)
- **Tài liệu chi tiết:** [§2 (B6) và §3 trong scanner_coverage_audit.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/scanner_coverage_audit.md#2-danh-mục-hành-vi-thị-trường-theo-vật-lý-quan-sát-được-không-theo-mẫu-hình-retail)
- **Vị trí code:** [`core_v3/mod.rs:122-126`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L122)
- **Nội dung:**
  - [ ] Đánh giá lại việc cắm lại `Range_Osc` vào Arbiter hoặc mở rộng `Absorption` để tiếp quản vùng thị trường đi ngang (Mean-reverting Range), tránh để trống vùng Chop Zone.

### 5. [ ] `TODO-SYS-05`: Xử Lý Nhịp Hồi Nông Trong Xu Hướng Mạnh (Hành vi B2)
- **Tài liệu chi tiết:** [§2 (B2) trong scanner_coverage_audit.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/scanner_coverage_audit.md#2-danh-mục-hành-vi-thị-trường-theo-vật-lý-quan-sát-được-không-theo-mẫu-hình-retail)
- **Nội dung:**
  - [ ] Cho phép `Trend_PB` mở vị thế khi `trend_strength` (ADX) cực mạnh dù Stochastic chưa rơi sâu về vùng quá bán (hồi nông đi ngang).
