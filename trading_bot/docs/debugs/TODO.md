# 📋 Master TODO: Kế Hoạch Sửa Lỗi & Tối Ưu Hệ Thống `trading_bot`

**Thư mục quản trị:** `trading_bot/docs/debugs/`  
**Mục tiêu:** Đồng bộ hóa hoàn hảo 1:1 giữa Backtest (`ferro_trade`), Chiến lược (`helios`), và Thực thi (`trading_bot`).

---

## 🚀 Giai Đoạn 1: Đồng Bộ Vị Thế & Bảo Toàn Stop Loss (`05_position_sync_and_sl_invariance`)

- [x] **1.1.** Kế toán `bars_held` chuẩn xác theo `(exit_ts - entry_ts) / 60_000` trong `PaperTrader`.
- [x] **1.2.** Ánh xạ `mapped_entry_bar` bằng Binary Search $O(\log N)$ trên `entry_timestamp` trong `HeliosBridge`.
- [x] **1.3.** Ràng buộc `current_pnl > 0.0` trong `RiskManager::manage_trailing_stop` để chống bóp nghẹt Initial SL.
- [ ] **1.4.** Viết Unit Test xác thực tính bất biến của Initial SL khi vị thế đang âm.

---

## 🚀 Giai Đoạn 2: Xử Lý Điểm Nghẽn Giao Tiếp & MTF Streaming (`06_mtf_streaming_and_bridge_desync`)

- [x] **2.1. Sửa Lỗi Đóng Băng MTF trong `HeliosBridge`:**
  - Đồng bộ hóa `precompute_timeframes` chạy tươi mới trên toàn bộ cửa sổ nến mỗi tick, xóa bỏ bẫy cache bucket.
  - Viết test độc lập `test_fast_resample_parity.rs` đối chứng 1:1 độ lệch 0.0000000000 giữa Polars vs Pure-Rust.
- [x] **2.2. Đồng Bộ Hóa Vòng Đời Kiểm Tra Rủi Ro & Monotonic Ratchet Guard:**
  - Áp dụng Monotonic Ratchet Guard: SL chỉ dời theo chiều bảo vệ vốn (Tiến, cấm Lùi) trong `risk.rs`.
- [x] **2.3. Sửa Lỗi Phân Loại `ExitReason` trong `PaperTrader`:**
  - So sánh `current_sl_price` với `initial_sl_price` để ghi nhận chuẩn xác `ExitReason::TrailingStopLoss`.
- [x] **2.4. Phao Cứu Sinh Khóa Hòa Vốn (Break-Even Safety Trigger):**
  - Khóa hòa vốn + phí sàn khi lợi nhuận đạt $\ge +5\%$ ROE hoặc $\ge 1.5\times\text{ATR}$ trong `risk.rs`.

---

## 🚀 Giai Đoạn 3: Kiểm Tra Thực Nghiệm & Đo Lường Kết Quả (Verification)

- [x] **3.1.** Chạy `cargo check` & `cargo test` trên các dự án (Tất cả 24 unit test của `bot_engine` và 48 unit test của `ferro_trade` đều pass 100%).
- [ ] **3.2.** Chạy kịch bản Walk-Forward Out-Of-Sample (`walk_forward_smart_seed.rs`) để đối chiếu kết quả giữa Single Backtest Eval và Paper Trader.
- [ ] **3.3.** Ghi nhận kết quả và đóng các issue tương ứng trong tài liệu debugs.
