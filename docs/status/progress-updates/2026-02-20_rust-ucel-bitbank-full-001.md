# RUST-UCEL-BITBANK-FULL-001 Progress Update

- Task: bitbank Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-bitbank-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/bitbank/catalog.json`)

- REST rows: 28
- WS rows: 16
- Contract target rows (REST+WS): 44

## Mapping rule fixed (single source of truth)

1. Primary mapping uses the single `catalog.operation` literal-to-`OpName` table in `ucel-registry`.
2. If `operation` is absent (mainly WS), mapping falls back to one explicit `id/prefix -> OpName` table in the same place.
3. `requires_auth` is mechanically fixed from `visibility` only (`private=true`, `public=false`) with no heuristic override.

## Coverage gate design (bitbank track)

- Manifest path: `ucel/coverage/bitbank.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 44 catalog ids are enumerated as gate targets with `implemented/tested` flags.
- Gate behavior: no gaps => pass, gaps+strict=true => fail, gaps+strict=false => warn.

## Next task declaration

- Task 2/3 will fill implementation/tests against all catalog rows and move the manifest toward full coverage.

## perf base note

- bitbank implementation rail follows UCEL baseline: typed deserialize (no `serde_json::Value`), `Bytes`-first payload handling to avoid extra copies, and bounded WS channel/backpressure as non-optional defaults.
