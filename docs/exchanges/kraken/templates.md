# Kraken Catalog Update Templates

## sources.md row template
| category | title | url | last_checked_at(YYYY-MM-DD) | notes |
|---|---|---|---|---|
| spot-rest-public | <title> | <https://...> | <YYYY-MM-DD> | <coverage note> |

## rest-api.md row template
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.rest.<resource>.<action> | GET | https://api.kraken.com | /0/public/... | v0 | <operation> | none | <q(req/opt)> | - | - | object | <fields(req/opt + type)> | object | <official rule> | <notes> | <source url> |

## websocket.md row template
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.public.ws.v2.<family>.<event> | wss://ws.kraken.com/v2 | v2 | <channel> | <json> | <json/not_applicable> | object | <fields(req/opt + type)> | ping-pong | none | <limits> | <notes> | <source url> |

## fix.md row template
| id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spot.private.fix.<session>.<flow> | fix.kraken.com | <port> | FIX.4.4 | <session> | fix-logon | <summary> | <types> | <fields> | ping-pong | <restrictions> | <notes> | <source url> |

## data.md row template
| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| data.<kind>.<granularity>.<format> | <kind> | <url> | <format> | <compression> | <freq> | <retention> | <schema> | <notes> | <source url> |

## diffs.md row template
| date(YYYY-MM-DD) | change_type | affected_area | market_scope | summary | action_required | source_url |
|---|---|---|---|---|---|---|
| <YYYY-MM-DD> | breaking | ws | spot | <official changelog summary only> | yes | <official changelog entry url> |
