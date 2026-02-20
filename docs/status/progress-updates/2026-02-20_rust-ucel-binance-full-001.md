# RUST-UCEL-BINANCE-FULL-001 Progress Update

- Task: binance Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-binance-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/binance/catalog.json`)

- REST rows: 9
- WS rows: 5
- Contract target rows (REST+WS): 14

## Mapping rule fixed (single source of truth)

1. Primary mapping uses one centralized rule in `ucel-registry` (`map_operation_literal` + `map_operation_by_id`) as the Binance op-name SSOT.
2. Every Binance catalog row now resolves to `OpName` from this SSOT mapping table (no ad-hoc fallback outside registry).
3. `requires_auth` is determined mechanically from `visibility` only (`private => true`, `public => false`), with no heuristic override.

## Coverage gate design (Binance track)

- Manifest path: `ucel/coverage/binance.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 14 Binance catalog ids are listed and tracked.
- Gate behavior: gaps are detected and reported as warning now; Task3 will switch Binance to strict mode.

## perf base note

- Typed deserialize baseline is preserved (`serde` typed structs; no `serde_json::Value` rail in this scope).
- Transport baseline remains bytes-first and avoids unnecessary copies.
- WS contract rail keeps bounded-channel/backpressure policy as mandatory design baseline.

## Next task declaration

- In the next Binance full-coverage task, all 14 catalog rows will be filled to implemented+tested and the coverage gate will be tightened to strict pass.
