# 🚨 SỔ TAY GHI NHẬN LỖI THIẾT KẾ & LOGIC (ERRORS.MD) - HELIOS

---

## 📅 Lỗi Ngày 2026-09-17 / 2026-09-18

### ❌ Lỗi #001: Scanner `ANOMALY` Cố Chấp Bắt Dao Rơi Do Hoàn Toàn Mù Xu Hướng (Trend-Blindness)
- **Hiện tượng:** Trong đợt sập mạnh của ETH từ $2426 về $2371, `AnomalyScanner` spam 12 lệnh Long liên tiếp, gây ra 6 lệnh Hard SL (-16% ROE mỗi lệnh), hòa 4 lệnh BE và làm bốc hơi -$12.16.
- **Nguyên nhân gốc rễ:** 
  1. `AnomalyScanner` chỉ nhìn vào $Z\text{-score}$ giá lệch chuẩn và độ dài râu nến đơn lẻ ($f_{\text{wick}}$). Trong sóng sập mạnh, $Z\text{-score}$ luôn âm sâu, khiến điểm số luôn đạt điều kiện vào lệnh.
  2. Scanner này hoàn toàn không kiểm tra độ dốc KAMA hay xu hướng vĩ mô (Macro Trend), ngộ nhận nhịp rút chân vi mô 5 phút thành đáy đảo chiều.
- **Hệ quả:** Biến một con sóng Downtrend lớn thành "nghĩa địa" cho tài khoản vì liên tục cản đầu xe tăng.
- **Giải pháp khắc phục:** 
  - Hoặc tắt hẳn `ANOMALY` khỏi Arbiter và chuyển sự tập trung sang `REVERSAL` (vốn có đo lường Volume kiệt sức).
  - Hoặc bắt buộc bổ sung bộ lọc xu hướng vĩ mô: Nếu $\text{kama\_slope\_atr} < -\text{thresh}$, cấm tuyệt đối mở Long.

---

### ❌ Lỗi #002: Cooldown Tracker Bị "Mất Trí Nhớ Chuỗi Thua" (Losing Streak Amnesia)
- **Hiện tượng:** Cùng một Scanner sau khi bị dính Hard SL chỉ bị cách ly một khoảng thời gian cố định ngắn (~30 nến). Ngay khi hết 30 nến, Scanner đó lại lập tức nhảy vào mở lệnh cùng chiều và tiếp tục bị cắt lỗ.
- **Nguyên nhân gốc rễ:**
  1. `CooldownTracker` chỉ có thời gian khóa cố định tuyến tính (`locked_until_bar = i + re_entry_cooldown_bars`), không có bộ nhớ đếm số lần thua liên tiếp (`consecutive_losses`).
  2. Không có cơ chế phạt lũy tiến (Exponential Backoff) để kéo dài thời gian cách ly khi thị trường đang rơi vào pha xấu đối với chiến thuật đó.
- **Giải pháp khắc phục:** Bổ sung `consecutive_losses` và công thức phạt lũy tiến $2^{(\text{losses}-1)}$, tự động reset về 0 khi có lệnh thắng và có trần `max_cooldown_cap` chống khóa vô hạn.

---

### ❌ Lỗi #003: `ANOMALY` và `RANGE_OSC` Chưa Được Tắt Khỏi Arbiter
- **Hiện tượng:** Dù đã có định hướng chỉ tập trung vào 4 Playbook chủ lực, nhưng trong `core_v3/mod.rs`, cả 6 Scanner vẫn được gọi `arbiter.add_scanner(...)`.
- **Hệ quả:** `ANOMALY` với điều kiện vào lệnh dễ dãi đã chiếm tới 77.3% tổng số lệnh, làm sai lệch bức tranh kiểm thử thực tế của các Playbook khác.
- **Giải pháp khắc phục:** Điều chỉnh đăng ký scanner trong `core_v3/mod.rs` đồng bộ với định hướng kiến trúc.

---

### ❌ Lỗi #004: Thiếu Định Lượng Telemetry & Heartbeat khi Gatekeeper Veto Flatline/Chaos 65.4 Giờ
- **Hiện tượng:** ETHUSDT trong bài test Walk-Forward 5 ngày OOS chỉ thực hiện giao dịch trong 2 ngày, còn 65.4 giờ bị rơi vào trạng thái Volatility Flatline. Log trước đó chỉ in một dòng chuyển trạng thái ban đầu, thiếu định lượng cụ thể tại sao Flatline (giá trị NATR thực tế vs Fee Floor vs DNA Alpha) và thiếu nhịp đập Heartbeat khiến việc audit OOS thiếu dữ kiện.
- **Nguyên nhân gốc rễ:**
  1. `GateEvaluation` không lưu lại giá trị `required_min_natr_pct`, `fee_floor_pct`, `min_alpha_pct`, `chaos_threshold`.
  2. `DefenseLedger` chỉ ghi nhận edge-trigger (vào/ra bão), khi bão kéo dài nhiều ngày thì hoàn toàn im lặng (silence trap).
  3. Thiếu công cụ trực quan hóa (Visual Report) để soi ~7,200 nến OOS, vị trí vào/thoát lệnh, dải bảo vệ Flatline/Chaos và đường cong vốn.
