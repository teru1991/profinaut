# Bybit Official Docs Complete Catalog (V5 REST/WS)

This directory contains an official-docs-only catalog for Bybit V5, covering Spot / Derivatives / Options scopes, with REST + WebSocket + changelog diffs.

## Files
- `sources.md`: official source roots and evidence links
- `rest-api.md`: REST endpoint catalog (from V5 API Explorer)
- `websocket.md`: V5 WebSocket topic catalog (public/private/trade/system)
- `data.md`: official data distribution catalog
- `diffs.md`: API/WS impacting entries extracted from V5 changelog
- `catalog.json`: machine-readable SSOT aligned to markdown IDs
- `templates.md`: reusable row templates
- `CHANGELOG.md`: append-only update history

## Verification
```bash
python -c "import json; json.load(open('docs/exchanges/bybit/catalog.json')); print('OK JSON parse')"
python - <<'PY'
import json
d=json.load(open('docs/exchanges/bybit/catalog.json'))
ids=[x['id'] for x in d['rest_endpoints']]+[x['id'] for x in d['ws_channels']]+[x['id'] for x in d['data_feeds']]
assert len(ids)==len(set(ids))
print('OK unique ids:',len(ids))
PY
```
