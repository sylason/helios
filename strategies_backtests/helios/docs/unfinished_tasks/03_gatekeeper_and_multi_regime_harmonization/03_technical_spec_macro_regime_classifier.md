# ĐẶC TẢ KỸ THUẬT: BỘ PHÂN LOẠI CHẾ ĐỘ THỊ TRƯỜNG VĨ MÔ & PHÂN LUỒNG PLAYBOOK TẠI ARBITER
## Mã Task: `TODO-AUDIT-NEW-115` (`H-GATE-02`)

> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Thư mục phân hệ:** `strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/`  
> **Module liên quan:** `core_v3/config.rs`, `core_v3/gatekeeper.rs`, `core_v3/arbiter.rs`  
> **Tham chiếu nền tảng:** [`01_gatekeeper_and_multi_regime_harmonization_blueprint.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/01_gatekeeper_and_multi_regime_harmonization_blueprint.md), [Note 23: Standard Quant Methodology Market Regimes](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/23_standard_quant_methodology_market_regimes.md)  
> **Trạng thái:** 🟢 SẴN SÀNG TRIỂN KHAI (APPROVED ARCHITECTURE)

---

## 1. Bối Cảnh Lỗ Hổng & Bản Chất Nghiệp Vụ

### 1.1. Lỗ Hổng Kiến Trúc Cũ (`TODO-AUDIT-NEW-115`)
Trong kiến trúc ban đầu của Helios V3:
1. `RegimeGatekeeper` chỉ đóng vai trò là một bộ lọc nhị phân Go/No-Go (`passed: bool`). Nó chỉ biết phát hiện bão hỗn loạn (`ChaosWhipsaw`) hoặc thị trường chết đứng thanh khoản (`MacroFlatline`, `MicroFlatline`).
2. Khi Gatekeeper mở cửa (`passed == true`), Arbiter vận hành trong tình trạng "mù chế độ thị trường" (Regime-Blind):
   - Cả 6 Tactical Scanners (`TrendPullback`, `MomentumIgnition`, `AbsorptionScout`, `RangeOscillator`, `ClimaxReversal`, `AnomalyFade`) đều được quét cào bằng và đưa vào sàn đấu thầu.
   - **Hậu quả thực tế:**
     - Khi thị trường có xu hướng mạnh dốc đứng (`Trending`), các playbook bắt đỉnh đáy (`ClimaxReversal`, `AnomalyFade`) vẫn đưa ra proposal ngược xu hướng và thậm chí thắng thầu do điểm raw score cao, gây ra hiện tượng "chặn đầu xe lửa / bắt dao rơi" làm cháy tài khoản.
     - Khi thị trường đi ngang nén biên độ (`RangeBound`), `TrendPullback` vẫn tìm kiếm các nhịp hồi giả tạo để mở lệnh và liên tục bị dính Stop Loss.

### 1.2. Nguyên Lý Giải Quyết Định Lượng (First-Principles)
Dựa trên mô hình trực giao 2x2 chuẩn Quant quốc tế (Directional Drift vs Volatility), thị trường được phân loại khách quan thành các chế độ vật lý độc lập:
1. **Thước đo trớn xu hướng phi thứ nguyên:** Sử dụng **Kaufman Efficiency Ratio ($KER$)** và **Welles Wilder $ADX$** lấy trực tiếp từ `sensors`. Đây là các đại lượng thuần túy đo lường cấu trúc giá, không bị lây nhiễm hay co giãn bởi bất kỳ gen DNA nào khác.
2. **Thước đo độ nén biến động:** Sử dụng $SqueezeIntensity \in [0.0, 1.0]$ từ `matrix` (tỷ lệ hình học giữa Bollinger Bands và Keltner Channel).
3. **Phân luồng Thẩm quyền Chọn lọc tại Arbiter (Selective Playbook Routing):** Arbiter tiếp nhận nhãn `MarketRegime` và loại bỏ ngay lập tức các scanner không phù hợp trước khi đưa vào đấu thầu.

---

## 2. Thiết Kế Kiến Trúc & Chi Tiết Triển Khai

### 2.1. Cấu Hình Siêu Tham Số: `MacroRegimeConfig`
Được định nghĩa trong `strategies_backtests/helios/src/strategy/core_v3/config.rs`:
```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MacroRegimeConfig {
    #[serde(default = "default_min_trending_er")]
    pub min_trending_er: f64,             // Mặc định 0.35 (Vượt ngưỡng nhiễu trắng ngẫu nhiên)
    #[serde(default = "default_min_trending_adx")]
    pub min_trending_adx: f64,            // Mặc định 25.0 (Ngưỡng xu hướng chuẩn Welles Wilder)
    #[serde(default = "default_range_squeeze_percentile")]
    pub range_squeeze_percentile: f64,    // Mặc định 0.65 (Dải nén chặt tích lũy)
    #[serde(default = "default_breakout_squeeze_percentile")]
    pub breakout_squeeze_percentile: f64, // Mặc định 0.35 (Dải bung nổ bứt phá)
    #[serde(default = "default_min_breakout_adx")]
    pub min_breakout_adx: f64,            // Mặc định 20.0 (Xung lực bùng nổ nến đầu)
}
```
* Bổ sung vào `HeliosConfigV3` với `#[serde(default)]` và `#[dna(skip)]` để **bảo toàn chính xác ngân sách 104 chiều DNA**.

### 2.2. Enum 6 Trạng Thái Khép Kín: `MarketRegime`
Được định nghĩa trong `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarketRegime {
    #[default]
    VolatilityFlatline, // 🛑 Tử địa thanh khoản (NATR < Sàn chi phí)
    TurbulentChaos,     // 🌪️ Bão biến động hỗn loạn (Chaos >= 0.70 hoặc Hysteresis)
    DataAnomaly,        // ⚠️ Dữ liệu bất thường hoặc chứa NaN/Inf
    Trending,           // 📈 Xu hướng bền vững có trớn (KER >= 0.35 hoặc ADX >= 25, không nén)
    RangeBound,         // 📦 Vùng dao động đi ngang / nén tích lũy (Squeeze >= 0.65 hoặc KER thấp & ADX < 20)
    Breakout,           // 🚀 Bứt phá nến đầu từ dải nén (Squeeze <= 0.35 và ADX >= 20, nhưng chưa đủ trớn Trend)
}
```

### 2.3. Quy Tắc Phân Loại Chế Độ Thị Trường (`classify_regime`)
Quy trình đánh giá theo thứ tự ưu tiên vật lý:
1. **Kiểm tra Veto An toàn:** Nếu `veto_reason` có giá trị $\implies$ Trả về `VolatilityFlatline`, `TurbulentChaos`, hoặc `DataAnomaly`.
2. **Kiểm tra NaN / Dữ liệu bất thường:** Nếu $KER$, $ADX$, hoặc $Squeeze$ là `NaN` $\implies$ Trả về `MarketRegime::DataAnomaly`.
3. **Ưu tiên kiểm tra Trớn Xu Hướng (`Trending`):**
   $$\text{is\_mature\_trend} = (KER \ge 0.35 \lor ADX \ge 25.0) \land Squeeze < 0.65$$
   Nếu đúng $\implies$ `MarketRegime::Trending`.
4. **Kiểm tra Bứt Phá Nổ Dải Nén (`Breakout`):**
   $$\text{is\_breakout} = Squeeze \le 0.35 \land ADX \ge 20.0$$
   Nếu đúng $\implies$ `MarketRegime::Breakout`.
5. **Mặc định Còn Lại:** `MarketRegime::RangeBound`.

---

## 3. Ma Trận Phân Luồng Thẩm Quyền Tại Arbiter (Selective Routing Matrix)

| Playbook Scanner | Trending (Xu Hướng) | RangeBound (Đi Ngang) | Breakout (Bứt Phá) | Veto Regimes (Chaos/Flatline/Anomaly) |
| :--- | :---: | :---: | :---: | :---: |
| **`TrendPullback`** | 🟢 **CHO PHÉP** (Đón nhịp hồi KAMA) | 🔴 **CẤM** (Không có trend) | 🔴 **CẤM** (Trend chưa hình thành) | 🛑 **VETO TOÀN BỘ** |
| **`MomentumIgnition`** | 🟢 **CHO PHÉP** (Đẩy sóng theo đà) | 🔴 **CẤM** (Bẫy phá vỡ giả) | 🟢 **CHO PHÉP** (Bắt cú nổ nến đầu) | 🛑 **VETO TOÀN BỘ** |
| **`AbsorptionScout`** | 🟢 **CHO PHÉP** (Hấp thụ nến hồi) | 🟢 **CHO PHÉP** (Hấp thụ biên range) | 🟢 **CHO PHÉP** (Hấp thụ thanh khoản nổ) | 🛑 **VETO TOÀN BỘ** |
| **`RangeOscillator`** | 🔴 **CẤM** (Chặn đầu xe lửa) | 🟢 **CHO PHÉP** (Đánh biên dải BB) | 🔴 **CẤM** (Chặn đầu xe lửa) | 🛑 **VETO TOÀN BỘ** |
| **`ClimaxReversal`** | 🔴 **CẤM** (Bắt dao rơi) | 🟢 **CHO PHÉP** (Bắt nến kiệt sức range) | 🔴 **CẤM** (Bắt dao rơi) | 🛑 **VETO TOÀN BỘ** |
| **`AnomalyFade`** | 🔴 **CẤM** (Chặn trend mạnh) | 🟢 **CHO PHÉP** (Hồi quy giá trị range) | 🔴 **CẤM** (Chặn trend mạnh) | 🛑 **VETO TOÀN BỘ** |

* Khi một Scanner bị cấm trong regime hiện tại:
  - Ghi nhận `AuctionStatus::RegimeIneligible(regime)` vào `auction_ledger`.
  - Không thêm vào `approved_proposals`, không tham gia sàn đấu thầu, bảo đảm an toàn $100\%$.

---

## 4. Danh Mục Ca Kiểm Thử Nghiệm Thu (Test Bench)

1. `test_regime_data_anomaly_on_nan`: Xác nhận `NaN` trả về `MarketRegime::DataAnomaly` và `passed == false`.
2. `test_regime_classification_trending`: $ADX = 30$, $ER = 0.50$, $Sqz = 0.20 \implies$ `MarketRegime::Trending`.
3. `test_regime_classification_breakout`: $Sqz = 0.25$, $ADX = 22$, $ER = 0.20 \implies$ `MarketRegime::Breakout`.
4. `test_regime_classification_range_bound`: $Sqz = 0.80$, $ADX = 14 \implies$ `MarketRegime::RangeBound`.
5. `test_arbiter_selective_routing_in_trending`: Xác nhận trong `Trending`, `TrendPullback` được duyệt, `RangeOscillator` và `ClimaxReversal` bị từ chối với `AuctionStatus::RegimeIneligible`.
6. `test_arbiter_selective_routing_in_range_bound`: Xác nhận trong `RangeBound`, `RangeOscillator` được duyệt, `TrendPullback` bị từ chối.
7. `test_dna_width_preserved_104`: Xác nhận `HeliosConfigV3::DNA_WIDTH == 104`.
