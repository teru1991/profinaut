# Coinbase Official Docs Catalog (SSOT)

This directory contains an **official-docs-only** catalog for Coinbase trading APIs across:
- Advanced Trade (Coinbase App)
- Exchange APIs (Institutional)
- International Exchange (INTX)

Completeness evidence is tracked in `sources.md` (official navigation-reachable pages that were used as catalog evidence).

## Files
- sources.md: SSOT evidence of coverage (official reachable pages)
- rest-api.md: REST endpoints/catalog rows (public/private; surface-separated)
- websocket.md: WebSocket catalog (public/private; surface-separated)
- fix.md: FIX catalog (if officially documented; else not_applicable w/ evidence)
- data.md: Official data distribution catalog (if documented)
- diffs.md: Official changelog diffs (official only)
- catalog.json: Machine-readable SSOT (1:1 with markdown tables)
- templates.md: Update templates
- CHANGELOG.md: Append-only change history

## Update Process (Must follow)
1. Fill `sources.md` first (official navigation reachable pages).
2. Use `sources.md` to fill `rest-api.md`, `websocket.md`, `fix.md`, and `data.md`.
3. Fill `diffs.md` from official changelog/upcoming-changes pages (official only).
4. Generate `catalog.json` matching markdown rows 1:1 (IDs unique).
5. Update `templates.md`.
6. Append to `CHANGELOG.md`.

## Verification
```bash
python -c "import json; json.load(open('docs/exchanges/coinbase/catalog.json')); print('OK JSON parse')"
```

## Notes
- Official docs only. No external blogs/SDK README inference.
- All table rows must include `source_url`.
