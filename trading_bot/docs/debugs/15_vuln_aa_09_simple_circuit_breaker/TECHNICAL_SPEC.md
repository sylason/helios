# THIẾT KẾ KỸ THUẬT: CẦU DAO TÀI KHOẢN ĐỘNG TINH GỌN (DYNAMIC ACCOUNT CIRCUIT BREAKER)
## KHẮC PHỤC TRIỆT ĐỂ: VULN-AA-09, VULN-AA-10, VULN-AA-11

---

## 1. Bối Cảnh & Phân Tích Bản Chất 3 Lỗ Hổng

### 1.1. `VULN-AA-09`: Deadlock Vĩnh Viễn Do `Same-Side Price Guard`
* **Vị trí:** [`asset_actor.rs:73-85`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L73-L85) và [`core_v3/mod.rs:66-76`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L66-L76).
* **Cơ chế lỗi:** Khi `consecutive_losses > 0`, hệ thống lưu `last_losing_price`. Nếu tín hiệu cùng chiều có `current_price < last_losing_price` (đối với Long), lệnh bị VETO ngay lập tức.
* **Tử huyệt:** Điều kiện này **không có thời hạn nến (No TTL)**. Khi thị trường sập sâu liên tục, giá nằm dưới điểm cắt lỗ suốt nhiều ngày $\implies$ Bot bị "tê liệt" suốt **2.64 ngày (3,803 nến)** trong log thực nghiệm.

### 1.2. `VULN-AA-10`: Bẫy Mở Khóa Sớm (`Early Release`) & Lệch Pha ATR Khung Nhỏ
* **Vị trí:** [`asset_actor.rs:87-101`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L87-L101) và hàm [`calculate_rolling_atr`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L107-L125).
* **Cơ chế lỗi:** Trong thời gian Cooldown, nếu giá bứt phá vượt qua $\text{last\_exit\_price} \pm 1.75 \times \text{ATR}_{1m}$, bot tự động "xé rào" mở khóa sớm.
* **Tử huyệt:**
  1. *Triệt tiêu bản chất Cầu Dao:* Cooldown sinh ra để hạ nhiệt sau chuỗi thua. Mở khóa sớm khi có nến giật mạnh sẽ dẫn bot vào bẫy đu đỉnh/bán đáy (FOMO Whipsaw).
  2. *Ô nhiễm dữ liệu:* Tự tính ATR 14 nến trên khung 1m cực kỳ nhiễu, lệch pha hoàn toàn với KAMA/ATR đa khung thời gian (MTF) của Helios Strategy.
  3. *Nghịch lý log:* Do bị `VULN-AA-09` chặn trước, log in ra chuỗi so sánh vô lý: `FavorableDiff: +55.28 < 1.75*ATR (4.45)`.

### 1.3. `VULN-AA-11`: Phân Mảnh Hai Nhánh Xử Lý Đóng Vị Thế
* **Vị trí:** [`asset_actor.rs:573-611`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L573-L611) (`intra-candle exit`) vs [`asset_actor.rs:723-745`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L723-L745) (`ClosePosition`).
* **Cơ chế lỗi:** 
  - Nhánh 1 (Intra-candle SL/TP): Cập nhật `consecutive_losses`, tính Exponential Cooldown (với trần cứng 60 nến).
  - Nhánh 2 (ClosePosition / Sàn): Gán thẳng `locked_until_bar = current + base_bars`, không cập nhật `consecutive_losses`, không tính lũy tiến, không reset khi thắng.
* **Tử huyệt:** Bất nhất trạng thái (State Inconsistency). Khi lệnh đóng bởi Strategy Action hoặc sàn, chuỗi thua không được ghi nhận chính xác.

---

## 2. Thiết Kế Mới: Cầu Dao Động Tinh Gọn (Dynamic TTL Circuit Breaker)

### 2.1. Triết Lý Cốt Lõi (First-Principles)
1. **Phân Định Ranh Giới (Separation of Concerns):**
   * **Strategy (`Helios`):** Chuyên tâm phân loại chế độ thị trường và sinh tín hiệu Alpha. Không tự bịt mắt bằng các phán đoán giá quá khứ.
   * **Bot Engine (`AssetActor`):** Nắm giữ số dư thật, lệnh thật, PnL thật. Độc quyền sở hữu **Cầu Dao Ngắt Mạch Tài Khoản (Account Circuit Breaker)**.
2. **Pure TTL (Hết Hạn Là Mở Khóa):**
   * Sau chuỗi thua, hệ thống nghỉ $N$ nến.
   * Khi $\text{current\_bar\_idx} \ge \text{locked\_until\_bar}$, Cầu Dao tự động đóng mạch 100%, không cần so sánh giá, không cần điều kiện bứt phá ATR.
3. **Không Hardcode Hằng Số (No Magic Numbers):**
   * $\text{base\_bars}$: Lấy từ DNA Optimizer (`stop_cfg.re_entry_cooldown_bars`).
   * $\text{streak}$: Số lần thua liên tiếp thực tế.
   * $\text{MaxCapBars} = \text{base\_bars} \times 4$ (Co giãn tự nhiên theo chu kỳ nến cơ sở).

---

## 3. Bảng Đối Chất Toàn Diện Giữa Các Phương Án Cho `VULN-AA-10`

