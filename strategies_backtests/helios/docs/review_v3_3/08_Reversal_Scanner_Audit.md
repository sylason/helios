# HELIOS V3.3 ARCHITECTURE REVIEW: PART 8 - REVERSAL SCANNER AUDIT & REDUNDANT PARAMETERS LIST

Tài liệu này tổng hợp toàn bộ kết quả rà soát chi tiết **FILE #8: [`core_v3/scanners/reversal.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs)** và danh sách tinh giảm các biến dư thừa.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `REVERSAL`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Săn sóng kiệt sức đảo chiều hồi về trung bình (Mean Reversion Physics) khi giá bị kéo dãn cực đại chệch xa đường trung vị `kama_mid` ($\ge \text{min\_reversal\_displacement} \times \text{ATR}$).
- **Bản chất Tiềm năng**: Reversal không phải chiến thuật gồng Siêu Trend dài hạn mà nhằm mục đích **"Ăn nhanh rút gọn tại đường Trung Vị (`kama_mid`)"**, chốt lời an toàn tại mốc dãn đàn hồi tự nhiên của giá.
- **Target Take Profit**: Đặt chốt lời cố định tại giá `kama_mid` của nến mở lệnh ($P_0$), giúp lệnh Chốt lời Limit sẵn sàng khớp $100\%$ lợi nhuận tối ưu mà không bị trượt giá.

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `REVERSAL`

### A. Trang Bị Bộ Cảm Biến RSI Độc Lập Per-Playbook:
- Thay vì dùng chung `sensors.rsi` với các Playbook khác, `ReversalScanner` sở hữu bộ cảm biến riêng:
  - **`rsi_reversal_period`**: Chu kỳ RSI riêng cho kiệt sức đảo chiều.
  - **`rsi_reversal_tf`**: Khung thời gian riêng.

### B. Bảo Vệ 2 Tầng Chống Kẹt Lệnh & Dao Rơi:
1. **Tầng 1 (Lọc volume cạn kiệt)**: `volume_force <= exhaustion_vol_drop` (Bắt buộc khối lượng giao dịch cạn kiệt mới được bắt đảo chiều, tránh bắt dao rơi khi bão xả hàng).
2. **Tầng 2 (Lọc khoảng cách dãn chệch)**: `displacement >= min_reversal_displacement * ATR` (Giá phải chệch xa `kama_mid` một khoảng đủ lớn mới mở lệnh, triệt tiêu $100\%$ micro-spam 1 nến).
3. **Thoát lệnh linh hoạt nếu giá nửa đường đứt gánh**: Nếu giá quay đầu mà chưa tới được $TP = P_0$, Progressive Trailing SL Decay tự động siết lũy thừa để bảo vệ lợi nhuận và đóng lệnh mượt mà.

---

## ✂️ 3. TỔNG HỢP DANH SÁCH BIẾN DƯ THỪA ĐÃ TINH GIẢM TRONG FILE #8 (`reversal.rs` & `ReversalParams`)

| Tên Biến Cũ | Lý Do Tinh Giảm / Loại Bỏ | Giải Pháp Thay Thế V3 |
|---|---|---|
| `z_vs_rsi_weight` | Phép cộng trọng số gượng ép làm dài dải DNA | Xóa bỏ. Kiểm tra trực tiếp Z-Score và RSI cực đại |
| `rsi.period` dùng chung | Gây giằng co tham số với các Playbook khác | Thay bằng bộ cảm biến riêng **`rsi_reversal_period`** & **`rsi_reversal_tf`** |
