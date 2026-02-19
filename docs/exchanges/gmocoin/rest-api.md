# REST API Catalog (Official)

Order:
1) Crypto Public REST
2) Crypto Private REST
3) FX Public REST
4) FX Private REST
5) Other

Fixed columns:
id | method | base_url | path | version | operation | auth.type |
params.query | params.path | params.body |
response.shape | response.fields |
errors.shape | rate_limit | notes | source_url

## Crypto Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.public.rest.status.get | GET | https://api.coin.z.com | /public/v1/status | v1 | Get service status | none | - | - | - | object | status(req):int; responsetime(req):string(iso8601) | object | docs reference | Public health endpoint | https://api.coin.z.com/docs/#/public/getStatus |
| crypto.public.rest.ticker.get | GET | https://api.coin.z.com | /public/v1/ticker | v1 | Get ticker | none | symbol(opt):string | - | - | object | symbol(req):string; ask(req):string(decimal); bid(req):string(decimal); timestamp(req):string(iso8601) | object | docs reference | Market snapshot by symbol | https://api.coin.z.com/docs/#/public/getTicker |
| crypto.public.rest.orderbooks.get | GET | https://api.coin.z.com | /public/v1/orderbooks | v1 | Get order book | none | symbol(req):string | - | - | object | asks(req):array<object>; bids(req):array<object>; timestamp(req):string(iso8601) | object | docs reference | Depth levels for a symbol | https://api.coin.z.com/docs/#/public/getOrderbooks |
| crypto.public.rest.trades.get | GET | https://api.coin.z.com | /public/v1/trades | v1 | Get recent trades | none | symbol(req):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | Public executions stream snapshot | https://api.coin.z.com/docs/#/public/getTrades |
| crypto.public.rest.klines.get | GET | https://api.coin.z.com | /public/v1/klines | v1 | Get candlesticks | none | symbol(req):string; interval(req):string; date(req):string | - | - | object | data(req):array<object> | object | docs reference | OHLCV history | https://api.coin.z.com/docs/#/public/getKlines |

## Crypto Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.private.rest.wsauth.post | POST | https://api.coin.z.com | /private/v1/ws-auth | v1 | Create WS auth token | signed | - | - | timestamp(req):int(ms); nonce(req):string | object | token(req):string | object | docs reference | Required before private WS connect | https://api.coin.z.com/docs/#/private/postWsAuth |
| crypto.private.rest.wsauth.extend.put | PUT | https://api.coin.z.com | /private/v1/ws-auth | v1 | Extend WS auth token | signed | - | - | token(req):string; timestamp(req):int(ms); nonce(req):string | object | status(req):int | object | docs reference | Keep private WS session alive | https://api.coin.z.com/docs/#/private/putWsAuth |
| crypto.private.rest.assets.get | GET | https://api.coin.z.com | /private/v1/account/assets | v1 | Get account assets | signed | - | - | - | object | assets(req):array<object> | object | docs reference | Spot margin balances | https://api.coin.z.com/docs/#/private/getAssets |
| crypto.private.rest.margin.get | GET | https://api.coin.z.com | /private/v1/account/margin | v1 | Get margin status | signed | - | - | - | object | actualProfitLoss(req):string(decimal); availableAmount(req):string(decimal) | object | docs reference | Account margin summary | https://api.coin.z.com/docs/#/private/getMargin |
| crypto.private.rest.activeorders.get | GET | https://api.coin.z.com | /private/v1/activeOrders | v1 | Get active orders | signed | symbol(opt):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | Open orders listing | https://api.coin.z.com/docs/#/private/getActiveOrders |
| crypto.private.rest.executions.get | GET | https://api.coin.z.com | /private/v1/executions | v1 | Get execution history | signed | symbol(opt):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | Historical private fills | https://api.coin.z.com/docs/#/private/getExecutions |
| crypto.private.rest.latestexecutions.get | GET | https://api.coin.z.com | /private/v1/latestExecutions | v1 | Get latest execution per order | signed | symbol(opt):string | - | - | object | list(req):array<object> | object | docs reference | Latest fill state | https://api.coin.z.com/docs/#/private/getLatestExecutions |
| crypto.private.rest.order.post | POST | https://api.coin.z.com | /private/v1/order | v1 | Create order | signed | - | - | symbol(req):string; side(req):string; executionType(req):string; timeInForce(opt):string; size(req):string(decimal); price(opt):string(decimal) | object | orderId(req):int|string | object | docs reference | Spot/derivative order placement | https://api.coin.z.com/docs/#/private/postOrder |
| crypto.private.rest.changeorder.post | POST | https://api.coin.z.com | /private/v1/changeOrder | v1 | Amend order | signed | - | - | orderId(req):int|string; price(opt):string(decimal); size(opt):string(decimal) | object | orderId(req):int|string | object | docs reference | Replace order params | https://api.coin.z.com/docs/#/private/postChangeOrder |
| crypto.private.rest.cancelorder.post | POST | https://api.coin.z.com | /private/v1/cancelOrder | v1 | Cancel order | signed | - | - | orderId(req):int|string | object | orderId(req):int|string | object | docs reference | Cancel single order | https://api.coin.z.com/docs/#/private/postCancelOrder |
| crypto.private.rest.openpositions.get | GET | https://api.coin.z.com | /private/v1/openPositions | v1 | Get open positions | signed | symbol(opt):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | Leveraged positions | https://api.coin.z.com/docs/#/private/getOpenPositions |
| crypto.private.rest.positionsummary.get | GET | https://api.coin.z.com | /private/v1/positionSummary | v1 | Get position summary | signed | symbol(req):string; side(opt):string | - | - | object | list(req):array<object> | object | docs reference | Summary by side/symbol | https://api.coin.z.com/docs/#/private/getPositionSummary |
| crypto.private.rest.closeorder.post | POST | https://api.coin.z.com | /private/v1/closeOrder | v1 | Close position by order | signed | - | - | symbol(req):string; side(req):string; size(req):string(decimal); positionId(req):int|string | object | orderId(req):int|string | object | docs reference | Position close order | https://api.coin.z.com/docs/#/private/postCloseOrder |

