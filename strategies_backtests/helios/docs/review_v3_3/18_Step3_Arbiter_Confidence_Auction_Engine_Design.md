# HELIOS V3.3 ARCHITECTURE REVIEW: PART 18 - STEP 3 ARBITER CONFIDENCE AUCTION ENGINE DESIGN

Tài liệu này ghi nhận chi tiết thiết kế nâng cấp cho **Bước 3: Bộ Trọng Tài Đấu Thầu Động Helios Arbiter (Confidence Auction Engine)** trong Helios V3.3. Thiết kế này loại bỏ hoàn toàn các câu lệnh cấm cứng `is_allowed` nhị phân, thiết lập cơ chế **Đấu Thầu Điểm Tự Tin (Confidence Auction)** đa bối cảnh, chuẩn hóa mã nguồn bằng **`enum ScannerId`** theo chuẩn Clean Code Rust và xây dựng 3 tầng phòng vệ chống bẫy **Fake Breakout**.

---

## 🏛️ 1. MỤC TIÊU TÁI CẤU TRÚC HELIOS ARBITER

1. **Xóa bỏ 100% Cổng Cấm Nhị Phân `is_allowed`**: Không còn bất kỳ câu lệnh `if-else` nào khóa cứng scanner theo bối cảnh. Tất cả 6 Scanners đều được đối xử bình đẳng và tự do gửi đề xuất (`Proposal`) về cho Arbiter.
2. **Đấu Thầu Tích Hỗ Tương (Interactivity Confidence Auction)**: Điểm tự tin $\text{Confidence}_j$ là sự giao thoa hài hòa giữa Điểm Mẫu Hình ($P_j$), Trọng Số Bối Cảnh Simplex ($\overline{w}_{\text{regime}}$) và Hệ Số Vi Mô Matrix ($\text{multiplier}$).
3. **Phê Duyệt Ngưỡng Động Continuous**: Ngưỡng phê duyệt $C_{\text{thresh}}[i]$ trôi mượt mà theo bối cảnh thị trường thực tế.

---

## 🦀 2. CHUẨN CLEAN CODE RUST: `enum ScannerId`

