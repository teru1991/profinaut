# HTX Catalog Update Templates

## sources.md row template
`category | title | url | last_checked_at(YYYY-MM-DD) | notes`

## rest-api.md row template
`id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url`

## websocket.md row template
`id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url`

## fix.md row template
`id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url`

## data.md row template
`id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url`

## diffs.md row template
`date(YYYY-MM-DD) | change_type | affected_area | scope | summary | action_required | source_url`

## catalog.json object templates
- rest endpoint: `{id, domain, visibility, method, base_url, path, version, operation, auth, params, response, errors, rate_limit, notes, source_url}`
- ws channel: `{id, domain, visibility, ws_url, version, channel, subscribe, unsubscribe, message, heartbeat, auth, restrictions, notes, source_url}`
- fix feed: `{id, domain, visibility, ...}`
- data feed: `{id, kind, ...}`
