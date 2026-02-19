# WebSocket Catalog (Official)

Order:
1) Options Public WS (market streams)
2) Options Private WS
3) Options WS Other (doc-ref)

Fixed columns:
id | channel | base_url | version | op/subscription |
request_message | response_message |
heartbeat | auth.type | restrictions | notes | source_url

## Options Public WS (Market Streams)
| id | channel | base_url | version | op/subscription | request_message | response_message | heartbeat | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|
| options.public.ws.trade | `<symbol>@trade` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | symbol naming per options convention | Real-time trade stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.public.ws.ticker | `<symbol>@ticker` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | symbol naming per options convention | 24h ticker stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.public.ws.kline | `<symbol>@kline_<interval>` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | interval must be in documented enum | Kline stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.public.ws.depth | `<symbol>@depth<levels>` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | levels/speed constrained by spec | Partial depth stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.public.ws.markprice | `<underlying>@markPrice` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | underlying stream scope | Mark price stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| options.public.ws.indexprice | `<underlying>@indexPrice` | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe stream name | n/a (raw stream endpoint) | object | ping-pong | none | underlying stream scope | Index price stream | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |

## Options Private WS

Official docs not found for a dedicated **Options private WebSocket API page** in the Options Trading navigation; user data stream access is documented via REST listenKey management.

| id | channel | base_url | version | op/subscription | request_message | response_message | heartbeat | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|

notes: official docs not found

evidence URL: https://developers.binance.com/docs/derivatives/option
