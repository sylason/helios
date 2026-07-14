use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{warn, error};


/// Ngưỡng cảnh báo (80%). Khi vượt qua, nên hạn chế các lệnh gọi không cần thiết.
const WARNING_THRESHOLD: u32 = 1920;

/// Ngưỡng khẩn cấp (92%). Khi vượt qua, phải đình chỉ hoàn toàn việc gọi REST API.
const EMERGENCY_THRESHOLD: u32 = 2200;

/// Thời gian đình chỉ khi chạm ngưỡng khẩn cấp (giây).
const EMERGENCY_SLEEP_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct WeightMonitor {
    used_weight: Arc<AtomicU32>,
}

impl WeightMonitor {
    /// Khởi tạo một Monitor mới.
    pub fn new() -> Self {
        Self {
            used_weight: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Lấy giá trị weight hiện tại.
    pub fn current_weight(&self) -> u32 {
        self.used_weight.load(Ordering::Acquire)
    }

    /// Cập nhật weight sau khi nhận phản hồi từ Binance.
    /// Binance trả về tổng weight đã dùng trong 1 phút qua thông qua header `x-mbx-used-weight-1m`.
    pub fn update_from_header(&self, weight: u32) {
        // Vì có thể có nhiều luồng (thread) cùng gọi API, ta lưu giá trị lớn nhất
        // để đảm bảo luôn dùng con số an toàn nhất. Tuy nhiên Binance tự reset về 0 mỗi phút,
        // nên nếu giá trị mới nhỏ hơn nhiều (ví dụ từ 2000 rớt xuống 10), nghĩa là Binance
        // đã sang chu kỳ phút mới, ta cần cập nhật trực tiếp.
        
        let current = self.current_weight();
        
        // Nếu giá trị trả về nhỏ hơn hẳn (ví dụ chênh lệch > 100), có khả năng đã sang chu kỳ 1 phút mới.
        // Ngược lại, nếu trong cùng chu kỳ, ta giữ lại giá trị cao hơn.
        if weight < current && current.saturating_sub(weight) > 100 {
            self.used_weight.store(weight, Ordering::Release);
        } else {
            // Giữ giá trị lớn nhất trong chu kỳ
            self.used_weight.fetch_max(weight, Ordering::Release);
        }
    }

    /// Trạng thái có an toàn để gọi REST API không (chưa tới mức Emergency).
    pub fn is_safe(&self) -> bool {
        self.current_weight() < EMERGENCY_THRESHOLD
    }

    /// Cảnh báo nếu sắp chạm ngưỡng.
    pub fn is_warning(&self) -> bool {
        let w = self.current_weight();
        w >= WARNING_THRESHOLD && w < EMERGENCY_THRESHOLD
    }

    /// Kiểm tra và tự động chặn luồng (ngủ) nếu chạm ngưỡng khẩn cấp.
    /// Trả về `true` nếu đã phải ngủ (để báo hiệu cho logic bên ngoài biết có sự cố).
    pub async fn check_and_sleep_if_emergency(&self) -> bool {
        let weight = self.current_weight();
        
        if weight >= EMERGENCY_THRESHOLD {
            error!(
                "🔥 BÁO ĐỘNG ĐỎ: Binance REST Rate Limit đã chạm {}. ĐÌNH CHỈ GỌI REST TRONG {} GIÂY!",
                weight, EMERGENCY_SLEEP_SECS
            );
            
            sleep(Duration::from_secs(EMERGENCY_SLEEP_SECS)).await;
            
            // Giả định sau khi ngủ, chu kỳ phút của Binance đã được reset.
            // Để an toàn, set về 0 để mở khóa. Lần gọi REST tiếp theo sẽ tự sửa giá trị này.
            self.used_weight.store(0, Ordering::Release);
            return true;
        } else if weight >= WARNING_THRESHOLD {
            warn!(
                "⚠️ CẢNH BÁO: Binance REST Rate Limit đang cao ({}). Hạn chế gọi API rác.",
                weight
            );
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weight_update_logic() {
        let monitor = WeightMonitor::new();

        // Cập nhật bình thường
        monitor.update_from_header(100);
        assert_eq!(monitor.current_weight(), 100);

        // Cập nhật giá trị lớn hơn
        monitor.update_from_header(150);
        assert_eq!(monitor.current_weight(), 150);

        // Cập nhật giá trị nhỏ hơn lắt nhắt (do response bị out-of-order)
        // Không nên làm tụt weight
        monitor.update_from_header(140);
        assert_eq!(monitor.current_weight(), 150);

        // Đột ngột rớt mạnh (Binance qua phút mới)
        monitor.update_from_header(5);
        assert_eq!(monitor.current_weight(), 5);
    }

    #[tokio::test]
    async fn test_thresholds() {
        let monitor = WeightMonitor::new();

        // Safe
        monitor.update_from_header(1000);
        assert!(monitor.is_safe());
        assert!(!monitor.is_warning());

        // Warning
        monitor.update_from_header(1950);
        assert!(monitor.is_safe());
        assert!(monitor.is_warning());

        // Emergency
        monitor.update_from_header(2250);
        assert!(!monitor.is_safe());
        assert!(!monitor.is_warning()); // is_warning trả false nếu >= Emergency
    }
}
