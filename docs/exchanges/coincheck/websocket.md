# Coincheck WebSocket Catalog

## 1) Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| coincheck.ws.public.trades | wss://ws-api.coincheck.com | v1 | {pair}-trades | {"type":"subscribe","channel":"btc_jpy-trades"} | not documented | array | [trade_id:number, pair:string, rate:string, amount:string, side:string, timestamp:string] | not documented | none | public stream | 取引履歴チャンネル。 | https://coincheck.com/ja/documents/exchange/api#websocket-trades |
| coincheck.ws.public.orderbook | wss://ws-api.coincheck.com | v1 | {pair}-orderbook | {"type":"subscribe","channel":"btc_jpy-orderbook"} | not documented | array | [pair:string, bids:array[[rate,amount]], asks:array[[rate,amount]], timestamp:string?] | not documented | none | public stream | 板情報チャンネル。 | https://coincheck.com/ja/documents/exchange/api#websocket-order-book |

## 2) Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| coincheck.ws.private.order_events | wss://stream.coincheck.com/private | v1 | order_events | {"command":"subscribe","channel":"order_events"} | {"command":"unsubscribe","channel":"order_events"} | object | event:string; order:object(id,pair,order_type,status,amount,rate,created_at,updated_at) | not documented | signature | login required | login 後に購読。 | https://coincheck.com/ja/documents/exchange/api#websocket-order-events |
| coincheck.ws.private.execution_events | wss://stream.coincheck.com/private | v1 | execution_events | {"command":"subscribe","channel":"execution_events"} | {"command":"unsubscribe","channel":"execution_events"} | object | event:string; execution:object(id,order_id,pair,amount,rate,side,liquidity,created_at) | not documented | signature | login required | login 後に購読。 | https://coincheck.com/ja/documents/exchange/api#websocket-execution-events |
