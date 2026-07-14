# Strategy Engine Specification

## 1. Trách nhiệm (Responsibilities)
- Nhận luồng K-Line 1m sạch sẽ từ Data Engine.
- Quản lý việc tích hợp với crate chiến lược (`helios`).
- Điều hành Optimizer liên tục để tìm tham số tốt nhất.

## 2. Vòng lặp Tối ưu hóa Liên tục (Continuous Optimization)
Đây là đặc tính làm nên điểm khác biệt của Bot:
1. **Background Optimizer**: Kế thừa logic từ `ferro_trade` (`run_helios_optimize.rs`), bot sẽ khởi chạy thuật toán Genetic Algorithm ngầm.
2. **Data Window**: Quá trình Optimize chỉ chạy trên tập dữ liệu **1 tháng gần nhất** tính đến thời điểm hiện tại.
3. **Continuous Loop**: Ngay khi Optimizer hoàn thành 1 phiên và tìm ra bộ Params tốt nhất -> Nó sẽ ghi JSON vào Database -> Ngay lập tức, nó gọi lại chính nó để chạy phiên tiếp theo (tịnh tiến Data Window thêm một chút).
4. **Forward Backtest Validation**: Lý do của việc chạy lặp này là để chứng minh Forward Backtest. Nếu các Parameter chạy "out-of-sample" ở nến tiếp theo đem lại Lợi nhuận Dương (Positive Return), chiến lược đó được chứng minh là khả thi!

## 3. Nạp nóng Tham số (Hot-Reloading JSON Params)
- Trong lúc Optimizer đang tính toán mướt mồ hôi, luồng chính của Asset Bot (Trade Engine) vẫn nhận nến và ra tín hiệu dựa trên bộ Params cũ.
- Mỗi khi có nến đóng (hoặc có Event báo hiệu), Strategy Engine sẽ check DB `strategy_configs`. Nếu thấy có bản ghi JSON mới với `created_at` cao hơn bản hiện tại, nó sẽ thực hiện ép kiểu JSON ngược ra Struct và **Hot-reload** thuật toán ngay trong lúc đang bay.
- Không cần restart bot!

## 4. Cơ chế Cắm và Chạy (Plug and Play)
Vì toàn bộ logic (như RSI, logic Helios) nằm ở crate ngoài (`strategies_backtests/helios`), Bot Engine chỉ định nghĩa Trait `StrategyRunner`. Nó đóng vai trò làm cầu nối (Adapter) để truyền Nến 1m vào `helios` và nhận về `Signal`.
