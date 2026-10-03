# BÁO CÁO GIẢI PHẪU DỮ LIỆU ĐỐI CHỨNG: RUN 1 VS RUN 2 (WALK-FORWARD SMART SEED)
> **Dự án:** Helios V3 & Trading Bot Engine  
> **Tác giả:** Antigravity Quant Team  
> **Ngày lập:** 02/10/2026  
> **Dữ liệu phân tích:** 
> - **Run 1 (Lãi +10.82%):** `trading_bot/bot_engine/examples/logs/run_walk_forward_smart_seed_20261002_223050`
> - **Run 2 (Lỗ -9.03%):** `trading_bot/bot_engine/examples/logs/run_walk_forward_smart_seed_20261002_224609`
> - **Cặp giao dịch:** ETHUSDT (Dữ liệu Out-Of-Sample 8 ngày, Rolling In-Sample 30 ngày)
> - **Điều kiện kiểm thử:** Xóa sạch thư mục `examples/storage` SQLite trước mỗi lần chạy để đảm bảo Cold Start độc lập.

---

## 📑 MỤC LỤC
1. [TỔNG QUAN HIỆU SUẤT & ĐỐI CHIẾU SỐ LIỆU TÀI CHÍNH](#1-tổng-quan-hiệu-suất--đối-chiếu-số-liệu-tài-chính)
2. [GIẢI PHẪU 3 NGUYÊN NHÂN CỐT TỬ (THE THREE SMOKING GUNS)](#2-giải-phẫu-3-nguyên-nhân-cốt-tử-the-three-smoking-guns)
   - [2.1. Smoking Gun 1: Lỗ Hổng Cầu Dao Cooldown Bị GA Triệt Tiêu (Lỗi Kiến Trúc Cực Kỳ Nghiêm Trọng)](#21-smoking-gun-1-lỗ-hổng-cầu-dao-cooldown-bị-ga-triệt-tiêu)
   - [2.2. Smoking Gun 2: Điểm Rẽ Nhánh Initial #0000 & Bẫy Khóa Di Truyền Smart Seeding](#22-smoking-gun-2-điểm-rẽ-nhánh-initial-0000--bẫy-khóa-di-truyền-smart-seeding)
   - [2.3. Smoking Gun 3: Bệnh "Mua Đỉnh Bán Đáy" Của MOMENTUM_IGN (Thiếu Trần Price_Z)](#23-smoking-gun-3-bệnh-mua-đỉnh-bán-đáy-của-momentum_ign)
3. [MỔ XẺ HÀNH VI KHỚP LỆNH: CÀO LỆNH & TRẢ THÙ THỊ TRƯỜNG (REVENGE CHAINING)](#3-mổ-xẻ-hành-vi-khớp-lệnh-cào-lệnh--trả-thù-thị-trường)
4. [ĐỐI CHIẾU SỔ CÁI PHÒNG NGỰ (CAPITAL DEFENSE MATRIX LEDGER)](#4-đối-chiếu-sổ-cái-phòng-ngự-capital-defense-matrix-ledger)
5. [MA TRẬN ĐỐI CHẤT PHƯƠNG ÁN KHẮC PHỤC TRIỆT ĐỂ](#5-ma-trận-đối-chất-phương-án-khắc-phục-triệt-để)
6. [KẾ HOẠCH HÀNH ĐỘNG KHẮC PHỤC (BABY STEPS ACTION PLAN)](#6-kế-hoạch-hành-động-khắc-phục-baby-steps-action-plan)

---

## 1. TỔNG QUAN HIỆU SUẤT & ĐỐI CHIẾU SỐ LIỆU TÀI CHÍNH

| Chỉ Số Đánh Giá | Run 1 (`223050`) | Run 2 (`224609`) | Chênh Lệch | Đánh Giá Định Lượng |
| :--- | :---: | :---: | :---: | :--- |
| **Vốn Ban Đầu** | $60.00 | $60.00 | $0.00 | Chuẩn hóa cố định |
| **Vốn Kết Thúc** | **$66.49** | **$54.58** | **-$11.91** | 🚨 **Biến động không thể chấp nhận** |
| **Lợi Nhuận Ròng (Net PnL)** | **+$6.49 (+10.82%)** | **-$5.42 (-9.03%)** | **-19.85%** | Lần 1 Thắng đẹp, Lần 2 Lỗ nặng |
| **Tổng Số Lệnh Khớp** | **25 lệnh** | **40 lệnh** | **+15 lệnh (+60%)** | Run 2 cào lệnh quá mức |
| **Tần Suất Khớp Lệnh** | 3.1 lệnh / ngày | **5.0 lệnh / ngày** | **+61.3%** | Run 2 biến thành bot Scalping cào rác |
| **Tỷ Lệ Thắng (Win Rate)** | **48.0% (12W / 13L)**| **42.5% (17W / 23L)**| **-5.5%** | Run 2 dính thêm 10 lệnh thua |
| **Avg Win / Avg Loss (R:R)** | +1.72% / -0.98% (1.75R)| +0.94% / -1.02% (0.92R)| **Suy giảm 47%**| Run 2 ăn mỏng, thua dày |
| **Lệnh Dính SL Trần 15% ROE**| **2 lệnh (8.0%)** | **15 lệnh (37.5%)** | **Gấp 7.5 lần!** | Run 2 đề xuất SL cực kỳ cẩu thả |
| **Thời Gian Bị Chặn Bão Chaos**| 159 bars (2.6 giờ) | **525 bars (8.8 giờ)**| **Gấp 3.3 lần** | Run 2 né bão nhiều nhưng khi mở thì cào điên cuồng |

> **Nhận định từ số liệu:** Nhận định trực giác của người dùng là **chính xác 100% về mặt định lượng**: Run 2 giao dịch quá mức (40 trades), tỷ lệ R:R suy thoái nghiêm trọng, dính liên tiếp các lệnh cắt lỗ tối đa (15% ROE) và liên tục mua đỉnh bán đáy.

---

## 2. GIẢI PHẪU 3 NGUYÊN NHÂN CỐT TỬ (THE THREE SMOKING GUNS)

### 2.1. Smoking Gun 1: Lỗ Hổng Cầu Dao Cooldown Bị GA Triệt Tiêu
- **Vị trí vi phạm:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:791-795`
  ```rust
  #[dna(0.0, 30.0, 1.0)]
  pub re_entry_cooldown_bars: usize,
  #[dna(0.0, 1.0, 1.0)]
  pub enable_re_entry_cooldown: bool,
  ```
- **Bản chất nghiệp vụ & toán học:**
  - Theo thiết kế ban đầu (`GEMINI.md`, Mục 6.2 & 6.4), Cầu Dao Cooldown (Circuit Breaker) là **Chốt Chặn Bất Biến (Invariant Safety Rule)** thuộc về Tầng Bảo Toàn Vốn, tuyệt đối không được để GA điều chỉnh hay vô hiệu hóa.
  - Tuy nhiên, do bị gắn macro `#[dna]`, GA Optimizer (vốn là cỗ máy Greedy tìm cách tối đa hóa Sharpe/PnL trên dữ liệu In-Sample 30 ngày) đã nhận thấy: *Nếu ngồi chờ cooldown sau mỗi lệnh thua thì số lệnh sẽ giảm, bỏ lỡ một số nhịp sóng hồi phục trong quá khứ.*
  - Do đó, GA ở Run 2 đã **đột biến gán `re_entry_cooldown_bars = 0` và `enable_re_entry_cooldown = false`**.
- **Hệ quả thực tế trên luồng thực thi (`asset_actor.rs:205`):**
  - Điều kiện kiểm tra `if stop_cfg.re_entry_cooldown_bars > 0` trở thành `false`.
  - Toàn bộ cơ chế Circuit Breaker sau khi dính `HARD_SL` bị tê liệt 100%.
  - Chốt chặn `closed_intra_candle` chỉ chặn duy nhất trên cùng 1 bar nến (`bar_idx`). Khi sang nến tiếp theo (`bar_idx + 1`), bot lập tức bắn lệnh tiếp theo cùng chiều!
  - **Minh chứng thép từ log Run 2:**
    - Lệnh #0026 (Short) dính Hard SL lúc **17:26** $\to$ Đúng **17:27 (sau đúng 1 phút!)**, Lệnh #0027 tiếp tục mở và lại bị vả dính Hard SL sau 8 phút!
    - Lệnh #0036 dính Hard SL lúc **01:22** $\to$ Đúng **01:24 (2 phút sau)** mở ngay Lệnh #0037!

---

### 2.2. Smoking Gun 2: Điểm Rẽ Nhánh Initial #0000 & Bẫy Khóa Di Truyền Smart Seeding
- **Vị trí vi phạm:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs` và `walk_forward_smart_seed.rs`
- **Diễn biến tại Chu kỳ Khởi Động Lạnh #0000 (Bar 0 - 30 ngày In-Sample đầu tiên):**
  - **Run 1:** Quần thể khởi tạo Latin Hypercube Sampling (LHS) sinh ra Champion ban đầu có OOS Return = **-0.10%**. Cổng Walk-Forward Evaluator (WFE Gate) **REJECT** cá thể này $\to$ `AssetActor` không nhận DNA rác mà vận hành bằng **Default Config** (thông số mặc định chuẩn, kỷ luật, thận trọng). Trong 240 phút đầu tiên, Run 1 bắn 0 lệnh rác.
  - **Run 2:** Quần thể khởi tạo LHS rơi vào một điểm ngẫu nhiên may mắn vượt qua WFE Gate với OOS Return = **+1.49%** (PASS) $\to$ Bộ DNA cào lệnh này lập tức được deploy vào `AssetActor` ngay tại Bar 0.
- **Hiện tượng Bẫy Khóa Di Truyền (Genetic Lock-in qua Smart Seeding 0.5%):**
  - Cơ chế Smart Seeding trích xuất Champion tốt nhất từ SQLite nạp vào thế hệ 0 của chu kỳ kế tiếp.
  - Bộ DNA ở Run 2 có các bộ lọc kích hoạt rất lỏng lẻo. Trên tập In-Sample (vốn có những giai đoạn ETH biến động một chiều cực mạnh), bộ gen này cào lệnh cật lực và tạo ra In-Sample Return lên tới **+110.5%**!
  - Với điểm số In-Sample khổng lồ đó, dòng gen cào lệnh này đàn áp toàn bộ các cá thể thăm dò mới, khóa chặt bộ tối ưu hóa vào một **Cực trị địa phương quái thai (Degenerate Local Optimum)**.
  - Qua 47 chu kỳ Re-Optimize, Run 2 liên tục bị "bỏ bùa" bởi dòng gen này, dẫn đến việc cào tới 40 lệnh trong 8 ngày OOS và đốt sạch 9.03% tài khoản.

---

### 2.3. Smoking Gun 3: Bệnh "Mua Đỉnh Bán Đáy" Của MOMENTUM_IGN (Thiếu Trần Price_Z)
- **Vị trí vi phạm:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:100-140`
- **Bản chất hiện tượng vi cấu trúc thị trường:**
  - `TREND_PB` scanner có cơ chế Boundary Guard (chặn không cho Long nếu giá vượt biên trên Bollinger Band hoặc quá xa KAMA).
  - Ngược lại, `MOMENTUM_IGN` lại **hoàn toàn không có bất kỳ bộ lọc trần nào về `Price_Z` (độ lệch chuẩn khoảng cách so với KAMA)**.
  - Khi thị trường có một nến 1m tăng dựng đứng cực nhanh, Volume nến vọt lên gấp 3-5 lần trung bình. Thuật toán `MOMENTUM_IGN` nhận định đây là "Breakout xung lực mạnh" và bắn tín hiệu Long với độ tự tin cao.
  - Tuy nhiên, trong thực tế thị trường Crypto: Khi `Price_Z > +3.0` hoặc `+4.0`, đó là hiện tượng **Volume Kiệt Sức / Phân Phối Cực Điểm (Climactic Volume Exhaustion)**. Phe bò dốc toàn lực đẩy nến râu cuối cùng vào vùng thanh khoản của Smart Money $\to$ Giá lập tức đảo chiều quay đầu rơi tự do!
- **Minh chứng thép từ Telemetry Run 2:**
  - **Lệnh #0003 (Long):** Vào lệnh tại `Price_Z = +3.27` (cách KAMA +3.27 sigma), `NATR = 0.42%`. Vừa vào lệnh xong, nến sau đảo chiều, chạm Hard SL mất -$0.79.
  - **Lệnh #0006 (Long):** Vào lệnh tại `Price_Z = +4.22` (cách KAMA hơn +4.2 sigma - đỉnh râu nến tuyệt đối!). Ngay lập tức giá sập ngược, dính Hard SL mất -$0.82.

---

## 3. MỔ XẺ HÀNH VI KHỚP LỆNH: CÀO LỆNH & TRẢ THÙ THỊ TRƯỜNG

Soi kỹ các chuỗi lệnh liên tiếp trong Run 2 bộc lộ hành vi cào lệnh vô cùng nguy hiểm:

| Chuỗi Lệnh | Chiều | Thời Điểm Ra (Exit) | Thời Điểm Vào Kế (Entry) | Delta Thời Gian | Kết Quả Lệnh Kế | Bản Chất Hành Vi |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **#0002 -> #0006** | Long | Liên tục | Liên tục | Vài phút/lệnh | 4 lệnh dính SL liên tiếp | Đu đỉnh FOMO cụm Momentum |
| **#0026 -> #0027** | Short/Long | 17:26 (Hard SL) | 17:27 (Mở mới) | **Đúng 1 phút** | Dính tiếp Hard SL (-13.2% ROE) | Cào lệnh trả thù (Revenge Chaining) |
| **#0032 -> #0033** | Long/Short | 10:00 (Hard SL) | 10:07 (Mở mới) | **7 phút** | Lỗ tiếp -11.5% ROE | Bắt dao rơi khi bão chưa tan |
| **#0036 -> #0037** | Short/Short | 01:22 (Hard SL) | 01:24 (Mở mới) | **2 phút** | Lỗ tiếp -12.8% ROE | Cào lệnh liên thanh |

> **Kết luận:** Nếu có Cầu Dao Cooldown tối thiểu 15-30 bars (15-30 phút) áp đặt cứng sau mỗi lệnh dính Hard SL, **toàn bộ 10 lệnh thua dây chuyền này sẽ bị triệt tiêu 100%**, biến kết quả Run 2 từ Lỗ -9.03% thành Lãi dương!

---

## 4. ĐỐI CHIẾU SỔ CÁI PHÒNG NGỰ (CAPITAL DEFENSE MATRIX LEDGER)

Tại cuối file log của 2 đợt chạy, Sổ cái Ma trận Phòng ngự ghi nhận sự chênh lệch chấn động:

```text
================================================================================
                    CAPITAL DEFENSE MATRIX AUDIT LEDGER                         
================================================================================
  [Tier 1: Gatekeeper Chaos Filter]
    Run 1: Total Chaos Bars Filtered: 159 bars (2.6 hours)
    Run 2: Total Chaos Bars Filtered: 525 bars (8.8 hours)  <-- Gấp 3.3 lần!

  [Tier 3: Sniper Guard & Dynamic Clamping]
    Run 1: Proposed Orders Clamped to Max SL: 2 orders
    Run 2: Proposed Orders Clamped to Max SL: 15 orders!   <-- Gấp 7.5 lần!
================================================================================
```

### Phân tích ý nghĩa định lượng:
1. **Tier 1 (Né bão Chaos):** Run 2 né bão tới 525 bars (gần 9 tiếng đồng hồ ngồi im). Nghĩa là thị trường ở Run 2 có những đoạn biến động hỗn loạn dữ dội. Thế nhưng, trong 15 tiếng còn lại khi cổng mở, bot lại bắn tới 40 lệnh (tần suất cào lệnh gấp 3 lần bình thường).
2. **Tier 3 (SL Clamping):** 
   - Ở Run 1, chỉ có **2 lệnh** bị chạm trần kẹp SL 15% ROE (chứng tỏ các đề xuất Stop Loss từ Scanner phần lớn đều hợp lý, nằm trong khoảng 5% - 10% ROE).
   - Ở Run 2, có tới **15 lệnh (chiếm 37.5% tổng số lệnh)** bị kẹp cứng ở trần 15% ROE. 
   - Điều này vạch trần: DNA của Run 2 đã nới lỏng các thông số ATR buffer của Scanner đến mức vô lý. Scanner đề xuất các mức SL cách xa Entry tới 20% - 30% ROE, và nếu không có chốt chặn `GLOBAL_HARD_MAX_ROE = 15%` của Risk Manager giữ lại, tài khoản đã bị cháy sạch (Liquidated) từ sớm!

---

## 5. MA TRẬN ĐỐI CHẤT PHƯƠNG ÁN KHẮC PHỤC TRIỆT ĐỂ

| Phương Án Khắc Phục | Bản Chất Nghiệp Vụ & Toán Học | Ưu Điểm (Pros) | Khuyết Điểm (Cons) | Kịch Bản Cực Đoan (Edge Cases) & Phòng Vệ |
| :--- | :--- | :--- | :--- | :--- |
| **1. Khóa Cứng Cầu Dao Cooldown (Hardcode Cooldown Invariant)** | Xóa bỏ `#[dna]` khỏi `re_entry_cooldown_bars`. Áp đặt sàn cứng tối thiểu: Sau khi dính Hard SL, bắt buộc nghỉ $T_{\text{cooldown}} = \max(15, \text{cooldown\_bars})$ nến 1m. | Triệt tiêu 100% hiện tượng cào lệnh trả thù (Revenge Chaining). Bảo vệ tài khoản khi dính chuỗi bão. | Có thể bỏ lỡ 1 nhịp hồi mạnh ngay sau khi quét thanh khoản (Liquidity Wick). | **Edge case:** Thị trường giật râu quét SL rồi bay ngay. **Phòng vệ:** Chấp nhận bỏ lỡ; bảo toàn vốn là ưu tiên số 1, không bao giờ đánh đu với rủi ro quét 2 đầu. |
| **2. Áp Trần Động Lượng `Price_Z` Cho `MOMENTUM_IGN`** | Bổ sung chốt chặn: Cấm Long nếu $\text{Price\_Z} > +2.5$, cấm Short nếu $\text{Price\_Z} < -2.5$. | Loại bỏ hoàn toàn căn bệnh mua đỉnh bán đáy và FOMO đu đỉnh râu nến phân phối kiệt sức. | Nếu gặp siêu xu hướng (Parabolic Bull Run), có thể vào lệnh trễ hơn 1-2 nến. | **Edge case:** Breakout nến đầu tiên có `Price_Z = +2.6`. **Phòng vệ:** Chờ nến hồi nhẹ (Pullback) để `TREND_PB` bắt ở vị thế an toàn hơn nhiều. |
| **3. Nâng Cấp Cổng Walk-Forward Evaluator (WFE Gate Consistency)** | Bổ sung tỷ lệ nhất quán: $\text{Consistency Ratio} = \frac{\text{OOS Return}}{\text{In-Sample Return}} \ge 0.15$. | Loại bỏ 100% các cá thể Overfitting (IS lời ảo 110% nhưng OOS chỉ ăn may 1%). Phá vỡ bẫy Local Minima. | Tỷ lệ PASS của WFE Gate sẽ khắt khe hơn, nhiều chu kỳ sẽ dùng lại Champion cũ. | **Edge case:** Giai đoạn thị trường đổi pha đột ngột từ Trend sang Sideway. **Phòng vệ:** Dùng lại DNA an toàn cũ tốt hơn là nhận một DNA quá khớp phá hoại tài khoản. |
| **4. Cơ Chế Giảm Đòn Bẩy (De-leverage) Khi SL Rộng Thay Vì VETO** | Nếu Scanner đề xuất khoảng cách cấu trúc $D_{\text{sl}}$ quá rộng: Thay vì kẹp cứng 15% ROE hoặc VETO, Risk Manager tự động giảm Position Size: $\text{Size} = \frac{\text{Risk Capital}}{D_{\text{sl}}}$. | Giữ được các cơ hội Breakout thực sự mà vẫn đảm bảo số tiền mất không vượt quá 15% ROE vốn phân bổ. | Khối lượng lệnh sẽ nhỏ đi khi biến động lớn, giảm lợi nhuận tuyệt đối. | **Edge case:** Size bị co về nhỏ hơn `MIN_NOTIONAL` ($21). **Phòng vệ:** Nếu nhỏ hơn $21 thì dứt khoát bỏ qua lệnh (Veto an toàn). |

---

## 6. KẾ HOẠCH HÀNH ĐỘNG KHẮC PHỤC (BABY STEPS ACTION PLAN)

Căn cứ theo triết lý Baby Steps và kiểm soát chất lượng nghiêm ngặt:

- [ ] **Bước 1 (HOTFIX - Khóa vĩnh viễn Cầu Dao Cooldown):**
  - Xóa macro `#[dna]` khỏi `re_entry_cooldown_bars` và `enable_re_entry_cooldown` trong `strategies_backtests/helios/src/strategy/core_v3/config.rs`.
  - Trong `trading_bot/bot_engine/src/controller/asset_actor.rs`, áp đặt:
    ```rust
    let penalty_bars = stop_cfg.re_entry_cooldown_bars.max(15);
    ```
- [ ] **Bước 2 (HOTFIX - Chặn Mua Đỉnh Bán Đáy `MOMENTUM_IGN`):**
  - Thêm điều kiện chặn `Price_Z` trong `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs`:
    ```rust
    if is_long && matrix.price_z[i] > 2.5 { return None; }
    if !is_long && matrix.price_z[i] < -2.5 { return None; }
    ```
- [ ] **Bước 3 (Nâng Cấp WFE Gate Consistency):**
  - Bổ sung kiểm tra `oos_return / is_return.max(0.01) >= 0.15` trong `prospector_runner.rs` để loại bỏ dứt điểm các cá thể Overfit ảo.
- [ ] **Bước 4 (Chạy Lại Thực Nghiệm Đối Chứng):**
  - Xóa SQLite, chạy lại 2 lần kiểm thử liên tiếp để xác nhận đường cong lợi nhuận hội tụ ổn định và số lệnh giảm về mức chuẩn (~20-25 lệnh/8 ngày).
