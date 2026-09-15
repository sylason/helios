# 🛡️ KIẾN TRÚC PHÒNG NGỰ ĐA TẦNG & QUẢN TRỊ TRẠNG THÁI UNTRADABLE (KHÔNG THỂ TRADE)

---

## 🏛️ 1. PHÂN TẦNG TRÁCH NHIỆM: CÁC TÌNH HUỐNG "KHÔNG THỂ TRADE" THUỘC VỀ FILE NÀO?

Một nguyên tắc cốt lõi trong **Clean Quant Architecture** là bảo toàn tính đơn nhiệm (Single Responsibility Principle - SRP): **Tuyệt đối không dồn toàn bộ gánh nặng nhận diện bão, thanh khoản chết hay biến động vĩ mô vào một Tactical Scanner con (`trend_pb.rs`)**.

Toàn bộ hệ sinh thái Helios V3 được phân chia ranh giới 3 tầng độc lập, khép kín:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ TẦNG 1: SENSORS & MATRIX (Cảm Biến & Ma Trận Trạng Thái Vĩ Mô)                         │
│ File: sensors.rs, matrix.rs                                                            │
│ • Nhiệm vụ: Đo lường khách quan độ hỗn loạn (chaos_ratio), độ nén (squeeze_intensity), │
│   độ dốc chuẩn hóa (kama_slope_adj), lực khối lượng (volume_force), độ dẹt (flatness).  │
│ • Sản phẩm: Continuous Simplex (w_t, w_r, w_c, w_ch ≡ 1.00) & Regime Multipliers.      │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ TẦNG 2: ARBITER (Hội Đồng Trọng Tài & Cầu Chì Toàn Cục)                                 │
│ File: arbiter.rs                                                                       │
│ • Nhiệm vụ: Quản trị các trạng thái UNTRADABLE (Không Thể Trade) ở cấp độ vĩ mô.       │
│ • Quyền hạn: VETO TOÀN CỤC (Flat Guard / Cash is King) ép bot ĐỨNG NGOÀI GIỮ TIỀN MẶT  │
│   khi bão tố, nhiễu ngẫu nhiên hoặc entropy quá cao.                                   │
│ • Quyền hạn: Phân xử quyền ưu tiên (Preemption) và hủy bỏ lệnh khi có tranh chấp điểm. │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ TẦNG 3: TACTICAL SCANNERS (Đội Tác Chiến Vi Mô - trend_pb.rs, range_osc.rs, ...)        │
│ File: scanners/trend_pb.rs                                                             │
│ • Nhiệm vụ duy nhất: Trả lời câu hỏi: "Nếu bối cảnh vĩ mô là Trend, thì tại nến này    │
│   nhịp hồi có chuẩn xác về mặt hình học toán học quanh KAMA hay không?"                │
│ • Tự vệ cục bộ: Chặn mua đu đỉnh (Distance Ceiling) & Chặn bắt dao rơi (Inval Floor).  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📊 2. BẢNG ĐỐI CHIẾU: MỌI TRẠNG THÁI KHÔNG THỂ TRADE ĐƯỢC CHẶN Ở ĐÂU?

