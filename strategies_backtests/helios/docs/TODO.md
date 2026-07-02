# TO-DO LIST CỦA DỰ ÁN HELIOS

> [!NOTE]
> File này dùng để theo dõi liên tục tiến độ sửa lỗi và nâng cấp tính năng của Helios Strategy.

## 🐛 BUGS CẦN FIX GẤP (CRITICAL)
- [ ] **Lỗi Vô hiệu hóa (The "Next" Bug):** Mở comment toàn bộ hàm `next()` trong file `src/strategy/mod.rs` để bot có thể thực sự Trade.
- [ ] **Lỗi Quản lý vốn (Sizing Bug):** Viết lại logic trong `src/strategy/logic/tactical.rs` để tính toán Size Hợp đồng theo Rủi ro Đô-la cố định (Fixed Dollar Risk), thay vì dùng `size_fraction` cứng.
- [ ] **Lỗi Mù Parabolic (Unit Mismatch):** Sửa phép tính điểm Parabolic trong `src/strategy/logic/regime.rs` và `risk.rs` (Đang dùng biến Percentile [0,1] chia cho độ (Degrees) -> điểm luôn = 0).
- [ ] **Lỗi Chiến thuật Chết (Dead Playbook):** Thêm điều kiện ánh xạ để Đội `MomentumIgnition` được gọi trong `src/strategy/logic/playbook.rs`. Hiện tại nó không bao giờ được dùng.

## 🛠 NÂNG CẤP KIẾN TRÚC (REFACTORING)
- [ ] **Gộp mã lặp (DRY SL/TP):** Tạo hàm `calculate_risk_params(entry, risk_config)` để loại bỏ 14 lần copy-paste code tính SL/TP trong `tactical.rs`.
- [ ] **Dọn rác (Dead Code):** Xóa các Struct không dùng đến trong `types.rs` (`MarketContext`, `MarketState`) và các hàm không gọi trong `normalization.rs`.
- [ ] **Tích hợp Slippage:** Cấu hình lại `ferro_trade` (hoặc RiskPolicy) để ép trừ Phí Giao Dịch (Taker Fee) và Slippage vào kết quả, giúp Optimizer tìm ra bộ thông số thực tế hơn.

## 🚀 TÍNH NĂNG MỚI (NEW FEATURES)
- [ ] **Triển khai Giao thức NUKE:** Bổ sung logic `Emergency Override` cho mọi vị thế khi điều kiện Squeeze/Parabolic đạt 100% hoàn hảo -> Đóng lệnh cũ, vào lệnh 100x với Tight SL.
- [ ] **Động cơ Chống Thanh lý (Liquidation Protector):** Code cầu chì từ chối lệnh nếu Khoảng cách SL vượt quá (100% / Leverage).
- [ ] **Dynamic Trailing Stop:** Thêm logic dời SL từ Tight (Hẹp) sang Loose (Rộng) theo phần trăm Lợi nhuận R:R hiện tại.
