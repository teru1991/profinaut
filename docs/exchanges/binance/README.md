# Binance Spot Official Docs Catalog (SSOT)

This directory contains an **official-docs-only** catalog for Binance Spot.
Completeness evidence is tracked in `sources.md`.

## Files
- sources.md: SSOT evidence of coverage (official reachable pages)
- rest-api.md: REST endpoints catalog (public/private + doc refs)
- websocket.md: WebSocket catalog (streams / WS API / user data)
- data.md: Official data distribution catalog (SBE/FIX/Testnet artifacts)
- catalog.json: Machine-readable SSOT (aligned with markdown tables)
- templates.md: Update templates (for future additions)
- CHANGELOG.md: Append-only change history

## Update Process (Must follow)
1) Fill `sources.md` first (official navigation reachable pages).
2) Use `sources.md` as evidence to fill `rest-api.md`, `websocket.md`, `data.md`.
3) Generate `catalog.json` matching markdown 1:1 (IDs unique).
4) Update `templates.md`.
5) Append to `CHANGELOG.md`.

## Verification
```bash
python -c "import json; json.load(open('docs/exchanges/binance/catalog.json')); print('OK JSON parse')"

python - <<'PY'
import json
d=json.load(open('docs/exchanges/binance/catalog.json'))
ids=[]
for x in d.get('rest_endpoints',[]): ids.append(x['id'])
for x in d.get('ws_channels',[]): ids.append(x['id'])
for x in d.get('data_feeds',[]): ids.append(x['id'])
assert len(ids)==len(set(ids)), "duplicate ids"
print("OK unique ids:", len(ids))
PY
```

Notes
- Official docs only. No external blogs or inference.
- All table rows include `source_url`.
