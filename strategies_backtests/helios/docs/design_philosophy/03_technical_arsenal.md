# Helios Technical Arsenal: Indicator Standard

Tài liệu này quy định các chỉ báo kỹ thuật được phép sử dụng trong Helios.
**Nguyên tắc:** Chỉ sử dụng những gì đã có sẵn trong thư viện Rust (`kand`, `yata`) hoặc tính toán cơ bản của `ferro_trade`. Không tự viết indicator phức tạp mới.

## 1. Primary Indicators (Từ Thư viện)

| Indicator | Implementation Source | Mục đích sử dụng |
| :--- | :--- | :--- |
| **KAMA** | `yata::methods::KAMA` | Xác định xu hướng thích ứng, lọc nhiễu (qua Efficiency Ratio). |
| **ATR** | `kand::ohlcv::atr` | Đo lường biến động, tính Stop Loss động. |
| **RSI** | `kand::ohlcv::rsi` | Đo động lượng, quá mua/quá bán. |
| **Stochastic** | `kand::ohlcv::stoch` | Động lượng ngắn hạn. |
| **ADX & DI** | `kand::ohlcv::adx` | Đo sức mạnh xu hướng. |
| **Bollinger Bands** | `kand::ohlcv::bbands` | Xác định biên độ dao động giá. |
| **MACD** | `kand::ohlcv::macd` | Xác định xu hướng và động lượng giao nhau. |
| **Linear Reg** | `yata::methods::LinearRegression` | (Dự kiến) Xác định kênh giá hồi quy. |

## 2. Derived Indicators (Tính toán Vector hóa)

Các chỉ báo này được tính bằng phép toán số học trên Series của Polars (không cần vòng lặp for).

*   **ATR Normalized:** `ATR / Close`. Dùng để so sánh biến động giữa các thời kỳ khác nhau.
*   **BBands Width Normalized:** `(Upper - Lower) / Middle`. Dùng để phát hiện Squeeze.
*   **Volume Ratio:** `Volume / VolumeSMA`. Xác định đột biến khối lượng.
*   **Efficiency Ratio (ER):** `KAMA` của `yata` lưu trạng thái nội bộ. Nếu không public, ta sẽ tính gần đúng bằng `abs(Close - Close[n]) / Sum(abs(Diff))`.

## 3. Composite Indicators (Tổ hợp)

*   **Keltner Channels:** Tổ hợp từ `EMA` (yata/kand) và `ATR` (kand).
*   **StochRSI:** Tổ hợp từ `RSI` -> `Stochastic`.

## 4. Quy tắc Triển khai

1.  **Batch Processing:** Tất cả indicator cùng khung thời gian (ví dụ 5m) phải được tính trong một closure `resample_apply` duy nhất.
2.  **No Custom Loops:** Tránh tối đa việc viết vòng lặp `for` để tính indicator thủ công. Hãy dùng hàm của thư viện.
3.  **Validation:** Mọi indicator sau khi tính xong phải được kiểm tra `NaN` (Warming up) trước khi sử dụng trong logic `next()`.
