# Đặc tả Kỹ thuật (Technical Specification) V3

Tài liệu này mô tả chi tiết các Structs, Traits và cách luồng dữ liệu (Data Flow) di chuyển trong hệ thống Helios V3. Đây là bản vẽ kỹ thuật dành cho quá trình lập trình (Implementation).

## 1. Data Structures & Types

### 1.1. Signal Proposal
Mỗi Scanner sẽ trả về một `Proposal` nếu phát hiện cơ hội.

```rust
pub enum SignalType {
    Long,
    Short,
}

pub struct Proposal {
    /// Loại tín hiệu
    pub signal_type: SignalType,
    /// Điểm tự tin do Scanner tự chấm dựa trên độ hoàn hảo của Pattern (0.0 đến 1.0)
    pub raw_score: f64,
    /// Tên của Scanner (Dùng để logging và Arbiter theo dõi)
    pub scanner_id: &'static str,
}
```

### 1.2. The Arbiter's Decision
Quyết định cuối cùng của Trọng tài.

```rust
pub struct ArbiterDecision {
    pub is_approved: bool,
    pub final_signal: Option<SignalType>,
    pub winning_scanner: Option<&'static str>,
    pub final_confidence: f64,
}
```

## 2. Core Traits

Hệ thống Scanner phải được module hóa thông qua Trait để đảm bảo nguyên tắc Open/Closed (Có thể thêm Scanner mới mà không cần sửa code cũ).

```rust
pub trait TacticalScanner {
    /// Tên định danh của Scanner
    fn id(&self) -> &'static str;
    
    /// Hàm quét tín hiệu trên nến hiện tại
    /// Trả về None nếu không có cơ hội. Trả về Some(Proposal) nếu có tín hiệu.
    fn scan(&self, 
            tick: &TickContext, 
            sensors: &HeliosSensors, 
            matrix: &MarketMatrix) -> Option<Proposal>;
}
```

## 3. The Arbiter Logic (Core Engine)

Bộ não phân xử của Helios V3 được định nghĩa là một struct giữ danh sách các Scanners.

```rust
pub struct HeliosArbiter {
    scanners: Vec<Box<dyn TacticalScanner>>,
    config: ArbiterConfig,
}

impl HeliosArbiter {
    pub fn process_tick(&mut self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrix) -> ArbiterDecision {
        let mut best_proposal: Option<(Proposal, f64)> = None;
        let mut max_confidence = 0.0;
        
        // 1. Tính toán Regime Multipliers
        let trend_multiplier = matrix.trend_strength[tick.i];
        let squeeze_multiplier = matrix.squeeze_intensity[tick.i];
        let range_multiplier = 1.0 - trend_multiplier; // Đơn giản hóa
        
        // 2. Thu thập Proposal từ tất cả Scanner
        for scanner in &self.scanners {
            if let Some(proposal) = scanner.scan(tick, sensors, matrix) {
                // Tùy thuộc vào Scanner ID, áp dụng Multiplier tương ứng
                let multiplier = match scanner.id() {
                    "TREND_PB" => trend_multiplier,
                    "MOMENTUM_IGNITION" => squeeze_multiplier,
                    "RANGE_OSC" => range_multiplier,
                    _ => 1.0,
                };
                
                let confidence = proposal.raw_score * multiplier;
                
                // Chọn Proposal có niềm tin cao nhất
                if confidence > max_confidence {
                    max_confidence = confidence;
                    best_proposal = Some((proposal, confidence));
                }
            }
        }
        
        // 3. Phê duyệt (Approval Gate)
        if let Some((proposal, confidence)) = best_proposal {
            if confidence >= self.config.min_approval_confidence {
                return ArbiterDecision {
                    is_approved: true,
                    final_signal: Some(proposal.signal_type),
                    winning_scanner: Some(proposal.scanner_id),
                    final_confidence: confidence,
                };
            }
        }
        
        // Không có lệnh nào đủ tốt
        ArbiterDecision {
            is_approved: false,
            final_signal: None,
            winning_scanner: None,
            final_confidence: 0.0,
        }
    }
}
```

## 4. Execution Pipeline (Hàm `next`)

Toàn bộ quy trình trong hàm `next` của `HeliosStrategyV3`:

```rust
fn next(&mut self, tick: &TickContext) {
    // 1. Update Market Matrix & Memory
    
    // 2. Quản lý Rủi ro ưu tiên (Dời Stoploss / Đóng lệnh)
    if tick.position().is_open() {
        self.risk_manager.manage_trailing_stop(tick, ...);
        return; // Đã có lệnh thì tập trung quản lý, không tìm lệnh mới (hoặc có thể cho phép nhồi lệnh tùy chiến lược)
    }
    
    // 3. Nếu không có lệnh, gọi Arbiter tìm lệnh mới
    let decision = self.arbiter.process_tick(tick, &self.sensors, &self.matrix);
    
    if decision.is_approved {
        // 4. Tính toán Size và Entry
        let atr = self.sensors.atr_sl[tick.i];
        self.risk_manager.execute_order(decision, tick, atr);
    }
}
```

## 5. Kết luận
Thiết kế này biến Helios từ một cỗ máy "bằng mọi giá phải xác định đúng Regime" thành một hệ thống **Chấm điểm Niềm tin Đa mục tiêu (Multi-Objective Confidence Scoring)**. Rất linh hoạt, rất sát thực tế và tối đa hóa sức mạnh của các chỉ báo toán học trong `MarketMatrix`.
