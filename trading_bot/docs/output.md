--- Prospector Finished in 5252.49s ---
[Prospector] 🧹 Starting cleanup process...
[Prospector] 💾 Successfully exported 10 champions to helios_champions_results.json
2026-07-05T13:18:20.609125Z DEBUG sled::pagecache::iobuf: rolling to new segment after clearing 4978724-5203562
2026-07-05T13:18:20.609342Z DEBUG sled::pagecache::segment: segment accountant returning offset: 0 for lsn 15204352 on deck: {524288, 3145728, 3670016}
2026-07-05T13:18:20.609643Z DEBUG sled::pagecache::iobuf: storing lsn 15204352 in beginning of buffer
2026-07-05T13:18:20.614633Z DEBUG sled::pagecache::iobuf: rolling to new segment after clearing 0-457010
2026-07-05T13:18:20.614877Z DEBUG sled::pagecache::segment: segment accountant returning offset: 524288 for lsn 15728640 on deck: {3145728, 3670016}
2026-07-05T13:18:20.615193Z DEBUG sled::pagecache::iobuf: storing lsn 15728640 in beginning of buffer
2026-07-05T13:18:20.621091Z DEBUG sled::pagecache::iobuf: rolling to new segment after clearing 524288-940302
2026-07-05T13:18:20.621444Z DEBUG sled::pagecache::segment: segment accountant returning offset: 3145728 for lsn 16252928 on deck: {3670016}
2026-07-05T13:18:20.621840Z DEBUG sled::pagecache::iobuf: storing lsn 16252928 in beginning of buffer
2026-07-05T13:18:20.625483Z DEBUG sled::pagecache::iobuf: rolling to new segment after clearing 3145728-3660705
2026-07-05T13:18:20.625769Z DEBUG sled::pagecache::segment: segment accountant returning offset: 3670016 for lsn 16777216 on deck: {}
2026-07-05T13:18:20.626129Z DEBUG sled::pagecache::iobuf: storing lsn 16777216 in beginning of buffer
2026-07-05T13:18:20.630741Z DEBUG sled::pagecache::iobuf: rolling to new segment after clearing 3670016-4177934
2026-07-05T13:18:20.630976Z DEBUG sled::pagecache::segment: segment accountant returning offset: 5242880 for lsn 17301504 on deck: {}
2026-07-05T13:18:20.631306Z DEBUG sled::pagecache::iobuf: storing lsn 17301504 in beginning of buffer
2026-07-05T13:18:20.632593Z DEBUG sled::pagecache::iobuf: advancing offset within the current segment from 5242880 to 5318203
2026-07-05T13:18:20.660419Z DEBUG sled::pagecache::iobuf: wrote lsns 14940196-15204351 to disk at offsets 4978724-5242879, maxed true complete_len 264156
2026-07-05T13:18:20.660791Z DEBUG sled::pagecache::iobuf: mark_interval(14940196, 264156)
2026-07-05T13:18:20.661151Z DEBUG sled::pagecache::iobuf: new highest interval: 14940196 - 15204351
2026-07-05T13:18:20.695013Z DEBUG sled::pagecache::iobuf: wrote lsns 15204352-15728639 to disk at offsets 0-524287, maxed true complete_len 524288
2026-07-05T13:18:20.695619Z DEBUG sled::pagecache::iobuf: mark_interval(15204352, 524288)
2026-07-05T13:18:20.696060Z DEBUG sled::pagecache::iobuf: new highest interval: 15204352 - 15728639
2026-07-05T13:18:20.728665Z DEBUG sled::pagecache::iobuf: wrote lsns 16252928-16777215 to disk at offsets 3145728-3670015, maxed true complete_len 524288
2026-07-05T13:18:20.729093Z DEBUG sled::pagecache::iobuf: mark_interval(16252928, 524288)
2026-07-05T13:18:20.791215Z DEBUG sled::pagecache::iobuf: wrote lsns 16777216-17301503 to disk at offsets 3670016-4194303, maxed true complete_len 524288
2026-07-05T13:18:20.791475Z DEBUG sled::pagecache::iobuf: wrote lsns 15728640-16252927 to disk at offsets 524288-1048575, maxed true complete_len 524288
2026-07-05T13:18:20.791676Z DEBUG sled::pagecache::iobuf: wrote lsns 17301504-17376826 to disk at offsets 5242880-5318202, maxed false complete_len 75323
2026-07-05T13:18:20.791841Z DEBUG sled::pagecache::iobuf: mark_interval(16777216, 524288)
2026-07-05T13:18:20.792196Z DEBUG sled::pagecache::iobuf: mark_interval(15728640, 524288)
2026-07-05T13:18:20.792523Z DEBUG sled::pagecache::iobuf: mark_interval(17301504, 75323)
2026-07-05T13:18:20.793121Z DEBUG sled::pagecache::iobuf: new highest interval: 15728640 - 17301503
2026-07-05T13:18:20.793716Z DEBUG sled::pagecache::iobuf: new highest interval: 17301504 - 17376826
2026-07-05T13:18:20.794127Z DEBUG sled::pagecache::iobuf: make_stable(17376826) returning
[Prospector] Database at 'storage/prospector.db' cleaned up.
2026-07-05T13:18:21.475160Z  INFO bot_engine::optimizer::prospector_runner: 🏆 [Optimizer] TOP 1 FOUND! Return: 137.44%, Max DD: -19.49%, Trades: 31
2026-07-05T13:18:21.521822Z  INFO bot_engine::data_engine::duckdb_impl: 🧬 [ETHUSDT] DNA for strategy 'helios_v1' saved to DB.
2026-07-05T13:18:21.522342Z  INFO bot_engine::optimizer::prospector_runner: 💾 [Optimizer] Đã lưu DNA mới nhất vào Database.
2026-07-05T13:18:21.522535Z  INFO bot_engine::optimizer::prospector_runner: 📢 [Optimizer] Đã phát tín hiệu (Event-Driven) mang DNA mới tới các Actor.
2026-07-05T13:18:21.522767Z DEBUG sled::pagecache::iobuf: advancing offset within the current segment from 5318203 to 5318219
2026-07-05T13:18:21.536294Z DEBUG sled::pagecache::iobuf: wrote lsns 17376827-17376842 to disk at offsets 5318203-5318218, maxed false complete_len 16
2026-07-05T13:18:21.536776Z DEBUG sled::pagecache::iobuf: mark_interval(17376827, 16)
2026-07-05T13:18:21.537166Z DEBUG sled::pagecache::iobuf: new highest interval: 17376827 - 17376842
2026-07-05T13:18:21.537560Z DEBUG sled::pagecache::iobuf: make_stable(17376842) returning
2026-07-05T13:18:21.538870Z DEBUG sled::pagecache::logger: IoBufs dropped
2026-07-05T13:18:21.565447Z  INFO bot_engine: 💤 [Optimizer] Tạm nghỉ 24 tiếng chờ chu kỳ mới.
2026-07-05T13:18:21.565959Z  INFO bot_engine::controller::asset_actor: 🧬 [ETHUSDT] Nhận được DNA mới từ Optimizer thông qua Event Bus!
2026-07-05T13:18:59.788078Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257480000, open: 1764.09, high: 1764.98, low: 1764.08, close: 1764.86, volume: 592.275 }
2026-07-05T13:18:59.788770Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257480000
2026-07-05T13:18:59.847271Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257480000 saved to DB.
2026-07-05T13:19:58.909703Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257540000, open: 1764.86, high: 1764.98, low: 1764.37, close: 1764.98, volume: 411.369 }
2026-07-05T13:19:58.910437Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257540000
2026-07-05T13:19:58.964534Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257540000 saved to DB.
2026-07-05T13:20:58.772902Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257600000, open: 1764.98, high: 1765.26, low: 1764.98, close: 1765.01, volume: 595.729 }
2026-07-05T13:20:58.773673Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257600000
2026-07-05T13:20:58.831869Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257600000 saved to DB.
2026-07-05T13:21:16.629920Z  INFO binance_sdk::common::websocket: PING received from server on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:21:16.630294Z  INFO binance_sdk::common::websocket: Responded PONG to server's PING message on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:21:58.767861Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257660000, open: 1765.0, high: 1765.33, low: 1765.0, close: 1765.18, volume: 508.398 }
2026-07-05T13:21:58.768606Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257660000
2026-07-05T13:21:58.831398Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257660000 saved to DB.
2026-07-05T13:22:58.738955Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257720000, open: 1765.18, high: 1765.18, low: 1763.75, close: 1764.0, volume: 464.727 }
2026-07-05T13:22:58.739682Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257720000
2026-07-05T13:22:58.799853Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257720000 saved to DB.
2026-07-05T13:23:58.806769Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257780000, open: 1764.0, high: 1764.6, low: 1764.0, close: 1764.21, volume: 506.455 }
2026-07-05T13:23:58.807520Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257780000
2026-07-05T13:23:58.861463Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257780000 saved to DB.
2026-07-05T13:24:16.605743Z  INFO binance_sdk::common::websocket: PING received from server on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:24:16.606265Z  INFO binance_sdk::common::websocket: Responded PONG to server's PING message on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:24:58.734946Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257840000, open: 1764.21, high: 1765.46, low: 1763.94, close: 1763.98, volume: 2923.203 }
2026-07-05T13:24:58.735669Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257840000
2026-07-05T13:24:58.800364Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257840000 saved to DB.
2026-07-05T13:25:59.056429Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257900000, open: 1763.98, high: 1764.63, low: 1763.98, close: 1764.32, volume: 620.999 }
2026-07-05T13:25:59.057148Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257900000
2026-07-05T13:25:59.104603Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257900000 saved to DB.
2026-07-05T13:26:58.702080Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783257960000, open: 1764.33, high: 1764.33, low: 1763.25, close: 1763.55, volume: 824.894 }
2026-07-05T13:26:58.702802Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783257960000
2026-07-05T13:26:58.768194Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783257960000 saved to DB.
2026-07-05T13:27:16.582751Z  INFO binance_sdk::common::websocket: PING received from server on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:27:16.583344Z  INFO binance_sdk::common::websocket: Responded PONG to server's PING message on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:27:58.824225Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783258020000, open: 1763.55, high: 1763.55, low: 1760.52, close: 1761.41, volume: 3755.912 }
2026-07-05T13:27:58.824979Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783258020000
2026-07-05T13:27:58.887027Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783258020000 saved to DB.
2026-07-05T13:28:59.009444Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783258080000, open: 1761.41, high: 1763.0, low: 1761.4, close: 1762.91, volume: 1495.312 }
2026-07-05T13:28:59.010179Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783258080000
2026-07-05T13:28:59.069690Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783258080000 saved to DB.
2026-07-05T13:29:58.954046Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783258140000, open: 1762.9, high: 1762.91, low: 1762.74, close: 1762.75, volume: 415.895 }
2026-07-05T13:29:58.954791Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783258140000
2026-07-05T13:29:59.013186Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783258140000 saved to DB.
2026-07-05T13:30:16.559240Z  INFO binance_sdk::common::websocket: PING received from server on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:30:16.559856Z  INFO binance_sdk::common::websocket: Responded PONG to server's PING message on 853547a3b7a1c4121c3ac3e8caf0fd94
2026-07-05T13:30:59.022358Z DEBUG bot_engine::data_engine::binance_stream: 📊 Nến mới đóng: Candle { symbol: "ETHUSDT", timestamp: 1783258200000, open: 1762.74, high: 1762.97, low: 1762.0, close: 1762.97, volume: 654.243 }
2026-07-05T13:30:59.023110Z DEBUG bot_engine::controller::asset_actor: 📬 [ETHUSDT] Received new candle: 1783258200000
2026-07-05T13:30:59.079789Z DEBUG bot_engine::data_engine::duckdb_impl: 💾 [ETHUSDT] Candle 1783258200000 saved to DB.