- **Giải pháp đã thực hiện (Phương án C):**
  1. Nâng cấp `GateEvaluation` mang đầy đủ 4 trường định lượng của Gatekeeper (`gatekeeper.rs`).
  2. Bổ sung `check_hourly_heartbeat` (phát log mỗi 60 nến = 1 giờ) hiển thị `[MARKET HEARTBEAT]` kèm giá và thông số NATR/Chaos thực tế (`defense_ledger.rs`).
  3. Xây dựng module `visual_report.rs` xuất file `report_oos.html` tương tác (TradingView Lightweight Charts v4.2 canvas 60fps) cắt lát đúng ~7,200 nến OOS, hiển thị candlestick, markers trade, dải shading Flatline/Chaos, và đường cong vốn Equity Subplot, tự động sinh vào thư mục log trước khi dọn dẹp storage trong `walk_forward_smart_seed.rs`.

---

### ❌ Lỗi #005: Bẫy Break-Even Tự Sát & Khối Lệnh Bán Tháo Hoảng Loạn `Emergency BE Rejection Close` Trong `risk.rs` (TODO-SEM-03)
- **Hiện tượng:** Khi giá chỉ tăng nhẹ $+0.02\%$, cơ chế Break-Even đã kích hoạt, dời SL lên Entry; sau đó khi nến dao động nhẹ chạm dưới Entry, bot thực hiện `tick.close(None)` bán tháo Market, khiến vị thế phải trả $0.10\%$ phí sàn 2 chiều (âm $-2.0\%$ ROE tại đòn bẩy 20x).
- **Nguyên nhân gốc rễ:**
  1. *Kích hoạt sớm dưới sàn phí:* Điều kiện `peak_profit >= current_atr * be_trigger_mult` kích hoạt khi giá chỉ tăng $+0.02\%$ trên nến 1m, trong khi phí sàn 2 chiều tối thiểu là $0.10\%$ ($2.0\%$ ROE tại đòn bẩy 20x).
  2. *Panic Market Close:* Khối lệnh `if current_close <= min_be { tick.close(None); }` thực thi lệnh Market đóng vị thế khẩn cấp ngay giữa thanh nến, cướp quyền khớp lệnh Limit/Stop Market trên sàn và ép vị thế chịu trượt giá cùng phí Taker.
  3. *Lệch ngữ nghĩa (Semantic Mismatch):* So sánh khập khiễng giữa biến số khoảng cách giá tuyệt đối và ROE đòn bẩy, bỏ qua chi phí sàn thực tế.
  4. *Ký sinh TTP:* Ghép nối điều kiện `is_be_triggered || peak_roe >= ttp_act_roe` khiến Trailing Take Profit bị kích hoạt sớm khi lợi nhuận chưa đủ lớn, vô hiệu hóa khả năng gồng lãi sóng dài.
  5. *Xung đột với Monotonic Ratchet Guard (`TODO-RSK-02`):* Khi giá retest về Entry khiến `current_pnl <= 0.0`, Guard chặn không cho cập nhật SL hòa vốn lên sàn Binance vì nhầm lẫn với vị thế đang chịu lỗ ngay từ đầu.
- **Giải pháp đã thực hiện:**
  1. *Sàn kinh tế kép (Dual Economic Floor):* Bắt buộc $\text{peak\_roe} \ge \max(2.0 \times \text{round\_trip\_fee\_roe},\ 6.0\%)$ VÀ $\text{peak\_dist} \ge \max(\text{Entry} \times 2.0 \times \text{round\_trip\_fee},\ \text{ATR} \times \text{be\_trigger\_mult})$.
  2. *Tách biệt hoàn toàn TTP:* `is_ttp_active = peak_roe >= ttp_act_roe;` (loại bỏ cờ BE khỏi TTP).
  3. *Triệt tiêu Panic Close & Đệm phí an toàn:* Xóa bỏ hoàn toàn `tick.close(None)`; tính toán mức SL hòa vốn `min_be` có đệm bù phí sàn và vi mô lợi nhuận $+0.03\%$ giá (`micro_cushion`), bảo vệ vị thế qua cơ chế Algo Stop Market đám mây.
  4. *Khai thông Ratchet Guard:* Cho phép dời SL khi `current_pnl > 0.0 || is_be_triggered`, bảo toàn tuyệt đối chốt chặn vị thế âm ban đầu nhưng không cản trở việc khóa hòa vốn.
  5. *Kiểm thử 100% Zero-Regression:* 3 unit tests chuyên sâu bảo vệ logic sàn kinh tế, tính toán SL và tính độc lập của TTP.

