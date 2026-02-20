# RUST-UCEL-BINANCE-COINM-FULL-001 Progress Update

- Task: binance-coinm Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-binance-coinm-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/binance-coinm/catalog.json`)

- REST rows: 7
- WS rows: 18
- Contract target rows (REST+WS): 25

## Mapping rule fixed (single source of truth)

1. OpName mapping is fixed in one table (`ucel-registry::map_operation_by_id`) and every binance-coinm catalog id is explicitly mapped.
2. `requires_auth` is mechanically derived from catalog visibility only (`private=true`, `public=false`).
3. For rows where `visibility` is omitted, visibility is mechanically resolved from catalog id segment (`.private.`/`.public.`) and rejected if unresolved.

## Coverage gate design (binance-coinm track)

- Manifest path: `ucel/coverage/binance-coinm.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 25 catalog ids are enumerated with `implemented`/`tested` fields.
- Gate behavior: gaps are detected and returned as warn-only until strict mode is enabled in Task 3.

## perf base note

- This task keeps the existing UCEL baseline policy unchanged: typed deserialize, bytes-first handling, and bounded WS/backpressure are the mandatory rail for exchange implementations.

## Next task declaration

- 次タスクで binance-coinm の catalog 全行について `implemented/tested` を埋め、coverage gate を strict に移行する。
