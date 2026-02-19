# Templates (Binance Options)

Use these templates when adding rows to markdown files and syncing `catalog.json`.

## REST row template

```md
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| options.<public|private>.rest.<name> | GET/POST/PUT/DELETE | https://eapi.binance.com | /eapi/v1/... | v1 | ... | none/keyed/signed | ... | ... | ... | object/array<object> | ... | object | ... | ... | https://developers.binance.com/docs/derivatives/option/... |
```

## WS row template

```md
| id | channel | base_url | version | op/subscription | request_message | response_message | heartbeat | auth.type | restrictions | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|---|---|
| options.<public|private>.ws.<name> | <stream> | wss://nbstream.binance.com/eoptions/ws | n/a | subscribe | ... | object | ping-pong | none/listenKey | ... | ... | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
```

## DATA row template

```md
| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |
|---|---|---|---|---|---|---|---|---|---|
| options.data.<name> | streaming/pull-api | ... | json | none | real-time/on-demand | not stated | ... | ... | https://developers.binance.com/docs/derivatives/option/... |
```

## DIFF row template

```md
| date(YYYY-MM-DD) | change_type | affected_area | summary | action_required | source_url |
|---|---|---|---|---|---|
| 2026-01-01 | added/changed/removed | rest-api/ws-streams/user-data | concise factual change | review-client/review-parser/review-limits/none | https://developers.binance.com/docs/derivatives/change-log |
```

## JSON skeletons

### REST entry (skeleton)
```json
{
  "id": "",
  "service": "crypto-derivatives",
  "visibility": "public",
  "method": "GET",
  "base_url": "",
  "path": "",
  "version": "",
  "operation": "",
  "auth": { "type": "none", "notes": "" },
  "params": { "query": [], "path": [], "body": [] },
  "response": { "shape": "object", "fields": [], "notes": "" },
  "errors": { "shape": "object", "notes": "" },
  "rate_limit": { "notes": "" },
  "notes": "",
  "source_url": ""
}
```

### WS entry (skeleton)
```json
{
  "id": "",
  "service": "crypto-derivatives",
  "visibility": "public",
  "channel": "",
  "base_url": "",
  "version": "n/a",
  "op": "subscribe",
  "request_message": { "shape": "n/a", "fields": [], "notes": "" },
  "response_message": { "shape": "object", "fields": [], "notes": "" },
  "heartbeat": { "type": "ping-pong", "notes": "" },
  "auth": { "type": "none", "notes": "" },
  "restrictions": "",
  "notes": "",
  "source_url": ""
}
```

### DATA entry (skeleton)
```json
{
  "id": "",
  "kind": "",
  "url_pattern": "",
  "format": "json",
  "compression": "none",
  "update_freq": "",
  "retention": "",
  "schema": { "summary": "", "notes": "" },
  "notes": "",
  "source_url": ""
}
```
