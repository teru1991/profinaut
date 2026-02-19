# Coinbase Catalog Update Templates

## sources.md row template
`| category | title | url | last_checked_at(YYYY-MM-DD) | notes |`

## rest-api.md row template
`| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |`

## websocket.md row template
`| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |`

## fix.md row template
`| id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url |`

## data.md row template
`| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |`

## diffs.md row template
`| date(YYYY-MM-DD) | change_type | affected_area | surface | market_scope | summary | action_required | source_url |`

## catalog.json object templates

### REST endpoint
```json
{
  "id": "advanced.crypto.public.rest.resource.action",
  "method": "GET",
  "base_url": "https://api.coinbase.com",
  "path": "/example",
  "version": "v1",
  "operation": "summary",
  "auth": { "type": "none" },
  "params": {
    "query": [],
    "path": [],
    "body": []
  },
  "response": {
    "shape": "object",
    "fields": []
  },
  "errors": { "shape": "object" },
  "rate_limit": "official page note",
  "notes": "",
  "source_url": "https://docs.cdp.coinbase.com/..."
}
```

### field template
```json
{
  "name": "field_name",
  "type": "string",
  "required": true,
  "notes": ""
}
```
