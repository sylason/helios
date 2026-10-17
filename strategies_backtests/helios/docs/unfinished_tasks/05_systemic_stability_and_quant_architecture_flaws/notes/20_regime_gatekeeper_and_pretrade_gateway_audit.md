# KIỂM TOÁN KIẾN TRÚC BỘ LỌC CỔNG VÀO CHIẾN LƯỢC & CHỐT CHẶN TIỀN GIAO DỊCH (REGIME GATEKEEPER & STRATEGY GATEWAY AUDIT)

> **Mã tài liệu:** `NOTE-20-GATEKEEPER-GATEWAY-AUDIT`  
> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Phạm vi kiểm toán:** `gatekeeper.rs`, `arbiter.rs`, `risk.rs`, `core_v3/mod.rs`, `sensors.rs`, `config.rs`, `dna_mapping.rs`  
> **Đối chiếu chéo:** `docs/audit_01/01_sensors/`, `15_semantic_mismatch_audit/`, `audit_01/ROADMAP.md`, `notes/12`, `notes/15`  
> **Mục tiêu:** Kiểm toán toàn diện cơ chế chốt chặn tiền giao dịch (Pre-trade Gateway), bẫy mù Flatline vi mô, hiện tượng ngáng chân chồng chéo với Arbiter/RiskManager và kiến trúc phân loại chế độ thị trường vĩ mô.

---

## 🧭 1. BẢNG ĐỐI CHIẾU TRẠNG THÁI TỔNG THỂ (STATUS MAPPING)

| Mã Lỗi / Hiện Tượng | Vị Trí Code (File & Dòng) | Hiện Trạng Thực Tế Trong Codebase | Phân Loại & Đánh Giá |
| :--- | :--- | :--- | :---: |
| **`TODO-SEM-01`**<br>(Bẫy Flatline Mù) | `core_v3/gatekeeper.rs`<br>`L24-32` | Đã bỏ phép so sánh nến vi mô 1m với phí 2 chiều; chuyển sang đọc `market_natr_pct` từ `sensors.natr` MTF ($15\text{m}/1\text{h}/4\text{h}$). | 🟢 **ĐÃ FIX TRONG CODE**<br>*(Đã gỡ Veto oan 95% nến)* |
| **`NEW-NATR-4H-BLIND`**<br>(Bẫy Mù NATR Khung Lớn) | `core_v3/gatekeeper.rs`<br>`L24-32` | Khi GA chọn `NAtrParams.timeframe = 4h`, `market_natr_pct` luôn $\ge 0.80\% - 2.50\% \implies$ Cầu dao Flatline bị mù hoàn toàn trước các phiên chết thanh khoản vi mô 1m/5m. | 🔴 **LỖI MỚI PHÁT HIỆN**<br>*(Mù thanh khoản vi mô)* |
| **`CATCH-22-STRUCTURAL-SL`**<br>(Nghịch lý Catch-22 Arbiter) | `core_v3/arbiter.rs`<br>`L254-280` | `validate_structural_sl` veto thẳng tay các proposal có SL $> 15\%$ ROE, biến cơ chế kẹp biên `clamp_sl_price` (`mod.rs:332-342`) thành **MÃ CHẾT (DEAD CODE)**! | 🔴 **CHƯA FIX - CHỒNG CHÉO**<br>*(Bóp chết 80% cơ hội)* |
| **`CHOP-DEAD-ZONE`**<br>(Vùng Chân Không Tê Liệt) | `scanners/trend_pb.rs:64`<br>`core_v3/mod.rs:48-53` | `Trend_PB` tự trói tay khi có Squeeze (`squeeze_intensity > max_sqz`) để nhường sân cho `Range_Osc`, nhưng `Range_Osc` lại đang bị comment out, tạo ra vùng tê liệt 0 lệnh. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Tê liệt khi thị trường tích lũy)* |
| **`NEW-DOUBLE-EVAL`**<br>(Đánh giá lặp lại 2 lần) | `core_v3/mod.rs:241`<br>`core_v3/arbiter.rs:169` | `RegimeGatekeeper::evaluate` bị gọi lặp lại 2 lần mỗi tick (lần 1 ở `mod.rs`, lần 2 ở `arbiter.rs`), lãng phí CPU và phân tán trách nhiệm. | 🟡 **LỖI MỚI PHÁT HIỆN**<br>*(Lãng phí tài nguyên CPU)* |
| **`TODO-H-05`**<br>(Macro Regime Classifier) | `core_v3/gatekeeper.rs`<br>`L1-60` | Gatekeeper hiện tại chỉ là công tắc ngắt cứng nhị phân ($O(1)$ Binary Kill-Switch), chưa phân loại được Trend vs Range vs Chaos để định hướng trọng số Playbooks. | 🟡 **CHƯA CÓ TRONG CODE**<br>*(Bảo lưu tính năng nâng cao)* |
| **`TODO-H-06`**<br>(MTF Phase Coherence) | `core_v3/gatekeeper.rs` | Không có chốt chặn đồng thuận pha trong Gatekeeper (đây là thiết kế chuẩn xác để tránh bóp chết điểm bứt phá chân sóng và chặn đứng Mean-Reversion). | 🟢 **ĐÃ PHÂN LẬP ĐÚNG CHỖ**<br>*(Phân bổ cục bộ từng Scanner)* |

