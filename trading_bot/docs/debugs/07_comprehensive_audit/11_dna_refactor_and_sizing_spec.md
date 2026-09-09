# 📐 BẢN THIẾT KẾ KỸ THUẬT: TÁI CẤU TRÚC KHÔNG GIAN DNA, LÀM MỊN BƯỚC NHẢY ATR & VÁ LỖI SIZING (AUDIT NHÓM 6)

---

## 🎯 1. BỐI CẢNH & MỤC TIÊU HỆ THỐNG

Sau các đợt kiểm toán chuyên sâu và kết quả thực nghiệm từ Walk-Forward Smart Seed (Log 26 - Log 29), hệ thống đã chứng minh được tính hiệu quả vượt bậc khi tỷ lệ lợi nhuận Out-Of-Sample đạt **+13.25% Net PnL** và đỉnh tài sản **+27.02% Peak Equity**. Tuy nhiên, để đảm bảo hệ thống hoàn toàn thích ứng cho **mọi asset (BTC, ETH, Altcoins), mọi thời điểm, mọi trạng thái thị trường và mọi cấu hình tài khoản**, việc tái cấu trúc không gian siêu tham số DNA và loại bỏ hoàn toàn các điểm nghẽn hardcode là bắt buộc.

### Các Mục Tiêu Cốt Lõi:
1. **Cố định đòn bẩy thông thường 20x & Tách `nuke_leverage` riêng:**
   - Tách `leverage` ra khỏi DNA (bỏ attribute `#[dna]`), biến nó thành Account/User Setting nhằm tránh xung đột với tỷ lệ ký quỹ của sàn (`margin_rate = 0.05`).
   - Bổ sung trường `nuke_leverage: f64` (mặc định 50.0x - 100.0x) cho chiến lược Nuke sau này, người dùng toàn quyền điều chỉnh.
2. **Chuẩn hóa Quản trị Vị thế & Triệt tiêu hoàn toàn lỗi toán học NaN:**
   - Tách `percent_per_trade` ra khỏi DNA, người dùng bot tự quyết định khẩu vị rủi ro (% vốn).
   - Thêm `min_order_usd: f64 = 21.0` và `max_cap_fraction: f64 = 0.25` vào cấu hình, xóa bỏ hoàn toàn magic numbers hardcode trong mã nguồn.
   - Bổ sung chốt chặn `if equity <= 0.0 || risk_pct <= 0.0 || leverage <= 0.0 { return 0.0; }` trong `RiskManager::calculate_size` để triệt tiêu vĩnh viễn lỗi `NaN.min(0.20) == 0.20` khi tài khoản sụt giảm mạnh.
3. **Cắt tỉa 6+ Gen Ma (Phantom Genes):**
   - Cắt bỏ 11 chiều gen không dùng đến: `macd` (4 genes), `vol_ma` (2 genes), `sl_min_dist_pct` (1 gene), `base_lot_size` (1 gene), `leverage` (1 gene), `percent_per_trade` (1 gene), `mega_trend_max_cycles` (1 gene).
   - Đưa số chiều DNA của `HeliosConfigV3` từ **118 chiều xuống 108 chiều** (sau khi thêm 1 gene `tsl_buffer_floor_atr`), giúp GA hội tụ nhanh hơn và giảm tải bộ nhớ cache.
4. **Làm Mịn Bước Nhảy ATR (Step Resolution) & Bao Phủ Toàn Bộ [Min, Max]:**
   - Thu nhỏ step ATR từ `0.20` - `0.25` xuống `0.10` để bước dời Stop Loss mượt mà, không bị giật cục khi quy đổi sang giá danh nghĩa của Altcoin.
   - Giữ nguyên hoặc mở rộng dải `[min, max]` để bao phủ 100% mọi điều kiện thị trường từ thanh khoản siêu dày đến biến động cực đoan.
   - Giới hạn trần ngâm lệnh Zombie `max_stagnant_hold_bars` về tối đa 720 nến (12 tiếng) với step 30 nến, ngăn chặn hoàn toàn việc ngâm lệnh lỗ tới 48 tiếng.

---

## 📊 2. BẢNG SO SÁNH THAY ĐỔI KHÔNG GIAN SIÊU THAM SỐ DNA

