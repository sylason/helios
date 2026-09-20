# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (ACTIONABLE TODO): ARBITER CORE V3
**Module:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`  
**Tài liệu kiểm toán gốc:** [technical_audit.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_arbiter/technical_audit.md)  
**Ngày lập kế hoạch:** 19/09/2026  
**Chuyên gia kiểm toán:** Quantitative & Systems Auditor  

---

## 🎯 BẢNG TỔNG QUAN ƯU TIÊN THỰC THI

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-ARB-01` | Xóa bỏ quy tắc cướp thầu SL hẹp (`dist_new < dist_curr`) | **CRITICAL** | 🔴 Chưa xử lý | `core_v3/arbiter.rs:173–183` |
| **2** | `TASK-ARB-02` | Tái cấu trúc bộ giải quyết xung đột 2 tầng (Two-Stage Auction) | **HIGH** | 🔴 Chưa xử lý | `core_v3/arbiter.rs:164–205` |
| **3** | `TASK-ARB-03` | Miễn trừ Macro Veto có điều kiện cho Climax Reversal | **HIGH** | 🔴 Chưa xử lý | `core_v3/arbiter.rs:108–118, 136–144` |
| **4** | `TASK-ARB-04` | Cấu hình hóa các tham số Hardcoded vào `ArbiterConfig` | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/config.rs:300–327`, `arbiter.rs:128` |
| **5** | `TASK-ARB-05` | Tối ưu hóa Zero-Allocation cho HeliosArbiter trong Bot Engine | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/mod.rs:338–346` |
| **6** | `TASK-ARB-06` | Bổ sung kiểm tra tường minh `is_nan()` cho `compute_confidence` | **LOW** | 🔴 Chưa xử lý | `core_v3/arbiter.rs:98–106` |
| **7** | `TASK-ARB-07` | Đánh giá phục hồi có kiểm soát cho `RangeOsc` & `AnomalyFade` | **MEDIUM** | 🔴 Chưa xử lý | `core_v3/mod.rs:340–345` |

---

## 📋 CHI TIẾT CÁC ĐẦU VIỆC CẦN THỰC HIỆN

### 🔴 TASK-ARB-01: Xóa bỏ quy tắc cướp thầu SL hẹp (`dist_new < dist_curr`)
- **Mã lỗi liên đới:** `VULN-ARB-01` (Critical).
- **Hiện trạng:**
  - Tại `arbiter.rs:173–183`, khi hai đề xuất cùng chiều, code đang so sánh `dist_new < dist_curr` để chọn đề xuất có SL gần giá hiện tại hơn, bất chấp điểm tự tin `conf` chênh lệch ra sao.
  - Điều này khiến `AbsorptionScout` (SL hẹp trong vùng Squeeze) liên tục cướp thầu của `TrendPullback` (SL đệm theo cấu trúc Swing), dẫn đến lệnh dễ bị quét stopout bởi nhiễu vi mô và thua lỗ vô ích.
- **Hành động cần làm:**
  1. Xóa bỏ hoàn toàn điều kiện `if dist_new < dist_curr` làm tiêu chí số 1.
  2. Thay thế bằng tiêu chí: **Đề xuất có `confidence` cao hơn sẽ chiến thắng**.
  3. Chỉ khi hai đề xuất có `(conf - current_conf).abs() <= 0.03` (tự tin tương đương), mới xét đến chất lượng của SL: Ưu tiên SL có khoảng cách an toàn nằm trong dải chuẩn cấu trúc $[1.0 \times \text{ATR}, 3.0 \times \text{ATR}]$, tránh SL quá chật ($< 0.5 \text{ATR}$) hoặc quá xa ($> 4.0 \text{ATR}$).
- **Tiêu chuẩn nghiệm thu (Acceptance Criteria):**
  - Unit test `test_conflict_resolution_closer_stop_loss` phải được cập nhật lại theo triết lý ưu tiên `confidence`.
  - Một proposal có `conf = 0.85` và `dist = 1.8 ATR` phải chiến thắng proposal có `conf = 0.40` và `dist = 0.5 ATR`.

---

### 🟠 TASK-ARB-02: Tái cấu trúc bộ giải quyết xung đột 2 tầng (Two-Stage Auction)
- **Mã lỗi liên đới:** `VULN-ARB-02` (High).
- **Hiện trạng:**
  - Tại `arbiter.rs:184–202`, câu lệnh `return ArbiterDecision` ở dòng 193 thoát ngay lập tức khỏi hàm khi phát hiện hai lệnh ngược chiều có chênh lệch $\le 0.15$.
  - Lỗi này khiến các scanner đứng phía sau trong mảng hoàn toàn bị bỏ rơi, dẫn đến việc thứ tự đăng ký scanner làm thay đổi hoàn toàn phán quyết của Arbiter.
