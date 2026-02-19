# RUST-UCEL-GMO-FULL-001 Progress Update

- Task: GMO Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-gmocoin-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/gmocoin/catalog.json`)

- REST rows: 30
- WS rows: 12
- Contract target rows (REST+WS): 42

## Mapping rule fixed (single source of truth)

1. Primary mapping uses `catalog.operation` literal-to-`OpName` mapping table.
2. If `operation` is absent or not canonical (e.g. WS rows), mapping falls back to **explicit `id -> OpName` table** in one place.
3. `requires_auth` is decided only by `visibility` (`private=true`, `public=false`), no heuristic fallback.

## Coverage gate design (fixed in this task)

- Manifest path: `ucel/coverage/gmocoin.yaml`
- Manifest format: one row per catalog `id` with:
  - `implemented: bool`
  - `tested: bool`
- Gate executes from Rust tests (`cargo test`) and reports unresolved rows.
- Task 1 mode: `strict: false` (warn-only).
- Task 3 plan: switch to `strict: true`; fail if any row has `implemented=false` or `tested=false`.

## Next task declaration

- Next tasks will fill all 42 rows with concrete implementation and contract tests until strict gate can be turned on without exceptions.