| Tham số | File / Vị trí | Cấu hình cũ (V3.2) | Cấu hình mới (V3.3 Refactor) | Rationale & Tác động |
| :--- | :--- | :--- | :--- | :--- |
| `leverage` | `RiskBaseParams` | `#[dna(1.0..50.0, step=1.0)]` | **Chuyển thành User Setting** (mặc định `20.0`) | Tránh xung đột thanh lý giữa sàn 20x và DNA 1x-50x |
| `nuke_leverage` | `RiskBaseParams` | *Chưa tồn tại* | **Thêm mới User Setting** (mặc định `50.0`) | Phục vụ kịch bản đòn bẩy cao Nuke trong tương lai |
| `percent_per_trade` | `RiskBaseParams` | `#[dna(0.02..0.50, step=0.02)]` | **Chuyển thành User Setting** (mặc định `0.20`) | Người dùng bot tự quyết định mức rủi ro danh mục |
| `min_order_usd` | `RiskBaseParams` | Hardcode `$21.0` | **Thêm mới Config Setting** (mặc định `21.0`) | Tránh sàn từ chối min notional mà không bị magic number |
| `max_cap_fraction` | `RiskBaseParams` | Hardcode `.min(0.20)` | **Thêm mới Config Setting** (mặc định `0.25`) | Trần bảo vệ ký quỹ linh hoạt thay cho hardcode 20% |
| `sl_min_dist_pct` | `RiskBaseParams` | `#[dna(0.001..0.030, step=0.001)]` | **Bỏ `#[dna]`** (Gen ma không dùng) | Giảm 1 chiều DNA vô nghĩa |
| `base_lot_size` | `RiskBaseParams` | `#[dna(0.1..2.0, step=0.1)]` | **Bỏ `#[dna]`** (Gen ma không dùng) | Giảm 1 chiều DNA vô nghĩa |
| `mega_trend_max_cycles` | `RiskStopParams` | `#[dna(2.0..30.0, step=1.0)]` | **Bỏ `#[dna]`** (Gen ma không dùng) | Giảm 1 chiều DNA vô nghĩa |
| `macd` (4 fields) | `IndicatorsConfig` | `#[dna]` (fast, slow, signal, tf) | **Bỏ `#[dna]`** (Gen ma không dùng) | Giảm 4 chiều DNA, loại bỏ tính toán thừa |
| `vol_ma` (2 fields) | `IndicatorsConfig` | `#[dna]` (period, tf) | **Bỏ `#[dna]`** (Gen ma không dùng) | Giảm 2 chiều DNA, triệt tiêu lỗi zero-period |
| `tsl_buffer_floor_atr` | `RiskStopParams` | Hardcode `1.0 ATR` | **Thêm mới `#[dna(0.4..2.5, step=0.10)]`** | GA tự tìm đệm an toàn râu nến cho từng cặp tiền |
| `base_sl_atr` | `RiskStopParams` | `min=0.5, max=6.0, step=0.20` | `min=0.5, max=6.0, step=0.10` | Làm mịn bước nhảy SL ban đầu |
| `be_atr_trigger_trend` | `RiskStopParams` | `min=1.5, max=4.0, step=0.25` | `min=1.0, max=4.5, step=0.10` | Mở rộng dải bao phủ, step mịn 0.10 |
| `be_buffer_atr_trend` | `RiskStopParams` | `min=1.0, max=2.5, step=0.25` | `min=0.8, max=3.0, step=0.10` | Mở rộng dải đệm thở cho trend |
| `be_atr_trigger_range` | `RiskStopParams` | `min=1.0, max=2.5, step=0.20` | `min=0.8, max=3.0, step=0.10` | Mở rộng dải, step mịn 0.10 |
| `be_buffer_atr_range` | `RiskStopParams` | `min=0.5, max=1.5, step=0.20` | `min=0.4, max=2.0, step=0.10` | Mở rộng dải đệm range |
| `be_atr_trigger_volatile` | `RiskStopParams` | `min=1.0, max=3.0, step=0.25` | `min=0.8, max=3.5, step=0.10` | Mở rộng dải cho xung lực |
| `be_buffer_atr_volatile` | `RiskStopParams` | `min=0.8, max=2.0, step=0.20` | `min=0.5, max=2.5, step=0.10` | Mở rộng dải đệm xung lực |
| `tsl_atr_normal` | `RiskStopParams` | `min=3.0, max=7.5, step=0.15` | `min=3.0, max=7.5, step=0.10` | Step mịn 0.10 cho xu hướng bậc thang |
| `tsl_atr_loose` | `RiskStopParams` | `min=7.5, max=15.0, step=0.25` | `min=7.5, max=15.0, step=0.15` | Mượt mà hóa phân tầng nới lỏng |
| `max_stagnant_hold_bars` | `RiskStopParams` | `min=120, max=2880, step=60` | `min=60, max=720, step=30` | Trần tối đa 12h, step 30m chống ngâm lệnh zombie |

