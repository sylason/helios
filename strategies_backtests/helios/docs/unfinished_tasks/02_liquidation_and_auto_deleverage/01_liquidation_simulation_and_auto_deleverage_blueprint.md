# THIẾT KẾ KIẾN TRÚC TƯƠNG LAI: LIQUIDATION SIMULATION & DYNAMIC AUTO-DELEVERAGE
> **Mã tài liệu:** `01_liquidation_simulation_and_auto_deleverage_blueprint.md`  
> **Thư mục:** `docs/unfinished_tasks/02_liquidation_and_auto_deleverage/`  
> **Phân hệ liên quan:** `ferro_trade` Simulated Broker, `trading_bot` PaperTrader & `helios` Risk Manager  
> **Trạng thái:** 📋 **ĐÃ XÁC MINH & BẢO LƯU CHO GIAI ĐOẠN ĐÒN BẨY CAO / VỐN LỚN**  
> **Thời điểm kích hoạt:** Khi hệ thống triển khai các chiến lược siêu đòn bẩy ($\ge 50\text{x}$), mở rộng giao dịch sang Altcoins có MMR biến thiên mạnh, hoặc nâng cấp `ferro_trade` thành Crypto Futures Engine chuẩn mực quốc tế.

---

## 🏛️ 1. BỐI CẢNH KIỂM CHỨNG & ĐỐI CHẤT KIẾN TRÚC

Trong quá trình kiểm toán và giải quyết `TODO-RSK-01` (Bổ sung chốt chặn MMR & Liquidation Guard), chúng ta đã đào sâu và phát hiện 3 vấn đề nâng cao:

### 🔍 1.1. Sự thiếu vắng cơ chế mô phỏng thanh lý trong `ferro_trade` (Broker Simulation Blindness):
- **Hiện trạng:** `ferro_trade/src/broker/broker.rs` chỉ kiểm tra lệnh Stop Loss và Take Profit.
- Nếu một vị thế bị lỗ vượt quá số vốn ký quỹ (Unrealized PnL âm sâu làm Equity âm): `Broker` **không hề có cơ chế cưỡng chế thanh lý (Margin Call / Liquidation)**.
- **Rủi ro:** Một chiến lược không có SL hoặc SL đặt quá xa có thể bị "ảo tưởng an toàn" trong Backtest (chỉ lỗ khi giá hồi lại), trong khi trên sàn Binance thực tế tài khoản đã bị giải chấp cháy rụi và mất sạch tiền ký quỹ.

### 🔍 1.2. Hạn chế của việc cố định đòn bẩy (Rigid Leverage Trap):
- Khi thị trường mở ra một cơ hội sóng lớn rất đẹp nhưng đòi hỏi khoảng cách Stop Loss kỹ thuật tương đối rộng ($\Delta_{\text{struct\_sl}} = 1.8\%$).
- Nếu cấu hình đòn bẩy của hệ thống đang cố định ở $L = 50\text{x}$:
  - Khoảng cách thanh lý an toàn chỉ là $\Delta_{\text{liq\_safe}} \approx 1.2\%$.
  - Chốt chặn an toàn `TODO-RSK-01` (Strict Veto) sẽ **từ chối mở lệnh hoàn toàn** để bảo vệ tài khoản.
  - **Hệ quả:** Bỏ lỡ một cơ hội giao dịch có tỷ lệ $EV$ dương chỉ vì đòn bẩy bị đóng cứng quá cao.

---

## 🚀 2. THIẾT KẾ CÁC TÍNH NĂNG NÂNG CẤP DÀI HẠN

### 🔹 **2.1. Động Cơ Mô Phỏng Thanh Lý Trong `ferro_trade` (Simulated Liquidation Engine)**
- **Vị trí tác động:** `ferro_trade/src/broker/broker.rs` (hàm `process_orders` hoặc vòng lặp nến `step`).
- **Toán học thanh lý:**
  Tại mỗi cây nến, Broker tính Giá thanh lý của từng vị thế mở:
  $$P_{\text{liq, Long}} = \text{Entry} \times \left(1 - \frac{1}{L} + \text{MMR}\right)$$
  $$P_{\text{liq, Short}} = \text{Entry} \times \left(1 + \frac{1}{L} - \text{MMR}\right)$$