---

## 🔬 2. MỔ XẺ CHI TIẾT CÁC LỖ HỔNG VÀ BẪY KIẾN TRÚC

### 2.1. Cầu Dao Flatline: Khắc Phục Bẫy Cũ Nhưng Xuất Hiện "Bẫy Mù Khung Lớn"
- **Khắc phục thành công lỗi cũ (`TODO-SEM-01`):**
  - Trước đây, `sensors.rs` so sánh biên độ nến đơn lẻ 1m với phí giao dịch hai chiều ($0.08\%$), khiến $95\%$ cây nến bình thường bị Gatekeeper Veto oan.
  - Hiện tại, tại `gatekeeper.rs:24-32`:
    ```rust
    let market_natr_pct = sensors.natr.get(tick.i).copied().unwrap_or(0.0);
    let roundtrip_fee_pct = 0.08;
    let min_spread_buffer_pct = config.gatekeeper.min_natr_threshold.max(0.01);
    if market_natr_pct < (roundtrip_fee_pct + min_spread_buffer_pct) {
        return GatekeeperDecision { can_trade: false, reason: Some("FLATLINE_CHOP_FILTER") };
    }
    ```
- **🔴 Bẫy mới phát sinh (`NEW-NATR-4H-BLIND`):**
  - `sensors.natr` được tính từ `ind.natr.timeframe`.
  - Trong quá trình tối ưu GA, gen `ind.natr.timeframe` có thể rơi vào **4h** hoặc **1h**.
  - Chỉ số NATR của nến 4h trên thị trường crypto (ETH, BTC) hầu như luôn dao động trong khoảng $0.80\% \dots 3.0\%$.
  - Biểu thức so sánh:
    $$\text{market\_natr\_pct} \ge 0.80\% \gg 0.08\% + 0.05\% = 0.13\%$$
  - **Hậu quả vi cấu trúc:** Khi thị trường rơi vào phiên thanh khoản chết vi mô (nến 1m/5m dẹt như que tăm, không có khối lượng và biên độ), `sensors.natr` khung 4h vẫn hiển thị mức $1.2\%$. Cầu dao Flatline bị **MÙ HOÀN TOÀN**, không thể ngăn chặn bot lao vào cào lệnh ở những thời điểm spread và trượt giá nuốt trọn biên độ!
- **Giải pháp dứt điểm:** Tách bạch rõ 2 thước đo:
  1. `Macro NATR` (15m/1h): Đánh giá biến động tổng thể.
  2. `Micro NATR` (1m/5m): Chốt chặn Flatline thực sự của phiên giao dịch hiện tại.

---

### 2.2. Nghịch Lý Catch-22: Arbiter Veto Sớm Biến `clamp_sl_price` Thành Mã Chết
- **Quan sát chuỗi logic từ Arbiter đến Execution:**
  1. **Tầng Arbiter (`core_v3/arbiter.rs:254-280`):**
     ```rust
     let max_allowed_roe = playbook_max_sl_roe.min(GLOBAL_HARD_MAX_ROE);
     let is_sl_valid = match signal.direction {
         SignalDirection::Long => {
             let roe = ((signal.entry_price - sl) / signal.entry_price) * leverage;
             roe > 0.0 && roe <= max_allowed_roe && sl < safe_liq_dist
         },
         ...
     };
     if !is_sl_valid {
         return false; // VETO THẲNG TAY TÍN HIỆU!
     }
     ```
  2. **Tầng Thực thi Strategy (`core_v3/mod.rs:332-342`):**
     ```rust
     // Kẹp cứng Stop Loss vào ngưỡng trần an toàn
     let clamped_sl = match signal.direction {
         SignalDirection::Long => {
             let max_dist = signal.entry_price * (max_roe_decimal / leverage);
             sl.max(signal.entry_price - max_dist)
         },
         ...
     };
     ```