---

## 🛡️ 3. CÔNG THỨC TOÁN HỌC & XỬ LÝ EDGE CASES

### 3.1. Vá Lỗi Sizing Triệt Tiêu NaN (`risk.rs`):
```rust
pub fn calculate_size(
    equity: f64, 
    risk_pct: f64, 
    min_order_usd: f64, 
    leverage: f64, 
    max_cap_fraction: f64
) -> f64 {
    // Chốt chặn 1: Triệt tiêu hoàn toàn trường hợp tài khoản sụt giảm mạnh hoặc dữ liệu âm/NaN
    if equity <= 0.0 || risk_pct <= 0.0 || leverage <= 0.0 {
        return 0.0;
    }
    
    let mut target_margin = equity * risk_pct;
    let mut order_size_usd = target_margin * leverage;
    
    // Chốt chặn 2: Đảm bảo kích thước tối thiểu để sàn không từ chối lệnh
    if order_size_usd < min_order_usd {
        order_size_usd = min_order_usd;
        target_margin = order_size_usd / leverage;
    }
    
    let size_fraction = target_margin / equity;
    let cap = if max_cap_fraction <= 0.0 { 0.25 } else { max_cap_fraction };
    
    // Chốt chặn 3: Khống chế trần ký quỹ bảo vệ tài khoản
    size_fraction.min(cap)
}
```

### 3.2. Thay Thế Magic Constant Đệm Râu Nến TSL (`risk.rs`):
```rust
// Thay vì cố định 1.0 ATR:
let floor_atr = if config.risk_management.stop_management.tsl_buffer_floor_atr <= 0.0 {
    1.0
} else {
    config.risk_management.stop_management.tsl_buffer_floor_atr
};
let min_buffer = current_atr * floor_atr;
```

---

## 🚀 4. KẾ HOẠCH TRIỂN KHAI THEO BABY STEPS

1. **Step 1: Cập nhật `config.rs` (`strategies_backtests/helios/src/strategy/core_v3/config.rs`)**
   - Loại bỏ `#[dna]` khỏi `macd`, `vol_ma`, `leverage`, `percent_per_trade`, `sl_min_dist_pct`, `base_lot_size`, `mega_trend_max_cycles`.
   - Bổ sung `nuke_leverage`, `min_order_usd`, `max_cap_fraction`.
   - Thêm `tsl_buffer_floor_atr` với macro `#[dna(min = 0.4, max = 2.5, step = 0.10)]`.
   - Cập nhật các step ATR thành `0.10` (và `0.15` cho `tsl_atr_loose`), mở rộng dải `[min, max]`.
2. **Step 2: Cập nhật `risk.rs` & `mod.rs` (`strategies_backtests/helios/src/strategy/core_v3/`)**
   - Cập nhật chữ ký hàm `calculate_size` và bổ sung xử lý an toàn.
   - Thay thế magic number `1.0 ATR` bằng `tsl_buffer_floor_atr`.
   - Bỏ kiểm tra `ind.vol_ma.period == 0` trong `core_v3/mod.rs:64` để triệt tiêu lỗi zero period warning.
   - Cập nhật test case `test_dna_width_and_parameter_defs_integrity`.
3. **Step 3: Cập nhật `walk_forward_smart_seed.rs`**
   - Bổ sung các hằng số cấu hình ở đầu file: `NUKE_LEVERAGE = 50.0`, `PERCENT_PER_TRADE = 0.20`, `MIN_ORDER_USD = 21.0`, `MAX_CAPITAL_FRACTION = 0.25`.
4. **Step 4: Kiểm thử và Bàn giao**
   - Chạy `cargo test --package helios`.
   - Cập nhật trạng thái vào `TODO_CRITICAL.md`.
   - Báo cáo và cung cấp lệnh git commit cho người dùng.
