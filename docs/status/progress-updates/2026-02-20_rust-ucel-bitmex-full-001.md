# RUST-UCEL-BITMEX-FULL-001 Progress Update

- Task: bitmex Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-bitmex-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/bitmex/catalog.json`)

- REST rows: 95
- WS rows: 30
- Contract target rows (REST+WS): 125

## Mapping rule fixed (single source of truth)

1. Primary mapping uses `catalog.operation` literal-to-`OpName` mapping table.
2. If `operation` is absent or non-canonical (BitMEX WS rows and non-standard REST action ids), mapping falls back to one fixed BitMEX `id`-pattern rule in registry (`public.ws.*.subscribe` / `private.ws.*.subscribe` and `*.rest.<resource>.<action>`).
3. `requires_auth` is decided only by `visibility` (`private=true`, `public=false`) with no heuristic override.

## Coverage gate design (BitMEX track)

- Manifest path: `ucel/coverage/bitmex.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 125 catalog ids are enumerated as gate targets with `implemented` / `tested` flags.
- Gate behavior: no gaps => pass, gaps+strict=true => fail, gaps+strict=false => warn.

## perf base note

- BitMEX follows UCEL baseline rails: typed deserialize (no `serde_json::Value` in adapters), `Bytes`-based transport handling to avoid unnecessary copies, and bounded WS channel/backpressure.

## Next task declaration

- Next task will fill `implemented/tested` per catalog row and move BitMEX gate to strict mode so unimplemented rows fail fast in CI.
