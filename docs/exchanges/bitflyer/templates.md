# Update Templates

## 1) New source evidence row (`sources.md`)

```md
| <category> | <title> | <url> | <YYYY-MM-DD> | <notes> |
```

## 2) New REST endpoint row (`rest-api.md`)

```md
| <id> | <method> | <base_url> | <path> | <version> | <operation> | <auth.type> | <params.query> | <params.path> | <params.body> | <response.shape> | <response.fields> | <errors.shape> | <rate_limit> | <notes> | <source_url> |
```

## 3) New WS channel row (`websocket.md`)

```md
| <id> | <ws_url> | <version> | <channel> | <subscribe.template> | <unsubscribe.template> | <message.shape> | <message.fields> | <heartbeat.type> | <auth.type> | <restrictions> | <notes> | <source_url> |
```

## 4) New data feed row (`data.md`)

```md
| <id> | <kind> | <url_pattern> | <format> | <compression> | <update_freq> | <retention> | <schema.summary> | <notes> | <source_url> |
```

## 5) `catalog.json` object templates

```json
{
  "id": "crypto.public.rest.example.get",
  "segment": "crypto",
  "access": "public",
  "protocol": "rest",
  "method": "GET",
  "base_url": "https://api.bitflyer.com",
  "path": "/v1/example",
  "version": "v1",
  "operation": "Example operation",
  "auth": { "type": "none" },
  "params": { "query": "-", "path": "-", "body": "-" },
  "response": { "shape": "object", "fields": "example(req):string" },
  "errors": { "shape": "object" },
  "rate_limit": "see official rate-limit doc",
  "notes": "",
  "source_url": "https://bf-lightning-api.readme.io/docs/http-api"
}
```

```json
{
  "id": "crypto.public.ws.example",
  "segment": "crypto",
  "access": "public",
  "protocol": "ws",
  "ws_url": "wss://ws.lightstream.bitflyer.com/json-rpc",
  "version": "json-rpc 2.0",
  "channel": "lightning_example_BTC_JPY",
  "subscribe": { "template": "{\"method\":\"subscribe\",\"params\":{\"channel\":\"lightning_example_BTC_JPY\"}}" },
  "unsubscribe": { "template": "{\"method\":\"unsubscribe\",\"params\":{\"channel\":\"lightning_example_BTC_JPY\"}}" },
  "message": { "shape": "object", "fields": "channel(req):string; message(req):object" },
  "heartbeat": { "type": "none documented" },
  "auth": { "type": "none" },
  "restrictions": "product must exist",
  "notes": "",
  "source_url": "https://bf-lightning-api.readme.io/docs/realtime-api"
}
```
