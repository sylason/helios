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
- [ ] **3.1. Thêm bộ đệm Cooldown sau khi đóng hòa vốn trong `AssetActor`:**
  - Khóa cùng Playbook Scanner trong $15$ nến nếu đóng lệnh ở mức PnL hòa vốn ($\approx 0.00\$$).
- [ ] **3.2. Điều kiện bứt phá hủy Cooldown (Volatility Breakout Override):**
  - Mở khóa tức thì nếu giá bứt phá vượt qua biên độ nến cũ $> 1.0 \times \text{ATR}$.

---

### Giai Đoạn 4: Tinh Chỉnh Hàm Mục Tiêu `Objective` trong `ProspectorRunner`
- [ ] **4.1. Bổ sung Profit Factor (PF) & SQN vào hàm Fitness:**
  - Phạt các bộ DNA có số lượng giao dịch ảo quá nhiều hoặc phân bổ PnL không đều.
- [ ] **4.2. Cân bằng giữa Return, Max Drawdown và Trade Expectancy.**

---

### Giai Đoạn 5: Realtime Mark Price Guard Độc Lập
- [ ] **5.1. Xây dựng luồng xử lý `markPriceUpdate` độc lập trong `bot_engine`:**
  - Khớp lệnh thoát khẩn cấp mili-giây khi sập giá mà không đợi hết nến 1m.
- [ ] **5.2. Viết Unit Test và Integration Test kiểm chứng khả năng phản ứng chớp nhoáng.**
