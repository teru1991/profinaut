# Data Distribution Catalog (Official)

Fixed columns:
id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url

| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| usdm.data.market.realtime.streams | market-data-stream | wss://fstream.binance.com/ws/<streamName> or /stream?streams=<a>/<b> | json | none | real-time | session-lifetime | event envelope with stream name + payload (`e`,`E`,symbol-specific fields) | Covers aggTrade, markPrice, kline, ticker, depth, liquidation, contract info families | https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams |
| usdm.data.market.snapshot.rest.depth | rest-snapshot | https://fapi.binance.com/fapi/v1/depth?symbol=<symbol>&limit=<n> | json | https gzip(opt) | on-demand | request-time snapshot | snapshot includes lastUpdateId, bids, asks | Used to initialize local order book before diff depth replay | https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api |
| usdm.data.account.realtime.userdata | user-data-stream | wss://fstream.binance.com/ws/<listenKey> | json | none | real-time | listenKey-lifetime | account/order/margin event objects by event type | listenKey managed via REST API endpoints | https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams |
| usdm.data.rpc.realtime.wsapi | websocket-api-rpc | wss://ws-fapi.binance.com/ws-fapi/v1 | json | none | request-response + push (method dependent) | session-lifetime | request/response object: id,status,result,error,rateLimits | Method coverage governed by WS API docs and change log | https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-api-general-info |
