# ĐẶC TẢ KỸ THUẬT: CỔNG PRE-TRADE HỢP NHẤT & TÁCH BẠCH DNA VI MÔ
## Mã Task: `TODO-AUDIT-NEW-24` (`NEW-NATR-4H-BLIND`)

> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Thư mục phân hệ:** `strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/`  
> **Module liên quan:** `core_v3/config.rs`, `sensors.rs`, `core_v3/mod.rs`, `core_v3/gatekeeper.rs`, `core_v3/telemetry/defense_ledger.rs`  
> **Tham chiếu nền tảng:** [`01_gatekeeper_and_multi_regime_harmonization_blueprint.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/01_gatekeeper_and_multi_regime_harmonization_blueprint.md), [Note 07: Micro vs Macro Standards](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/07_micro_vs_macro_timeframe_quant_standards.md)  
> **Trạng thái:** 🟢 SẴN SÀNG TRIỂN KHAI (APPROVED ARCHITECTURE)

---

## 1. Bối Cảnh Lỗ Hổng & Bản Chất Nghiệp Vụ

### 1.1. Lỗ Hổng Cũ (`NEW-NATR-4H-BLIND`)
Tại `gatekeeper.rs`, chỉ báo đo biến động để kích hoạt Cầu Dao Flatline đọc từ `sensors.natr`. Chỉ báo này thuộc khung thời gian vĩ mô do GA tự do tối ưu (`NAtrParams.timeframe` trong dải `15m, 1h, 4h`).
Khi GA chọn khung thời gian $4\text{h}$:
* $\text{NATR}_{4\text{h}}$ của Bitcoin/Ethereum luôn dao động trong khoảng $0.80\% \dots 2.50\%$.
* Biểu thức so sánh:
  $$\text{market\_natr\_pct} \ge 0.80\% \gg 0.10\% (\text{phí 2 chiều}) + 0.12\% (\text{min\_alpha}) = 0.22\%$$
* **Hậu quả:** Khi thị trường vi mô rơi vào trạng thái que tăm (nến 1m/3m/5m tê liệt thanh khoản, biên độ $< 0.02\%$, spread nuốt trọn giá), Cầu Dao Flatline bị **MÙ HOÀN TOÀN**. Bot mở lệnh trong điều kiện trượt giá và spread nuốt trọn biên độ, dẫn đến thua lỗ nặng nề.

### 1.2. Nguyên Lý Giải Quyết (First-Principles)
Tách bạch 2 hiện tượng vật lý độc lập:
1. **Dung lượng sóng vĩ mô toàn vòng đời (Macro Regime Capacity):** Cần đảm bảo biến động nến khung lớn đủ bù đắp phí 2 chiều và biên lợi nhuận kỳ vọng ròng ($\text{Roundtrip Fee} + \text{min\_alpha} \approx 0.22\%$).
2. **Thanh khoản khớp lệnh tức thời vi mô (Micro Execution Liquidity):** Cần đảm bảo nến khớp lệnh hiện tại không phải là nến que tăm, biên độ tối thiểu phải vượt qua chi phí vào lệnh 1 chiều ($\text{Taker Fee} = 0.04\%$).

---

## 2. Thiết Kế Kiến Trúc & Chi Tiết Triển Khai

### 2.1. Cấu hình DNA Độc lập: `MicroNAtrParams`
Đặt tại `strategies_backtests/helios/src/strategy/core_v3/config.rs`:
```rust
#[derive(Debug, Clone, Deserialize, DnaMappableMacro)]
pub struct MicroNAtrParams {
    #[dna(min = 5.0, max = 30.0, step = 1.0)]
    #[serde(deserialize_with = "ferro_trade::optimizer::config_parser::deserialize_usize_from_f64")]
    pub period: usize,
    #[dna(subset = "1m, 3m, 5m")]
    pub timeframe: Timeframe,
}

impl Default for MicroNAtrParams {
    fn default() -> Self {
        Self {
            period: 14,
            timeframe: Timeframe::T1m,
        }
    }
}
```
* Bổ sung vào `IndicatorsConfig`:
```rust
    #[dna(nest)]
    pub atr: AtrParams,
    #[dna(nest)]
    pub natr: NAtrParams,
    #[dna(nest)]
    #[serde(default)]
    pub micro_natr: MicroNAtrParams,
```

### 2.2. Tầng Cảm Biến: `HeliosSensors` & Khởi Tạo
* Tại `strategies_backtests/helios/src/strategy/sensors.rs`:
```rust
    pub natr: Arc<Vec<f64>>,
    pub micro_natr: Arc<Vec<f64>>,
    pub chaos_ratio: Arc<Vec<f64>>,
```
* Tại `strategies_backtests/helios/src/strategy/core_v3/mod.rs` (hàm `init`):
```rust
    s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
    s.micro_natr = ctx.natr(ind.micro_natr.timeframe, "NATR_Micro", ind.micro_natr.period);
```

### 2.3. Cổng Pre-Trade: `gatekeeper.rs`
* Cập nhật enum `GateVetoReason`:
  * `GateVetoReason::MacroFlatline` $\to$ `"MACRO_FLATLINE"`
  * `GateVetoReason::MicroFlatline` $\to$ `"MICRO_FLATLINE"`
* Mở rộng `GateEvaluation` lưu trữ:
  * `pub micro_natr_pct: f64`
  * `pub micro_fee_floor_pct: f64`
* Hai tầng kiểm tra:
  ```rust
  // Tầng 1: Vĩ mô
  if market_natr_pct < required_min_natr_pct {
      return GateEvaluation { veto_reason: Some(GateVetoReason::MacroFlatline), ... };
  }
  // Tầng 2: Vi mô
  let micro_fee_floor_pct = (risk_base.taker_fee_rate * 100.0).max(1e-4);
  if micro_natr_pct < micro_fee_floor_pct {
      return GateEvaluation { veto_reason: Some(GateVetoReason::MicroFlatline), ... };
  }
  ```

### 2.4. Tầng Telemetry: `defense_ledger.rs`
* Ghi nhận riêng biệt và log chi tiết cho từng lý do Veto:
  * `MacroFlatline`: in `Macro NATR` vs `Required NATR`.
  * `MicroFlatline`: in `Micro NATR` vs `MicroFeeFloor`.

---

## 3. Tiêu Chuẩn Nghiệm Thu & Ma Trận Kiểm Thử

1. **Test 1 (`test_gatekeeper_micro_flatline_veto`):**
   * $\text{NATR}_{macro} = 1.50\%$, $\text{NATR}_{micro} = 0.02\% < 0.04\% \implies$ `veto_reason == Some(GateVetoReason::MicroFlatline)`.
2. **Test 2 (`test_gatekeeper_macro_flatline_veto`):**
   * $\text{NATR}_{micro} = 0.08\%$, $\text{NATR}_{macro} = 0.12\% < 0.22\% \implies$ `veto_reason == Some(GateVetoReason::MacroFlatline)`.
3. **Test 3 (`test_gatekeeper_healthy_consolidation_passes`):**
   * $\text{NATR}_{macro} = 0.45\%$, $\text{NATR}_{micro} = 0.05\% \ge 0.04\% \implies$ `passed == true`.
4. **Test 4 (`test_gatekeeper_micro_data_anomaly`):**
   * $\text{NATR}_{micro} = \text{NaN} \implies$ `veto_reason == Some(GateVetoReason::DataAnomaly)`.
5. **Zero Regression:**
   * 100% tests của cả 3 crates (`helios`, `ferro_trade`, `bot_engine`) pass hoàn toàn.