---

### ❌ Lỗi #006: Bẫy Mù NATR 4h Của Cầu Dao Flatline & Sự Pha Tạp DNA Vi Mô (TODO-AUDIT-NEW-24 / NEW-NATR-4H-BLIND)
- **Hiện tượng:** Khi GA Optimizer lựa chọn `ind.natr.timeframe = Timeframe::T4h`, giá trị NATR khung 4h ($\text{NATR}_{4\text{h}} \approx 0.80\% \dots 2.50\%$) luôn vượt xa sàn chi phí sinh tồn ($0.22\%$). Do đó, Cầu Dao Flatline bị **MÙ HOÀN TOÀN** trước các đợt nến vi mô rơi vào trạng thái que tăm / đóng băng thanh khoản ($< 0.02\%$), khiến bot vẫn phê duyệt mở lệnh trong điều kiện thị trường không thể hấp thụ lệnh hoặc spread/slippage bào mòn tài khoản.
- **Nguyên nhân gốc rễ:**
  1. *Lệch ngữ nghĩa thang đo (Temporal Scale Mismatch):* Đem chỉ số biến động sóng lớn 4h làm thước đo độ thanh khoản tức thời cho việc khớp lệnh tại nến 1m.
  2. *Thiếu độc lập dòng dữ liệu (Data Lineage Violation):* Không có cấu hình DNA và cảm biến riêng biệt cho biến động vi mô, phụ thuộc ngầm vào `ind.natr` (vốn được thiết kế để đo biên độ vĩ mô và tính `chaos_ratio`).
  3. *Bẫy VETO nhầm nến nén tích lũy (TODO-SEM-01):* Nếu dùng sàn $0.22\%$ so với nến vi mô 1m thì lại Veto oan $95\%$ các cây nến bình thường ($0.06\% \dots 0.15\%$) và nến nén tích lũy chuẩn bị bứt phá ($0.05\%$).
- **Giải pháp đã thực hiện (Phương Án 1 Chốt Chuẩn Quant):**
  1. *Cổng Pre-Trade Hợp Nhất 2 Tầng (Unified Two-Tier Gateway):*
     - Tầng 1 (Vĩ mô): $\text{macro\_natr} \ge \text{required\_min\_natr\_pct} = \max(\text{fee\_floor\_pct},\ \text{alpha\_pct})$ (sàn $0.22\%$). Veto lý do: `GateVetoReason::MacroFlatline`.
     - Tầng 2 (Vi mô cơ học): $\text{micro\_natr} \ge \text{micro\_fee\_floor\_pct} = \text{taker\_fee\_rate} \times 100.0 = 0.04\%$ (phí Taker 1 chiều). Veto lý do: `GateVetoReason::MicroFlatline`.
     - Tầng 3 (Bão Chaos): $\text{chaos\_index} \le \text{chaos\_veto\_threshold}$ kèm Hysteresis Schmitt Trigger.
  2. *Struct DNA Vi Mô Độc Lập:* Tạo `MicroNAtrParams` (`timeframe: subset = "1m, 3m, 5m"`, `period: min = 5.0, max = 30.0, step = 1.0`, default: `14, Timeframe::T1m`). Tăng DNA_WIDTH từ 102 lên 104 chiều.
  3. *Tách bạch Telemetry & Defense Ledger:* Ghi nhận riêng biệt `MacroFlatline` và `MicroFlatline` trong telemetry và hourly heartbeat.
  4. *Kiểm thử 100% Zero-Regression:* 11/11 tests trong gatekeeper & telemetry pass; 104/104 tests toàn bộ `helios` pass; 50/50 tests `ferro_trade` pass; 46/46 tests `bot_engine` pass.

---

### ❌ Lỗi #007: Thảm Họa High Watermark Buffer Bleed Khi Lệnh Trôi Khỏi Buffer (TODO-AUDIT-NEW-12 / TODO-BRG-01)
- **Hiện tượng:** Khi một vị thế chạy dài ngày (ví dụ >30 ngày = 43,200 nến 1m) và nến vào lệnh trôi ra khỏi mép trái của Rolling Buffer trong RAM (`pos.entry_timestamp < candles[0].timestamp`):
  1. `helios_bridge.rs` buộc phải gán ép `mapped_entry_bar = 0`.
  2. `risk.rs` quét lịch sử nến `for j in (entry_bar + 1)..=i` (tức từ index 1 đến cuối buffer).
  3. Đỉnh lợi nhuận trong quá khứ trước buffer bị xóa sổ hoàn toàn (Buffer Bleed).
  4. Nếu giá hiện tại thấp hơn đỉnh cũ nhưng cao hơn đáy cục bộ của buffer, `decay_cycles` bùng nổ siết Trailing SL về sát sàn hoặc sập SL về Entry/đáy mới một cách oan uổng.
  5. Đồng thời, `bars_held` bị bóp nghẹt thành số nến trong buffer thay vì thời gian thực tế, khiến Cầu dao giải phóng vốn Zombie (`stagnant_bars`) bị tê liệt hoàn toàn.
