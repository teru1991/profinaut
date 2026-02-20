# RUST-UCEL-BINANCE-USDM-FULL-001 Progress Update

- Task: binance-usdm Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-binance-usdm-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/binance-usdm/catalog.json`)

- REST rows: 6
- WS rows: 10
- Contract target rows (REST+WS): 16

## Mapping rule fixed (single source of truth)

1. `ucel-registry::map_operation` now routes all `usdm.*` ids through one centralized mapper (`map_binance_usdm_operation`) as the op-name SSOT.
2. The mapper table is complete for every current Binance USDM catalog id, so missing rows fail with `NOT_SUPPORTED`.
3. `requires_auth` is mechanically derived from visibility only (`visibility == private` => `true`, otherwise `false`) with no heuristic override.

## Coverage gate design (Binance USDM track)

- Manifest path: `ucel/coverage/binance-usdm.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 16 Binance USDM catalog ids are listed and tracked.
- Gate behavior: unresolved `implemented/tested` gaps are detected and reported as warning; strict mode is deferred.

## perf base note

- Typed deserialize baseline remains mandatory (typed serde structs, no `serde_json::Value` fallback path in this rail).
- Transport policy remains bytes-first to avoid unnecessary copies.
- WS policy remains bounded-channel/backpressure-first.

## Next task declaration

- In the next Binance USDM full-coverage task, all 16 catalog rows will be filled to implemented+tested and the gate will be tightened to strict.
