# Templates

## sources.md row template
| category | title | url | last_checked_at(YYYY-MM-DD) | notes |

## rest-api.md row template
| id | method | base_url | path | version | operation | auth.type | params.query | params.path | params.body | response.shape | response.fields | errors.shape | rate_limit | notes | source_url |

## websocket.md row template
| id | ws_url | version | channel | subscribe.template | unsubscribe.template | message.shape | message.fields | heartbeat.type | auth.type | restrictions | notes | source_url |

## data.md row template
| id | kind | url_pattern | format | compression | update_freq | retention | schema.summary | notes | source_url |

## catalog.json entry templates
### REST entry (skeleton)
```json
{
  "id": "",
  "service": "crypto",
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
  "service": "crypto",
  "visibility": "public",
  "ws_url": "",
  "version": "",
  "channel": "",
  "subscribe": { "template": "", "notes": "" },
  "unsubscribe": { "template": "", "notes": "" },
  "message": { "shape": "object", "fields": [], "notes": "" },
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
  "format": "",
  "compression": "",
  "update_freq": "",
  "retention": "",
  "schema": { "summary": "", "notes": "" },
  "notes": "",
  "source_url": ""
}
```
