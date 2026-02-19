# WebSocket API Catalog (Official)

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
| crypto.public.ws.ticker | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_ticker_{product_code} | {"method":"subscribe","params":{"channel":"lightning_ticker_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_ticker_BTC_JPY"}} | object | channel(req):string; message(req):object(ticker fields) | none documented | none | product_code must be listed in /v1/getmarkets | realtime ticker updates | https://bf-lightning-api.readme.io/docs/realtime-ticker |
| crypto.public.ws.executions | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_executions_{product_code} | {"method":"subscribe","params":{"channel":"lightning_executions_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_executions_BTC_JPY"}} | object | channel(req):string; message(req):array<object execution> | none documented | none | product_code must be listed in /v1/getmarkets | realtime trade tape | https://bf-lightning-api.readme.io/docs/realtime-executions |
| crypto.public.ws.board | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_board_{product_code} | {"method":"subscribe","params":{"channel":"lightning_board_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_board_BTC_JPY"}} | object | channel(req):string; message(req):object(board diff: bids/asks/mid_price) | none documented | none | recommended to combine with snapshot channel | incremental board updates | https://bf-lightning-api.readme.io/docs/realtime-board |
| crypto.public.ws.board_snapshot | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_board_snapshot_{product_code} | {"method":"subscribe","params":{"channel":"lightning_board_snapshot_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_board_snapshot_BTC_JPY"}} | object | channel(req):string; message(req):object(full board snapshot) | none documented | none | initial sync channel for order book | snapshot then diff pattern | https://bf-lightning-api.readme.io/docs/realtime-board-snapshot |

## Crypto Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| crypto.private.ws.child_order_events | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | child_order_events | {"method":"auth","params":{"api_key":"<key>","timestamp":1234567890,"nonce":"<nonce>","signature":"<sign>"}} then {"method":"subscribe","params":{"channel":"child_order_events"}} | {"method":"unsubscribe","params":{"channel":"child_order_events"}} | object | channel(req):string; message(req):array<object event> | none documented | api_key+hmac signature | requires successful auth before subscribe | own child-order lifecycle events | https://bf-lightning-api.readme.io/docs/realtime-child-order-events |
| crypto.private.ws.parent_order_events | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | parent_order_events | {"method":"auth","params":{"api_key":"<key>","timestamp":1234567890,"nonce":"<nonce>","signature":"<sign>"}} then {"method":"subscribe","params":{"channel":"parent_order_events"}} | {"method":"unsubscribe","params":{"channel":"parent_order_events"}} | object | channel(req):string; message(req):array<object event> | none documented | api_key+hmac signature | requires successful auth before subscribe | own parent-order lifecycle events | https://bf-lightning-api.readme.io/docs/realtime-parent-order-events |

## FX Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.public.ws.ticker | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_ticker_FX_BTC_JPY | {"method":"subscribe","params":{"channel":"lightning_ticker_FX_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_ticker_FX_BTC_JPY"}} | object | channel(req):string; message(req):object(ticker fields) | none documented | none | FX_BTC_JPY must exist in product list | FX realtime ticker | https://bf-lightning-api.readme.io/docs/realtime-ticker |
| fx.public.ws.executions | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_executions_FX_BTC_JPY | {"method":"subscribe","params":{"channel":"lightning_executions_FX_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_executions_FX_BTC_JPY"}} | object | channel(req):string; message(req):array<object execution> | none documented | none | FX_BTC_JPY must exist in product list | FX realtime executions | https://bf-lightning-api.readme.io/docs/realtime-executions |
| fx.public.ws.board | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_board_FX_BTC_JPY | {"method":"subscribe","params":{"channel":"lightning_board_FX_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_board_FX_BTC_JPY"}} | object | channel(req):string; message(req):object(board diff) | none documented | none | use with board snapshot channel | FX realtime order book diff | https://bf-lightning-api.readme.io/docs/realtime-board |
| fx.public.ws.board_snapshot | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | lightning_board_snapshot_FX_BTC_JPY | {"method":"subscribe","params":{"channel":"lightning_board_snapshot_FX_BTC_JPY"}} | {"method":"unsubscribe","params":{"channel":"lightning_board_snapshot_FX_BTC_JPY"}} | object | channel(req):string; message(req):object(full board snapshot) | none documented | none | use for FX order book initialization | FX realtime order book snapshot | https://bf-lightning-api.readme.io/docs/realtime-board-snapshot |

## FX Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fx.private.ws.child_order_events | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | child_order_events | {"method":"auth","params":{"api_key":"<key>","timestamp":1234567890,"nonce":"<nonce>","signature":"<sign>"}} then {"method":"subscribe","params":{"channel":"child_order_events"}} | {"method":"unsubscribe","params":{"channel":"child_order_events"}} | object | channel(req):string; message(req):array<object event with FX product_code> | none documented | api_key+hmac signature | payload may include multiple products; filter by product_code=FX_BTC_JPY client-side | private FX order events on shared channel | https://bf-lightning-api.readme.io/docs/realtime-child-order-events |
| fx.private.ws.parent_order_events | wss://ws.lightstream.bitflyer.com/json-rpc | json-rpc 2.0 | parent_order_events | {"method":"auth","params":{"api_key":"<key>","timestamp":1234567890,"nonce":"<nonce>","signature":"<sign>"}} then {"method":"subscribe","params":{"channel":"parent_order_events"}} | {"method":"unsubscribe","params":{"channel":"parent_order_events"}} | object | channel(req):string; message(req):array<object event with FX product_code> | none documented | api_key+hmac signature | payload may include multiple products; filter by product_code=FX_BTC_JPY client-side | private FX parent-order events on shared channel | https://bf-lightning-api.readme.io/docs/realtime-parent-order-events |