- **Hành động cần làm:**
  1. Tách thuật toán đấu thầu thành 2 tầng độc lập:
     - **Tầng 1 (Cùng chiều):** Tìm ra `best_long: Option<(Proposal, f64)>` và `best_short: Option<(Proposal, f64)>` tốt nhất từ toàn bộ danh sách `self.scanners`.
     - **Tầng 2 (Đối đầu Long vs Short):** So sánh duy nhất giữa `best_long` và `best_short`:
       + Nếu chỉ có Long: Chọn Long.
       + Nếu chỉ có Short: Chọn Short.
       + Nếu có cả hai: Xét hiệu số $\Delta = |\text{conf}_{\text{long}} - \text{conf}_{\text{short}}|$. Nếu $\Delta > \text{conflict\_margin}$ chọn bên mạnh hơn; nếu $\Delta \le \text{conflict\_margin}$ thì trả về Flat.
  2. Đảm bảo tính giao hoán: Thứ tự add scanner vào `self.scanners` không được phép làm thay đổi kết quả đầu ra.
- **Tiêu chuẩn nghiệm thu:**
  - Viết unit test chứng minh hoán vị mảng scanners (duyệt A trước B hoặc B trước A) đều cho ra cùng một `ArbiterDecision`.

---

### 🟠 TASK-ARB-03: Miễn trừ Macro Veto có điều kiện cho Climax Reversal
- **Mã lỗi liên đới:** `VULN-ARB-03` (High).
- **Hiện trạng:**
  - Tại `arbiter.rs:115–116, 136–144`, bộ lọc `is_direction_vetoed` cấm Long khi `kama_slope_atr < -0.10 && trend_axis >= 0.40`.
  - Điều này triệt hạ `ClimaxReversalScanner` trong các cú sụp đổ thanh lý (Capitulation Dump), khiến scanner này bị "starvation" hoàn toàn.
- **Hành động cần làm:**
  1. Thêm phương thức kiểm tra miễn trừ hoặc bổ sung logic ngoại lệ trong `is_direction_vetoed`:
     - Nếu `proposal.scanner_id == ScannerId::ClimaxReversal`: Cho phép bỏ qua Macro Veto nếu `z_score` cực đại ($|Z| \ge 2.5$) và có tín hiệu cạn kiệt khối lượng (Volume Exhaustion).
  2. Bổ sung cờ cấu hình `exempt_reversal_from_veto: bool` trong `ArbiterConfig` để người dùng có thể bật/tắt hành vi này.
- **Tiêu chuẩn nghiệm thu:**
  - Khi KAMA dốc $-0.25$ và TrendAxis $0.60$, tín hiệu Long của `TrendPullback` bị VETO, nhưng tín hiệu Long cực hạn của `ClimaxReversal` được PHÉP thông qua vào vòng đấu thầu.

---

### 🟡 TASK-ARB-04: Cấu hình hóa các tham số Hardcoded vào `ArbiterConfig`
- **Mã lỗi liên đới:** `VULN-ARB-04` (Medium).
- **Hiện trạng:**
  - `macro_thresh = 0.10` bị gán cứng tại dòng 128.
  - `trend_axis >= 0.40` bị gán cứng tại dòng 115–116.
  - `diff > 0.15` (ngưỡng phân xử xung đột) bị gán cứng tại dòng 188.
- **Hành động cần làm:**
  1. Mở rộng `ArbiterConfig` trong `core_v3/config.rs`:
     - `macro_veto_slope_threshold: f64` (mặc định 0.10).
     - `macro_veto_trend_axis_min: f64` (mặc định 0.40).
     - `conflict_confidence_margin: f64` (mặc định 0.15).
  2. Cập nhật macro DNA (nếu muốn tối ưu hóa) hoặc giữ serde default để đảm bảo tính tương thích ngược.
  3. Truyền các giá trị này từ `config.arbiter` vào `arbiter.process_tick(...)`.
- **Tiêu chuẩn nghiệm thu:**
  - Không còn bất kỳ hằng số ma thuật (magic numbers) nào liên quan đến ngưỡng veto hay phân xử xung đột bị hardcode trong `arbiter.rs`.

---

### 🟡 TASK-ARB-05: Tối ưu hóa Zero-Allocation cho HeliosArbiter trong Bot Engine
- **Mã lỗi liên đới:** `VULN-ARB-05` (Medium).
- **Hiện trạng:**
  - `core_v3/mod.rs:338–344` khởi tạo `let mut arbiter = HeliosArbiter::new();` và 4 lần `Box::new(...)` ở MỖI TICK.
  - Lãng phí hàng triệu lần cấp phát heap khi chạy backtest và GA Optimizer.
