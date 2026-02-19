# Update Templates

## sources.md 追記テンプレ
| category | title | url | last_checked_at(YYYY-MM-DD) | notes |
|---|---|---|---|---|
| rest | <title> | <url> | <date> | <note> |

## rest-api.md 行テンプレ
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| <id> | GET/POST | https://api-cloud.bittrade.co.jp | </path> | v1 | <operation> | none/signed-hmac-sha256 | <query> | <path> | <body> | object/array | <fields> | object | <limit> | <notes> | <source_url> |

## websocket.md 行テンプレ
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| <id> | wss://api-cloud.bittrade.co.jp/ws | v1/v2 | <channel> | <sub json> | <unsub json> | object | <fields> | ping-pong | none/signed-hmac-sha256 | <restriction> | <notes> | <source_url> |

## data.md 行テンプレ
| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| <id> | <kind> | <url_pattern> | <format> | <compression> | <freq> | <retention> | <schema> | <notes> | <source_url> |