- **Nguyên nhân gốc rễ:**
  1. *Thiếu cấu trúc tích lũy trạng thái (Missing State Accumulator):* Cả `Trade` (ferro_trade) và `LivePositionState` (bot_engine) chỉ lưu các giá trị ban đầu (entry_price, entry_bar/timestamp) mà không lưu trạng thái đơn điệu của đỉnh lợi nhuận qua thời gian.
  2. *Thuật toán quét lịch sử $O(N)$ phụ thuộc lát cắt bộ nhớ:* Đẩy trách nhiệm tìm đỉnh cho vòng lặp duyệt nến trong `RiskManager`, vi phạm tính độc lập với không gian bộ nhớ.
  3. *Lệch ngữ nghĩa thời gian nến và thời gian vật lý:* Đếm số nến lát cắt thay vì tính khoảng cách thời gian vật lý $\Delta t$ chuẩn xác.
- **Giải pháp đã thực hiện (Phương Án B+ State Accumulator Đơn Điệu):**
  1. *Mở rộng State Accumulator $O(1)$:* Bổ sung `highest_favorable_price` và `peak_timestamp` (kèm `peak_bar` trong backtest) vào `Trade` (`ferro_trade`), `LivePositionState`, `PaperTrader` và `LiveEngine` (`bot_engine`).
  2. *Cập nhật đỉnh liên tục $O(1)$:* 
     - Trong `Broker::next`: Tự động cập nhật `update_peak` cho tất cả lệnh đang mở tại mỗi bước nến (chống Lookahead bias).
     - Trong `LiveEngine` & `PaperTrader`: Cập nhật đỉnh theo Mark Price và nến thời gian thực.
  3. *Chuyển tiếp đầy đủ qua Cầu nối (`helios_bridge.rs`):* Chuyển tiếp `highest_favorable_price`, `peak_timestamp` và `entry_time_opt` vào `Trade` giả lập của `TickContext`.
  4. *Tính toán $O(1)$ & Triệt tiêu Vòng lặp Nến Trong `risk.rs`:*
     - Xóa sổ hoàn toàn vòng lặp quét slice `for j in (entry_bar + 1)..=i`.
     - Lấy trực tiếp `peak_val = trade.highest_favorable_price`.
     - Tính `effective_stagnant_bars` và `bars_held` dựa trên khoảng cách thời gian vật lý: $\frac{t_{\text{current}} - t_{\text{peak}}}{\Delta t_{\text{candle}}}$ và $\frac{t_{\text{current}} - t_{\text{entry}}}{\Delta t_{\text{candle}}}$.
  5. *Động hóa $\Delta t$:* Bổ sung `TickContext::candle_interval_ms()` để tự động nội suy chu kỳ nến từ `DataView`, triệt tiêu hoàn toàn magic number 60,000 ms.
  6. *Kiểm thử 100% Zero-Regression:* 
     - 50/50 tests `ferro_trade` pass.
     - 30/30 unit tests `helios::risk` pass (0.00s latency).
     - 6/6 tests `trailing_sl_test` pass, kiểm chứng thực nghiệm bảo toàn nguyên vẹn đỉnh $1850 khi nến trôi khỏi buffer và giải phóng vốn Zombie 3,000 nến chuẩn xác.

---

### ❌ Lỗi #008: Bẫy Catch-22 VETO Triệt Tiêu Nến Đẹp Của Arbiter (TODO-AUDIT-NEW-04 / TODO-AUDIT-NEW-23 / CATCH-22-STRUCTURAL-SL)
- **Hiện tượng:** Khi Tactical Scanner phát hiện một mẫu hình nến đảo chiều hoặc pullback hoàn hảo nhưng có râu nến/swing biên độ rộng (khoảng cách Stop Loss cấu trúc $\text{ROE} > \text{playbook\_max\_sl\_roe}$, ví dụ $18\% > 15\%$ tại đòn bẩy $20\text{x}$):
  1. Arbiter lập tức kích hoạt phán quyết nhị phân cứng: `passed = false` và gán nhãn `AuctionStatus::StructuralRiskVeto`.
  2. Toàn bộ proposal có điểm tự tin cực cao bị ném vào sọt rác, bị loại bỏ khỏi cuộc đấu thầu tín hiệu Two-Stage Auction.
  3. Hậu quả trực tiếp: Khối mã kẹp SL tự động `RiskManager::clamp_sl_price` và cơ chế ghi sổ cái phòng vệ `defense_ledger.on_sl_clamped` tại [`mod.rs:332-370`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L332-L370) bị biến thành **$100\%$ Dead Code** (không bao giờ có cơ hội được thực thi khi gặp nến có SL rộng). Bot bị "đói lệnh" và bỏ lỡ hàng loạt setup đẹp ngoài thị trường.
