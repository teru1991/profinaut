# HTX WebSocket Catalog (Official Only)

## 1) Spot Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.ws.market.catalog.index | wss://api-aws.huobi.pro/ws | v2 | market.* | {"sub":"market.$symbol.$topic"} | {"unsub":"market.$symbol.$topic"} | object | object<k:string,v:any> | ping-pong | none | connection limits updated (see announcement) | ws v2 policy update documented by official announcement | https://www.htx.com/support/900000916183/ |

## 2) Spot Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.ws.account.catalog.index | wss://api-aws.huobi.pro/ws/v2 | v2 | accounts.* | {"action":"sub","ch":"accounts.update"} | {"action":"unsub","ch":"accounts.update"} | object | object<k:string,v:any> | ping-pong | signed | ws v2 restrictions apply | private ws category tracked via official ws v2 update notice | https://www.htx.com/support/900000916183/ |

## 3) Futures Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.public.ws.market.catalog.index | wss://www.hbdm.com/ws | unknown | market.* | {"sub":"market.$contract_code.$topic"} | {"unsub":"market.$contract_code.$topic"} | object | object<k:string,v:any> | ping-pong | none | unknown | futures ws product family evidenced by official futures api support entry | https://www.htx.com/support/360000188382 |

## 4) Futures Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| futures.private.ws.account.catalog.index | wss://www.hbdm.com/notification | unknown | accounts.* | {"op":"sub","topic":"orders.$symbol"} | {"op":"unsub","topic":"orders.$symbol"} | object | object<k:string,v:any> | ping-pong | signed | unknown | futures private ws channels expected in official futures api docs tree | https://www.htx.com/support/360000188382 |

## 5) Swap Public WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| swap.public.ws.market.catalog.index | wss://api.hbdm.com/swap-ws | unknown | market.* | {"sub":"market.$contract_code.$topic"} | {"unsub":"market.$contract_code.$topic"} | object | object<k:string,v:any> | ping-pong | none | unknown | swap ws domain tracked from official OpenAPI entrypoint and derivatives support docs | https://www.htx.com/en-us/opend/newApiPages/ |

## 6) Swap Private WS

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| swap.private.ws.account.catalog.index | wss://api.hbdm.com/swap-notification | unknown | orders.* | {"op":"sub","topic":"orders.$contract_code"} | {"op":"unsub","topic":"orders.$contract_code"} | object | object<k:string,v:any> | ping-pong | signed | unknown | swap private ws expected under official derivatives API docs family | https://www.htx.com/en-us/opend/newApiPages/ |

## 7) Options WS（as documented）

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.public.ws.market.catalog.index | wss://api.hbdm.com/option-ws | unknown | option.* | {"sub":"market.$symbol.$topic"} | {"unsub":"market.$symbol.$topic"} | object | object<k:string,v:any> | ping-pong | none | unknown | options ws family inferred from official options api version history coverage | https://www.htx.com/support/900003014323/ |

## 8) WS Common（token/login/heartbeat/limits/error codes等）

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.common.protocol | wss://api-aws.huobi.pro/ws/v2 | v2 | common | {"action":"sub","ch":"$channel"} | {"action":"unsub","ch":"$channel"} | object | object<k:string,v:any> | ping-pong | other | ws connection/session limits changed by announcement | official ws v2 notice used as canonical common constraints source | https://www.htx.com/support/900000916183/ |

## 9) Other

| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| other.public.ws.other.not_applicable | not_applicable | not_applicable | not_applicable | {} | {} | object | object | none | other | unknown | no additional official ws family identified from current accessible source set | https://www.htx.com/en-us/opend/newApiPages/ |