- **Hành động cần làm:**
  1. Đưa `arbiter: HeliosArbiter` thành một trường trong struct `HeliosStrategyV3`.
  2. Khởi tạo và nạp danh sách scanners một lần duy nhất trong `HeliosStrategyV3::new(...)`.
  3. Trong hàm `next()`, chỉ việc gọi `self.arbiter.process_tick(...)` trực tiếp trên tham chiếu `&self.arbiter`.
- **Tiêu chuẩn nghiệm thu:**
  - Zero dynamic heap allocation cho Arbiter trong vòng lặp `next()`.
  - Tốc độ backtest tăng ít nhất 15% – 25%.

---

### 🟢 TASK-ARB-06: Bổ sung kiểm tra tường minh `is_nan()` cho `compute_confidence`
- **Mã lỗi liên đới:** `VULN-ARB-06` (Low).
- **Hiện trạng:**
  - Dòng 100–105 chỉ kiểm tra `raw_score <= 0.0`. Nếu `raw_score` hoặc `chaos_index` mang giá trị `f64::NAN`, phép tính có thể tạo ra `NaN` không an toàn.
- **Hành động cần làm:**
  1. Sửa hàm `compute_confidence`:
     ```rust
     #[inline]
     pub fn compute_confidence(raw_score: f64, chaos_index: f64) -> f64 {
         if raw_score.is_nan() || chaos_index.is_nan() || raw_score <= 0.0 {
             return 0.0;
         }
         let penalty = chaos_index.max(0.0).min(1.0);
         (raw_score * (1.0 - penalty)).max(0.0).min(1.0)
     }
     ```
  2. Bổ sung unit test kiểm tra đầu vào `NaN` luôn trả về `0.0`.
- **Tiêu chuẩn nghiệm thu:**
  - `compute_confidence(f64::NAN, 0.2) == 0.0`
  - `compute_confidence(0.8, f64::NAN) == 0.0`

---

### 🟡 TASK-ARB-07: Đánh giá phục hồi có kiểm soát cho `RangeOsc` & `AnomalyFade`
- **Mã lỗi liên đới:** Phân tích Starvation trong `technical_audit.md` (Mục 7).
- **Hiện trạng:**
  - `mod.rs:340–345` loại bỏ cứng `RangeOscScanner` và `AnomalyScanner`, khiến 2 playbook này bị "starvation" hoàn toàn (0.0% execution).
- **Hành động cần làm:**
  1. Xem xét thiết kế cờ kích hoạt trong config:
     - `enable_range_scanner: bool`
     - `enable_anomaly_scanner: bool`
  2. Nếu người dùng bật cờ, cho phép đăng ký vào Arbiter.
  3. Thiết lập bộ lọc phối hợp an toàn (chỉ kích hoạt `RangeOsc` khi `trend_axis < 0.20 && bb_width_norm > min_width`).
- **Tiêu chuẩn nghiệm thu:**
  - Các scanner không bị xóa cứng trong mã nguồn, có thể bật/tắt linh hoạt qua tham số cấu hình.

---

## 🧪 KẾ HOẠCH TEST HỒI QUY (REGRESSION TEST SUITE)

Sau khi hoàn tất các task trên, bộ test tại `arbiter.rs:230–298` phải được mở rộng với các kịch bản sau:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nan_resilience_in_confidence() {
        assert_eq!(HeliosArbiter::compute_confidence(f64::NAN, 0.5), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(0.8, f64::NAN), 0.0);
        assert_eq!(HeliosArbiter::compute_confidence(f64::NAN, f64::NAN), 0.0);
    }

    #[test]
    fn test_confidence_superiority_over_tight_sl() {
        // Proposal A: Tự tin cao (0.80), SL rộng (1.5 ATR)
        // Proposal B: Tự tin thấp (0.30), SL hẹp (0.4 ATR)
        // KẾT QUẢ MONG ĐỢI: Proposal A phải THẮNG thầu!
    }

    #[test]
    fn test_conflict_resolution_commutativity() {
        // Danh sách 1: [Long(0.50), Short(0.52), Long(0.95)]
        // Danh sách 2: [Long(0.95), Short(0.52), Long(0.50)]
        // KẾT QUẢ MONG ĐỢI: Cả 2 danh sách phải cho ra cùng kết quả Long(0.95) thắng!
    }

    #[test]
    fn test_reversal_exemption_from_macro_veto() {
        // Thị trường sập mạnh: kama_slope = -0.30, trend_axis = 0.65
        // TrendPullback Long: Bị VETO
        // ClimaxReversal Long (với Z-score = -3.0): ĐƯỢC PHÉP thông qua!
    }
}
```
