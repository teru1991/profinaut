# Kraken WebSocket Catalog (Official)

Fixed columns:
id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url

## 1) Spot Public WS (v1)

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.ws.v1.market.book.subscribe | wss://ws.kraken.com | v1 | book | {"event":"subscribe","pair":["XBT/USD"],"subscription":{"name":"book","depth":10}} | {"event":"unsubscribe","pair":["XBT/USD"],"subscription":{"name":"book"}} | array<object> | channelID(req):int,pair(req):string,book(req):object | server-push | none | channel depth variants by docs | v1 public book feed | https://docs.kraken.com/api/docs/websocket-v1/book/ |
| spot.public.ws.v1.market.trade.subscribe | wss://ws.kraken.com | v1 | trade | {"event":"subscribe","pair":["XBT/USD"],"subscription":{"name":"trade"}} | {"event":"unsubscribe","pair":["XBT/USD"],"subscription":{"name":"trade"}} | array<object> | channelID(req),pair(req),trades(req):array<object> | server-push | none | public only | v1 trade feed | https://docs.kraken.com/api/docs/websocket-v1/trade/ |

## 2) Spot Private WS (v1)

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.ws.v1.account.open_orders.subscribe | wss://ws-auth.kraken.com | v1 | openOrders | {"event":"subscribe","subscription":{"name":"openOrders","token":"<ws_token>"}} | {"event":"unsubscribe","subscription":{"name":"openOrders","token":"<ws_token>"}} | array<object> | channelName(req),sequence(opt):int,data(req):array<object> | server-push | token | token required from REST /GetWebSocketsToken | private account state stream | https://docs.kraken.com/api/docs/websocket-v1/openorders/ |
| spot.private.ws.v1.trade.add_order.request | wss://ws-auth.kraken.com | v1 | addOrder | {"event":"addOrder","token":"<ws_token>","pair":"XBT/USD","type":"buy","ordertype":"limit","price":"30000","volume":"0.01"} | not_applicable | object | event(req):string,status(req):string,errorMessage(opt):string,txid(opt):array<string> | ping-pong | token | private trading permissions required | private trading over WS v1 | https://support.kraken.com/articles/360039849211-websocket-api-v1-placing-and-cancelling-orders |

## 3) Spot Public WS (v2)

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.ws.v2.market.book.subscribe | wss://ws.kraken.com/v2 | v2 | book | {"method":"subscribe","params":{"channel":"book","symbol":["BTC/USD"],"depth":10}} | {"method":"unsubscribe","params":{"channel":"book","symbol":["BTC/USD"]}} | object | channel(req),type(req),data(req):array<object> | ping-pong | none | depth and snapshot behavior per channel doc | v2 normalized envelope | https://docs.kraken.com/api/docs/websocket-v2/book |
| spot.public.ws.v2.market.instrument.subscribe | wss://ws.kraken.com/v2 | v2 | instrument | {"method":"subscribe","params":{"channel":"instrument","symbol":["BTC/USD"]}} | {"method":"unsubscribe","params":{"channel":"instrument","symbol":["BTC/USD"]}} | object | channel(req),type(req),data(req):array<object> | ping-pong | none | symbol format per docs | instrument metadata feed | https://docs.kraken.com/api/docs/websocket-v2/instrument/ |

## 4) Spot Private WS (v2)

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.ws.v2.trade.add_order | wss://ws.kraken.com/v2 | v2 | add_order | {"method":"add_order","params":{"token":"<token>","order_type":"limit","side":"buy","order_qty":0.01,"symbol":"BTC/USD","limit_price":30000}} | not_applicable | object | method(req),success(req):bool,error(opt):string,result(opt):object | ping-pong | token | authenticated token/challenge needed for private methods | private v2 trading method example present in docs | https://docs.kraken.com/api/docs/websocket-v2/add_order |

## 5) Futures Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.public.ws.v1.market.ticker.subscribe | wss://futures.kraken.com/ws/v1 | other | ticker | {"event":"subscribe","feed":"ticker","product_ids":["PI_XBTUSD"]} | {"event":"unsubscribe","feed":"ticker","product_ids":["PI_XBTUSD"]} | object | feed(req),product_id(req):string,bid/ask(opt):string(decimal) | ping-pong | none | periodic ping required to keep connection alive | futures market data ticker | https://docs.kraken.com/api/docs/futures-api/websocket/ticker |
| futures.public.ws.v1.market.book.subscribe | wss://futures.kraken.com/ws/v1 | other | book | {"event":"subscribe","feed":"book","product_ids":["PI_XBTUSD"]} | {"event":"unsubscribe","feed":"book","product_ids":["PI_XBTUSD"]} | object | feed(req),product_id(req),bids/asks(req):array<object> | ping-pong | none | message rates depend on feed/update mode | futures L2 book feed | https://docs.kraken.com/api/docs/futures-api/websocket/book |

## 6) Futures Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.private.ws.v1.account.open_positions.subscribe | wss://futures.kraken.com/ws/v1 | other | open_positions | {"event":"subscribe","feed":"open_positions","api_key":"<api_key>","original_challenge":"<challenge>","signed_challenge":"<signed>"} | {"event":"unsubscribe","feed":"open_positions"} | object | feed(req),account(req):string,positions(req):array<object> | ping-pong | signed | requires challenge signing flow | private futures account stream | https://docs.kraken.com/api/docs/futures-api/websocket/open-positions |

## 7) WS Common（protocol/auth/limits等）

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.v1.common.heartbeat | wss://ws.kraken.com | v1 | heartbeat | not_applicable | not_applicable | object | event(req):"heartbeat" | server-push | none | none | v1 server heartbeat event | https://docs.kraken.com/api/docs/websocket-v1/heartbeat/ |
| other.public.ws.v2.common.ping | wss://ws.kraken.com/v2 | v2 | ping | {"method":"ping"} | not_applicable | object | method(req),req_id(opt),success(req):bool,time_in(req):string(iso8601),time_out(req):string(iso8601) | ping-pong | none | none | v2 ping request/response | https://docs.kraken.com/api/docs/websocket-v2/ping |
| other.public.ws.other.futures.ping | wss://futures.kraken.com/ws/v1 | other | ping | {"event":"ping"} | not_applicable | object | event(req):"pong" | ping-pong | none | required to avoid disconnect after inactivity | futures ws keepalive requirement | https://docs.kraken.com/api/docs/guides/futures-websockets/ |

## 8) Other

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.doc.spot-intro | https://docs.kraken.com/api/docs/guides/spot-ws-intro/ | other | docs-reference | - | - | object | v1(req):wss://ws.kraken.com,v2(req):wss://ws.kraken.com/v2 | none | none | docs reference only | architecture guide for spot ws versions | https://docs.kraken.com/api/docs/guides/spot-ws-intro/ |
