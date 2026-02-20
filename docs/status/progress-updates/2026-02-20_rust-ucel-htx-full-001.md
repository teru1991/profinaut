# RUST-UCEL-HTX-FULL-001 Progress Update

## Scope
- Task ID: `RUST-UCEL-HTX-FULL-001`
- Goal: HTX full-coverage rail (catalog loader validation, op-name SSOT, contract index wiring, coverage manifest/gate scaffold, perf-base policy alignment).

## Catalog facts (SSOT source)
- Source of truth: `docs/exchanges/htx/catalog.json`
- REST rows: **13**
- WS rows: **9**
- Total tracked rows: **22**

## Implemented mapping / decision rules
- **OpName SSOT rule (single location)**: `ucel-registry::map_operation` now centrally resolves op names from catalog metadata (`operation`) and fallback id heuristics; HTX rows are included via this same centralized rule.
- **requires_auth rule (mechanical)**: `requires_auth` is derived from `visibility == private` only (`op_meta_from_entry`), with contradiction fail-fast when catalog-provided `requires_auth` conflicts.

## Coverage gate design (HTX)
- Added `ucel/coverage/htx.yaml` with all 22 IDs (REST+WS), each carrying:
  - `implemented`
  - `tested`
- Gate mode for this task: `strict: false` (warn-only)
- `ucel-testkit` test asserts warn behavior and detects uncovered rows (22 gaps).

## Contract index coverage wiring
- `CatalogContractIndex` path verified for HTX: every catalog id can be registered and checked as zero-missing.

## Perf base alignment notes
- Loader/test utilities use typed serde structures (no `serde_json::Value` in added code paths).
- This task establishes SSOT/gate/index rail; runtime transport-level bounded-channel/backpressure enforcement remains under transport/exchange implementation tasks.

## Next task declaration
- Next HTX task will switch from scaffold to **full implementation/test fill for all 22 rows**, then tighten gate to strict mode so unresolved rows fail CI deterministically.
