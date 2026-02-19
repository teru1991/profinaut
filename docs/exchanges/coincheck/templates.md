# Update Templates

## sources.md 追記テンプレ
| category | title | url | last_checked_at(YYYY-MM-DD) | notes |
|---|---|---|---|---|
| rest | <title> | <url> | <date> | <note> |

## rest-api.md 行テンプレ
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| coincheck.rest.public.<x> | GET/POST/DELETE | https://coincheck.com | /api/<path> | v1 | <operation> | none/signature | <query> | <path> | <body> | object/array | <fields> | object(success/error) | <limit> | <notes> | <source_url> |

## websocket.md 行テンプレ
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| coincheck.ws.public.<x> | wss://ws-api.coincheck.com | v1 | <channel> | <subscribe json> | <unsubscribe json> | object/array | <fields> | <heartbeat> | none/signature | <restrictions> | <notes> | <source_url> |

## data.md 行テンプレ
| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| coincheck.data.<x> | <kind> | <url_pattern> | <format> | <compression> | <freq> | <retention> | <schema> | <notes> | <source_url> |
