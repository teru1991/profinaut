# Kraken REST API Catalog (Official)

Fixed columns:
id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url

## 1) Spot Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.rest.assets.list | GET | https://api.kraken.com | /0/public/Assets | v0 | Get asset metadata | none | aclass(opt),asset(opt) | - | - | object | error(req):array<string>,result(req):object<k:string,v:object> | object | see spot REST rate limits guide | public asset universe | https://docs.kraken.com/api/docs/rest-api/get-asset-info/ |
| spot.public.rest.asset-pairs.list | GET | https://api.kraken.com | /0/public/AssetPairs | v0 | Get tradable pairs | none | pair(opt),info(opt) | - | - | object | error(req),result(req):object<k:string,v:object> | object | see spot REST rate limits guide | includes lot/price precision | https://docs.kraken.com/api/docs/rest-api/get-tradable-asset-pairs/ |
| spot.public.rest.ticker.get | GET | https://api.kraken.com | /0/public/Ticker | v0 | Get ticker information | none | pair(req):string | - | - | object | error(req),result(req):object<k:string,v:object> | object | see spot REST rate limits guide | multi-pair supported by comma list | https://docs.kraken.com/api/docs/rest-api/get-ticker-information/ |

## 2) Spot Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.rest.balance.get | POST | https://api.kraken.com | /0/private/Balance | v0 | Get account balances | signed | - | - | nonce(req):int|string | object | error(req),result(req):object<k:string,v:string(decimal)> | object | private tiered rate/counter | API-Key + API-Sign headers required | https://docs.kraken.com/api/docs/rest-api/get-account-balance/ |
| spot.private.rest.order.add | POST | https://api.kraken.com | /0/private/AddOrder | v0 | Add order | signed | validate(opt):bool | - | nonce(req),ordertype(req),type(req),pair(req),volume(req),price(opt),price2(opt),oflags(opt),timeinforce(opt) | object | error(req),result(req):object(txid:array<string>(opt),descr:object(opt)) | object | private trading counter applies | request body is x-www-form-urlencoded | https://docs.kraken.com/api/docs/rest-api/add-order/ |
| spot.private.rest.token.ws.get | POST | https://api.kraken.com | /0/private/GetWebSocketsToken | v0 | Get WebSocket auth token | signed | - | - | nonce(req) | object | error(req),result(req):object(token(req),expires(req):int(s)) | object | private counter applies | token used for WS private subscriptions | https://docs.kraken.com/api/docs/rest-api/get-websockets-token/ |

## 3) Futures Public REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.public.rest.instruments.list | GET | https://futures.kraken.com/derivatives | /api/v3/instruments | v3 | List futures instruments | none | - | - | - | object | serverTime(req):string(iso8601),instruments(req):array<object> | object | see futures REST guide limits | includes symbols and contract spec | https://docs.kraken.com/api/docs/futures-api/trading/get-instruments/ |
| futures.public.rest.tickers.list | GET | https://futures.kraken.com/derivatives | /api/v3/tickers | v3 | Get futures tickers | none | - | - | - | object | serverTime(req),tickers(req):array<object> | object | see futures REST guide limits | market snapshot feed by HTTP | https://docs.kraken.com/api/docs/futures-api/trading/get-tickers/ |

## 4) Futures Private REST

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.private.rest.accounts.get | GET | https://futures.kraken.com/derivatives | /api/v3/accounts | v3 | Get account information | signed | - | - | - | object | accounts(req):array<object> | object | see futures REST guide limits | requires APIKey, Authent, Nonce, APIExpires headers | https://docs.kraken.com/api/docs/futures-api/trading/get-accounts/ |
| futures.private.rest.order.send | POST | https://futures.kraken.com/derivatives | /api/v3/sendorder | v3 | Send futures order | signed | - | - | orderType(req),symbol(req),side(req),size(req):int,limitPrice(opt):string(decimal),cliOrdId(opt):string | object | sendStatus(req):object | object | private endpoint rate limits | signed with Authent header per guide | https://docs.kraken.com/api/docs/futures-api/trading/send-order |

## 5) Other (doc-ref)

| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.rest.meta.auth | GET | https://docs.kraken.com | /api/docs/guides/spot-rest-auth/ | docs | Spot REST auth model reference | none | - | - | - | object | API-Key(req),API-Sign(req),nonce(req) described in guide | object | see guide | documentation reference row | https://docs.kraken.com/api/docs/guides/spot-rest-auth/ |
