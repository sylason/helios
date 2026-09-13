# 📑 TÀI LIỆU KỸ THUẬT: FLAW-TSL.7 - UNIVERSAL BIDIRECTIONAL COOLDOWN & DIRECTIONAL EARLY RELEASE

> **Mã lỗi:** FLAW-TSL.7  
> **Dự án liên quan:** strategies_backtests/helios & 	rading_bot/bot_engine  
> **Thư mục tài liệu:** 	rading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl7_bidirectional_cooldown/  
> **Trạng thái:** Đang thẩm định thiết kế (Ready for Implementation)

---

## 1. BỐI CẢNH & PHÂN TÍCH BẢN CHẤT LỖI LOGIC

### 1.1. Hiện trạng thực tế từ Log 33 & Log 34 (Bẫy Alternating Whipsaw)
Trong vùng thị trường đi ngang quét râu 2 đầu (Choppy Whipsaw Market):
1. **Lệnh Long bị cắt lỗ ở đáy:** Một cây nến đỏ xả mạnh đâm thủng đáy hỗ trợ -> Quét sạch Stop Loss của lệnh Long.
2. **Kích hoạt bẫy Short ngay tại đáy:**
   - Cú xả làm giá đâm toạc dải dưới Bollinger Band, RSI rơi cắm đầu vào vùng quá bán sâu -> Các Scanners (MomentumIgnition, TrendPullback) phát tín hiệu Short với điểm số rất cao (> 0.90).
   - Cơ chế Cooldown hiện tại chỉ khóa 1 chiều: locked_side = Some(SignalType::Long).
   - Khi kiểm tra tín hiệu Short mới: locked_side == Some(signal) cho kết quả alse -> **KHÔNG BỊ CHẶN!**
   - Bot lập tức **MỞ LỆNH SHORT NGAY ĐÁY TUYỆT ĐỐI!**
3. **Lệnh Short bị cắt lỗ ở đỉnh:**
   - Giá sau đó rút chân giật ngược chữ V tăng bốc đầu -> Quét sạch Stop Loss lệnh Short.
   - Cooldown khóa 1 chiều: locked_side = Some(SignalType::Short).
   - Nến xanh giật lên lại kích hoạt tín hiệu Long đỉnh -> Bot lại **MỞ LỆNH LONG ĐU ĐỈNH!**
   - Lại bị sập hầm và quét SL tiếp.
👉 **Hậu quả:** Trong ngày 02/09 (Log 33), tài khoản bị vả 2 đầu liên tiếp 4 lệnh so le (#0004, #0005, #0007, #0008), bốc hơi 16% tổng vốn chỉ trong vài giờ!

### 1.2. Hiện trạng phân mảnh & Lệch pha Cooldown giữa 2 tầng
- **Tầng Strategy (HeliosStrategyV3):** Khóa theo số bar trong DNA e_entry_cooldown_bars (thường 30-35 bars), kiểm tra bứt phá 1.5 * ATR.
- **Tầng Thực thi (AssetActor):** Tự sinh ra struct CooldownGuard riêng, hardcode thời gian khóa tĩnh 15 bars, kiểm tra bứt phá bằng .abs() >= 1.5% (gây lỗi bắt dao rơi khi giá càng sập sâu).
👉 **Hậu quả:** Hai bộ não Cooldown chạy lệch nhau cả về thời gian mở khóa lẫn giá tham chiếu exit!

---

## 2. TRIẾT LÝ THIẾT KẾ TINH GỌN (KISS: SLEEP CẢ 2 CHIỀU + DIRECTIONAL EARLY RELEASE)

Thay vì dựng lên cỗ máy phân tầng phức tạp, hệ thống áp dụng nguyên lý **Thua lỗ = Ngủ đông (Sleep 2 chiều)**:
1. **Sleep Cả 2 Chiều (Bidirectional Sleep):**
   - Khi lệnh đóng bị lỗ (eal_pnl <= 0.001), hệ thống lập tức khóa **CẢ 2 CHIỀU Long & Short** trong cooldown_bars nến (lấy từ DNA).
   - Tuyệt đối không cho phép mở lệnh ngược chiều ngay tại đáy hoặc đỉnh vừa quét Stop Loss.
2. **Directional Early Release (Mở khóa sớm nếu có sóng bứt phá thực sự):**
   - Tín hiệu mới là **LONG**: Chỉ mở khóa sớm nếu giá đã bứt phá tăng vượt lên: Close - ExitPrice >= 1.5 * ATR. (Nếu giá tiếp tục sập sâu thì cấm tuyệt đối Long bắt dao rơi!).
   - Tín hiệu mới là **SHORT**: Chỉ mở khóa sớm nếu giá đã bứt phá sập sâu xuống: ExitPrice - Close >= 1.5 * ATR. (Nếu giá giật tăng tiếp thì cấm tuyệt đối Short cản tàu!).
   - Nếu biến động nhỏ (< 1.5 ATR): Tiếp tục ngủ đủ N nến, bảo vệ vốn tuyệt đối!
3. **Mô Hình Phòng Ngự Kép (Dual Sentinels - Zero Desync):**
   - **Tầng Chiến Lược (`HeliosStrategyV3`):** Quản lý `CooldownTracker` cho Backtest & GA Optimizer (Single Source of Truth cho quá trình tiến hóa).
   - **Tầng Thực Thi (`AssetActor`):** Do `evaluate_live_tick` chạy trên Dummy Broker nên Strategy không nhận thức được lệnh thật vừa bị cắt lỗ. Vì vậy, `AssetActor` trang bị `EngineCooldownGuard` tại cổng ra:
     + Đọc trực tiếp tham số `re_entry_cooldown_bars` và `cooldown_breakout_atr_mult` từ DNA của chiến lược (hoàn toàn Dynamic, loại bỏ hoàn toàn các hằng số cứng tự phát).
     + Tự động kích hoạt khóa 2 chiều khi phát hiện vị thế thật (PaperTrader/Binance) bị đóng lỗ hoặc hòa vốn danh nghĩa (`trade_pnl <= 0.001`).
     + Áp dụng chốt chặn kép: Cấm mở lệnh cùng cây nến vừa đóng vị thế (Intra-candle) VÀ cấm mở lệnh nếu chưa thỏa mãn điều kiện Directional Early Release ($\ge k \times ATR$).

---

## 3. DANH MỤC FILE THAY ĐỔI & KIỂM THỬ

1. `strategies_backtests/helios/src/strategy/core_v3/mod.rs` (Cập nhật CooldownTracker & logic Bidirectional Sleep trong Strategy).
2. `strategies_backtests/helios/src/strategy/core_v3/config.rs` (Thêm gene `cooldown_breakout_atr_mult`).
3. `trading_bot/bot_engine/src/controller/asset_actor.rs` (Triển khai `EngineCooldownGuard` tại cổng thực thi, đọc tham số từ Strategy DNA).
4. `trading_bot/bot_engine/src/controller/asset_actor.rs` (Thêm unit tests kiểm chứng Engine Cooldown Guard Veto và Directional Early Release).
5. `strategies_backtests/helios/src/strategy/core_v3/tests.rs` (Bộ unit test Bidirectional Cooldown trong Helios).
