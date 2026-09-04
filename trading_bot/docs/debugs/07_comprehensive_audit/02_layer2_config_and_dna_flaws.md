# 🚨 Báo Cáo Kiểm Toán Tầng 2: Cấu Hình JSON & Ánh Xạ Gen DNA (Configs & DNA Mapping)

**Dự án:** `helios` & `ferro_trade` & `trading_bot`  
**Ngày kiểm toán:** 2026-09-04  
**Trạng thái:** ⚠️ **PHÁT HIỆN 8 LỖ HỔNG LOGIC (3 CRITICAL, 2 MAJOR/HIGH, 3 MEDIUM)**

---

## 📌 Bảng Tổng Hợp Lỗ Hổng Phát Hiện Tại Tầng 2

| STT | Mức Độ | Tên Lỗ Hổng | Vị Trí Mã Nguồn | Hậu Quả Thực Tế |
| :---: | :---: | :--- | :--- | :--- |
| **1** | 🔴 **CRITICAL** | **21 Tham Số Ma (Phantom Genes) Chiếm 19.1% DNA** | [`config.rs:L80-L530`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)<br>Toàn bộ 6 Scanners, Matrix, Risk | 21 trường cấu hình trong DNA được GA tối ưu và lưu DB nhưng **code runtime KHÔNG HỀ ĐỌC hoặc BỊ GHI ĐÈ BỞI HẰNG SỐ CỨNG**. Lãng phí ~20% tài nguyên GA vào các biến tàng hình. |
| **2** | 🔴 **CRITICAL** | **Xung Đột Đòn Bẩy (Leverage Mismatch) Giữa JSON & Code** | [`helios_optimizer_config.json:L5`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_optimizer_config.json#L5)<br>[`config.rs:L455`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L455) | JSON khóa cứng sàn 20x (`margin_rate: 0.05`), nhưng DNA lại cho GA chọn 1x - 50x. Nếu GA chọn 1x: kẹp trần SL nới ra 80%, nhưng tài khoản 20x bị **thanh lý ở -5% trước khi chạm SL**! |
| **3** | 🔴 **CRITICAL** | **Nguy Cơ Sập Bot (`Index Out Of Bounds`) Khi Đổi Kích Thước DNA** | [`ferro_trade_derive/src/lib.rs:L154`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/ferro_trade_derive/src/lib.rs#L154)<br>[`asset_actor.rs:L235`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L235) | Macro truy cập mảng trực tiếp `dna[offset]` không kiểm tra độ dài. Khi code thêm/bớt gen (ví dụ từ 110 sang 111 hoặc 89), nạp DNA cũ từ SQLite sẽ **PANIC sập AssetActor ngay khi khởi động**. |
| **4** | 🟠 **MAJOR** | **Lỗi Parse Enum `MaType` (`LinReg` & `Vidya`) Âm Thầm Fallback Về `SMA`** | [`config.rs:L120, L150, L152`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L120)<br>[`identifier.rs:L52-54`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L52-L54) | `MaType` dùng `UPPERCASE` serialization nhưng `config.rs` truyền chuỗi `"LinReg"` và `"Vidya"`. `FromStr` fail và **âm thầm fallback về `SMA`** (`default`), GA bị đánh lừa gán nhãn sai loại MA! |
| **5** | 🟠 **MAJOR** | **Smart Seeding Bị Loại Bỏ Trong Im Lặng (Silent Drop)** | [`ga_worker.rs:L392-L397`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/ga_worker.rs#L392-L397) | `dnas.retain(|dna| dna.len() == param_names.len())`. Nếu độ dài lệch dù chỉ 1 gen, hạt giống Champion bị xóa bỏ hoàn toàn trong im lặng không có cảnh báo nào, làm liệt Smart Seeding. |
| **6** | 🟡 **HIGH** | **Lệch Quy Mô (Semantic Mismatch) Làm Tê Liệt 2 Scanners** | [`scanners/absorption.rs:L30-34`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L30-L34)<br>[`scanners/momentum_ignition.rs:L32`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L32) | `vol_force` trong `matrix.rs` có dải $[0.0, 1.0]$, nhưng `vol_z_threshold` trong `config.rs` lại có bounds $[0.20, 5.00]$. Khi GA chọn $\ge 2.0$, hàm tanh cho kết quả cực nhỏ, **làm liệt kích hoạt scanner**! |
| **7** | 🔵 **MEDIUM** | **Thuộc Tính Treo `#[dna]` Trên Biến Kiểu `bool`** | [`config.rs:L533-L537`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L533-L537) | `#[dna(min = 0.01, max = 0.99, step = 0.03)]` bị gắn nhầm lên `enable_re_entry_cooldown: bool` (tàn dư của biến squeeze cũ bị xóa). |
| **8** | 🔵 **MEDIUM** | **Hàng Loạt Hằng Số Cứng (Magic Numbers) Trong Quản Trị Rủi Ro** | [`mod.rs:L240`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L240)<br>[`risk.rs:L23, L29, L148, L211`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) | `$21.0` Min order USD, trần `0.20` ký quỹ, `0.80` đệm thanh lý, `4` chu kỳ zombie, `0.0008` đệm hòa vốn bị hardcode trực tiếp thay vì đọc từ cấu hình. |

---

## 🔎 Chi Tiết Danh Sách 21 Tham Số Ma (Phantom Parameters)

1. **`indicators.rsi_reversal.period` & `timeframe`** (Reversal scanner không hề đọc RSI).
2. **`indicators.rsi_anomaly.period` & `timeframe`** (Anomaly scanner không hề đọc RSI).
3. **`indicators.macd.fast`, `slow`, `signal`, `timeframe`** (Tính xong để đó, không scanner/matrix nào đọc).
4. **`indicators.stoch_trend.d_period` & `d_ma_type`** (Trend scanner chỉ đọc %K, vứt bỏ %D).
5. **`indicators.stoch_trend.zone` & `indicators.stoch_range.zone`** (Chỉ ảnh hưởng trigger tín hiệu nhưng bị hàm init vứt bỏ).
6. **`indicators.vol_ma.period` & `timeframe`** (Matrix dùng price Z-score thay vì volume SMA).
7. **`matrix.volume_lookback`** (Trong `matrix.rs` không xuất hiện biến này).
8. **`arbiter.scanner_weight`** (Công thức đấu thầu tin cậy bỏ qua trọng số này).
9. **`risk_management.risk_base.sl_min_dist_pct`** (Bị ghi đè bởi `min_initial_sl_pct`).
10. **`risk_management.risk_base.base_lot_size`** (Bị ghi đè bởi hardcoded `$21.0`).
11. **`risk_management.stop_management.mega_trend_max_cycles`** (Không có phép so sánh nào).
12. **`risk_management.stop_management.max_stagnant_hold_bars`** (Bị ghi đè bởi hardcoded `decay_cycles >= 4`).
13. **`risk_management.stop_management.stagnant_min_profit_pct`** (Bị ghi đè bởi hardcoded `current_pnl <= 0.0`).

---

## 🛡️ Điểm Sáng Đã Được Xác Minh
1. **Khớp chỉ số $1:1$:** Macro `DnaMappableMacro` duyệt theo đúng thứ tự khai báo trường struct trong Rust, đảm bảo tính đối xứng hoàn toàn giữa `update_from_dna` và `parameter_defs`. Không bị lệch pha chỉ số.
2. **An toàn giá trị biên:** Công thức `.clamp(min, max)` bảo vệ cứng các chu kỳ chỉ báo luôn $\ge 2.0$, không bao giờ bị $0.0$ gây chia cho 0 hay panic. Dải Fast và Slow được thiết kế rời rạc $[2, 18]$ vs $[20, 80]$.

---

## 🛠️ Kế Hoạch Khắc Phục Khuyến Nghị (Action Plan Cho Tầng 2)

1. **Dọn dẹp 21 Gen Ma:** Thu hẹp DNA từ **110 gen xuống 89 gen**, giúp GA hội tụ nhanh hơn **~25%**, giảm kích thước vector lưu SQLite và tiết kiệm RAM.
2. **Khóa cứng Đòn Bẩy 20x:** Đặt `leverage` cố định là `20.0` khớp với `margin_rate: 0.05` trong JSON, loại bỏ rủi ro thanh lý ảo.
3. **Bổ sung `dna_signature` Guard & Safe Slice:**
   - Trong `update_from_dna`: Nếu `dna.len() != DNA_WIDTH`, fallback an toàn và log cảnh báo thay vì panic.
   - Thêm `dna_signature` vào bảng SQLite để tự động bỏ qua DNA cũ khi nâng cấp mã nguồn.
4. **Sửa Enum `MaType` Parsing:** Đổi `"LinReg,Vidya"` thành `"LINREG,VIDYA"` trong `config.rs`.
5. **Sửa dải `vol_z_threshold`:** Thu hẹp về $[0.05, 1.00]$ để khớp với chuẩn hóa `vol_force`.
