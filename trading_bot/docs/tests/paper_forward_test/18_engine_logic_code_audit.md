# Báo Cáo Phân Tích Mã Nguồn: Kiểm Toán Logic Thực Thi & Luồng Nến Trong Bot Engine (`paper_forward_test.rs`)

- **Tác giả:** Phân tích viên Mã Nguồn Engine (Research Subagent)  
- **Ngày lập:** 13/08/2026  
- **Đối tượng kiểm toán:** 
  - `trading_bot/bot_engine/examples/paper_forward_test.rs`
  - `trading_bot/bot_engine/src/execution/paper_trader.rs`
  - `trading_bot/bot_engine/src/controller/asset_actor.rs`
  - `trading_bot/bot_engine/src/optimizer/prospector_runner.rs`
- **File báo cáo:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\tests\paper_forward_test\18_engine_logic_code_audit.md`  

---

## 📌 Executive Summary (Tóm Tắt Kiểm Toán Mã Nguồn & Sửa Lỗi)

Báo cáo này tiến hành kiểm toán đối chiếu từng dòng mã nguồn (line-by-line code audit) của hạ tầng `bot_engine` để xác minh luồng lặp nến, kế toán vốn và trật tự quét SL/TP.

---

## 1. 🔍 Kiểm Toán Luồng Lặp Nến Out-Of-Sample trong `paper_forward_test.rs`

- **Vòng lặp nến `for candle in &oos_candles`:** Khi Re-Opt chạy dưới dạng `.await` đồng bộ, luồng lặp nến tạm dừng, nạp DNA mới rồi **tiếp tục lặp nến kế tiếp ($t+1$) mà KHÔNG BỎ NHẢY BẤT KỲ NẾN NÀO**.
- **Khẳng định:** Toàn bộ 14,386 nến Out-Of-Sample được bơm 100% đầy đủ vào `AssetActor`.

---

## 2. 💰 Kiểm Toán Logic Kế Toán Vốn Trong `PaperTrader`

- Biến `self.equity` được bảo tồn liên tục qua tất cả các chu kỳ Re-Opt.
- **Không có lỗi reset vốn trong code.** `PaperTrader` bảo tồn và tích lũy vốn ròng hoàn toàn chính xác từ $60.00 $\rightarrow$ $56.58$.

---

## 3. 🚨 Phát Hiện & Đã Sửa Lỗi Trật Tự Quét Trailing SL Trong `AssetActor`

- **Phát hiện lỗi Lookahead Trailing SL:** Trước đây, trong `asset_actor.rs`, BƯỚC 3.1 (`evaluate_live_tick`) chạy TRƯỚC BƯỚC 3.2 (`paper_trader.on_candle`). Điều này khiến giá Close của nến $T$ dời Trailing SL lên mức mới rồi mới dùng mức SL mới đó quét giá Low của chính nến $T$ (vi phạm tính nhân quả thời gian).
- **Đã sửa đổi (Fixed):** Đã đảo ngược trật tự xử lý trong `AssetActor::on_new_candle`:
  1. Quét Trailing SL hiện tại bằng High/Low của nến $T$ (**BƯỚC 3.1** - `paper_trader.on_candle`).
  2. Đánh giá chiến lược và dời Trailing SL cho nến tiếp theo (**BƯỚC 3.2** - `evaluate_live_tick`).
- **Xác minh:** Biên dịch `cargo check --example paper_forward_test` thành công 100%.