| STT | Trạng Thái Cực Đoan Không Thể Trade | Biểu Hiện Thị Trường Vi Mô | File Nào Chịu Trách Nhiệm Chặn? | Cơ Chế Phòng Ngự Toán Học Cụ Thể |
| :---: | :--- | :--- | :---: | :--- |
| **1** | **Bão Whipsaw Quét Râu 2 Đầu** | Nến đổi màu liên tục, râu dài $>70\%$, bước đi ngẫu nhiên ($Hurst \approx 0.50$). | `matrix.rs` & `arbiter.rs` | $w_{\text{chaos}}$ vọt lên $> 0.70$, dìm $w_t \to 0$ $\implies$ Arbiter Veto loại `TrendPB`. |
| **2** | **Thị Trường Đóng Băng / Sổ Lệnh Rỗng (Zombie Tape)** | Volume $\to 0$, nến dẹp sợi chỉ, chênh lệch Bid/Ask dãn toạc. | `sensors.rs` & `arbiter.rs` | Sàng lọc Quote Volume Floor $\implies$ Arbiter từ chối mọi đề xuất vào lệnh (Zero Liquidity Veto). |
| **3** | **Flash Crash / Thiên Nga Đen Đột Ngột** | Tin tức giật sập 1 cây nến đỏ khổng lồ xuyên thủng mọi đường hỗ trợ. | `trend_pb.rs` (Cục bộ) | **Invalidation Floor:** $\text{Close} < \text{KAMA} - \text{Inval} \times \text{ATR} \implies$ Triệt tiêu điểm về $0.0$ ngay lập tức! |
| **4** | **Parabolic Pump (Hưng Phấn Đỉnh Sóng)** | Giá tăng dựng đứng cách xa KAMA $4 - 6$ lần ATR. | `trend_pb.rs` (Cục bộ) | **Distance Ceiling:** $\text{Close} > \text{KAMA} + \text{MaxDist} \times \text{ATR} \implies$ Triệt tiêu điểm về $0.0$ (chống mua đu đỉnh). |
| **5** | **Tranh Chấp Tử Thần (Deadlock)** | Cả Long và Short cùng bùng nổ điểm cao trên 1 cây nến (vùng giằng co mạnh). | `arbiter.rs` | Arbiter hủy cả 2 lệnh, tuyên bố hòa hoãn để đứng ngoài quan sát (FLAW-REG.4). |
| **6** | **Chuỗi Thua Liên Tiếp Sau Cắt Lỗ (Trauma Streak)** | Bot vừa bị quét SL ở nhịp trước, thị trường chưa tìm được điểm cân bằng mới. | `asset_actor.rs` & `core_v3/mod.rs` | **Dual Sentinels Cooldown:** Khóa 2 chiều 15 nến, cấm mở lệnh kế tiếp cho đến khi giá bứt phá có lợi $\ge 1.5 \times \text{ATR}$. |

---

## 🎯 3. PHÂN TÍCH CHUYÊN SÂU BÀI TOÁN FAKE BREAKOUT TRONG TREND PULLBACK

### A. Phân định ranh giới giữa `MomentumIgnition` và `TrendPullback`
*   `MomentumIgnition`: Mua khi giá **bứt phá phá cản (Breakout)** $\implies$ Đối mặt với bẫy *Bull Trap / False Breakout đỉnh*.
*   `TrendPullback`: Tuyệt đối không mua phá đỉnh, mà **chờ giá thoái lui về trục cân bằng KAMA (Retracement)** $\implies$ Đối mặt với 2 dạng bẫy đặc thù:

### B. Hai dạng bẫy giả tạo trong nhịp hồi KAMA & Cách triệt tiêu
1.  **Fake Bounce / Dead Cat Bounce (Cú Nảy Giả Tạo):**
    *   *Hiện tượng:* Giá chạm KAMA nảy lên một cây nến xanh nhỏ, nhưng cây nến tiếp theo xả gãy hỗ trợ KAMA.
    *   *Giải pháp:*
        - Bắt buộc kiểm tra **CLV (Close Location Value) $> 0.30$**: Nến phải đóng cửa ở nửa trên của biên độ, chứng minh lực cầu thực sự áp đảo lực cung.
        - **Structural Stop Loss:** Luôn neo dưới $\min(\text{Low}[i], \text{KAMA}[i]) - \text{Buffer}$. Nếu cú nảy thất bại, lệnh bị cắt lỗ ở mức tối thiểu ($0.5 - 1.0$ ATR), không bao giờ chịu lỗ sâu.
2.  **Fake Breakdown / Liquidity Sweep (Quét Râu Giả Qua KAMA):**
    *   *Hiện tượng:* Nến thò râu quét qua KAMA để cắn Stop Loss của thị trường rồi rút chân đóng nến trên KAMA.
    *   *Giải pháp:* Khoảng cách $\delta_{\text{KAMA}}$ được tính theo giá đóng nến $\text{Close}[i]$ chứ không tính theo râu nến. Nếu nến chỉ quét râu rồi rút chân đóng cửa trên KAMA $\implies$ Vẫn được công nhận là nhịp hồi săn thanh khoản hợp lệ!
