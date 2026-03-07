# Domestic Public Compatibility Matrix

Final machine-checked summary consumed by `domestic_public_compat_*` tests.

summary.total_entries: 79
summary.rest_entries: 50
summary.ws_entries: 29
summary.canonical_core: 43
summary.canonical_extended: 10
summary.vendor_public_extension: 26
summary.not_supported: 0

## Coverage dimensions
| check | status | source |
| --- | --- | --- |
| inventory lock | pass | `ucel/coverage_v2/domestic_public/jp_public_inventory.lock.json` |
| route reachability | pass | registry hub + inventory stable identifiers |
| docs/matrices | pass | domestic public exchange docs |
| fixture/golden families | pass | `ucel/fixtures/domestic_public_goldens/*.json` |
| schema/runtime policy | pass | specs under `docs/specs/ucel` |
| workspace scope drift | pass | `ucel/crates/ucel-cex-*` domestic venues |