- **Nguyên nhân gốc rễ:**
  1. *Vi phạm nguyên tắc phân định trách nhiệm (Separation of Concerns Violation):* Arbiter là bộ phận chấm điểm tự tin và phân xử xung đột tín hiệu (Confidence Auction), nhưng lại tự tiện áp đặt phán quyết nhị phân về rủi ro cắt lỗ để hủy lệnh.
  2. *Mâu thuẫn kiến trúc nội tại (Catch-22 Paradox):* Tầng trên (`arbiter.rs`) giết lệnh vì SL $> 15\%$, trong khi tầng dưới (`mod.rs`) đã dày công thiết kế sẵn thuật toán `clamp_sl_price` để tự động kẹp SL về trần $15\%$. Hai cơ chế triệt tiêu lẫn nhau.
  3. *Không phân biệt giữa Rủi ro Cấu trúc (Structural Risk) và Rủi ro Cháy sàn (Liquidation Risk):* SL cấu trúc $18\%$ ROE tương đương $0.90\%$ biến động giá ở $20\text{x}$, nằm an toàn tuyệt đối bên trong khoảng cách cháy sàn Binance MMR ($\approx 4.42\%$ giá), nhưng lại bị Arbiter đối xử như một lệnh tự sát sàn (Kamikaze).
- **Giải pháp đã thực hiện (Phương Án A: Auto-Clamping Pass-Through):**
  1. *Phá bỏ phán quyết nhị phân tại Arbiter:* Trong `arbiter.rs`, khi gặp `SlValidationResult::StructuralRiskVeto`, nếu `confidence >= min_threshold`, Arbiter phê duyệt `passed = true` và đưa proposal vào `approved_proposals` để tham gia đấu thầu Two-Stage bình thường.
  2. *Bảo toàn 100% Chốt Chặn Cháy Sàn Binance (Liquidation Guard):* Giữ nguyên lệnh VETO hủy bỏ đối với `SlValidationResult::LiquidationRiskVeto` (bảo vệ chống cháy tài khoản trước khi chạm SL) và `SlValidationResult::InvalidPrice` (giá NaN, $\le 0.0$).
  3. *Ủy thác Kẹp SL Xuống Tầng Thực Thi:* Arbiter giữ nguyên `suggested_sl` thô truyền xuống `decision.final_sl`. Tại `mod.rs:332`, hàm `RiskManager::clamp_sl_price` tự động kẹp SL về trần an toàn của playbook (`playbook_max_sl_roe`) và ghi nhận vào `defense_ledger.on_sl_clamped`.
  4. *Bảo toàn Telemetry:* Khi proposal có SL rộng thắng thầu hoặc là runner-up, `AuctionCandidate.reason` ghi nhận rõ thông điệp `[Structural SL exceeds ROE cap (X% > Y%), passed to RiskManager for clamping]`.
  5. *Kiểm thử 100% Zero-Regression:*
     - 11/11 tests trong `arbiter::tests` pass (bao gồm test case phê duyệt SL 30% ROE chuyển giao cho RiskManager và test case VETO lệnh Kamikaze vi phạm thanh lý sàn).
     - 108/108 tests toàn bộ thư viện `helios` pass.
     - 46/46 tests `bot_engine` pass.

---

### ❌ Lỗi #009: Lệch Pha Cắt Lát Buffer & Nến Cụt MTF Khởi Tạo (TODO-AUDIT-NEW-15 / TODO-ACT-04 / SLICE-WARMUP-MISMATCH)
- **Hiện tượng:** 
  1. Trong `AssetActor::get_live_candles_slice`, buffer RAM bị cắt lát cứng xuống 15,000 nến (`max_live_mem_candles`). Cây nến đầu tiên sau lát cắt rơi vào một phút ngẫu nhiên không chia hết cho 240m (4h), dẫn đến cây nến 4h đầu tiên bị cắt cụt (truncated candle) làm biến dạng chỉ báo MTF Resampling.
  2. Sự mất đồng bộ giữa GA In-Sample Window (40 ngày = 57,600 nến) và Actor Buffer (30 ngày hoặc 15,000 nến) khiến GA tối ưu một tập dữ liệu, còn Actor vận hành trên một tập dữ liệu hoàn toàn khác, gây sai số khởi tạo chỉ báo (Warmup Initialization Drift).
