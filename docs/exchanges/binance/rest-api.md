# REST API Catalog (Official)

Order:
1) Spot Public REST
2) Spot Private REST
3) Spot Other (doc-ref)
4) Other

Fixed columns:
id | method | base_url | path | version | operation | auth.type |
params.query | params.path | params.body |
response.shape | response.fields |
errors.shape | rate_limit | notes | source_url

## Spot Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.rest.ping.get | GET | https://api.binance.com | /api/v3/ping | v3 | Test connectivity | none | - | - | - | object | empty object | object | Weight: 1 | Connectivity check endpoint | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
| crypto.public.rest.time.get | GET | https://api.binance.com | /api/v3/time | v3 | Get server time | none | - | - | - | object | serverTime(req):int(ms) | object | Weight: 1 | Used for timestamp sync | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
| crypto.public.rest.exchangeinfo.get | GET | https://api.binance.com | /api/v3/exchangeInfo | v3 | Exchange/symbol metadata | none | symbol(opt):string; symbols(opt):array<string>; permissions(opt):array<string> | - | - | object | timezone(req):string; serverTime(req):int(ms); symbols(req):array<object>; rateLimits(req):array<object>; exchangeFilters(req):array<object> | object | Weight varies by params | Includes filters/enums references | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
| crypto.public.rest.depth.get | GET | https://api.binance.com | /api/v3/depth | v3 | Order book snapshot | none | symbol(req):string; limit(opt):int | - | - | object | lastUpdateId(req):int; bids(req):array<array<string>>; asks(req):array<array<string>> | object | Weight depends on limit | Snapshot for diff-depth sync | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
| crypto.public.rest.trades.get | GET | https://api.binance.com | /api/v3/trades | v3 | Recent trades list | none | symbol(req):string; limit(opt):int | - | - | array<object> | id(req):int; price(req):string(decimal); qty(req):string(decimal); time(req):int(ms); isBuyerMaker(req):bool | object | Weight: 25 | Recent market trades | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |
| crypto.public.rest.klines.get | GET | https://api.binance.com | /api/v3/klines | v3 | Kline/candlestick data | none | symbol(req):string; interval(req):string; startTime(opt):int(ms); endTime(opt):int(ms); limit(opt):int; timeZone(opt):string | - | - | array<array<string|int>> | [openTime,open,high,low,close,volume,closeTime,quoteAssetVolume,numberOfTrades,takerBuyBaseVolume,takerBuyQuoteVolume,ignore] | object | Weight: 2 | Array-based OHLCV schema | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints |

## Spot Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.private.rest.order.post | POST | https://api.binance.com | /api/v3/order | v3 | Place new order | signed | symbol(req):string; side(req):string; type(req):string; timeInForce(opt):string; quantity(opt):string(decimal); quoteOrderQty(opt):string(decimal); price(opt):string(decimal); recvWindow(opt):float; timestamp(req):int(ms) | - | - | object | symbol(req):string; orderId(req):int; clientOrderId(req):string; transactTime(req):int(ms) | object | Order count + weight rules apply | Signed TRADE endpoint | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints |
| crypto.private.rest.order.get | GET | https://api.binance.com | /api/v3/order | v3 | Query order | signed | symbol(req):string; orderId(opt):int; origClientOrderId(opt):string; recvWindow(opt):float; timestamp(req):int(ms) | - | - | object | symbol(req):string; orderId(req):int; status(req):string; price(req):string(decimal); origQty(req):string(decimal); executedQty(req):string(decimal) | object | Weight: 4 | Signed USER_DATA endpoint | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints |
| crypto.private.rest.openorders.get | GET | https://api.binance.com | /api/v3/openOrders | v3 | Current open orders | signed | symbol(opt):string; recvWindow(opt):float; timestamp(req):int(ms) | - | - | array<object> | symbol(req):string; orderId(req):int; status(req):string; type(req):string; side(req):string | object | Weight varies by symbol presence | Open order list | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints |
| crypto.private.rest.account.get | GET | https://api.binance.com | /api/v3/account | v3 | Account information | signed | recvWindow(opt):float; timestamp(req):int(ms); omitZeroBalances(opt):bool | - | - | object | makerCommission(req):int; takerCommission(req):int; canTrade(req):bool; balances(req):array<object> | object | Weight: 20 | Signed USER_DATA endpoint | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints |
| crypto.private.rest.mytrades.get | GET | https://api.binance.com | /api/v3/myTrades | v3 | Account trade history | signed | symbol(req):string; orderId(opt):int; startTime(opt):int(ms); endTime(opt):int(ms); fromId(opt):int; limit(opt):int; recvWindow(opt):float; timestamp(req):int(ms) | - | - | array<object> | symbol(req):string; id(req):int; orderId(req):int; price(req):string(decimal); qty(req):string(decimal); quoteQty(req):string(decimal); commission(req):string(decimal); time(req):int(ms) | object | Weight: 20 | Signed trade history | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints |
| crypto.private.rest.listenkey.post | POST | https://api.binance.com | /api/v3/userDataStream | v3 | Start user data stream (legacy listenKey) | api-key | - | - | - | object | listenKey(req):string | object | Weight: 1 | Legacy flow; see changelog for WS API migration | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/user-data-stream-endpoints |

## Spot Other (doc-ref)
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.rest.docs.enums.ref | GET | docs://binance-spot | /rest-api/enum-definitions | n/a | doc-ref | none | - | - | - | object | enum catalogs (req):array<object> | object | n/a | Non-endpoint shared enums | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/enum-definitions |
| crypto.public.rest.docs.filters.ref | GET | docs://binance-spot | /rest-api/filters | n/a | doc-ref | none | - | - | - | object | filter definitions (req):array<object> | object | n/a | Non-endpoint filter rules | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/filters |
| crypto.public.rest.docs.errors.ref | GET | docs://binance-spot | /rest-api/error-codes | n/a | doc-ref | none | - | - | - | object | error code list (req):array<object> | object | n/a | Shared error code reference | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/error-codes |
| crypto.public.rest.docs.limits.ref | GET | docs://binance-spot | /rest-api/limits | n/a | doc-ref | none | - | - | - | object | REQUEST_WEIGHT/ORDERS limits (req):array<object> | object | n/a | Shared rate limit rules | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits |

## Other
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.rest.changelog.ref | GET | docs://binance-spot | /changelog | n/a | doc-ref | none | - | - | - | object | dated change entries(req):array<object> | object | n/a | Track behavior changes incl. listenKey and ws-api updates | https://developers.binance.com/docs/binance-spot-api-docs/changelog |
