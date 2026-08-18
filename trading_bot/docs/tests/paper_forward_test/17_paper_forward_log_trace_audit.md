# Báo Cáo Kiểm Toán Dấu Vết (Trace Audit): Khách Quan Dữ Liệu Log Paper Forward Test v3

- **Tác giả:** Phân tích viên Phân tích Vết Log (Research Subagent)  
- **Ngày lập:** 13/08/2026  
- **Đối tượng kiểm toán:** Nhật ký thực thi `trading_bot/bot_engine/examples/logs/paper_forward_v3.log`  
- **File báo cáo:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\tests\paper_forward_test\17_paper_forward_log_trace_audit.md`  

---

## 📌 Executive Summary (Tóm Tắt Kiểm Toán Khách Quan)

Báo cáo này tiến hành kiểm toán dấu vết (trace audit) 100% dữ liệu thực thi của `paper_forward_v3.log` nhằm làm rõ hai nghi vấn logic của người dùng:
1. **Nghi vấn A:** Liệu có lỗi logic làm phát sinh 27 lệnh giao dịch chỉ trong 1 ngày hay không?
2. **Nghi vấn B:** Liệu dòng log `[TRADE #001] Equity: $59.02` trong Periodic Report #5 có chứng minh vốn bị reset về $60.00 hay không?

---

## 1. 🔍 Kiểm Toán Mốc Thời Gian & Tần Suất Lệnh (Nghi vấn A)

### 1.1 Trích xuất Mốc Thời gian trong Log
- **Mốc khởi đầu In-Sample Train:** `2026-07-03 09:30 UTC`
- **Mốc kết thúc In-Sample Train / Bắt đầu Out-Of-Sample:** `2026-08-02 09:30 UTC`
- **Mốc Periodic Report #0005:** `2026-08-07 09:30 UTC` (Line 2144)
- **Mốc Periodic Report #0006:** `2026-08-08 09:30 UTC` (Line 2286)

### 1.2 Phân tích Tần suất
- Khoảng thời gian từ khi bắt đầu bơm nến Out-Of-Sample (02/08/2026) đến Periodic Report #0005 (07/08/2026) là **5 NGÀY THỰC THI CHẠY LIÊN TỤC** (tương đương 7,200 nến 1-phút).
- Trong 5 ngày đó, hệ thống thực hiện 27 lệnh giao dịch đã đóng.
- **Tần suất trung bình:** $27 / 5 = \mathbf{5.4\text{ lệnh / ngày}}$ (tương đương 1 lệnh mỗi 4.4 giờ).

👉 **Kết luận 1:** **KHÔNG CÓ LỖI LOGIC 27 LỆNH TRONG 1 NGÀY.** Tần suất 5.4 lệnh/ngày hoàn toàn bình thường và phản ánh đúng thực tế nến 1-phút.

---

## 2. 💰 Kiểm Toán Sổ Cái Tích Lũy Vốn `closed_trades` (Nghi vấn B)

### 2.1 Cơ chế Lưu trữ Snapshot Lịch sử của `PaperTrader`
Trong `paper_trader.rs`, mảng `closed_trades` được thiết kế dưới dạng **Append-Only History Log**:
```rust
self.closed_trades.push(PaperTradeRecord {
    id: self.total_trades,
    pos_type: old_position.clone(),
    entry_price: self.entry_price,
    exit_price: current_price,
    net_pnl: net_pnl_trade,
    pnl_pct,
    equity_after: self.equity, // SNAPSOT SỐ DƯ TẠI THỜI ĐIỂM ĐÓNG LỆNH ĐÓ!
});
```

### 2.2 Trace Tiến trình Vốn Thực tế Qua Chi Tiết Từng Lệnh

| Trade # | Thời điểm xảy ra | Vốn khởi tạo trước lệnh | Net PnL ($) | Equity sau lệnh (`equity_after`) | Trạng thái Sổ cái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Start**| Ngày 0 (02/08) | **$60.00** | $0.00 | **$60.00** | Khởi tạo tài khoản |
| **#001** | Ngày 1 (02/08) | $60.00 | -$0.98 | **$59.02** | Snapshot Ngày 1 |
| **#002** | Ngày 1 (02/08) | $59.02 | -$0.86 | **$58.16** | Snapshot Ngày 1 |
| **#003** | Ngày 2 (03/08) | $58.16 | +$2.15 | **$60.32** | Snapshot Ngày 2 |
| ... | ... | ... | ... | ... | ... |
| **#027** | Ngày 5 (07/08) | $58.04 | -$0.54 | **$57.50** | Snapshot Report #5 |
| **#028** | Ngày 6 (08/08) | $57.50 | -$1.46 | **$56.03** | Snapshot Ngày 6 |
| **#029** | Ngày 6 (08/08) | $56.03 | +$0.55 | **$56.58** | Snapshot Report #6 |

### 2.3 Phân tích Báo cáo Periodic Report #5
Khi Periodic Report #5 được gọi ở Ngày 5, phương thức `log_all_trades()` duy trì việc in lại **toàn bộ lịch sử từ Lệnh #001 đến #027**. 
- Dòng `[TRADE #001] ... Equity: $59.02` hiển thị số dư `$59.02` **chính là snapshot lịch sử của Ngày 1**, không phải vốn ở Ngày 5.
- Số dư ròng tại Ngày 5 báo về chính xác `Current: $57.61` (Line 2145).

👉 **Kết luận 2:** **SỐ DƯ VỐN KHÔNG BỊ RESET VỀ $60.00.** Sổ cái `closed_trades` ghi nhận chính xác 100% sự sụt giảm vốn tích lũy từ $60.00 $\rightarrow$ $56.58$.
