# PHÂN TÍCH SO SÁNH THỰC NGHIỆM: RUN 1 VS RUN 2 (WALK-FORWARD SMART SEED)
> **Ngày thực hiện:** 02/10/2026  
> **Hai lần chạy liên tiếp (Cold Start - Xóa sạch DB trước mỗi lần):**
> 1. **Run 1:** `run_walk_forward_smart_seed_20261002_223050`
> 2. **Run 2:** `run_walk_forward_smart_seed_20261002_224609`

---

## 🎯 BẢNG SO SÁNH TỔNG THỂ HIỆU SUẤT TÀI CHÍNH (OOS 8 NGÀY)

| Chỉ Số Đánh Giá | Run 1 (`223050`) | Run 2 (`224609`) | Mức Độ Chênh Lệch / Biến Động | Đánh Giá Độ Ổn Định |
| :--- | :---: | :---: | :---: | :---: |
| **Vốn Ban Đầu** | $60.00 | $60.00 | $0.00 | Chuẩn hóa |
| **Vốn Kết Thúc** | **$66.49** | **$54.58** | **-$11.91** | 🚨 **Bất ổn định cực độ** |
| **Lợi Nhuận Ròng (Net PnL)** | **+$6.49 (+10.82%)** | **-$5.42 (-9.03%)** | **-19.85%** | Lần 1 LÃI, Lần 2 LỖ NẶNG |
| **Tổng Số Lệnh Khớp** | **25 lệnh** | **40 lệnh** | **+15 lệnh (+60%)** | Run 2 cào lệnh quá mức |
| **Tần Suất Vào Lệnh** | 3.1 lệnh / ngày | **5.0 lệnh / ngày** | +60% tần suất | Run 2 rơi vào bẫy Scalping cào rác |
| **Số Lệnh Thắng vs Thua** | 12 Thắng / 13 Thua | 17 Thắng / 23 Thua | +10 lệnh thua thêm | Run 2 dính chuỗi cào lệnh thua |
| **Tỷ Lệ Thắng (Win Rate)** | **48.0%** | **42.5%** | **-5.5%** | Càng trade nhiều càng thua |
| **Số Lệnh Bị Kẹp SL Trần 15%**| **2 lệnh** | **15 lệnh** | **Gấp 7.5 lần!** | Run 2 đề xuất SL cực kỳ cẩu thả |
| **Thời Gian Né Bão Chaos** | 159 bars (2.6 giờ) | **525 bars (8.8 giờ)**| Gấp 3.3 lần | Thị trường bão nhiều hơn ở Run 2 |

---

## 🔍 CÁC VẤN ĐỀ TRỌNG TÂM CẦN ĐIỀU TRA

1. **Tại sao cùng 1 tập dữ liệu, xóa sạch DB lại ra 2 kết quả trái ngược?**
   - Sự phụ thuộc vào hạt giống ngẫu nhiên ban đầu (Latin Hypercube Sampling) trong không gian 102 chiều.
   - Hiện tượng Stochastic Overfitting và Local Minima Trap.
2. **Tại sao Run 2 lại bắn tới 40 lệnh (tăng 60%)?**
   - Hiện tượng nới lỏng bộ lọc kích hoạt tín hiệu trong DNA được tối ưu.
   - Hiện tượng cào lệnh liên tiếp (Chaining Re-entries) chỉ sau 1-2 nến dừng lỗ.
3. **Hiện tượng "Mua đỉnh bán đáy" và Cầu Dao Cooldown bị tê liệt:**
   - Tại sao vừa dính Stop Loss nến trước, nến sau lại vào lệnh cùng chiều tiếp tục dính SL?
   - Cầu Dao Cooldown đang ở đâu trong các pha này?
