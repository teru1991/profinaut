# bitbank Official Docs Catalog (SSOT)

This directory contains an **official-docs-only** catalog for bitbank.
JP/EN both are covered. Completeness is evidenced by `sources.md`.

## Files
- sources.md: SSOT evidence of coverage (JP/EN, reachable pages)
- rest-api.md: REST endpoints catalog (public/private, crypto/fx)
- websocket.md: Stream / Realtime channels catalog (public/private, crypto/fx)
- data.md: Official data distribution catalog (if any; otherwise "not documented" with evidence)
- catalog.json: Machine-readable SSOT (1:1 with markdown tables)
- templates.md: Update templates (for future additions)
- CHANGELOG.md: Append-only change history

## Update Process (Must follow)
1) Fill `sources.md` first (JP/EN, navigation reachable pages).
2) Use `sources.md` as evidence to fill `rest-api.md`, `websocket.md`, `data.md`.
3) Generate `catalog.json` matching markdown 1:1 (IDs unique).
4) Update `templates.md`.
5) Append to `CHANGELOG.md`.

## Verification
```bash
python -c "import json; json.load(open('docs/exchanges/bitbank/catalog.json')); print('OK JSON parse')"

python - <<'PY'
import json
d=json.load(open('docs/exchanges/bitbank/catalog.json'))
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
- All table rows must include source_url.
