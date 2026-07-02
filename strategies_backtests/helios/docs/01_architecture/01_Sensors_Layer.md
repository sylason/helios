# 01. Lớp Cảm biến (Sensors Layer)

Tài liệu phản ánh file `src/strategy/sensors.rs`.

## Chức năng
Lớp này là con mắt của Helios, nơi thu thập mọi tín hiệu kỹ thuật trước khi não bộ xử lý. Mọi dữ liệu ở đây là **dữ liệu thô (Raw Data)**.

## Cấu trúc dữ liệu
Struct `HeliosSensors` chứa 22 kênh dữ liệu. Để tối ưu tốc độ và bộ nhớ trong Rust, tất cả được lưu dưới dạng tham chiếu chia sẻ an toàn `Arc<Vec<f64>>`.

Các nhóm Cảm biến chính:
1. **Dữ liệu Nến:** `closes`
2. **Đo lường Biến động (Volatility):** `natr` (Normalized ATR), `chaos_ratio` (NATR chia cho đường trung bình của chính nó), `atr_sl` (dùng để tính Stop Loss).
3. **Đo lường Khối lượng:** `vol_ma` (Volume Moving Average), `mfi` (Money Flow Index).
4. **Đo lường Xu hướng:** 
   - `adx` (Độ mạnh xu hướng).
   - `linreg_slope` & `linreg_angle` (Góc dốc để phát hiện Parabolic).
   - `kama_mid` (Trục xu hướng chính).
5. **Đo lường Biên độ (Vùng nén):** `bb_upper/lower`, `kc_upper/lower`.
6. **Đo lường Động lượng (Momentum Triggers):** `rsi`, `macd_main/signal/hist`, `stoch_k/d/cross`.
7. **Đo lường Độ mượt:** `efficiency_ratio` (Nhiễu hay Tuyến tính).

> [!NOTE] 
> Tầng này chỉ chứa Dữ liệu, không có Logic tính toán. Việc tính toán Indicator được ủy thác hoàn toàn cho thư viện `ta` (Technical Analysis) nằm ở hàm `init()` trong `mod.rs`.
