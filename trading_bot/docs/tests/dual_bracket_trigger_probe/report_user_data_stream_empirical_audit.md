# Báo Cáo Thực Nghiệm Chuyên Sâu: Kiểm Chứng Toàn Diện User Data Stream (Binance Futures)

> **Thời gian thực nghiệm:** 2026-09-22 16:30:53 UTC  
> **Môi trường:** Binance Futures USDⓈ-M Testnet  
> **Cặp tiền:** `ETHUSDT`  
> **File JSON lưu trữ dữ liệu thô:** [`trading_bot/bot_engine/examples/logs/dual_bracket_runs/run_20260922_163003.json`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/logs/dual_bracket_runs/run_20260922_163003.json)  
> **File JSON latest:** [`trading_bot/bot_engine/examples/logs/dual_bracket_runs/latest.json`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/logs/dual_bracket_runs/latest.json)  
> **Mã nguồn thực nghiệm:** [`trading_bot/bot_engine/examples/binance_dual_bracket_trigger_probe.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/binance_dual_bracket_trigger_probe.rs)

---

## 🎯 1. Mục Tiêu & Kịch Bản Kiểm Chứng

Kiểm chứng thực tế 100% bằng dữ liệu socket thời gian thực:
1. **User Data Stream phản hồi ra sao trong suốt vòng đời lệnh?** (Mở lệnh Market, đặt song song SL/TP, liên tục dời Trailing SL, đóng lệnh).
2. **Khi vị thế bị đóng, User Data Stream bắn những sự kiện gì?**
3. **User Data Stream có bắn sự kiện `ALGO_UPDATE` thông báo lệnh Take Profit còn lại bị `EXPIRED` hay không?**

---

## 🔬 2. Bảng Phân Tích Sự Kiện User Data Stream Thực Tế (30 Events)

Trong suốt 47 giây chạy bài test, User Data Stream đã thu thập được **chính xác 30 sự kiện** bắn về từ Binance theo thứ tự thời gian:

| Thứ Tự | Loại Sự Kiện (`event_type`) | Trường Dữ Liệu Trọng Yếu Từ Binance | Ý Nghĩa Kỹ Thuật Đối Với Bot Engine |
| :--- | :--- | :--- | :--- |
| **#1** | **`TRADE_LITE`** | `L: "2733.29", S: "BUY", q: "0.010"` | Thông báo khớp lệnh siêu nhanh cấp micro-giây. |
| **#2** | **`ORDER_TRADE_UPDATE`** | `X: "NEW", S: "BUY", i: 16802164451` | Lệnh Market Buy vừa được hệ thống tạo. |
| **#3** | **`ACCOUNT_UPDATE`** | `pa: "0.01", ep: "2733.29", wb: "5285.8399"` | **Vị thế Long 0.01 ETH đã mở thành công.** Cập nhật ngay số dư và entry price vào RAM! |
| **#4** | **`ORDER_TRADE_UPDATE`** | `X: "FILLED", ap: "2733.29", n: "0.0109 USDT"` | Lệnh Market Buy đã khớp hoàn toàn, ghi nhận phí sàn 0.0109 USDT. |
| **#5** | **`ALGO_UPDATE`** | `aid: 1000000214617944, o: "STOP_MARKET", X: "NEW", tp: "2678.62"` | Binance xác nhận đặt lệnh Trailing SL ban đầu (-2%). |
| **#6** | **`ALGO_UPDATE`** | `aid: 1000000214617947, o: "TAKE_PROFIT_MARKET", X: "NEW", tp: "2787.96"` | Binance xác nhận đặt lệnh Take Profit ban đầu (+2%). |
| **#7 - #24** | **Chuỗi `ALGO_UPDATE` (Dời SL)** | `X: "CANCELED"` (cho SL cũ) $\rightarrow$ `X: "NEW"` (cho SL mới áp sát) | Mỗi khi bot dời SL, Binance bắn ngay 2 sự kiện: Hủy SL cũ và Kích hoạt SL mới. |
| **#25** | **`ORDER_TRADE_UPDATE`** | `X: "NEW", S: "SELL", i: 16802164911` | Lệnh Market Sell đóng vị thế được tạo. |
| **#26** | **`ACCOUNT_UPDATE`** | `pa: "0", ep: "0", up: "0", wb: "5285.8172"` | **CHÂN LÝ VỊ THẾ:** Vị thế lập tức về đúng `0.000 ETH`! |
| **#27** | **`ORDER_TRADE_UPDATE`** | `X: "FILLED", ap: "2732.11", rp: "-0.0118", n: "0.0109"` | Lệnh Market Sell đã khớp xong, ghi nhận PnL thực tế và phí sàn. |
| **#28** | **`ALGO_UPDATE`** | `aid: 1000000214618734, o: "STOP_MARKET", X: "EXPIRED"` | **Sàn tự động EXPIRED lệnh SL vì vị thế đã về 0!** |
| **#29** | **`ALGO_UPDATE`** | `aid: 1000000214617947, o: "TAKE_PROFIT_MARKET", X: "EXPIRED"` | **🎯 BẰNG CHỨNG CỐT TỬ: Sàn tự động EXPIRED lệnh Take Profit còn lại và bắn socket về bot!** |

---

## 💡 3. Trích Dẫn Trực Tiếp Payload JSON (Raw Payloads)

### 📌 A. Khi Vị Thế Đóng: `ACCOUNT_UPDATE`
```json
{
  "event_type": "ACCOUNT_UPDATE",
  "payload": {
    "E": 1790094653160,
    "a": {
      "B": [{ "a": "USDT", "wb": "5285.81721256" }],
      "P": [{ "s": "ETHUSDT", "pa": "0", "ep": "0", "up": "0" }],
      "m": "ORDER"
    }
  },
  "received_at_iso": "2026-09-22T16:30:51.724522500+00:00"
}
```

### 📌 B. Khi Khớp Lệnh Đóng: `ORDER_TRADE_UPDATE`
```json
{
  "event_type": "ORDER_TRADE_UPDATE",
  "payload": {
    "E": 1790094653160,
    "o": {
      "S": "SELL",
      "X": "FILLED",
      "ap": "2732.11",
      "i": 16802164911,
      "l": "0.01",
      "n": "0.01092844",
      "rp": "-0.0118",
      "s": "ETHUSDT"
    }
  },
  "received_at_iso": "2026-09-22T16:30:51.724901600+00:00"
}
```

### 📌 C. Bằng Chứng Sống Còn: Lệnh Take Profit Bị `EXPIRED` Được Bắn Về Socket
```json
{
  "event_type": "ALGO_UPDATE",
  "payload": {
    "E": 1790094653167,
    "o": {
      "aid": 1000000214617947,
      "o": "TAKE_PROFIT_MARKET",
      "X": "EXPIRED",
      "s": "ETHUSDT",
      "tp": "2787.96",
      "cp": true
    }
  },
  "received_at_iso": "2026-09-22T16:30:51.725313200+00:00"
}
```

---

## 🏆 4. Kết Luận Khoa Học & Định Hướng Kiến Trúc

1. **User Data Stream là "Mắt Thần" Tuyệt Đối:**
   - Khi vị thế đóng, 3 sự kiện (`ACCOUNT_UPDATE`, `ORDER_TRADE_UPDATE`, `ALGO_UPDATE`) được Binance bắn về socket **cùng lúc chỉ cách nhau 0.7 mili-giây**!
   - Không cần bất kỳ lệnh polling REST API nào, bot đã nắm trọn vẹn:
     + Vị thế hiện tại là bao nhiêu (`pa: "0"`).
     + Lợi nhuận thực tế là bao nhiêu (`rp: "-0.0118"`).
     + Phí sàn là bao nhiêu (`n: "0.01092844"`).
     + Lệnh Take Profit còn lại đã bị hủy an toàn chưa (`X: "EXPIRED"`).
2. **Tiết Kiệm 100% API Weight:**
   - Hệ thống có thể chuyển hoàn toàn việc giám sát tài khoản sang User Data Stream, bảo toàn trọn vẹn hạn mức 2400 weight cho việc gửi lệnh.
