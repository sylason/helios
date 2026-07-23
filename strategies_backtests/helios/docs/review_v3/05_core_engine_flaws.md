# Helios V3 Core Review: Core Engine (mod.rs & risk.rs)

## Review: `mod.rs` & Hàm `calculate_size`

### Quan sát & Phân tích
1. **Hardcode Đòn bẩy (Leverage) và Min Order:**
   Trong `mod.rs`, tại dòng 157, khi tính toán Position Size, hệ thống gọi hàm:
   ```rust
   let size = RiskManager::calculate_size(
       tick.equity, 
       self.config.risk_management.risk_base.percent_per_trade, 
       21.0, 
       20.0
   );
   ```
   Các con số `21.0` (Min Order USD) và `20.0` (Leverage) đang bị hardcode cứng vào mã nguồn. 
   - **Vấn đề & Mục tiêu tương lai:** Ta không muốn lưu Leverage vào biến môi trường `.env` (vì bảo mật) hay đặt cố định. Trong tương lai, khi hệ thống hoàn mỹ, ta sẽ bổ sung Playbook siêu cấp tên là **NUKE** (Đánh chắc thắng, xác suất cực cao). Khi NUKE kích hoạt, ta cần dỡ bỏ Leverage 20x hiện tại và bơm đòn bẩy cực đại (100x - 125x) để tối đa hóa lợi nhuận trong tích tắc. Sau khi NUKE chốt lời, hệ thống phải tự động trả Leverage về mức Normal (ví dụ 20x) cho các Playbook thông thường.
   - **Đề xuất:** Cấu trúc lại hàm `calculate_size` và cơ chế `Proposal`. Mỗi `Scanner` (hoặc `Playbook`) sẽ được quyền đính kèm một `requested_leverage` bên cạnh điểm số tín hiệu. Arbiter khi duyệt tín hiệu sẽ chuyển Leverage yêu cầu này xuống `mod.rs` để điều khiển `ferro_trade` mô phỏng việc Set Leverage/Ký quỹ linh hoạt theo đúng cấp độ của Playbook đó.

2. **Chế độ Single-Position (Chặn nhồi lệnh):**
   Tại dòng 139 trong `mod.rs`:
   ```rust
   if pos.is_open() {
       return; 
   }
   ```
   Hệ thống đang hoạt động dưới dạng Single-Position (chỉ mở 1 lệnh duy nhất tại 1 thời điểm).
   - **Đánh giá:** Đây là một thiết kế **tốt và an toàn** cho một Tactical Sniper Bot (phong cách "One shot, One kill"). Tuy nhiên, nếu sau này muốn phát triển cơ chế Nhồi lệnh (Pyramiding) khi Trend mạnh, ta sẽ phải cấu trúc lại đoạn này để cho phép mở thêm vị thế. Tạm thời, nên giữ nguyên để đảm bảo độ rủi ro thấp.
