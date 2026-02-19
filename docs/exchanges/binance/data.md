# Data Distribution Catalog (Official)

Fixed columns:
id | kind | url_pattern | format | compression | update_freq | retention |
schema.summary | notes | source_url

| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| data.market.realtime.json | market-data-stream | wss://stream.binance.com:9443/ws/<streamName> | json | none | real-time | session-lifetime | trade/depth/bookTicker/kline event objects | Spot market streams (raw/combined) | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |
| data.account.realtime.json | user-data-stream | wss://stream.binance.com:9443/ws/<listenKey> | json | none | real-time | listenKey or ws-api subscription lifetime | outboundAccountPosition/balanceUpdate/executionReport payloads | Legacy listenKey user data channel | https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream |
| data.market.realtime.sbe | market-data-stream | SBE gateway endpoints per docs | sbe(binary) | none | real-time | session-lifetime | SBE message header + template payloads by schema | Spot SBE binary feeds | https://developers.binance.com/docs/binance-spot-api-docs/sbe-market-data-streams |
| data.tradeevents.session.fix | drop-copy | FIX Drop Copy endpoint per docs | fix(tag-value) | tls | real-time | session-lifetime | ExecutionReport and session-level FIX messages | Official FIX drop-copy distribution | https://developers.binance.com/docs/binance-spot-api-docs/fix-api/drop-copy |
| data.market.snapshot.json | rest-snapshot | https://api.binance.com/api/v3/depth?symbol=<symbol>&limit=<n> | json | https gzip(opt) | on-demand | request-time snapshot | snapshot object with lastUpdateId,bids,asks | Used with diff-depth recovery procedure | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