- **Nguyên nhân gốc rễ:**
  1. *Đề xuất ban đầu sai bản chất:* Ý định hardcode 4h UTC (`timestamp % 14_400_000 == 0`) vi phạm nguyên tắc trừu tượng phân tầng (Leaky Abstraction) và xung đột với kiến trúc độc lập khung thời gian `TODO-AUDIT-NEW-159` ($T_{micro}, T_{meso}, T_{macro}$).
  2. *Thiếu Single Source of Truth (SSOT) về thời gian:* `Asset.candles` bị hardcode trần dung lượng 43,200 nến trong `models.rs`, `sync_history` hardcode 30 ngày, trong khi các hằng số ngày cấu hình phân mảnh ở nhiều nơi.
- **Giải pháp đã thực hiện (Kiến Trúc Dẫn Xuất Động Theo Số Ngày - Dynamic Days-Driven Engine):**
  1. *Single Source of Truth:* Thêm hàm khởi tạo `EngineConfig::for_days(days: u64)` tự động tính $\text{Candles} = \text{days} \times 1,440$ nến 1m và `historical_sync_days = days`.
  2. *Động hóa `Asset`:* Mở rộng `Asset` với trường `pub max_capacity: usize`, bổ sung constructor `new_with_capacity(symbol, capacity)` và tự động cập nhật dung lượng khi gọi `with_config` hoặc `with_max_mem_candles`.
  3. *Đồng bộ `sync_history`:* Nạp dữ liệu lịch sử từ SQLite theo đúng `self.config.historical_sync_days`, triệt tiêu hardcode 30 ngày.
  4. *Đồng bộ 1:1 trong `walk_forward_smart_seed.rs`:* Cấu hình Actor bằng `EngineConfig::for_days(TRAIN_WINDOW_DAYS as u64)` (40 ngày = 57,600 nến). Khi đó `slice_start = 0`, GA và Actor nhìn thấy tập dữ liệu 1:1, nến cụt và sai số khởi tạo tự động biến mất 100%.
  5. *Chuẩn hóa Mock Candles:* Loại bỏ data artifact trong `create_mock_candles` (cập nhật `last_p = p` liên tục và làm mượt warmup candles kết thúc tại `start_price`).
  6. *Kiểm thử 100% Zero-Regression:*
     - 53/53 tests `bot_engine::lib` pass.
     - 6/6 tests `trailing_sl_test` pass.
     - `cargo check` trên cả `helios`, `ferro_trade`, và ví dụ `walk_forward_smart_seed` đều pass 100%.

---

### ❌ Lỗi #010: Bỏ Quên Hard SL Ban Đầu Khi Mở Lệnh Live & Nguy Cơ Vị Thế Trần Trụi (TODO-AUDIT-NEW-10 / BUG-EXEC-NEW-01)
- **Hiện tượng:** Khi chiến lược Helios V3 phát sinh tín hiệu mở vị thế mới (`Action::Buy` / `Action::Sell`), bot trên sàn Binance Futures Live/Testnet mở Market Order thành công nhưng **hoàn toàn bỏ qua việc đặt lệnh Algo `STOP_MARKET` bảo vệ ban đầu**. Vị thế bị thả nổi trần trụi (Naked Position) trong suốt thời gian của cây nến đầu tiên ($\ge 60$s) hoặc cho tới khi có nhịp dời Trailing SL. Nếu xảy ra râu nến quét thanh khoản hoặc sập giá bất ngờ, tài khoản phải đối mặt với nguy cơ thanh lý cưỡng bức (Liquidation).
- **Nguyên nhân gốc rễ:**
  1. *Lệch pha mô hình dữ liệu (Data Contract Mismatch):* Helios V3 đóng gói giá dừng lỗ ban đầu vào `policy: Some(RiskPolicy::FixedPrice { sl_price, .. })` và gán `stop = None`. Tuy nhiên, `OrderExecutor` và `LiveEngine` chỉ đọc biến `sl_price_to_set = stop;`, dẫn tới `sl_price_to_set` luôn luôn là `None`.
  2. *Bẫy tranh chấp luồng mạng (Network Race Condition):* Khi Market Order khớp trên sàn, sự kiện WebSocket `PositionUpdate` thường đến máy khách trước khi chu trình đặt lệnh Algo SL hoàn tất và ghi vào RAM `active_sl`. `LiveEngine` rơi vào trạng thái "mù" SL hoặc phải gọi REST API `sync_active_algo_orders` làm tốn API Weight vô ích.
  3. *Thiếu cơ chế phòng vệ chốt chặn cuối (Absence of Panic Defense):* Cơ chế đặt SL chỉ có 3 lần retry sơ sài (20ms) và nếu thất bại hoàn toàn thì chỉ in log lỗi `error!` rồi bỏ mặc vị thế mở trần trụi ngoài thị trường, không có phản xạ thoát lệnh khẩn cấp.
