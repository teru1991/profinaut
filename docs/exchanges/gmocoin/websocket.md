# WebSocket Catalog (Official)

Order:
1) Crypto Public WS
2) Crypto Private WS
3) FX Public WS
4) FX Private WS

Fixed columns:
id | ws_url | version | channel |
subscribe.template | unsubscribe.template |
message.shape | message.fields |
heartbeat.type | auth.type | restrictions | notes | source_url

## Crypto Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.ws.ticker.update | wss://api.coin.z.com/ws/public/v1 | v1 | ticker | {"command":"subscribe","channel":"ticker","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"ticker","symbol":"<symbol>"} | object | symbol(req):string; ask(req):string(decimal); bid(req):string(decimal); timestamp(req):string(iso8601) | server-push | none | symbol required | Best bid/ask updates | https://api.coin.z.com/docs/#/ws/ticker |
| crypto.public.ws.trades.update | wss://api.coin.z.com/ws/public/v1 | v1 | trades | {"command":"subscribe","channel":"trades","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"trades","symbol":"<symbol>"} | object | symbol(req):string; side(req):string; price(req):string(decimal); size(req):string(decimal); timestamp(req):string(iso8601) | server-push | none | symbol required | Trade-by-trade execution stream | https://api.coin.z.com/docs/#/ws/trades |
| crypto.public.ws.orderbooks.update | wss://api.coin.z.com/ws/public/v1 | v1 | orderbooks | {"command":"subscribe","channel":"orderbooks","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"orderbooks","symbol":"<symbol>"} | object | asks(req):array<object>; bids(req):array<object>; timestamp(req):string(iso8601) | server-push | none | symbol required | Incremental/full book updates | https://api.coin.z.com/docs/#/ws/orderbooks |

## Crypto Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.private.ws.executionevents.update | wss://api.coin.z.com/ws/private/v1/{token} | v1 | executionEvents | {"command":"subscribe","channel":"executionEvents"} | {"command":"unsubscribe","channel":"executionEvents"} | object | orderId(req):int|string; executionId(req):int|string; symbol(req):string; size(req):string(decimal) | ping-pong | token | valid ws-auth token required | Private execution notifications | https://api.coin.z.com/docs/#/privateWebsocket/executionEvents |
| crypto.private.ws.orderevents.update | wss://api.coin.z.com/ws/private/v1/{token} | v1 | orderEvents | {"command":"subscribe","channel":"orderEvents"} | {"command":"unsubscribe","channel":"orderEvents"} | object | orderId(req):int|string; status(req):string; symbol(req):string | ping-pong | token | valid ws-auth token required | Order acceptance/cancel updates | https://api.coin.z.com/docs/#/privateWebsocket/orderEvents |
| crypto.private.ws.positionevents.update | wss://api.coin.z.com/ws/private/v1/{token} | v1 | positionEvents | {"command":"subscribe","channel":"positionEvents"} | {"command":"unsubscribe","channel":"positionEvents"} | object | positionId(req):int|string; symbol(req):string; side(req):string; size(req):string(decimal) | ping-pong | token | leveraged account only | Position lifecycle stream | https://api.coin.z.com/docs/#/privateWebsocket/positionEvents |

## FX Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.public.ws.ticker.update | wss://api.coin.z.com/fx/ws/public/v1 | v1 | ticker | {"command":"subscribe","channel":"ticker","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"ticker","symbol":"<symbol>"} | object | symbol(req):string; ask(req):string(decimal); bid(req):string(decimal); timestamp(req):string(iso8601) | server-push | none | symbol required | FX quote stream | https://api.coin.z.com/fxdocs/#/ws/ticker |
| fx.public.ws.trades.update | wss://api.coin.z.com/fx/ws/public/v1 | v1 | trades | {"command":"subscribe","channel":"trades","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"trades","symbol":"<symbol>"} | object | symbol(req):string; side(req):string; price(req):string(decimal); size(req):string(decimal) | server-push | none | symbol required | FX trades stream | https://api.coin.z.com/fxdocs/#/ws/trades |
| fx.public.ws.orderbooks.update | wss://api.coin.z.com/fx/ws/public/v1 | v1 | orderbooks | {"command":"subscribe","channel":"orderbooks","symbol":"<symbol>"} | {"command":"unsubscribe","channel":"orderbooks","symbol":"<symbol>"} | object | asks(req):array<object>; bids(req):array<object>; timestamp(req):string(iso8601) | server-push | none | symbol required | FX orderbook stream | https://api.coin.z.com/fxdocs/#/ws/orderbooks |

## FX Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.private.ws.executionevents.update | wss://api.coin.z.com/fx/ws/private/v1/{token} | v1 | executionEvents | {"command":"subscribe","channel":"executionEvents"} | {"command":"unsubscribe","channel":"executionEvents"} | object | orderId(req):int|string; executionId(req):int|string; symbol(req):string; size(req):string(decimal) | ping-pong | token | valid ws-auth token required | FX private executions | https://api.coin.z.com/fxdocs/#/privateWebsocket/executionEvents |
| fx.private.ws.orderevents.update | wss://api.coin.z.com/fx/ws/private/v1/{token} | v1 | orderEvents | {"command":"subscribe","channel":"orderEvents"} | {"command":"unsubscribe","channel":"orderEvents"} | object | orderId(req):int|string; status(req):string; symbol(req):string | ping-pong | token | valid ws-auth token required | FX private order updates | https://api.coin.z.com/fxdocs/#/privateWebsocket/orderEvents |
| fx.private.ws.positionevents.update | wss://api.coin.z.com/fx/ws/private/v1/{token} | v1 | positionEvents | {"command":"subscribe","channel":"positionEvents"} | {"command":"unsubscribe","channel":"positionEvents"} | object | positionId(req):int|string; symbol(req):string; side(req):string; size(req):string(decimal) | ping-pong | token | margin account required | FX private position updates | https://api.coin.z.com/fxdocs/#/privateWebsocket/positionEvents |
