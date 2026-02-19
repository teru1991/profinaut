# WebSocket Catalog (Official)

Order:
1) Spot Public WS Streams
2) Spot Private WS (User Data)
3) Spot WebSocket API
4) Other

Fixed columns:
id | ws_url | version | channel |
subscribe.template | unsubscribe.template |
message.shape | message.fields |
heartbeat.type | auth.type | restrictions | notes | source_url

## Spot Public WS Streams
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.ws.trades.trade | wss://stream.binance.com:9443/ws/<symbol>@trade | stream-v3 | <symbol>@trade | auto on connect (raw stream URL) | close socket | object | e(req):string; E(req):int(ms); s(req):string; t(req):int; p(req):string(decimal); q(req):string(decimal); T(req):int(ms); m(req):bool | ping-pong | none | symbol must be lowercase in stream name | Real-time trade stream | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |
| crypto.public.ws.aggtrade.trade | wss://stream.binance.com:9443/ws/<symbol>@aggTrade | stream-v3 | <symbol>@aggTrade | auto on connect | close socket | object | e(req):string; E(req):int(ms); s(req):string; a(req):int; p(req):string(decimal); q(req):string(decimal); f(req):int; l(req):int; T(req):int(ms); m(req):bool | ping-pong | none | lowercase symbol convention | Aggregated trade stream | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |
| crypto.public.ws.kline.update | wss://stream.binance.com:9443/ws/<symbol>@kline_<interval> | stream-v3 | <symbol>@kline_<interval> | auto on connect | close socket | object | e(req):string; E(req):int(ms); s(req):string; k(req):object(open,close,high,low,volume,start,end,closed) | ping-pong | none | interval must be supported enum | Candlestick updates | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |
| crypto.public.ws.depth.update | wss://stream.binance.com:9443/ws/<symbol>@depth | stream-v3 | <symbol>@depth | auto on connect | close socket | object | e(req):string; E(req):int(ms); s(req):string; U(req):int; u(req):int; b(req):array<array<string>>; a(req):array<array<string>> | ping-pong | none | sync with REST depth snapshot sequence | Diff depth stream | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |
| crypto.public.ws.bookticker.update | wss://stream.binance.com:9443/ws/<symbol>@bookTicker | stream-v3 | <symbol>@bookTicker | auto on connect | close socket | object | u(req):int; s(req):string; b(req):string(decimal); B(req):string(decimal); a(req):string(decimal); A(req):string(decimal) | ping-pong | none | per-symbol best bid/ask | Book ticker stream | https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams |

## Spot Private WS (User Data)
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.private.ws.userdata.outboundaccountposition | wss://stream.binance.com:9443/ws/<listenKey> | stream-v3 | outboundAccountPosition | POST /api/v3/userDataStream -> connect listenKey | DELETE /api/v3/userDataStream | object | e(req):string; E(req):int(ms); u(req):int(ms); B(req):array<object(asset,free,locked)> | ping-pong | api-key | listenKey keepalive required when legacy mode is used | Account balance delta event | https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream |
| crypto.private.ws.userdata.balanceupdate | wss://stream.binance.com:9443/ws/<listenKey> | stream-v3 | balanceUpdate | POST /api/v3/userDataStream -> connect listenKey | DELETE /api/v3/userDataStream | object | e(req):string; E(req):int(ms); a(req):string; d(req):string(decimal); T(req):int(ms) | ping-pong | api-key | emitted on deposit/withdraw/internal transfer | Balance update event | https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream |
| crypto.private.ws.userdata.executionreport | wss://stream.binance.com:9443/ws/<listenKey> | stream-v3 | executionReport | POST /api/v3/userDataStream -> connect listenKey | DELETE /api/v3/userDataStream | object | e(req):string; E(req):int(ms); s(req):string; c(req):string; S(req):string; o(req):string; f(req):string; q(req):string(decimal); p(req):string(decimal); X(req):string; x(req):string; i(req):int; l(req):string(decimal); z(req):string(decimal); T(req):int(ms) | ping-pong | api-key | payload fields vary by execution type | Order lifecycle event | https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream |

## Spot WebSocket API
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.ws.wsapi.time | wss://ws-api.binance.com/ws-api/v3 | v3 | time | {"id":"<id>","method":"time"} | n/a | object | id(req):int|string; status(req):int; result(req):object(serverTime:int(ms)); rateLimits(opt):array<object> | ping-pong | none | request-response schema with status code | WS API server time request | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/general-api-information |
| crypto.public.ws.wsapi.exchangeinfo | wss://ws-api.binance.com/ws-api/v3 | v3 | exchangeInfo | {"id":"<id>","method":"exchangeInfo","params":{...}} | n/a | object | id(req):int|string; status(req):int; result(req):object(symbols,rateLimits,exchangeFilters) | ping-pong | none | params optional and mirror REST semantics | WS API exchange metadata request | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/market-data-requests |
| crypto.private.ws.wsapi.order.place | wss://ws-api.binance.com/ws-api/v3 | v3 | order.place | {"id":"<id>","method":"order.place","params":{"apiKey":"...","timestamp":0,"signature":"...",...}} | n/a | object | id(req):int|string; status(req):int; result(req):object(symbol,orderId,clientOrderId,transactTime) | ping-pong | signed | request security signature mandatory | WS API trading request | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/trading-requests |
| crypto.private.ws.wsapi.account.status | wss://ws-api.binance.com/ws-api/v3 | v3 | account.status | {"id":"<id>","method":"account.status","params":{"apiKey":"...","timestamp":0,"signature":"..."}} | n/a | object | id(req):int|string; status(req):int; result(req):object(canTrade,balances,permissions) | ping-pong | signed | USER_DATA permission required | WS API account query | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/account-requests |
| crypto.private.ws.wsapi.userdata.subscribe | wss://ws-api.binance.com/ws-api/v3 | v3 | userDataStream.subscribe | {"id":"<id>","method":"userDataStream.subscribe","params":{"apiKey":"...","timestamp":0,"signature":"..."}} | {"id":"<id>","method":"userDataStream.unsubscribe","params":{"subscriptionId":<int>}} | object | id(req):int|string; status(req):int; result(req):object(subscriptionId:int) | ping-pong | signed | ws-api user data subscription lifetime defined in docs/changelog | Preferred replacement for listenKey mode | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/user-data-stream-requests |

## Other
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.sbe.marketdata | udp+tcp (see docs) | sbe | sbe-market-data | session/login per spec | session logout | binary | messageHeader(req):object; templateId(req):int; payload(req):object | server-push | other | dedicated gateway and schema version coordination | Spot SBE market data channel reference | https://developers.binance.com/docs/binance-spot-api-docs/sbe-market-data-streams |
| other.private.ws.fix.orderentry | FIX TLS endpoint (see docs) | fix-4.4 | order-entry | Logon(A) | Logout(5) | object | MsgType(req):string; SeqNum(req):int; ClOrdID(req):string; Symbol(req):string | ping-pong | api-key | FIX cert/session onboarding required | Spot FIX order entry overview | https://developers.binance.com/docs/binance-spot-api-docs/fix-api/order-entry |
