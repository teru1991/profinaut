# Bithumb WebSocket Catalog (Official Only)

## 1) Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| openapi.public.ws.ticker.snapshot | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | ticker | {"type":"ticker","symbols":["BTC_KRW"],"tickTypes":["30M"]} | {"type":"ticker","symbols":["BTC_KRW"],"isOnlyRealtime":false} | object | type:string(req), content:object(req), timestamp:int(ms)(opt) | ping-pong | none | request count limits announced in changelog | Public ticker stream. | https://apidocs.bithumb.com/reference/%ED%98%84%EC%9E%AC%EA%B0%80-ticker |
| openapi.public.ws.trade.snapshot | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | trade | {"type":"trade","symbols":["BTC_KRW"]} | {"type":"trade","symbols":["BTC_KRW"],"isOnlyRealtime":false} | object | type:string(req), content:array<object>(req) | ping-pong | none | request count limits announced in changelog | Public trade stream. | https://apidocs.bithumb.com/reference/%EC%B2%B4%EA%B2%B0-trade |
| openapi.public.ws.orderbook.snapshot | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | orderbook | {"type":"orderbook","symbols":["BTC_KRW"]} | {"type":"orderbook","symbols":["BTC_KRW"],"isOnlyRealtime":false} | object | type:string(req), content:object(req), list:array<object>(opt) | ping-pong | none | request count limits announced in changelog | Public orderbook stream. | https://apidocs.bithumb.com/reference/%ED%98%B8%EA%B0%80-orderbook |

## 2) Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| openapi.private.ws.myorder.update | wss://pubwss.bithumb.com/private/ws | v2.1.5 | myOrder | {"type":"myOrder","symbols":["BTC_KRW"],"token":"<jwt_or_signed_token>"} | {"type":"myOrder","symbols":["BTC_KRW"],"token":"<token>","isOnlyRealtime":false} | object | type:string(req), content:array<object>(req), orderId:string(opt) | ping-pong | signed | requires private auth token | Private order/fill stream. | https://apidocs.bithumb.com/reference/%EB%82%B4-%EC%A3%BC%EB%AC%B8-%EB%B0%8F-%EC%B2%B4%EA%B2%B0-myorder |
| openapi.private.ws.myasset.update | wss://pubwss.bithumb.com/private/ws | v2.1.5 | myAsset | {"type":"myAsset","symbols":["ALL"],"token":"<jwt_or_signed_token>"} | {"type":"myAsset","symbols":["ALL"],"token":"<token>","isOnlyRealtime":false} | object | type:string(req), content:array<object>(req), balance:string(decimal)(opt) | ping-pong | signed | requires private auth token | Private balance stream. | https://apidocs.bithumb.com/reference/%EB%82%B4-%EC%9E%90%EC%82%B0-myasset |

## 3) WS Common（요청 방법/포맷/에러/연결관리/제한）

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| openapi.public.ws.common.connection | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | connection-management | {"ping":<int(ms)>} | n/a | object | status:string(req), message:string(opt) | ping-pong | none | connection and idle handling per docs | WS 연결 관리 page baseline. | https://apidocs.bithumb.com/reference/%EC%97%B0%EA%B2%B0-%EA%B4%80%EB%A6%AC |
| openapi.public.ws.common.request_format | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | request-format | {"type":"<channel>","symbols":array<string>} | {"type":"<channel>","symbols":array<string>,"isOnlyRealtime":bool} | object | type:string(req), symbols:array<string>(req), tickTypes:array<string>(opt) | none | none | malformed payload returns websocket error object | WS 요청 포맷 reference. | https://apidocs.bithumb.com/reference/%EC%9A%94%EC%B2%AD-%ED%8F%AC%EB%A7%B7 |
| openapi.public.ws.common.errors | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | websocket-errors | n/a | n/a | object | status:string(req), resmsg:string(req), errorCode:string(opt) | none | none | includes invalid type/param handling | WS 에러 schema page. | https://apidocs.bithumb.com/reference/%EC%9B%B9%EC%86%8C%EC%BC%93-%EC%97%90%EB%9F%AC |
| openapi.public.ws.common.type_response | wss://pubwss.bithumb.com/pub/ws | v2.1.5 | type-response-map | {"type":"ticker|trade|orderbook|myOrder|myAsset"} | n/a | object | type:string(req), content:object|array<object>(req) | none | other | private types require auth token | Type-specific request/response matrix. | https://apidocs.bithumb.com/reference/%ED%83%80%EC%9E%85%EB%B3%84-%EC%9A%94%EC%B2%AD-%EB%B0%8F-%EC%9D%91%EB%8B%B5 |
