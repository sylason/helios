# Sự Mất Cân Bằng Của Các Trọng Số Regime (Matrix Multipliers)

Trong `core_v3/matrix.rs`, kiến trúc v3 đưa ra một ý tưởng xuất sắc: tính toán các biến số nhân (multipliers) liên tục từ 0.0 đến 1.0 cho từng Market Regime để hỗ trợ Arbiter chấm điểm tín hiệu.

## 🚨 Vấn đề Hiện Tại
Cách tính toán hiện tại đang làm sai lệch trọng lượng của các regime, đặc biệt là `trend_multiplier` và `range_multiplier`.

```rust
// Trong matrix.rs (v3)
trend_strength.push(sensors.adx[i] / 100.0);
let t_mul = trend_strength[i].max(0.0).min(1.0);
trend_multiplier.push(t_mul);
range_multiplier.push(1.0 - t_mul);
```

### Phân tích Lỗ hổng:
1. **Bản chất của ADX:** Chỉ số ADX (Average Directional Index) có thang điểm lý thuyết từ 0 đến 100, nhưng trong thực tế thị trường, ADX rất hiếm khi vượt quá mức `60.0`. Phần lớn thời gian ADX dao động ở mức `15.0 - 40.0`.
2. **Sự Thiên Vị Của Phép Chia:** Bằng cách chia trực tiếp `ADX / 100.0`, giá trị của `trend_multiplier` bị nén xuống chỉ còn trung bình khoảng `0.2` đến `0.4`.
3. **Phản Ứng Dây Chuyền Lên Range:** Vì `range_multiplier = 1.0 - t_mul`, trọng số của phe Range sẽ luôn tự động nhận được điểm số cực kỳ cao, trung bình từ `0.6` đến `0.8`.

**Hậu Quả:** Arbiter sẽ bị "tẩy não" và liên tục thiên vị các đề xuất từ `RangeOscScanner`, ngay cả khi thị trường đang có xu hướng rõ ràng, vì trọng số của Trend không bao giờ cạnh tranh lại Range.

## 🛠️ Giải Pháp Đề Xuất
Phục hồi lại kỹ thuật chuẩn hóa bằng `rolling_percentile` từ V2:
```rust
let adx_percentile = normalization::rolling_percentile(&sensors.adx, config.matrix.trend_lookback);
```
Kỹ thuật này sẽ biến ADX thành một thang đo tỷ lệ phần trăm phân phối chuẩn từ 0.0 đến 1.0 thực thụ (Bất chấp giá trị tuyệt đối của ADX là bao nhiêu, miễn là nó nằm ở đỉnh của chu kỳ gần nhất thì nó là 1.0). Từ đó `trend_multiplier` và `range_multiplier` mới tạo ra sự cạnh tranh công bằng.
