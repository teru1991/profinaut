# Deribit Catalog Update Templates

## 1) sources.md row template
`| category | title | url | last_checked_at(YYYY-MM-DD) | notes |`

## 2) jsonrpc-ws/http row template
`| id | base_url | transport | version | method | operation | auth.type | params | result.shape | result.fields | errors.shape | rate_limit | notes | source_url |`

## 3) subscriptions row template
`| id | ws_url | channel | subscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |`

## 4) fix row template
`| id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url |`

## 5) data row template
`| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |`

## 6) diffs row template
`| date(YYYY-MM-DD) | change_type | affected_area | surface | scope | summary | action_required | source_url |`

## 7) catalog.json object template (per item)
```json
{
  "id": "jsonrpc.http.public.namespace.method",
  "source_url": "https://docs.deribit.com/...",
  "fields": [
    {"name": "field_name", "type": "string", "required": true}
  ]
}
```
