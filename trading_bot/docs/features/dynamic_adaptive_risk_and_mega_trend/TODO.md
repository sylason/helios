# Checklist Triển Khai: Dynamic Adaptive Risk & Mega Trend Engine

**Thư mục tính năng:** `trading_bot/docs/features/dynamic_adaptive_risk_and_mega_trend/`  
**Trạng thái:** Đã hoàn thành đặc tả kiến trúc, sẵn sàng triển khai theo Baby Steps.

---

## 📋 Danh Sách Nhiệm Vụ (Task Checklist)

### Giai Đoạn 1: Sửa "Điểm Gãy" (The Disconnect Gap) trong `Helios RiskManager`
- [x] **1.1. Xóa bỏ các con số gán cứng trong Break-Even Trigger:**
  - Thay thế `be_roe_trigger = 0.05` và `1.5 * current_atr` bằng công thức động phụ thuộc vào `matrix.squeeze_multiplier` và `risk_cfg.base_sl_atr`.
- [x] **1.2. Kích hoạt cơ chế Thả Diều Siêu Sóng (Mega Trend Kite Flying with Hard Floor):**
  - Khi `current_pnl >= risk_cfg.mega_trend_pnl_trigger`, kết hợp đồng thời:
    1. Khóa sàn lợi nhuận cứng: $\text{SL}_{\text{Floor}} = \text{EntryPrice} \pm (\text{EntryPrice} \times \text{PeakROE} \times 0.5 / \text{Leverage})$.
    2. Nới lỏng Trailing Stop sang $\text{risk\_cfg.tsl\_atr\_loose}$ ($8.0 \dots 15.0\text{ ATR}$) để bảo vệ vị thế không bị quét non khi sóng lên $+200\% \dots +300\%$ ROE.
- [x] **1.3. Áp dụng Monotonic Ratchet Guard toàn diện:**
  - Bảo đảm mức SL chỉ tịnh tiến bảo vệ vốn, bất biến trước mọi chu kỳ Re-Opt Hot-Swap DNA.

---

### Giai Đoạn 2: Nâng Cấp Báo Cáo & Trực Quan Hóa Trong `PaperTrader`
- [x] **2.1. Bổ sung các cột thông tin chi tiết vào bảng log `log_all_trades`:**
  - `OPEN TIME` (Định dạng `MM-dd HH:mm`)
  - `CLOSE TIME` (Định dạng `MM-dd HH:mm`)
  - `HELD` (Số nến / số phút thực tế `bars_held`)
- [x] **2.2. Kiểm tra tính chính xác của `PaperTradeRecord` khi xuất báo cáo định kỳ.**

---

### Giai Đoạn 3: Chống Lặp Lệnh (Micro-Churn Re-entry Cooldown)
- [x] **3.1. Thêm cờ bật/tắt `enable_re_entry_cooldown` và bộ đệm Cooldown sau khi đóng hòa vốn trong `config.rs` & `HeliosStrategyV3`:**
  - Khóa cùng Playbook Scanner / Chiều lệnh trong $N$ nến (`re_entry_cooldown_bars`) nếu đóng lệnh ở mức PnL hòa vốn hoặc cắt lỗ nhỏ ($\le 0.001$).
- [x] **3.2. Điều kiện bứt phá hủy Cooldown (Volatility Breakout Override):**
  - Mở khóa tức thì nếu giá bứt phá vượt qua biên độ nến thoát lệnh cũ $\ge 1.5 \times \text{ATR}$.
- [x] **3.3. Bộ Unit Tests kiểm chứng toàn diện:**
  - Đã viết 5 test cases kiểm tra: Tắt toggle `false`, Chặn cùng chiều trong range, Cho phép ngược chiều, Mở khóa sớm khi có bứt phá biến động, Cho phép khi hết thời gian Cooldown. (100% tests PASS).

---

### Giai Đoạn 4: Tinh Chỉnh Hàm Mục Tiêu `Objective` trong `ProspectorRunner`
- [x] **4.1. Bổ sung Profit Factor (PF) & SQN vào hàm Fitness:**
  - Phạt nặng các bộ DNA có Profit Factor mỏng ($PF < 1.15$), thưởng cho $PF \ge 1.50$ và SQN cao.
- [x] **4.2. Cân bằng giữa Return, Max Drawdown và Trade Frequency:**
  - Áp dụng trọng số tần suất lệnh $W_{\text{Freq}}$ (tối ưu $8 - 55$ lệnh/tháng, phạt quá ít hoặc quá nhiều).
- [x] **4.3. Bộ Unit Tests kiểm chứng toàn diện:**
  - Đã viết 3 test cases kiểm tra: Phạt return âm/NaN, Phạt số lệnh quá ít, Thưởng hệ thống có PF dày và SQN cao. (100% tests PASS).

---

### Giai Đoạn 5: Realtime Mark Price Guard Độc Lập
- [ ] **5.1. Xây dựng luồng xử lý `markPriceUpdate` độc lập trong `bot_engine`:**
  - Khớp lệnh thoát khẩn cấp mili-giây khi sập giá mà không đợi hết nến 1m.
- [ ] **5.2. Viết Unit Test và Integration Test kiểm chứng khả năng phản ứng chớp nhoáng.**
