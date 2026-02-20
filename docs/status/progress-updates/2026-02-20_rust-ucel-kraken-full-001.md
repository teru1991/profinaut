# RUST-UCEL-KRAKEN-FULL-001 Progress Update

- Task: Kraken Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-kraken-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/kraken/catalog.json`)

- REST rows: 10
- WS rows: 10
- Contract target rows (REST+WS): 20

## Mapping rule fixed (single source of truth)

1. Primary mapping uses `catalog.operation` literal-to-`OpName` mapping table.
2. If `operation` is absent or non-canonical (mainly WS rows), mapping falls back to one explicit `id -> OpName` table.
3. `requires_auth` is decided only by `visibility` (`private=true`, `public=false`) with no heuristic override.

## Coverage gate design (Kraken track)

- Manifest path: `ucel/coverage/kraken.yaml`
- Manifest mode: `strict: false` (warn-only)
- Entries: all 20 catalog ids are enumerated as gate targets.
- Gate behavior is fixed as: no gaps => pass, gaps+strict=true => fail, gaps+strict=false => warn.

## perf base note

- Existing UCEL transport baseline keeps typed deserialize (`Bytes` + typed `serde` decode) and bounded WS channel backpressure as the default rail; Kraken follows this baseline as implementation starts.
