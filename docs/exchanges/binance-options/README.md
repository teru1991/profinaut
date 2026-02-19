# Binance Options Trading Official Docs Catalog (SSOT)

This directory contains an official-docs-only catalog for **Binance Derivatives → Options Trading**.

## Files
- `sources.md`: Official navigation/source evidence (Options Trading + Derivatives Change Log)
- `rest-api.md`: REST categories/endpoints catalog with source URLs
- `websocket.md`: WebSocket market/private stream catalog
- `data.md`: Data feed distribution catalog
- `diffs.md`: Options-impacting changes extracted from Derivatives Change Log
- `catalog.json`: Machine-readable SSOT aligned 1:1 with markdown tables
- `templates.md`: Reusable row templates
- `CHANGELOG.md`: Append-only update history

## Update order (mandatory)
1. Complete `sources.md` first.
2. Fill `rest-api.md`, `websocket.md`, and `data.md` using `sources.md` evidence.
3. Extract Options-impacting entries into `diffs.md`.
4. Keep `catalog.json` synchronized 1:1 with markdown rows and unique IDs.
5. Append work log to `CHANGELOG.md`.

## Verification
```bash
python -c "import json; json.load(open('docs/exchanges/binance-options/catalog.json')); print('OK JSON parse')"

python - <<'PY'
import json
p='docs/exchanges/binance-options/catalog.json'
d=json.load(open(p))
ids=[x['id'] for x in d.get('rest_endpoints',[])] + [x['id'] for x in d.get('ws_channels',[])] + [x['id'] for x in d.get('data_feeds',[])]
assert len(ids)==len(set(ids)), 'duplicate ids'
print('OK unique ids:', len(ids))
PY
```