- **Quy tắc khớp giải chấp:**
  - Nếu `tick.low <= P_{\text{liq, Long}}` (cho Long) hoặc `tick.high >= P_{\text{liq, Short}}` (cho Short):
    - Cưỡng chế đóng vị thế tại đúng $P_{\text{liq}}$ (hoặc giá xấu hơn theo slippage).
    - Khấu trừ **Phí phạt thanh lý (Liquidation Clearance Fee)** từ $0.5\% - 1.5\%$ notional.
    - Đánh dấu `trade.exit_tag = Some("LIQUIDATED")`.

### 🔹 **2.2. Cơ Chế Tự Động Hạ Đòn Bẩy Thông Minh (Dynamic Auto-Deleverage Guard)**
- **Triết lý:** Thích ứng tối thượng theo phong cách Mahoraga: *"Thị trường đòi hỏi không gian rộng bao nhiêu thì ta thích nghi đòn bẩy tương ứng bấy nhiêu"*.
- **Toán học tính đòn bẩy an toàn:**
  Nếu $\Delta_{\text{struct\_sl}} > \Delta_{\text{liq\_safe}}(L_{\text{config}})$, hệ thống tự động giải phương trình tìm đòn bẩy tối đa cho phép:
  $$L_{\text{safe}} = \left\lfloor \frac{1}{\frac{\Delta_{\text{struct\_sl}}}{1.0 - \text{buffer\_ratio}} + \text{MMR} + 2 \times \text{fee}} \right\rfloor$$
- **Cơ chế thực thi trên sàn Binance (`trading_bot`):**
  - Trước khi gửi lệnh Market Open: Nếu $L_{\text{safe}} < L_{\text{config}}$, bot gọi REST API `POST /fapi/v1/leverage` để hạ đòn bẩy cặp tiền đó xuống $L_{\text{safe}}$.
  - Mở vị thế và đặt Stop Loss an toàn.
  - Khi lệnh đóng: Tự động khôi phục lại đòn bẩy chuẩn $L_{\text{config}}$.

### 🔹 **2.3. Nạp Bảng MMR Bậc Thang Tự Động (Binance Leverage Bracket Integration)**
- Tự động nạp toàn bộ danh sách `leverageBracket` của Binance để biết chính xác MMR theo từng Tier khối lượng danh nghĩa, không cần người dùng tự tra cứu bằng tay.

---

## 🛡️ 3. LÝ DO BẢO LƯU & GIẢI PHÁP TẠM THỜI CHO HIỆN TẠI

- **Lý do bảo lưu:**
  1. Đòn bẩy hiện tại của hệ thống được chuẩn hóa ở mức **$20\text{x}$ cố định**. Tại $20\text{x}$, khoảng cách thanh lý ($4.0\% - 4.5\%$) rộng gấp hơn 5 lần trần dừng lỗ $15\%$ ROE ($0.75\%$). Rủi ro thanh lý ở $20\text{x}$ là cực kỳ thấp.
  2. Việc gọi API đổi đòn bẩy liên tục sinh thêm độ trễ, hao tổn API Weight và có nguy cơ phát sinh Race Condition.
  3. Cần tập trung nguồn lực giải quyết các lỗ hổng P0 trực tiếp để hoàn thành chu kỳ Walk-Forward và Paper Trading.
- **Giải pháp áp dụng ngay cho hiện tại (`TODO-RSK-01`):**
  - Triển khai **Strict Veto** và **Fractional Margin Clamping (15%)** trực tiếp trong `helios::RiskManager`.
  - Mọi lệnh có SL chạm vùng nguy hiểm thanh lý đều bị chặn đứng ngay từ tầng Arbiter, đảm bảo an toàn 100% cho tài khoản mà không cần can thiệp phức tạp vào API sàn.