| Tiêu Chí | Phương Án A: Loại Bỏ Hoàn Toàn Early Release (Đề Xuất) | Phương Án B: Giữ Lại Early Release & Sửa MTF ATR |
| :--- | :--- | :--- |
| **Bản chất nghiệp vụ** | Đúng bản chất Cầu Dao: Nghỉ ngơi tuyệt đối sau chuỗi thua để tránh Whipsaw | Cố đoán thị trường: Cho rằng bứt phá giá là xu hướng mới đã hình thành |
| **Độ phức tạp code** | Cực kỳ tối giản: Xóa bỏ 100% `calculate_rolling_atr`, `last_exit_price`, `FavorableDiff` | Phức tạp: Phải truyền ATR từ Matrix/Strategy sang Engine; xử lý bẫy chia cho 0 |
| **Rủi ro Whipsaw / FOMO** | **0% rủi ro**: Không thể bị lừa bởi nến giật ảo (Fakeout) | **Cao**: Râu nến quét thanh khoản 1m có thể kích hoạt Early Release rồi sập tiếp |
| **Tính độc lập thời gian** | Đảm bảo tính toán nhịp thở thị trường theo đúng số nến định lượng | Phá vỡ tính chu kỳ của hình phạt lũy tiến |
| **Edge Cases** | Bỏ lỡ một nhịp bứt phá sớm nếu thị trường quay đầu cực mạnh ngay sau khi dính SL | Vào lệnh sớm đúng ngay đỉnh một cú "bull trap" rút chân |

### 3.1. Quyết Định Thiết Kế Chính Thức Cho `VULN-AA-10`
* **Đồng thuận: Chọn Phương Án A (The Pure Circuit Breaker).**
* **Hành động kỹ thuật:**
  1. Xóa bỏ hoàn toàn hàm `calculate_rolling_atr` trong `asset_actor.rs`.
  2. Xóa bỏ toàn bộ nhánh kiểm tra `is_favorable_breakout`, xóa biến `last_exit_price` và `FavorableDiff`.
  3. Cầu Dao chỉ kiểm tra duy nhất: `current_bar_idx < self.locked_until_bar`. Hết thời gian là tự động mở khóa 100%.

---

## 4. Quyết Định Thiết Kế Chính Thức Cho `VULN-AA-11` (Unified Lifecycle Sentinel)

### 4.1. Kiến Trúc Cốt Lõi: Hybrid Engine-Drain + Snapshot Fallback
* **Đồng thuận: Chọn Phương Pháp 4 (Unified Lifecycle Sentinel).**
* **Mục tiêu:** 
  - Khắc phục triệt để hiện tượng phân mảnh trạng thái giữa `intra-candle exit`, `ClosePosition` và `WebSocket`.
  - Khử bỏ hoàn toàn bẫy so sánh PnL tuyệt đối `trade_pnl <= 0.001` (bẫy mệnh giá BTC vs LSK).
  - Tự động trừ phí sàn 2 chiều ($2 \times \text{CommissionRate} \approx 0.08\%$) để bảo vệ tài khoản khỏi bẫy hòa vốn danh nghĩa nhưng lỗ tiền phí.

### 4.2. Luồng Xử Lý 3 Bước Tại 1 Điểm Chốt Duy Nhất (`sync_and_handle_closed_positions`):
1. **Bước 1 (Đồng bộ & Nhận diện Đóng Vị Thế):**
   * So sánh ảnh chụp vị thế trước đó (`previous_position`) và vị thế hiện tại (`current_position`).
   * Nếu `previous_position.is_some() && current_position.is_none()` $\implies$ Vị thế vừa đóng hoàn toàn.
2. **Bước 2 (Giải Quyết Dữ Liệu Hai Nguồn - Dual-Feed Data Resolution):**
   * **Nguồn 1 (Ưu tiên số 1 - Engine Drain):** Gọi `self.execution_engine.take_last_closed_trade()`.
     - Nếu có `ClosedTrade` từ `PaperTrader` $\implies$ Dùng trực tiếp `is_net_win = trade.net_pnl > 0.0`.
   * **Nguồn 2 (Dự phòng số 2 - Snapshot Fallback cho Live Binance):**
     - Nếu Engine không trả về struct (như WebSocket `PositionUpdate` trên sàn thật), tự động tính từ `previous_position`:
       $$\text{RawReturn} = \begin{cases} \frac{\text{Exit} - \text{Entry}}{\text{Entry}} & \text{nếu Long} \\ \frac{\text{Entry} - \text{Exit}}{\text{Entry}} & \text{nếu Short} \end{cases}$$
       $$\text{NetReturn} = \text{RawReturn} - (2 \times \text{CommissionRate})$$
       $$\text{is\_net\_win} = \text{NetReturn} > 0.0$$
3. **Bước 3 (Cập Nhật Cầu Dao & Giải Phóng DNA Mới):**
   * Nếu `is_net_win == true`:
     - $\text{consecutive\_losses} = 0$, $\text{locked\_until\_bar} = 0$.
     - Ghi log: `🎉 [COOLDOWN FULL RESET - WINNING TRADE]`.
   * Nếu `is_net_win == false`:
     - $\text{consecutive\_losses} += 1$.
     - Tính phạt lũy tiến: $\text{PenaltyBars} = \text{base\_bars} \times 2^{\min(\text{streak} - 1, 2)}$.
     - Trần động: $\text{MaxCapBars} = \text{base\_bars} \times 4$.
     - $\text{locked\_until\_bar} = \text{current\_bar\_idx} + \min(\text{PenaltyBars}, \text{MaxCapBars})$.
     - Ghi log: `🛑 [CIRCUIT BREAKER ACTIVATED]`.
   * **Tự động áp dụng Pending DNA (nếu có):**
     - Lệnh vừa đóng xong $\implies$ Bung ngay `self.pending_dna.take()` áp dụng vào Strategy và Config cho các lệnh tương lai!


