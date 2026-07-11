# Kế hoạch Triển khai Helios V3 (Signal-First)

## Giai đoạn 1: Thiết kế Cơ sở Hạ tầng (Infrastructure)
- [x] Tạo kiến trúc khung thư mục `core_v3` (Skeletons).
- [x] Soạn thảo Blueprint và Logic Toán học (Kiến trúc, Scanners, Arbiter).
- [ ] Xây dựng struct `HeliosStrategyV3` implement trait `Strategy` (chứa `next()` và `init()`).
- [ ] Chuyển đổi (Port) logic Market Matrix từ V2 sang dùng cho V3.

## Giai đoạn 2: Phát triển Các Modules Cốt lõi
- [ ] Định nghĩa `Trait Scanner` với hàm `scan(ctx, matrix) -> Option<Proposal>`.
- [ ] Viết `Trend_PB_Scanner`.
- [ ] Viết `Momentum_Scanner`.
- [ ] Viết `Range_Scanner`.
- [ ] Xây dựng bộ máy `Arbiter` nhận các Proposals và duyệt lệnh dựa trên `Regime Multiplier` và `Confidence Score`.

## Giai đoạn 3: Quản trị Rủi ro & Thực thi
- [ ] Viết hệ thống `RiskManager` để xử lý Trailing Stop.
- [ ] Tích hợp tính toán Size theo biến động ATR.
- [ ] Kiểm soát tín hiệu Nhiễu (Whipsaw/Slippage protection).

## Giai đoạn 4: Tích hợp & Kiểm thử (Testing)
- [ ] Viết Config V3 và JSON Schema.
- [ ] Xây dựng File Example `v3_run_helios.rs` để chạy thử nghiệm độc lập.
- [ ] Xác minh kết quả so sánh giữa V2 và V3 về tần suất vào lệnh (Frequency) và Tỉ suất Lợi nhuận (Profit Factor).
- [ ] Chạy Genetic Algorithm Optimizer trên V3 để tìm cấu hình mượt nhất.
