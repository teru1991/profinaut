# Kraken Official Docs Catalog (SSOT)

This directory contains an **official-docs-only** catalog for Kraken:
- Spot REST / Spot WebSocket (v1 & v2)
- Futures REST / Futures WebSocket
- FIX API (officially documented)
- Change Log diffs (official only)

Completeness evidence is tracked in `sources.md`.

## Files
- `sources.md`: SSOT evidence of coverage
- `rest-api.md`: REST endpoints catalog (spot/futures; public/private)
- `websocket.md`: WebSocket catalog (v1/v2 + futures; public/private + common)
- `fix.md`: FIX catalog (official)
- `data.md`: Official data distribution catalog (if documented)
- `diffs.md`: Official changelog diffs (official only)
- `catalog.json`: Machine-readable SSOT (aligned with markdown tables)
- `templates.md`: Update templates
- `CHANGELOG.md`: Append-only change history

## Notes
- This catalog intentionally references only Kraken official docs (`docs.kraken.com`) and official Kraken support docs (`support.kraken.com`).
- If a surface is not explicitly present in official docs, it is marked as `not_applicable` with source URL evidence.
