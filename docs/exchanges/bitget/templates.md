# Bitget Catalog Update Templates

## 1) `sources.md` row template
`| category | title | url | last_checked_at(YYYY-MM-DD) | notes |`

## 2) `rest-api.md` row template
`| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |`

## 3) `websocket.md` row template
`| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |`

## 4) `fix.md` row template
`| id | fix_host | fix_port | version | session | auth.type | flow.summary | message.types | fields.summary | heartbeat.type | restrictions | notes | source_url |`

## 5) `data.md` row template
`| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |`

## 6) `diffs.md` row template
`| date(YYYY-MM-DD) | change_type | affected_area | scope | summary | action_required | source_url |`

## 7) `catalog.json` object templates

REST endpoint:
```json
{
  "id": "spot.public.rest.market.tickers",
  "domain": "spot",
  "visibility": "public",
  "method": "GET",
  "base_url": "https://api.bitget.com",
  "path": "/api/v2/spot/market/tickers",
  "version": "v2",
  "operation": "Get tickers",
  "auth": { "type": "none" },
  "params": {
    "query": [{ "name": "symbol", "type": "string", "required": false }],
    "path": [],
    "body": []
  },
  "response": {
    "shape": "object",
    "fields": [{ "name": "data", "type": "array<object>", "required": true }]
  },
  "errors": { "shape": "object" },
  "rate_limit": "as documented",
  "notes": "",
  "source_url": "https://www.bitget.com/api-doc/..."
}
```

WS channel:
```json
{
  "id": "mix.public.ws.ticker.snapshot",
  "domain": "mix",
  "visibility": "public",
  "ws_url": "wss://ws.bitget.com/...",
  "version": "v2",
  "channel": "ticker",
  "subscribe_template": "{\"op\":\"subscribe\",\"args\":[...]}",
  "unsubscribe_template": "{\"op\":\"unsubscribe\",\"args\":[...]}",
  "message": {
    "shape": "object",
    "fields": [{ "name": "arg", "type": "object", "required": true }]
  },
  "heartbeat": { "type": "ping-pong" },
  "auth": { "type": "none" },
  "restrictions": "as documented",
  "notes": "",
  "source_url": "https://www.bitget.com/api-doc/..."
}
```
