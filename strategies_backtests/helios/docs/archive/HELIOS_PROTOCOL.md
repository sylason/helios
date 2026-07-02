# HELIOS STRATEGY PROTOCOL (Giao thức Chiến lược Helios)

Tài liệu này là "Nguồn Chân Lý" (Source of Truth) duy nhất cho logic vận hành của chiến lược Helios. Mọi thay đổi trong mã nguồn Rust và cấu hình JSON phải tuân thủ nghiêm ngặt các quy tắc này.

## 1. Quản lý Vị thế & Rủi ro (Position & Risk Management)

- **Nguyên tắc "Độc tôn" (No Pyramiding):** Tuyệt đối không nhồi lệnh. Chỉ cho phép một vị thế (LONG hoặc SHORT) tồn tại tại một thời điểm. Mọi tín hiệu cùng chiều khi đang có lệnh sẽ bị bỏ qua.
- **Quy mô lệnh cố định (Fixed Sizing):** Sử dụng một tỷ lệ % vốn cố định (X% Equity) cho mọi giao dịch để đảm bảo tính ổn định và kiểm soát rủi ro đồng nhất.
- **Cơ chế Đảo chiều & Cắt ngang (Reversal & Override):**
    - **Mặc định (Bảo thủ):** Lệnh hiện tại phải kết thúc (trúng Trailing SL hoặc TP) thì mới xét đến lệnh tiếp theo.
    - **Ngoại lệ Khẩn cấp (Emergency Override):** Chỉ cho phép đóng vị thế cũ để mở vị thế ngược chiều ngay lập tức nếu Squad cứu hộ (**Anomaly** hoặc **Reversal**) đạt điểm số cực đại (**Score > 0.95**).

## 2. Hệ thống Hội đồng Helios (Council Arbitration System)

Hệ thống phân xử được thiết kế để loại bỏ sự thiên kiến và độc tài của một Squad duy nhất.

- **Điểm số Nhạy bén Ngữ cảnh (Context-Aware Scoring):**
    - Mỗi Squad tính điểm dựa trên các chỉ báo (Indicators) riêng.
    - **Ràng buộc Ma trận:** Điểm số phải bị điều chỉnh bởi "Ma trận nhận thức" chung (Market Matrix). Ví dụ: Squad Trend không được có điểm cao nếu chỉ số `entropy` (nhiễu) vượt quá ngưỡng an toàn.
- **Phân xử Xếp hạng (Ranked Arbitration):**
    - Loại bỏ cơ chế `if-else` (kẻ đứng sau thắng).
    - Toàn bộ Squad vượt ngưỡng `min_score_threshold` sẽ được đưa vào một danh sách.
    - **Sắp xếp (Sorting):** Sắp xếp danh sách theo `Weighted_Score = Score * Priority_DNA`.
    - **Kẻ chiến thắng:** Squad đứng đầu danh sách xếp hạng được quyền "bóp cò".
- **Sự Toàn vẹn của Playbook:** Khuyến khích GA tìm ra DNA mà ở đó các Squad bổ trợ cho nhau tùy theo Market Regime (Trạng thái thị trường).

## 3. Cầu chì An toàn & Giác quan (Safety & Perception)

- **Cầu chì Môi trường (Environmental Fuse):**
    - Nếu trạng thái là `Chaotic` (Hỗn loạn/Nhiễu cực cao): Dừng tuyệt đối mọi hoạt động vào lệnh mới. An toàn vốn là ưu tiên số 1. Cơ chế này đóng vai trò là "Khoảng lặng" (Cool-down) tự nhiên của hệ thống.
- **Màng lọc Volatility:**
    - Sử dụng `volatility_benchmark` (SMA của ATR) để tính toán SL/TP, tránh việc bị quét lệnh bởi các biến động ảo (Noise) của khung 1-phút.
- **Tính năng Động cho Indicators:** Code phải đảm bảo các chỉ số (như Squeeze) trả về giá trị biến thiên liên tục (ví dụ dùng Percentile) thay vì các giá trị "đơ" (0 hoặc 1), giúp Optimizer có "manh mối" để tiến hóa.

## 4. Phân cấp Đa khung thời gian (MTF Authority)

- **Trọng số theo Khung thời gian:** Các chỉ báo ở khung thời gian lớn (1h, 4h) sẽ có trọng số đóng góp điểm số vào Hội đồng cao hơn các khung nhỏ (1m, 5m). Điều này đảm bảo các quyết định tác chiến luôn thuận theo bối cảnh vĩ mô.

## 5. Nhất quán Nhiệm vụ (Mission Consistency)

- **Trailing SL theo Playbook:** Một khi lệnh đã được mở, cơ chế Trailing Stoploss sẽ bám sát logic của Playbook đã mở lệnh đó cho đến khi kết thúc. Điều này giữ cho "ý chí" quản trị rủi ro ban đầu không bị thay đổi bởi các biến động nhất thời của Regime sau đó.
- **Tập trung vào Mở lệnh:** Mọi tinh hoa của Strategy tập trung vào việc tìm điểm vào lệnh tối ưu. Việc thoát lệnh hoàn toàn được ủy thác cho hệ thống Trailing SL co giãn.

## 6. Mục tiêu Chẩn đoán (Diagnostic Goals)

Hệ thống Log phải in ra được các thông tin sau để giám định:
1. Danh sách điểm số của tất cả Squad tại nến hiện tại.
2. Lý do một Squad bị loại (không vượt ngưỡng hoặc thua điểm xếp hạng).
3. Lý do Sniper không bóp cò (do vị thế đang mở hoặc lọc kỹ thuật).
