// trading_bot/bot_engine/src/telemetry/visual_report.rs
//
// 📊 ZERO-DEPENDENCY INTERACTIVE OOS VISUAL REPORT GENERATOR
// Renders TradingView Lightweight Charts (v4.2) canvas 60fps HTML report.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::execution::paper_trader::{ExitReason, PaperTradeRecord, PositionType};
use crate::models::models::Candle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatekeeperZoneInput {
    pub start_bar: usize,
    pub end_bar: usize,
    pub start_ts: i64,
    pub end_ts: i64,
    pub reason: String,
}

pub struct VisualReportGenerator;

impl VisualReportGenerator {
    pub fn generate(
        symbol: &str,
        candles: &[Candle],
        trades: &[PaperTradeRecord],
        reopt_events: &[(i64, String)],
        zones_json: Option<&str>,
        initial_cash: f64,
        output_dir: &str,
    ) -> Result<String, std::io::Error> {
        if candles.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Candles slice is empty, cannot generate report",
            ));
        }

        // 1. Phân tích vùng bảo vệ từ JSON
        let zones: Vec<GatekeeperZoneInput> = zones_json
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();

        let flatline_zones_count = zones
            .iter()
            .filter(|z| z.reason.contains("Flatline") || z.reason.contains("FLATLINE"))
            .count();

        // 2. Thống kê số liệu tài khoản & giao dịch
        let final_equity = trades.last().map(|t| t.equity_after).unwrap_or(initial_cash);
        let net_pnl = final_equity - initial_cash;
        let return_pct = (net_pnl / initial_cash.max(1e-5)) * 100.0;
        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.net_pnl > 0.0).count();
        let losing_trades = trades.iter().filter(|t| t.net_pnl <= 0.0).count();
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let start_time_str = Utc
            .timestamp_millis_opt(candles.first().unwrap().timestamp)
            .unwrap()
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
        let end_time_str = Utc
            .timestamp_millis_opt(candles.last().unwrap().timestamp)
            .unwrap()
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();

        // 3. Chuẩn bị dữ liệu Candlestick (Lightweight Charts format)
        let mut candle_data_json = String::with_capacity(candles.len() * 70);
        candle_data_json.push('[');
        for (idx, c) in candles.iter().enumerate() {
            if idx > 0 {
                candle_data_json.push(',');
            }
            candle_data_json.push_str(&format!(
                "{{\"time\":{},\"open\":{:.2},\"high\":{:.2},\"low\":{:.2},\"close\":{:.2}}}",
                c.timestamp / 1000,
                c.open,
                c.high,
                c.low,
                c.close
            ));
        }
        candle_data_json.push(']');

        // 4. Chuẩn bị Volume Data
        let mut volume_data_json = String::with_capacity(candles.len() * 75);
        volume_data_json.push('[');
        for (idx, c) in candles.iter().enumerate() {
            if idx > 0 {
                volume_data_json.push(',');
            }
            let color = if c.close >= c.open {
                "rgba(8, 153, 129, 0.35)"
            } else {
                "rgba(242, 54, 69, 0.35)"
            };
            volume_data_json.push_str(&format!(
                "{{\"time\":{},\"value\":{:.2},\"color\":\"{}\"}}",
                c.timestamp / 1000,
                c.volume,
                color
            ));
        }
        volume_data_json.push(']');

        // 5. Chuẩn bị Markers (Entry / Exit / Re-Opt)
        #[derive(Serialize)]
        struct ChartMarker {
            time: i64,
            position: &'static str,
            color: &'static str,
            shape: &'static str,
            text: String,
        }

        let mut markers: Vec<ChartMarker> = Vec::with_capacity(trades.len() * 2 + reopt_events.len());

        for t in trades {
            let is_long = t.pos_type == PositionType::Long;
            let side_str = if is_long { "LONG" } else { "SHORT" };
            let playbook_tag = t.opening_playbook_tag.as_deref().unwrap_or("STD");

            // Entry marker
            markers.push(ChartMarker {
                time: t.entry_timestamp / 1000,
                position: if is_long { "belowBar" } else { "aboveBar" },
                color: if is_long { "#089981" } else { "#f23645" },
                shape: if is_long { "arrowUp" } else { "arrowDown" },
                text: format!("#{}: {} @ {:.2} [{}]", t.id, side_str, t.entry_price, playbook_tag),
            });

            // Exit marker
            let exit_color = if t.net_pnl > 0.0 {
                "#22c55e"
            } else if t.exit_reason == ExitReason::HardStopLoss {
                "#ef4444"
            } else {
                "#f59e0b"
            };

            markers.push(ChartMarker {
                time: t.exit_timestamp / 1000,
                position: if is_long { "aboveBar" } else { "belowBar" },
                color: exit_color,
                shape: "circle",
                text: format!(
                    "#{}: Exit {:.2} ({:+.2}%) [{}]",
                    t.id,
                    t.exit_price,
                    t.pnl_pct,
                    t.exit_reason.as_str()
                ),
            });
        }

        for (ts, label) in reopt_events {
            markers.push(ChartMarker {
                time: ts / 1000,
                position: "aboveBar",
                color: "#3b82f6",
                shape: "square",
                text: label.clone(),
            });
        }

        markers.sort_by_key(|m| m.time);
        let markers_json = serde_json::to_string(&markers).unwrap_or_else(|_| "[]".to_string());

        // 6. Chuẩn bị Regime Shading Data (Overlay Histogram)
        let mut regime_data_json = String::with_capacity(candles.len() * 70);
        regime_data_json.push('[');
        let mut flatline_bar_count = 0usize;
        let mut chaos_bar_count = 0usize;

        for (idx, c) in candles.iter().enumerate() {
            if idx > 0 {
                regime_data_json.push(',');
            }
            let in_flatline = zones.iter().any(|z| {
                (z.reason.contains("Flatline") || z.reason.contains("FLATLINE"))
                    && c.timestamp >= z.start_ts
                    && c.timestamp <= z.end_ts
            });
            let in_chaos = zones.iter().any(|z| {
                (z.reason.contains("Chaos") || z.reason.contains("CHAOS"))
                    && c.timestamp >= z.start_ts
                    && c.timestamp <= z.end_ts
            });

            if in_flatline {
                flatline_bar_count += 1;
                regime_data_json.push_str(&format!(
                    "{{\"time\":{},\"value\":1,\"color\":\"rgba(100, 116, 139, 0.28)\"}}",
                    c.timestamp / 1000
                ));
            } else if in_chaos {
                chaos_bar_count += 1;
                regime_data_json.push_str(&format!(
                    "{{\"time\":{},\"value\":1,\"color\":\"rgba(239, 68, 68, 0.28)\"}}",
                    c.timestamp / 1000
                ));
            } else {
                regime_data_json.push_str(&format!(
                    "{{\"time\":{},\"value\":0,\"color\":\"transparent\"}}",
                    c.timestamp / 1000
                ));
            }
        }
        regime_data_json.push(']');

        // 7. Chuẩn bị Equity Curve (Step-wise qua từng trade)
        #[derive(Serialize)]
        struct EquityPoint {
            time: i64,
            value: f64,
        }
        let mut equity_points: Vec<EquityPoint> = Vec::with_capacity(trades.len() + 2);
        equity_points.push(EquityPoint {
            time: candles.first().unwrap().timestamp / 1000,
            value: initial_cash,
        });

        for t in trades {
            equity_points.push(EquityPoint {
                time: t.exit_timestamp / 1000,
                value: t.equity_after,
            });
        }

        if let Some(last_c) = candles.last() {
            let last_t = last_c.timestamp / 1000;
            if equity_points.last().map(|p| p.time).unwrap_or(0) < last_t {
                equity_points.push(EquityPoint {
                    time: last_t,
                    value: final_equity,
                });
            }
        }
        let equity_data_json = serde_json::to_string(&equity_points).unwrap_or_else(|_| "[]".to_string());

        // 8. Chuẩn bị Trade Ledger Table rows
        let mut table_rows_html = String::new();
        for t in trades {
            let is_win = t.net_pnl > 0.0;
            let badge_class = if is_win { "badge-win" } else { "badge-loss" };
            let entry_time = Utc
                .timestamp_millis_opt(t.entry_timestamp)
                .unwrap()
                .format("%m-%d %H:%M")
                .to_string();
            let exit_time = Utc
                .timestamp_millis_opt(t.exit_timestamp)
                .unwrap()
                .format("%m-%d %H:%M")
                .to_string();
            let playbook_tag = t.opening_playbook_tag.as_deref().unwrap_or("STD");
            let side_str = if t.pos_type == PositionType::Long { "LONG" } else { "SHORT" };
            let side_color = if t.pos_type == PositionType::Long { "#089981" } else { "#f23645" };

            table_rows_html.push_str(&format!(
                "<tr onclick=\"zoomToTrade({}, {})\" style=\"cursor:pointer;\">\
                    <td><b>#{}</b></td>\
                    <td style=\"color:{};font-weight:bold;\">{}</td>\
                    <td><span class=\"playbook-tag\">{}</span></td>\
                    <td>{}</td>\
                    <td>${:.2}</td>\
                    <td>{}</td>\
                    <td>${:.2}</td>\
                    <td>{}</td>\
                    <td><code>{}</code></td>\
                    <td class=\"{}\">{:+.2} USD</td>\
                    <td class=\"{}\">{:+.2}%</td>\
                    <td><b>${:.2}</b></td>\
                </tr>\n",
                t.entry_timestamp / 1000,
                t.exit_timestamp / 1000,
                t.id,
                side_color,
                side_str,
                playbook_tag,
                entry_time,
                t.entry_price,
                exit_time,
                t.exit_price,
                t.bars_held,
                t.exit_reason.as_str(),
                badge_class,
                t.net_pnl,
                badge_class,
                t.pnl_pct,
                t.equity_after
            ));
        }

        // 9. Lắp ráp toàn bộ HTML template hoàn chỉnh
        let html_content = format!(
r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Helios V3 OOS Telemetry Report - {symbol}</title>
    <script src="https://unpkg.com/lightweight-charts@4.2.0/dist/lightweight-charts.standalone.production.js"></script>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: #0f172a;
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
            padding: 20px;
        }}
        .header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: #1e293b;
            padding: 16px 24px;
            border-radius: 12px;
            border: 1px solid #334155;
            margin-bottom: 20px;
        }}
        .header h1 {{ font-size: 20px; font-weight: 700; color: #f8fafc; }}
        .header .meta {{ font-size: 13px; color: #94a3b8; margin-top: 4px; }}
        .badge-regime {{
            display: inline-flex;
            gap: 12px;
            align-items: center;
            font-size: 13px;
        }}
        .dot {{ width: 10px; height: 10px; border-radius: 50%; display: inline-block; margin-right: 4px; }}
        .dot-flatline {{ background-color: #64748b; }}
        .dot-chaos {{ background-color: #ef4444; }}
        .dot-trade {{ background-color: #22c55e; }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 12px;
            margin-bottom: 20px;
        }}
        .stat-card {{
            background: #1e293b;
            border: 1px solid #334155;
            padding: 14px 18px;
            border-radius: 10px;
        }}
        .stat-card .label {{ font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 600; }}
        .stat-card .value {{ font-size: 20px; font-weight: 700; margin-top: 4px; }}
        .val-positive {{ color: #22c55e; }}
        .val-negative {{ color: #ef4444; }}

        .chart-box {{
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 20px;
        }}
        .chart-title {{
            font-size: 14px;
            font-weight: 600;
            color: #cbd5e1;
            margin-bottom: 12px;
            display: flex;
            justify-content: space-between;
        }}
        #candlestickChart {{ height: 550px; width: 100%; }}
        #equityChart {{ height: 180px; width: 100%; margin-top: 10px; }}

        .table-box {{
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 12px;
            padding: 16px;
            overflow-x: auto;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
        }}
        th {{
            background: #0f172a;
            color: #94a3b8;
            text-align: left;
            padding: 10px 12px;
            font-weight: 600;
            border-bottom: 1px solid #334155;
        }}
        td {{
            padding: 10px 12px;
            border-bottom: 1px solid #334155;
        }}
        tr:hover td {{ background: #334155; }}
        .badge-win {{ color: #22c55e; font-weight: 700; }}
        .badge-loss {{ color: #ef4444; font-weight: 700; }}
        .playbook-tag {{
            background: #0284c7;
            color: #fff;
            padding: 2px 6px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 600;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div>
            <h1>🛡️ HELIOS V3 QUANT EXECUTION AUDIT: OOS TELEMETRY REPORT</h1>
            <div class="meta">Asset: <b>{symbol}</b> | OOS Span: <b>{start_time_str}</b> &rarr; <b>{end_time_str}</b> | Bars: <b>{candles_len}</b></div>
        </div>
        <div class="badge-regime">
            <span><span class="dot dot-flatline"></span>Flatline Protected ({flatline_bar_count} bars / {flatline_hours:.1}h)</span>
            <span><span class="dot dot-chaos"></span>Chaos Whipsaw Protected ({chaos_bar_count} bars / {chaos_hours:.1}h)</span>
            <span><span class="dot dot-trade"></span>Executed Trades ({total_trades})</span>
        </div>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <div class="label">Initial Cash</div>
            <div class="value">${initial_cash:.2}</div>
        </div>
        <div class="stat-card">
            <div class="label">Final Equity</div>
            <div class="value">${final_equity:.2}</div>
        </div>
        <div class="stat-card">
            <div class="label">Net Realized PnL</div>
            <div class="value {pnl_class}">{net_pnl:+.2} USD ({return_pct:+.2}%)</div>
        </div>
        <div class="stat-card">
            <div class="label">Total Trades / Win Rate</div>
            <div class="value">{total_trades} ({win_rate:.1}%)</div>
        </div>
        <div class="stat-card">
            <div class="label">Win / Loss Count</div>
            <div class="value"><span class="val-positive">{winning_trades}W</span> / <span class="val-negative">{losing_trades}L</span></div>
        </div>
        <div class="stat-card">
            <div class="label">Flatline Protection</div>
            <div class="value">{flatline_bar_count} bars ({flatline_zones_count} zones)</div>
        </div>
    </div>

    <div class="chart-box">
        <div class="chart-title">
            <span>📈 OOS CANDLESTICK CANVAS, REGIME SHADING & TRADE ORDERS</span>
            <span style="font-size:12px;color:#94a3b8;">Click vào dòng lệnh ở bảng bên dưới để Zoom nhanh đến lệnh</span>
        </div>
        <div id="candlestickChart"></div>
        <div class="chart-title" style="margin-top:16px;">
            <span>💰 SUBPLOT: COMPOUNDING EQUITY CURVE (${initial_cash:.2} &rarr; ${final_equity:.2})</span>
        </div>
        <div id="equityChart"></div>
    </div>

    <div class="table-box">
        <div class="chart-title">📋 AUDITED TRADES LEDGER ({total_trades} TRADES)</div>
        <table>
            <thead>
                <tr>
                    <th>#ID</th>
                    <th>Type</th>
                    <th>Playbook</th>
                    <th>Entry Time</th>
                    <th>Entry Price</th>
                    <th>Exit Time</th>
                    <th>Exit Price</th>
                    <th>Bars Held</th>
                    <th>Exit Reason</th>
                    <th>Net PnL</th>
                    <th>ROE %</th>
                    <th>Equity After</th>
                </tr>
            </thead>
            <tbody>
                {table_rows_html}
            </tbody>
        </table>
    </div>

    <script>
        const candleData = {candle_data_json};
        const volumeData = {volume_data_json};
        const regimeData = {regime_data_json};
        const markersData = {markers_json};
        const equityData = {equity_data_json};

        // 1. Tạo Main Candlestick Chart
        const mainContainer = document.getElementById('candlestickChart');
        const mainChart = LightweightCharts.createChart(mainContainer, {{
            layout: {{
                background: {{ color: '#1e293b' }},
                textColor: '#94a3b8',
            }},
            grid: {{
                vertLines: {{ color: '#334155' }},
                horzLines: {{ color: '#334155' }},
            }},
            crosshair: {{
                mode: LightweightCharts.CrosshairMode.Normal,
            }},
            rightPriceScale: {{
                borderColor: '#334155',
            }},
            timeScale: {{
                borderColor: '#334155',
                timeVisible: true,
                secondsVisible: false,
            }},
        }});

        // Regime Shading Band (Background Histogram Overlay)
        const regimeSeries = mainChart.addHistogramSeries({{
            priceScaleId: 'regime_overlay',
            priceFormat: {{ type: 'custom', formatter: () => '' }},
        }});
        mainChart.priceScale('regime_overlay').applyOptions({{
            scaleMargins: {{ top: 0, bottom: 0 }},
            visible: false,
        }});
        regimeSeries.setData(regimeData);

        // Candlestick Series
        const candleSeries = mainChart.addCandlestickSeries({{
            upColor: '#089981',
            downColor: '#f23645',
            borderVisible: false,
            wickUpColor: '#089981',
            wickDownColor: '#f23645',
        }});
        candleSeries.setData(candleData);
        candleSeries.setMarkers(markersData);

        // Volume Series
        const volumeSeries = mainChart.addHistogramSeries({{
            priceFormat: {{ type: 'volume' }},
            priceScaleId: 'volume',
        }});
        mainChart.priceScale('volume').applyOptions({{
            scaleMargins: {{ top: 0.82, bottom: 0 }},
            visible: false,
        }});
        volumeSeries.setData(volumeData);

        // 2. Tạo Subplot Equity Chart
        const eqContainer = document.getElementById('equityChart');
        const eqChart = LightweightCharts.createChart(eqContainer, {{
            layout: {{
                background: {{ color: '#1e293b' }},
                textColor: '#94a3b8',
            }},
            grid: {{
                vertLines: {{ color: '#334155' }},
                horzLines: {{ color: '#334155' }},
            }},
            rightPriceScale: {{
                borderColor: '#334155',
            }},
            timeScale: {{
                borderColor: '#334155',
                timeVisible: true,
                secondsVisible: false,
            }},
        }});

        const eqSeries = eqChart.addAreaSeries({{
            topColor: 'rgba(56, 189, 248, 0.45)',
            bottomColor: 'rgba(56, 189, 248, 0.02)',
            lineColor: '#38bdf8',
            lineWidth: 2,
        }});
        eqSeries.setData(equityData);

        // Đồng bộ hóa TimeScale giữa 2 biểu đồ
        mainChart.timeScale().subscribeVisibleLogicalRangeChange(range => {{
            if (range) eqChart.timeScale().setVisibleLogicalRange(range);
        }});
        eqChart.timeScale().subscribeVisibleLogicalRangeChange(range => {{
            if (range) mainChart.timeScale().setVisibleLogicalRange(range);
        }});

        window.addEventListener('resize', () => {{
            mainChart.applyOptions({{ width: mainContainer.clientWidth }});
            eqChart.applyOptions({{ width: eqContainer.clientWidth }});
        }});

        function zoomToTrade(entryTs, exitTs) {{
            const buffer = 3600; // 1 giờ trước và sau lệnh
            mainChart.timeScale().setVisibleRange({{
                from: entryTs - buffer,
                to: exitTs + buffer,
            }});
        }}
    </script>
</body>
</html>"#,
            symbol = symbol,
            start_time_str = start_time_str,
            end_time_str = end_time_str,
            candles_len = candles.len(),
            flatline_bar_count = flatline_bar_count,
            flatline_hours = flatline_bar_count as f64 / 60.0,
            chaos_bar_count = chaos_bar_count,
            chaos_hours = chaos_bar_count as f64 / 60.0,
            total_trades = total_trades,
            initial_cash = initial_cash,
            final_equity = final_equity,
            net_pnl = net_pnl,
            return_pct = return_pct,
            pnl_class = if net_pnl >= 0.0 { "val-positive" } else { "val-negative" },
            win_rate = win_rate,
            winning_trades = winning_trades,
            losing_trades = losing_trades,
            flatline_zones_count = flatline_zones_count,
            table_rows_html = table_rows_html,
            candle_data_json = candle_data_json,
            volume_data_json = volume_data_json,
            regime_data_json = regime_data_json,
            markers_json = markers_json,
            equity_data_json = equity_data_json,
        );

        let out_path = Path::new(output_dir).join("report_oos.html");
        let mut file = File::create(&out_path)?;
        file.write_all(html_content.as_bytes())?;

        Ok(out_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_report_generation() {
        let temp_dir = std::env::temp_dir();
        let dummy_candles = vec![
            Candle {
                symbol: "ETHUSDT".to_string(),
                timestamp: 1728000000000,
                open: 2500.0,
                high: 2510.0,
                low: 2490.0,
                close: 2505.0,
                volume: 100.0,
                quote_volume: 250500.0,
            },
            Candle {
                symbol: "ETHUSDT".to_string(),
                timestamp: 1728000060000,
                open: 2505.0,
                high: 2515.0,
                low: 2500.0,
                close: 2510.0,
                volume: 120.0,
                quote_volume: 301200.0,
            },
        ];

        let dummy_trades = vec![PaperTradeRecord {
            id: 1,
            pos_type: PositionType::Long,
            entry_price: 2500.0,
            exit_price: 2510.0,
            entry_timestamp: 1728000000000,
            exit_timestamp: 1728000060000,
            bars_held: 1,
            exit_reason: ExitReason::TakeProfit,
            opening_playbook_tag: Some("TREND_PB".to_string()),
            net_pnl: 1.50,
            pnl_pct: 2.50,
            equity_after: 61.50,
            mfe_pct: 3.0,
            mae_pct: -0.5,
        }];

        let reopt_events = vec![(1728000000000, "Re-Opt #0001".to_string())];
        let zones_json = r#"[{"start_bar":0,"end_bar":1,"start_ts":1728000000000,"end_ts":1728000060000,"reason":"VOLATILITY_FLATLINE"}]"#;

        let res = VisualReportGenerator::generate(
            "ETHUSDT",
            &dummy_candles,
            &dummy_trades,
            &reopt_events,
            Some(zones_json),
            60.0,
            temp_dir.to_str().unwrap(),
        );

        assert!(res.is_ok());
        let path = res.unwrap();
        assert!(std::path::Path::new(&path).exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("LightweightCharts"));
        assert!(content.contains("ETHUSDT"));
        assert!(content.contains("TREND_PB"));
        assert!(content.contains("Flatline Protected"));
        let _ = std::fs::remove_file(path);
    }
}