- **Giải pháp đã thực hiện:**
  1. *Hàm trích xuất chuẩn hóa:* Xây dựng pure helper function `OrderExecutor::extract_sl_price(action: &Action) -> Option<f64>` ưu tiên bóc tách `sl_price` từ `RiskPolicy::FixedPrice`, fallback sang `stop` và `new_sl_price` của `UpdateTradeSlTp`.
  2. *Bộ đệm bảo vệ đón đầu (`pending_initial_sl`):* Mở rộng `LiveEngine` với trường `pending_initial_sl`, lưu giá SL ngay khi nhận `Action` từ Strategy trước khi gửi lệnh. Khi `PositionUpdate` đến, `LiveEngine` ưu tiên gán ngay `pending_initial_sl` vào `live_position.current_sl_price` mà không cần chờ hay gọi REST API, và tự động dọn sạch khi vị thế flat (`position_amt == 0.0`).
  3. *Kháng nhiễu mạng 7 tầng & Tự hủy trùng lệnh:* Nâng cấp vòng lặp đặt Hard SL qua WebSocket lên 7 lần retry với exponential backoff (`[10, 20, 40, 80, 160, 320, 500]ms`); tự động bắt mã lỗi trùng lệnh `-4061` để đồng bộ và hủy lệnh Algo cũ trước khi retry.
  4. *Cơ chế 2-Tier Emergency Panic Close:* Nếu sau cả 7 lần retry vẫn thất bại, bot lập tức kích hoạt Emergency Market Close đảo chiều qua WebSocket Trade API (kèm Last-Resort REST API fallback) để đóng ngay vị thế, bảo đảm không bao giờ để Naked Position tồn tại ngoài thị trường.
  5. *Kiểm thử 100% Zero-Regression:*
     - Viết mới `test_extract_sl_price_from_action` và `test_live_engine_initial_sl_from_policy`.
     - 57/57 tests `bot_engine::lib` pass 100%.
     - 117/117 tests `helios::lib` pass 100%.
     - 51/51 tests `ferro_trade::lib` pass 100%.

---

### ❌ Lỗi #011: Vùng Chết (Deadzone) & Sàn Bão Hòa Cực Đoan Trong Stochastic Hook Guard Của Trend_PB (TODO-PB-04 / TODO-AUDIT-NEW-39)
- **Hiện tượng:**
  1. *Vùng Chết (Deadzone) $[climax\_thresh, stoch\_thresh]$:* Khi Stochastic nằm trong khoảng $[15.0, 30.0]$ và đang cắm đầu rơi tự do dốc đứng, do điều kiện kẹp `stoch_k < climax_thresh && is_falling`, scanner không chặn mà vẫn duyệt lệnh Long, biến Trend_PB thành hành vi bắt dao rơi liều mạng.
  2. *Sàn Bão Hòa Cực Đoan ($K = D = 0.0$ cho Long / $K = D = 100.0$ cho Short):* Khi thị trường bán tháo xả lũ cực mạnh đè bẹp Stochastic xuống sàn $0.0$, phép so sánh `stoch_k < prev_stoch_k` và `stoch_k < stoch_d` đều cho kết quả `false`, dẫn đến `is_falling = false` và bot nhảy vào mua ngay tại tâm bão xả lũ.
  3. *Lỗ hổng NaN Warmup:* `sensors.stoch_trend_d` chưa lọc giá trị `NaN` trong giai đoạn khởi động chỉ báo.
- **Nguyên nhân gốc rễ:**
  1. Giả định ngầm sai lầm rằng "chỉ khi rơi xuống vùng xả lũ cực đoan ($< climax\_thresh$) mới cần chặn rơi tự do". Trong thực tế, bất kỳ nhịp hồi nào đang rơi dốc đứng ở mức 20 hay 25 cũng đều chưa hình thành điểm uốn đảo chiều.
  2. Bỏ quên trường hợp suy biến bão hòa số học biên tại $0.0$ và $100.0$ của bộ dao động $[0, 100]$.
