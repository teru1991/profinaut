# rust-ucel-coincheck-full-001

## Catalog evidence
- SSOT source: `docs/exchanges/coincheck/catalog.json`
- REST rows: 25
- WS rows: 4
- Total tracked rows: 29

## Op mapping SSOT rule
- Mapping is centralized in `ucel-registry::map_operation` and `map_coincheck_operation_by_id`.
- Coincheck rows are mapped by `id` only (`coincheck.*` prefix), so operation names are mechanically decided from catalog IDs.
- `requires_auth` is mechanically derived from visibility/private scope (`visibility=private` or `.private.` in id) and never inferred from ad-hoc heuristics.

## Coverage gate design (coincheck)
- Added `ucel/coverage/coincheck.yaml` enumerating all 29 catalog IDs.
- Each entry carries `implemented` and `tested` flags.
- Gate mode is `strict: false` for this task (warn-only), so missing implementation/tests are surfaced without blocking.

## Contract test index
- Enabled coincheck catalog coverage checks through `CatalogContractIndex` tests.
- Added checks for:
  - all rows missing when no tests are registered
  - zero missing rows when all catalog IDs are registered

## Next task declaration
- Next task will fill implementation and contract tests for all 29 rows and then flip the coincheck coverage gate to strict.
