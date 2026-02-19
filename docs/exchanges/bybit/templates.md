# Templates

## REST row template
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| bybit.{public\|private}.rest.{group}.{name} | GET/POST | https://api.bybit.com | /v5/... | v5 | ... | none\|api-key+sign | a,b | - | - | object | retCode,retMsg,result,time | object | See docs | market_scope=Spot/Derivatives/Options | https://bybit-exchange.github.io/docs/v5/... |

## WebSocket row template
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| bybit.{public\|private}.ws.{name} | wss://stream.bybit.com/v5/... | v5 | topic | {"op":"subscribe","args":["topic"]} | {"op":"unsubscribe","args":["topic"]} | object | topic,type,ts,data | ping/pong | none\|api-key+sign | market_scope=... | ... | https://bybit-exchange.github.io/docs/v5/websocket/... |
