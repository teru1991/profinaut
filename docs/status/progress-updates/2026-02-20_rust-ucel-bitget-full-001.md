# RUST-UCEL-BITGET-FULL-001 progress update

- Task ID: `RUST-UCEL-BITGET-FULL-001`
- Scope: `rust-ucel-bitget-ssot-contracts-coverage-gate-perf-base`

## Catalog counts (SSOT source)
- Source of truth: `docs/exchanges/bitget/catalog.json`
- REST rows: **1**
- WS rows: **1**
- Total tracked rows: **2**

## Op mapping SSOT rule
- Operation mapping is fixed to `ucel-registry::map_operation`.
- `requires_auth` is mechanically derived from `visibility == "private"` in `op_meta_from_entry`.
- No inferred auth behavior is allowed beyond catalog visibility.

## Coverage gate design (bitget)
- Added `ucel/coverage/bitget.yaml` with all current catalog ids.
- Each entry includes `implemented` and `tested` flags.
- Gate mode is **warn-only** (`strict: false`) in this task.
- `ucel-testkit` now verifies bitget manifest gaps are detected as warnings.

## Contract index coverage behavior
- `CatalogContractIndex` can register catalog ids and detect missing tests against full catalog id set.
- Added bitget test coverage to ensure all catalog ids can be indexed.

## Perf base alignment
- This task keeps the existing typed catalog/manifest deserialization path (no `serde_json::Value` for core structures).
- No additional copies introduced in this task.
- WS bounded channel / backpressure policy remains part of transport-level implementation and is unchanged here.

## Next task declaration
- In the next task, all bitget catalog rows will be fully implemented and tested so the gate can be moved from warn-only to strict.
