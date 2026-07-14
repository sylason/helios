use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tracing::{warn, info, debug};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

/// Quản lý Request Weight dựa trên cơ chế Fixed Window của Binance.
/// Tự động reset mỗi khi bước sang phút mới (UTC).
#[derive(Debug)]
pub struct WeightMonitor {
    used_weight: AtomicU32,
    limit: u32,
    /// Thời điểm cập nhật cuối cùng (milliseconds)
    last_update_ms: AtomicU64,
}

impl WeightMonitor {
    pub fn new(limit: u32) -> Self {
        Self {
            used_weight: AtomicU32::new(0),
            limit,
            last_update_ms: AtomicU64::new(0),
        }
    }

    /// Cập nhật trọng số đã dùng từ HTTP Header `X-MBX-USED-WEIGHT-1M`.
    pub fn update(&self, used_weight: u32) {
        let now = self.now_ms();
        self.used_weight.store(used_weight, Ordering::SeqCst);
        self.last_update_ms.store(now, Ordering::SeqCst);

        let usage_pct = (used_weight as f32 / self.limit as f32) * 100.0;
        if usage_pct > 85.0 {
            warn!("🚨 API Load Critical: {}/{} ({}%)", used_weight, self.limit, usage_pct);
        } else if usage_pct > 60.0 {
            info!("⚠️ API Load Rising: {}/{} ({}%)", used_weight, self.limit, usage_pct);
        }
    }

    /// [CIRCUIT BREAKER] Kiểm tra và tạm dừng nếu sắp vượt ngưỡng an toàn.
    /// `required_weight`: Dự tính số weight mà request sắp tới sẽ tiêu tốn.
    pub async fn wait_if_needed(&self, required_weight: u32) {
        let current = self.get_current_weight();
        
        // Ngưỡng bảo vệ: 90% giới hạn
        if current + required_weight >= (self.limit as f32 * 0.9) as u32 {
            let ms_until_next_minute = 60000 - (self.now_ms() % 60000);
            warn!("🛑 Weight Limit protection activated. Waiting {}ms for reset...", ms_until_next_minute);
            sleep(Duration::from_millis(ms_until_next_minute as u64 + 100)).await; // Thêm 100ms trừ hao
        }
    }

    /// Lấy weight hiện tại, có tính đến việc tự động reset theo phút UTC.
    pub fn get_current_weight(&self) -> u32 {
        let now = self.now_ms();
        let last_update = self.last_update_ms.load(Ordering::Relaxed);
        
        // Logic Fixed Window: Nếu nến hiện tại đã sang phút mới so với lần update cuối
        if (now / 60000) > (last_update / 60000) {
            0
        } else {
            self.used_weight.load(Ordering::Relaxed)
        }
    }

    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn report_status(&self) {
        let current = self.get_current_weight();
        debug!("📊 Weight Monitor: {}/{} (Used: {}%)", current, self.limit, (current as f32 / self.limit as f32) * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_weight_update_and_get() {
        let monitor = WeightMonitor::new(100);
        monitor.update(50);
        assert_eq!(monitor.get_current_weight(), 50);
    }

    #[test]
    fn test_weight_reset_after_minute_passes() {
        let monitor = WeightMonitor::new(100);
        monitor.update(50);
        
        // Giả lập thời gian update cuối cùng là 2 phút trước
        let now = monitor.now_ms();
        monitor.last_update_ms.store(now - 120_000, Ordering::SeqCst);
        
        // Vì đã sang phút mới, weight hiện tại phải trả về 0
        assert_eq!(monitor.get_current_weight(), 0);
    }
}
