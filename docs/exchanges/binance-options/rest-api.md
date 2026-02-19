# REST API Catalog (Official)

Order:
1) Options Public REST
2) Options Private REST
3) Options Other (doc-ref)

Fixed columns:
id | method | base_url | path | version | operation | auth.type |
params.query | params.path | params.body |
response.shape | response.fields |
errors.shape | rate_limit | notes | source_url

## Options Public REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.public.rest.general.ref | GET | docs://binance-options | /general-info | n/a | doc-ref (general rules) | none | - | - | - | object | base_rules(req):object | object | See General Info | Timing/signature/base URL guidance for all options REST APIs | https://developers.binance.com/docs/derivatives/option/general-info |
| options.public.rest.errors.ref | GET | docs://binance-options | /error-code | n/a | doc-ref (error catalog) | none | - | - | - | object | error_codes(req):array<object> | object | n/a | Shared options error definitions | https://developers.binance.com/docs/derivatives/option/error-code |
| options.public.rest.market.ref | GET | docs://binance-options | /market-data/rest-api | n/a | doc-ref (market data family) | none | symbol(opt):string; underlying(opt):string; expiryDate(opt):string; limit(opt):int | - | - | object/array<object> | market snapshots(req):object/array | object | Per endpoint weight | Public options market data endpoints family | https://developers.binance.com/docs/derivatives/option/market-data/rest-api |

## Options Private REST
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.private.rest.trade.ref | GET/POST/DELETE | docs://binance-options | /trade/rest-api | n/a | doc-ref (trade family) | signed | symbol(req/opt):string; side(opt):string; type(opt):string; timestamp(req):int(ms); recvWindow(opt):int | - | - | object/array<object> | order lifecycle payloads(req):various | object | REQUEST_WEIGHT + ORDERS (per endpoint) | Order placement/change/cancel/query family for options | https://developers.binance.com/docs/derivatives/option/trade/rest-api |
| options.private.rest.account.ref | GET/POST | docs://binance-options | /account/rest-api | n/a | doc-ref (account family) | signed | timestamp(req):int(ms); recvWindow(opt):int | - | - | object/array<object> | balances/positions/history(req):various | object | Per endpoint weight | Account state/configuration/history family for options | https://developers.binance.com/docs/derivatives/option/account/rest-api |
| options.private.rest.listenkey.post | POST | https://eapi.binance.com | /eapi/v1/listenKey | v1 | Start user data stream | keyed | - | - | - | object | listenKey(req):string | object | Per endpoint spec | Create listenKey for options user stream | https://developers.binance.com/docs/derivatives/option/user-data-streams |
| options.private.rest.listenkey.put | PUT | https://eapi.binance.com | /eapi/v1/listenKey | v1 | Keepalive user data stream | keyed | listenKey(opt):string | - | - | object | result(req):string/object | object | Per endpoint spec | Extend listenKey validity | https://developers.binance.com/docs/derivatives/option/user-data-streams |
| options.private.rest.listenkey.delete | DELETE | https://eapi.binance.com | /eapi/v1/listenKey | v1 | Close user data stream | keyed | listenKey(opt):string | - | - | object | result(req):string/object | object | Per endpoint spec | Invalidate listenKey | https://developers.binance.com/docs/derivatives/option/user-data-streams |
