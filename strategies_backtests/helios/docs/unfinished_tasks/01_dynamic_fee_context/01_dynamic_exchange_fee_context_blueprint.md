# THIẾT KẾ KIẾN TRÚC TƯƠNG LAI: ENGINE-INJECTED DYNAMIC FEE CONTEXT
> **Mã tài liệu:** `01_dynamic_exchange_fee_context_blueprint.md`  
> **Thư mục:** `docs/unfinished_tasks/01_dynamic_fee_context/`  
> **Phân hệ liên quan:** `ferro_trade` Core Context & `trading_bot` Multi-Exchange Dynamic Fee Engine  
> **Trạng thái:** 📋 **ĐÃ XÁC MINH & BẢO LƯU CHO GIAI ĐOẠN SAU FORWARD TEST**  
> **Thời điểm kích hoạt:** Khi `bot_engine` mở rộng sang hỗ trợ nhiều sàn giao dịch đồng thời (Binance, Bybit, OKX) hoặc cần cập nhật phí theo VIP Tier tự động.

---

## 🏛️ 1. BỐI CẢNH KIỂM CHỨNG & VẤN ĐỀ ĐÃ XÁC MINH

Trong đợt kiểm toán phân hệ cảm biến (`TODO-SEN-03`), khi nghiên cứu cơ chế bảo vệ chống sàn Flatline (thị trường tê liệt biến động khiến lệnh bị phí sàn nuốt chửng), một câu hỏi kiến trúc cốt tử đã được đặt ra:

> *"Liệu có nên truyền `commission_rate` trực tiếp từ Engine/Context vào Strategy thay vì lưu trong cấu hình `RiskBaseParams` không?"*

### 🔍 Kết quả kiểm chứng thực tế trong mã nguồn:
1. **Trong `ferro_trade` (`ferro_trade/src/backtest/`):**
   - `BacktestBuilder` có nhận `.commission(commission_rate)` để trừ phí trong `Broker`.
   - **Điểm nghẽn:** Struct [`InitContext`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs#L26-L31) và [`TickContext`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs#L224-L233) **chưa có trường `commission_rate`**. Do đó, `Strategy` hiện tại không thể đọc trực tiếp phí sàn từ Engine nếu không thông qua config của chính mình.
2. **Trong `strategies_backtests/helios` (`config.rs:771-778`):**
   - Struct `RiskBaseParams` **đã có sẵn** hai trường Non-DNA:
     - `pub taker_fee_rate: f64` (mặc định $0.0004 = 0.04\%$).
     - `pub slippage_allowance_pct: f64` (mặc định $0.0002 = 0.02\%$).
3. **Trong `trading_bot` (`walk_forward_smart_seed.rs:188` & `helios_bridge.rs`):**
   - `walk_forward_smart_seed.rs` đang gán trực tiếp: `strategy_inst.config.risk_management.risk_base.taker_fee_rate = COMMISSION_RATE;`.
   - `HeliosBridge` tự dựng mock `InitContext` và `TickContext` khi nạp nến thời gian thực.

---

## 🚀 2. THIẾT KẾ PHƯƠNG ÁN A: ENGINE-INJECTED DYNAMIC FEE CONTEXT

Mục tiêu dài hạn: Biến `bot_engine` thành nền tảng giao dịch đa sàn, đa tài sản (Multi-Asset / Multi-Exchange), nơi phí giao dịch được nạp tự động qua WebSocket/REST, lưu SQLite cache, và tự động tiêm vào `TickContext`.

```mermaid
graph TD
    subgraph Exchange ["Sàn Giao Dịch (Binance / Bybit / OKX)"]
        WS_REST["WebSocket / REST API (Fee Tiers & Commission Rates)"]
    end

    subgraph BotEngine ["trading_bot / bot_engine"]
        FeeCache["SQLite Store / RAM Cache (Asset Fee Mapping)"]
        AssetActor["AssetActor (per Symbol: ETHUSDT, SOLUSDT, EURUSD)"]
    end

    subgraph EngineCore ["ferro_trade (Backtest Engine & Context)"]
        Context["TickContext { ..., pub commission_rate: f64 }"]
    end

    subgraph StrategyCore ["helios (Strategy Core V3)"]
        Gatekeeper["RegimeGatekeeper: Effective_Floor = 2 * tick.commission_rate + slippage + DNA_Alpha"]
    end

    WS_REST -->|Cập nhật phí động theo VIP Tier / Asset| FeeCache
    FeeCache --> AssetActor
    AssetActor -->|Nạp phí động vào| Context
    Context -->|Strategy đọc phí O(1) không cần config| Gatekeeper
```

### Chi tiết các bước nâng cấp tương lai:
1. **Cập nhật `ferro_trade`:**
   - Trong `ferro_trade/src/backtest/context.rs`: Thêm `pub commission_rate: f64` vào `InitContext` và `TickContext`.
   - Trong `ferro_trade/src/backtest/engine.rs`: Gán `commission_rate: self.commission` khi khởi tạo context.
2. **Cập nhật `trading_bot`:**
   - Trong `InitialSync` / `MarketFilter`: Bổ sung API truy vấn `taker_commission_rate` và `maker_commission_rate` của từng symbol.
   - Lưu vào SQLite `market_filters` table.
   - Trong `HeliosBridge`: Đọc phí từ `market_filter` và tiêm thẳng vào `TickContext`.
3. **Cập nhật `helios`:**
   - `RegimeGatekeeper` đọc trực tiếp `tick.commission_rate` để tính sàn sinh tồn vật lý.

---

## 🛡️ 3. LÝ DO BẢO LƯU & GIẢI PHÁP TẠM THỜI CHO HIỆN TẠI

- **Lý do bảo lưu:** Phương án A đòi hỏi sửa đổi đồng thời cả 3 crate (`ferro_trade`, `helios`, `trading_bot`) và cập nhật toàn bộ các bài mock test. Ở giai đoạn hiện tại, ưu tiên số 1 là **ổn định hệ thống để Forward Test và Paper Trade sinh lời kiểm chứng**.
- **Giải pháp áp dụng ngay cho `TODO-SEN-03`:**
  - Tận dụng ngay hai trường Non-DNA đã có sẵn trong `RiskBaseParams`:
    $$\text{Roundtrip\_Cost} = 2 \times \text{taker\_fee\_rate} + \text{slippage\_allowance\_pct}$$
    $$\text{Effective\_Min\_NATR} = \text{Roundtrip\_Cost} + \text{min\_natr\_threshold}$$
  - `min_natr_threshold` trong DNA đóng vai trò là **Net Alpha Spread** (Biên lợi nhuận ròng tối thiểu ngoài chi phí sàn).