## FX Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.public.rest.status.get | GET | https://api.coin.z.com | /fx/public/v1/status | v1 | Get FX API status | none | - | - | - | object | status(req):int; responsetime(req):string(iso8601) | object | docs reference | FX service status | https://api.coin.z.com/fxdocs/#/public/getStatus |
| fx.public.rest.ticker.get | GET | https://api.coin.z.com | /fx/public/v1/ticker | v1 | Get FX ticker | none | symbol(req):string | - | - | object | symbol(req):string; ask(req):string(decimal); bid(req):string(decimal); timestamp(req):string(iso8601) | object | docs reference | FX quote snapshot | https://api.coin.z.com/fxdocs/#/public/getTicker |
| fx.public.rest.orderbooks.get | GET | https://api.coin.z.com | /fx/public/v1/orderbooks | v1 | Get FX order book | none | symbol(req):string | - | - | object | asks(req):array<object>; bids(req):array<object> | object | docs reference | FX depth data | https://api.coin.z.com/fxdocs/#/public/getOrderbooks |
| fx.public.rest.trades.get | GET | https://api.coin.z.com | /fx/public/v1/trades | v1 | Get FX trades | none | symbol(req):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | Public FX trades | https://api.coin.z.com/fxdocs/#/public/getTrades |
| fx.public.rest.klines.get | GET | https://api.coin.z.com | /fx/public/v1/klines | v1 | Get FX klines | none | symbol(req):string; interval(req):string; date(req):string | - | - | object | data(req):array<object> | object | docs reference | FX OHLCV | https://api.coin.z.com/fxdocs/#/public/getKlines |

## FX Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.private.rest.wsauth.post | POST | https://api.coin.z.com | /fx/private/v1/ws-auth | v1 | Create FX WS auth token | signed | - | - | timestamp(req):int(ms); nonce(req):string | object | token(req):string | object | docs reference | FX private WS bootstrap | https://api.coin.z.com/fxdocs/#/private/postWsAuth |
| fx.private.rest.assets.get | GET | https://api.coin.z.com | /fx/private/v1/account/assets | v1 | Get FX account assets | signed | - | - | - | object | assets(req):array<object> | object | docs reference | FX account balances | https://api.coin.z.com/fxdocs/#/private/getAssets |
| fx.private.rest.activeorders.get | GET | https://api.coin.z.com | /fx/private/v1/activeOrders | v1 | Get FX active orders | signed | symbol(opt):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | FX open orders | https://api.coin.z.com/fxdocs/#/private/getActiveOrders |
| fx.private.rest.order.post | POST | https://api.coin.z.com | /fx/private/v1/order | v1 | Create FX order | signed | - | - | symbol(req):string; side(req):string; executionType(req):string; size(req):string(decimal); price(opt):string(decimal) | object | orderId(req):int|string | object | docs reference | FX order placement | https://api.coin.z.com/fxdocs/#/private/postOrder |
| fx.private.rest.cancelorder.post | POST | https://api.coin.z.com | /fx/private/v1/cancelOrder | v1 | Cancel FX order | signed | - | - | orderId(req):int|string | object | orderId(req):int|string | object | docs reference | FX order cancel | https://api.coin.z.com/fxdocs/#/private/postCancelOrder |
| fx.private.rest.openpositions.get | GET | https://api.coin.z.com | /fx/private/v1/openPositions | v1 | Get FX open positions | signed | symbol(opt):string; page(opt):int; count(opt):int | - | - | object | list(req):array<object> | object | docs reference | FX positions | https://api.coin.z.com/fxdocs/#/private/getOpenPositions |
| fx.private.rest.closeorder.post | POST | https://api.coin.z.com | /fx/private/v1/closeOrder | v1 | Close FX position | signed | - | - | symbol(req):string; side(req):string; size(req):string(decimal); positionId(req):int|string | object | orderId(req):int|string | object | docs reference | FX close order | https://api.coin.z.com/fxdocs/#/private/postCloseOrder |

## Other
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