Thay vì sử dụng chuỗi ký tự `&'static str` dễ gây lỗi chính tả và tốn chi phí so sánh, hệ thống chuyển sang sử dụng `enum ScannerId`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScannerId {
    TrendPullback,      // TREND_PB
    MomentumIgnition,   // MOMENTUM_IGN
    RangeOscillator,    // RANGE_OSC
    ClimaxReversal,     // REVERSAL
    AnomalyFade,        // ANOMALY
    AbsorptionScout,    // ABSORPTION
}
```

### Lợi ích định lượng:
- **Type Safety & Exhaustive Matching**: Trình biên dịch `rustc` bắt buộc nhánh `match` phải cover đủ $100\%$ các Scanner tại thời điểm compile, loại bỏ hoàn toàn lỗi runtime.
- **Zero Cost Abstraction**: So sánh 2 enum trong Rust là so sánh 1 số nguyên `u8` trên thanh ghi CPU ($0\text{ns}$), nhanh hơn so sánh chuỗi ký tự rất nhiều lần.

---

## 📋 3. THỨ TỰ LUỒNG LOGIC THỰC THI CHI TIẾT (TỪNG BƯỚC)

Tại mỗi cây nến $i$ (chu kỳ 1 phút), khi bot nhận dữ liệu `tick`:

- **Bước 1: Khởi Tạo Trạng Thái Đấu Thầu Trên Nến Hiện Tại**:
  - Đặt `best_proposal = None` (chưa có đề xuất nào dẫn đầu).
  - Đặt `max_confidence = 0.0` (mức điểm cao nhất khởi điểm bằng 0).
  - Tính toán **Ngưỡng Phê Duyệt Động $C_{\text{thresh}}[i]$** từ Vector Simplex:
    $$C_{\text{thresh}}[i] = \overline{w}_{\text{trend}}[i] \cdot \text{min\_conf\_trend} + \overline{w}_{\text{range}}[i] \cdot \text{min\_conf\_range} + \overline{w}_{\text{volatile}}[i] \cdot \text{min\_conf\_vol}$$

- **Bước 2: Vòng Lặp Thu Thập & Chấm Điểm Đấu Thầu (Duyệt qua 6 Scanners)**:
  - Lần lượt với từng Scanner trong danh sách:
    - **2.1. Quét tín hiệu vi mô (`scanner.scan`)**: Scanner tự tính điểm mẫu hình $P_j[i] \in [0.0, 1.0]$. Nếu $P_j[i] < \text{min\_activation\_score}$, Scanner trả về `None` (bỏ qua). Nếu đạt, trả về `Some(Proposal)`.
    - **2.2. Tra cứu Trọng số Bối cảnh Vĩ mô**: Dùng `match proposal.scanner_id` để lấy trọng số Simplex $\overline{w}_{\text{regime}(j)}[i]$ tương ứng ($O(1)$).
    - **2.3. Tra cứu Multiplier Dòng tiền**: Lấy `multiplier` tương ứng từ `MarketMatrixV3`.
    - **2.4. Màng lọc Phủ quyết (Veto Check)**: Nếu $\text{multiplier} < \text{min\_multiplier\_veto} \implies$ Loại ngay đề xuất này (gán điểm $= 0.0$), chuyển sang Scanner kế tiếp.
    - **2.5. Tính Điểm Tự Tin Đấu Thầu (Confidence Score)**:
      $$\text{Confidence}_j = P_j[i] \times \overline{w}_{\text{regime}(j)}[i] \times \text{multiplier}_j[i]$$
    - **2.6. Cạnh tranh Ngôi Quán Quân (Max Tracking)**: So sánh `Confidence_j` với `max_confidence`. Nếu $\text{Confidence}_j > \text{max\_confidence}$, cập nhật:
      - `max_confidence = Confidence_j`
      - `best_proposal = Some(proposal)`

- **Bước 3: Hội Đồng Trọng Tài Ra Quyết Định Cuối Cùng**:
  - Kiểm tra xem có đề xuất quán quân (`best_proposal`) hay không:
    - **Nếu có đề xuất VÀ `max_confidence >= C_thresh[i]`**:
      $\implies$ **PHÊ DUYỆT (`is_approved = true`)**, đóng gói `ArbiterDecision` chuyển giao cho `RiskManager` để đặt lệnh lên sàn.
    - **Ngược lại (Không có đề xuất nào hoặc điểm cao nhất chưa vượt ngưỡng duyệt)**:
      $\implies$ **TỪ CHỐI (`is_approved = false`)**, bot đứng ngoài quan sát để bảo toàn vốn.

---

## 🛡️ 4. CƠ CHẾ PHÒNG THỦ 3 TẦNG CHỐNG BẪY "FAKE BREAKOUT"

Fake Breakout (Bull Trap / Bear Trap) là hiện tượng giá giật mạnh qua dải Bollinger Bands rồi lập tức đảo chiều cắm đầu. Hệ thống Helios V3.3 xử lý bằng 3 tầng phòng thủ:

1. **Tầng 1 - Màng lọc hợp lưu trước khi vào lệnh**: `MOMENTUM_IGN` đòi hỏi hợp lưu đồng thời: $f_{\text{break}}$ (Độ bục dải BB) + $f_{\text{rsi}}$ (RSI bứt phá) + $f_{\text{vol}}$ (Khối lượng xác nhận). Nếu chỉ là cú giật râu thanh khoản với volume yếu $\implies$ Bị Arbiter loại ngay.
2. **Tầng 2 - Kẹp trần Stop Loss khởi tạo**: Ngay khi lệnh khớp, SL được đặt theo ATR và kẹp trần an toàn $\le \frac{0.80}{\text{Leverage}}$ (ví dụ $4.0\%$ với đòn bẩy $20x$). Nếu giá sập đột ngột, lệnh bị cắt ngay với mức lỗ tối đa $1\% - 2\%$ tài khoản, tuyệt đối không bị thanh lý.
3. **Tầng 3 - Trailing Stop & Progressive Time Decay**: 
   - Nếu Fake Breakout giật nhẹ rồi đi ngang đuối sức $\implies$ `stagnant_bars` tăng $\implies$ SL tự động siết sát vào giá và **Stagnant Capital Release** đóng chủ động lệnh lỗ sau 1 chu kỳ decay.
   - Nếu Fake Breakout có lãi nhẹ rồi mới quay đầu $\implies$ **High-Watermark Peak Tracking** đã kéo SL lên mức hòa vốn để bảo toàn vốn.

---

## 🔒 5. ĐỒNG NHẤT ĐÒN BẨY `leverage = 20.0` TRONG GIAI ĐOẠN NÀY

- **Khóa cố định đòn bẩy `leverage = 20.0`**:
  - Triệt tiêu bẫy Overfit đòn bẩy của GA Optimizer (không để Optimizer ăn gian bằng cách nâng đòn bẩy lên $50x$).
  - Giảm chiều không gian tìm kiếm, giúp GA hội tụ nhanh hơn.
  - Đồng nhất $100\%$ giữa `common.rs` (`MARGIN = 1.0 / 20.0`) và `HeliosConfigV3` (`leverage = 20.0`).
- Mức trần Stop Loss an toàn được cố định ở mức: $\frac{0.80}{20} = \mathbf{4.0\%}$.
- Các tính năng mở rộng đòn bẩy động (Dynamic Leverage) và Playbook "NUKE" sẽ được phát triển sau khi hệ thống cốt lõi hoàn thiện và ổn định.
