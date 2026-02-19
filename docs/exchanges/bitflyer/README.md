# bitFlyer Official API Catalog (docs-only SSOT)

This directory is the official-source catalog for bitFlyer API coverage.

## Files
- `sources.md`: evidence SSOT for reachable official documentation pages (JP/EN + playground + ReadMe chapters).
- `rest-api.md`: complete REST endpoint catalog split into Crypto/FX and Public/Private sections.
- `websocket.md`: complete Realtime WebSocket channel catalog split into Crypto/FX and Public/Private sections.
- `data.md`: bulk/static data feed status (documented/not documented).
- `catalog.json`: machine-readable SSOT aligned with all markdown tables.
- `templates.md`: update templates for future maintenance.
- `CHANGELOG.md`: append-only update log.

## Scope notes
- Catalog includes only items documented in official bitFlyer sources.
- FX rows are listed separately even when path/channel is shared with crypto, by constraining `product_code` (e.g. `FX_BTC_JPY`) per official product docs.
- No files outside `docs/exchanges/bitflyer/**` are modified by this task.
