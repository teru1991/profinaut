# BitTrade WebSocket Catalog

## 1) Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| public.ws.market.kline | wss://api-cloud.bittrade.co.jp/ws | v1 | market.$symbol.kline.$period | {"sub":"market.btcjpy.kline.1min","id":"client-id"} | {"unsub":"market.btcjpy.kline.1min","id":"client-id"} | object | ch:string; ts:number; tick:object | ping-pong | none | public stream | ローソク足push。 | https://api-doc.bittrade.co.jp/#websocket-public |
| public.ws.market.depth | wss://api-cloud.bittrade.co.jp/ws | v1 | market.$symbol.depth.$type | {"sub":"market.btcjpy.depth.step1","id":"client-id"} | {"unsub":"market.btcjpy.depth.step1","id":"client-id"} | object | ch:string; ts:number; tick:object(bids,asks) | ping-pong | none | public stream | 板情報push。 | https://api-doc.bittrade.co.jp/#websocket-public |
| public.ws.market.bbo | wss://api-cloud.bittrade.co.jp/ws | v1 | market.$symbol.bbo | {"sub":"market.btcjpy.bbo","id":"client-id"} | {"unsub":"market.btcjpy.bbo","id":"client-id"} | object | ch:string; ts:number; tick:object(bid,ask) | ping-pong | none | public stream | BBO push。 | https://api-doc.bittrade.co.jp/#websocket-public |
| public.ws.market.detail | wss://api-cloud.bittrade.co.jp/ws | v1 | market.$symbol.detail | {"sub":"market.ethbtc.detail","id":"client-id"} | {"unsub":"market.ethbtc.detail","id":"client-id"} | object | ch:string; ts:number; tick:object | ping-pong | none | public stream | ティッカーpush。 | https://api-doc.bittrade.co.jp/#websocket-public |
| public.ws.market.trade.detail | wss://api-cloud.bittrade.co.jp/ws | v1 | market.$symbol.trade.detail | {"sub":"market.ethjpy.trade.detail","id":"client-id"} | {"unsub":"market.ethjpy.trade.detail","id":"client-id"} | object | ch:string; ts:number; tick:object | ping-pong | none | public stream | 約定push。 | https://api-doc.bittrade.co.jp/#websocket-public |

## 2) Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| private.ws.accounts.update | wss://api-cloud.bittrade.co.jp/ws/v2 | v2 | accounts.update | {"action":"sub","ch":"accounts.update","params":{"auth":{...}}} | {"action":"unsub","ch":"accounts.update"} | object | ch:string; ts:number; data:list<object> | ping-pong | signed-hmac-sha256 | api key required | 口座更新通知。 | https://api-doc.bittrade.co.jp/#websocket-private |
| private.ws.trade.clearing | wss://api-cloud.bittrade.co.jp/retail/ws | v1 | trade.clearing | {"sub":"trade.clearing","id":"client-id","auth":{...}} | {"unsub":"trade.clearing","id":"client-id"} | object | ch:string; ts:number; data:object | ping-pong | signed-hmac-sha256 | api key required | 販売所約定/清算系通知。 | https://api-doc.bittrade.co.jp/#websocket-private |