- **Bản chất nghịch lý Catch-22:**
  - Nếu Scanner (ví dụ `Trend_PB` hoặc `Absorption`) phát hiện một cấu trúc sóng đảo chiều đẹp nhưng do cây nến quét trước đó dài khiến Stop Loss cấu trúc vượt quá $15\%$ ROE (ví dụ $18\%$ ROE):
    - Thay vì cho phép tín hiệu đi qua để `clamp_sl_price` tự động co SL về mức trần $15\%$ ROE an toàn, `validate_structural_sl` trong Arbiter đã **VETO BỎ HOÀN TOÀN TÍN HIỆU TỪ VÒNG GỬI XE**!
    - Hậu quả: Khối code `clamp_sl_price` trong `mod.rs` **KHÔNG BAO GIỜ CÓ CƠ HỘI ĐƯỢC CHẠY (DEAD CODE)**!
  - Điều này lý giải tại sao trong các đợt kiểm thử thực tế vừa qua, các Scanner bị tê liệt lệnh hàng loạt: $80\%$ các cơ hội giao dịch có râu nến biến động lớn đều bị Arbiter giết chết tức tưởi thay vì được quản trị rủi ro bằng cách kẹp biên SL.
- **Giải pháp kiến trúc:** Xóa bỏ lệnh Veto giết chết tín hiệu trong `validate_structural_sl`. Chuyển sang cơ chế **Auto-Clamping SL**: Nếu SL cấu trúc quá rộng, tự động kẹp về `max_allowed_roe` và tiếp tục cho phép vào lệnh với khối lượng bảo toàn rủi ro!

---

### 2.3. Hiện Tượng Tự Trói Tay & Vùng Chân Không Tê Liệt (Chop Dead Zone)
- **Tình trạng trong code:**
  - Tại `scanners/trend_pb.rs:64`:
    ```rust
    let is_squeeze_safe = squeeze <= pb_cfg.max_squeeze_intensity;
    if !is_squeeze_safe {
        return None; // Thoát ngay nếu thị trường đang nén (Squeeze)
    }
    ```
  - Thiết kế ban đầu: Khi thị trường Squeeze (đi ngang nén biên độ), `Trend_PB` phải nhường sân cho `Range_Osc` (giao dịch biên Bollinger/Keltner).
  - Tuy nhiên, tại `core_v3/mod.rs:48-53`:
    ```rust
    // [V3.3 TINH GIẢN] Tắt RangeOscillator khỏi vòng lặp quét lệnh
    // let range_signal = self.range_osc.scan(...);
    ```
- **Hậu quả thực tế:**
  - `Trend_PB` tự trói tay khi thị trường nén.
  - `Range_Osc` lại bị vô hiệu hóa hoàn toàn.
  - **Tạo ra một Vùng Chân Không Tê Liệt (Dead Zone):** Suốt những giai đoạn thị trường tích lũy nén biên độ (chiếm tới $60\% - 70\%$ thời gian của thị trường crypto), hệ thống hoàn toàn không có bất kỳ Playbook nào hoạt động, dẫn tới chuỗi ngày 0 lệnh kéo dài làm sập tiêu chuẩn WFE Gate!

---

### 2.4. Đánh Giá Lặp Lại 2 Lần Gatekeeper Mỗi Tick (`NEW-DOUBLE-EVAL`)
- **Vị trí code:**
  - **Lần 1:** Tại `core_v3/mod.rs:241-247`:
    ```rust
    let gate = self.gatekeeper.evaluate(tick, &self.sensors, &self.matrix, &self.config);
    if !gate.can_trade {
        return; // Dừng ngay vòng lặp next()
    }
    ```
  - **Lần 2:** Tại `core_v3/arbiter.rs:168-173`:
    ```rust
    let gate = self.gatekeeper.evaluate(tick, sensors, matrix, config);
    if !gate.can_trade {
        return ArbiterDecision { is_approved: false, final_signal: None, ... };
    }
    ```
