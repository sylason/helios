# 💬 BIÊN BẢN THẢO LUẬN: ĐỒNG BỘ DỮ LIỆU BẰNG SNAPSHOT & DELTA EVENTS

- **Thời gian:** 2026-09-09
- **Chủ đề:** Cơ chế truyền tin thời gian thực giữa Rust Core (`src-tauri`) và Giao diện React.
- **Trạng thái:** ✅ **ĐÃ THỐNG NHẤT (FINAL DECISION)**

---

## 📌 1. Bối Cảnh & Vấn Đề

Cần đồng bộ dữ liệu (trạng thái bot, giá nến, lãi/lỗ uPnL, mức Trailing SL, nhật ký log) giữa lõi tính toán Rust và màn hình hiển thị React sao cho đạt hiệu năng cao nhất, không gây đơ giật UI và không ngốn tài nguyên CPU của máy tính.

---

## 🎯 2. Quyết Định Kỹ Thuật (Decision Log)

1. **Thống nhất:** Áp dụng mô hình **Snapshot + Delta Event Stream**.
2. **Quy trình hoạt động:**
   - **Giai đoạn Khởi động (Initial Snapshot):** React gọi lệnh `cmd_get_full_snapshot` đúng 1 lần khi vừa mở giao diện để lấy toàn bộ dữ liệu hiện có (Account state, Health, Danh sách Asset).
   - **Giai đoạn Vận hành (Delta Events):** Rust Core chỉ phát các sự kiện nhỏ khi có thay đổi thực tế thông qua Tauri Event Emitter:
     - `event_asset_state_updated`: Khi bot chuyển trạng thái FSM (`INACTIVE` $\rightarrow$ `STARTING` $\rightarrow$ `RUNNING_ACTIVE`...).
     - `event_position_updated`: Khi vị thế hoặc mức Trailing SL thay đổi.
     - `event_health_tick`: Khi có cập nhật về API weight hoặc ping latency.
     - `event_toast_notification`: Khi có thông báo lỗi cô lập từ sàn.
3. **Lợi ích:** Tiết kiệm tối đa CPU (gần như 0% khi thị trường đi ngang), cập nhật tức thì <1ms khi có biến động nến.