- **Giải pháp đã thực hiện (Two-Tier Stochastic Hook Guard):**
  1. *Khử sàn/trần bão hòa:* Cưỡng chế $is\_falling = true$ khi $K \le 1.0 \land D \le 1.0$; $is\_surging = true$ khi $K \ge 99.0 \land D \ge 99.0$.
  2. *Vùng Climax ($K < climax\_thresh$):* Đòi hỏi xác nhận kép (không rơi $K \ge prev\_K$ VÀ không nằm dưới $D$: $K \ge D$). Chặn đứng 100% Bull Trap khi $K$ giật nhẹ nhưng vẫn nằm dưới $D$.
  3. *Vùng Pullback thường ($climax \le K \le stoch\_thresh$):* Triệt tiêu hoàn toàn Vùng Chết, chặn đứng khi nến đang rơi dốc ($K < prev$) hoặc nến phẳng đang chúi xuống ($K == prev \land K < D$).
  4. *Sanitize NaN:* Thêm `.filter(|v| !v.is_nan()).unwrap_or(stoch_k)`.
  5. *Kiểm thử 100% Zero-Regression:*
     - Viết mới 3 tests: `test_trend_pb_hook_guard_blocks_zero_and_hundred_saturation`, `test_trend_pb_hook_guard_blocks_deadzone_falling`, `test_trend_pb_hook_guard_climax_dual_confirmation`.
     - 17/17 tests `trend_pb` pass 100%.
     - `test_dna_width_and_parameter_defs_integrity` pass 100% (bảo toàn 107 chiều DNA).

---

### ❌ Lỗi #012: Bẫy Đóng Lệnh Non Whipsaw Trên Nến 1m Do Nhầm Lẫn Giữa Micro Retest & Macro Structure Break (TODO-AUDIT-NEW-184 / TREND-MSB-EXIT)
- **Hiện tượng & Giả định ngầm ban đầu:**
  - Đề xuất task `TODO-AUDIT-NEW-184` yêu cầu kích hoạt cắt lỗ khẩn cấp bằng lệnh Market khi nến 1m đóng cửa $Close[i] < Low[\text{entry\_bar}]$ (Long) hoặc $Close[i] > High[\text{entry\_bar}]$ (Short) vì cho rằng "đáy nến vào lệnh là Higher Low bất khả xâm phạm, nếu nến sau đóng thủng thì xu hướng tăng đã tử trận".
- **Phân tích bản chất vi cấu trúc & Nguyên nhân bị bác bỏ (Falsified):**
  1. *Nhiễu vi mô 1m (High Noise):* Nến 1m có tỷ số tín hiệu trên nhiễu (SNR) cực thấp. Nhịp thoái lui thực tế của thị trường là một cụm $3 \sim 8$ cây nến. Đáy của cây nến vào lệnh chỉ là đáy vi mô cục bộ, không phải True Swing Low của toàn bộ xu hướng.
  2. *Bẫy mô hình 2 đáy & Phá hủy kỳ vọng ($EV$):* Thị trường thường xuyên tạo mô hình Double Bottom hoặc Spring test (Wyckoff) nhúng nhẹ qua đáy nến trước để vét nốt thanh khoản trước khi đảo chiều tăng thật. Nếu chém lệnh ngay tại $Close[i] < Low[\text{entry\_bar}]$, hệ thống sẽ cắt non ngay tại chân sóng, biến các lệnh thắng tiềm năng $+20\%$ ROE thành các khoản lỗ nhỏ $-3\%$ ROE liên tiếp kèm phí sàn, làm sụp đổ hoàn toàn kỳ vọng toán học ($EV$) của chiến lược.
  3. *Phát hiện lệch pha DNA & Clamp ngầm:* Trong quá trình thẩm định Initial SL, phát hiện `config.rs` cấu hình DNA `struct_sl_buffer_atr` từ $[0.10, 3.00]\ \text{ATR}$ nhưng trong `trend_pb.rs` lại bị clamp cứng ở `1.20` (lãng phí 60% không gian tìm kiếm của GA Optimizer).
- **Giải pháp chuẩn hóa đã thực hiện:**
  1. *Bác bỏ cơ chế chém non nến đơn lẻ:* Khẳng định $\text{Initial Structural SL} = Low[\text{entry\_bar}] - \text{buffer ATR}$ đã là một chốt chặn bảo vệ cấu trúc hoàn chỉnh trên sàn (được bảo hiểm trần ở $15\%$ ROE bởi `RiskManager::clamp_sl_price`), và `Tiered Profit Ratchet` (Nấc 3) bảo vệ khi có lãi.
  2. *Chuẩn hóa đồng bộ 1:1 giữa DNA và Clamp:* Nới clamp trong `trend_pb.rs` từ `1.20` lên `2.00` và thắt chặt DNA trong `config.rs` về $[0.10, 2.00]\ \text{ATR}$ (step $0.10 \implies 20$ nấc gene thực chất), đồng bộ chuẩn mực với các Scanner lân cận (`Momentum`: $[0.20, 2.00]$, `Absorption`: $[0.10, 2.50]$).
  3. *Kiểm thử 100% Zero-Regression:*
     - Viết mới `test_trend_pb_sl_buffer_supports_up_to_2_0_atr` xác minh đệm $2.0\ \text{ATR}$ không còn bị nghẽn.
     - 180/180 tests `helios::lib` pass 100%.