- **Hậu quả:**
  - Trong mỗi tick nến, `gatekeeper.evaluate` bị gọi lặp lại 2 lần với các phép tính toán giống hệt nhau.
  - Phân tán trách nhiệm kiến trúc: Cổng vào chiến lược nên được kiểm soát duy nhất tại một chốt chặn tập trung ở cửa ngõ `core_v3/mod.rs`, không nên để Arbiter phải gọi lại lần thứ hai.

---

## ⚖️ 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành Phần / Đề Xuất | Ưu Điểm | Khuyết Điểm | Edge Cases Nguy Hiểm & Giải Pháp |
| :--- | :--- | :--- | :--- |
| **Gỡ Bỏ Veto Structural SL $\to$ Auto Clamping SL** | • Tái sinh lại khối code `clamp_sl_price` bị chết.<br>• Giải phóng $80\%$ số lệnh bị chặn oan của Trend_PB & Absorption.<br>• Tuân thủ nghiêm ngặt nguyên lý bảo toàn rủi ro. | • Vị thế sẽ chịu SL chặt hơn cấu trúc râu nến (có thể dễ dính SL sớm hơn nếu thị trường quét râu). | **Edge Case:** SL bị kẹp quá sát giá Entry $< 0.20\%$ do đòn bẩy quá cao.<br>$\implies$ Bắt buộc giữ khoảng cách SL tối thiểu $\ge 0.30\%$ để tránh trượt giá cắn SL ngay khi khớp. |
| **Tách Micro NATR vs Macro NATR** | • Khử $100\%$ bẫy mù Flatline khi nến 4h cao nhưng nến 1m chết thanh khoản.<br>• Bảo vệ bot khỏi trượt giá và giãn spread. | • Cần thêm 1 cảm biến `natr_micro` (1m/5m) vào struct `HeliosSensors`. | **Edge Case:** Nến 1m giật bất thường do nhiễu micro-spread.<br>$\implies$ Dùng SMA trượt 5 nến của Micro NATR để làm mịn. |
| **Gỡ Bỏ Lặp Lại Gatekeeper Lần 2 Trong Arbiter** | • Tiết kiệm chu kỳ CPU trong vòng lặp tick.<br>• Quy hoạch chuẩn xác: Gatekeeper là Pre-trade Gateway ở `mod.rs`, Arbiter chỉ chuyên tâm đấu thầu tín hiệu. | • Cần điều chỉnh chữ ký của `arbiter.process_tick`. | **Edge Case:** Tín hiệu lọt qua `mod.rs` nhưng thị trường biến động cực đoan ngay trong tick.<br>$\implies$ Không xảy ra vì dữ liệu tick là bất biến trong suốt hàm `next()`. |
| **Kích Hoạt Lại RangeOsc Hoặc Nới Lỏng Squeeze Ở Trend_PB** | • Xóa bỏ vùng chân không tê liệt (Dead Zone).<br>• Tăng tần suất giao dịch trong giai đoạn nén tích lũy. | • Giao dịch trong vùng tích lũy đòi hỏi quản trị R:R chặt chẽ vì dễ bị quét 2 đầu. | **Edge Case:** Breakout giả trong vùng nén.<br>$\implies$ Áp dụng Break-Even Ratchet sớm khi đạt $1.0R$. |

---

## 📋 4. DANH MỤC CÔNG VIỆC CẦN TRIỂN KHAI (TODO ACTION ITEMS)

- [ ] **Task GTW-01:** Chuyển đổi `validate_structural_sl` trong `arbiter.rs:254-280` từ cơ chế Hard Veto sang cơ chế Auto-Clamping (kẹp SL về trần an toàn thay vì hủy lệnh).
- [ ] **Task GTW-02:** Bổ sung `micro_natr` (khung 1m/5m cố định) vào chốt chặn Flatline của `gatekeeper.rs`, triệt tiêu hoàn toàn bẫy mù khi nạp NATR 4h.
- [ ] **Task GTW-03:** Loại bỏ lệnh gọi `gatekeeper.evaluate` lặp lại thừa thãi trong `arbiter.rs:169`.
- [ ] **Task GTW-04:** Giải quyết vùng chân không tê liệt (Chop Dead Zone): Đánh giá việc nới lỏng ngưỡng `max_squeeze_intensity` của `Trend_PB` hoặc phục hồi `Range_Osc` với bộ thông số tinh giản.
