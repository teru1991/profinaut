# Upbit Catalog Update Templates

## 1) sources.md row template

`| category | title | url | last_checked_at(YYYY-MM-DD) | notes |`

## 2) rest-api.md row template

`| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |`

## 3) websocket.md row template

`| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |`

## 4) fix.md row template

`| id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url |`

## 5) data.md row template

`| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |`

## 6) diffs.md row template

`| date(YYYY-MM-DD) | change_type | affected_area | scope | summary | action_required | source_url |`

## 7) catalog.json object template

```json
{
  "id": "exchange.private.rest.example.operation",
  "domain": "exchange",
  "visibility": "private",
  "auth": { "type": "signed" },
  "fields": [
    { "name": "field_name", "type": "string", "required": true }
  ],
  "source_url": "https://global-docs.upbit.com/reference"
}
```
