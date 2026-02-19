# WebSocket Catalog (Official) — Bitget

Order:
1) Spot Public WS
2) Spot Private WS
3) Mix(Futures) Public WS
4) Mix(Futures) Private WS
5) UTA WS (as documented)
6) WS Common
7) Other

Fixed columns:
id | ws_url | version | channel |
subscribe.template | unsubscribe.template |
message.shape | message.fields |
heartbeat.type | auth.type | restrictions | notes | source_url

## Spot Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

## Spot Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

## Mix(Futures) Public WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

## Mix(Futures) Private WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

## UTA WS
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|

## WS Common
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.nav.blocked | unknown | unknown | common | {"op":"subscribe","args":[]} | {"op":"unsubscribe","args":[]} | object | object<k:string,v:any> | other | other | unknown | Official WebSocket pages could not be fetched in this runtime (HTTP 403 via proxy), so channel-level extraction is pending. | https://www.bitget.com/api-doc/common/websocket-intro |

## Other
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
