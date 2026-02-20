# RUST-UCEL-BITFLYER-FULL-001 Progress Update

- Task: `rust-ucel-bitflyer-ssot-contracts-coverage-gate-perf-base`
- Scope lock: `LOCK:shared-docs`
- Date: 2026-02-20

## Catalog baseline (SSOT source)
Source catalog: `docs/exchanges/bitflyer/catalog.json`

- REST rows: **49**
- WS rows: **12**
- Total tracked rows: **61**

## OpName SSOT mapping rule
Bitflyer op naming is fixed to catalog-id driven mapping (single mapping entrypoint in registry), and `requires_auth` is mechanically derived from visibility semantics (`private => true`, otherwise false).

No inference outside catalog row metadata/id pattern is allowed.

## Contract index design
`CatalogContractIndex` is used to verify that **every** catalog id has a registered contract test id.
This task enables the same all-row coverage check for bitflyer.

## Coverage gate design (Task1 mode)
Added `ucel/coverage/bitflyer.yaml` as the manifest containing all 61 catalog ids.

- `strict: false` (warn-only in this task)
- every row has `implemented` / `tested` booleans
- current baseline intentionally leaves unimplemented rows as `false` to surface gaps in gate output

## Perf base policy alignment
Bitflyer track follows shared UCEL perf baseline policy:

- typed deserialization (no `serde_json::Value` SSOT contract path)
- bytes-oriented data path to avoid unnecessary copy
- WS path must use bounded channels with backpressure

## Next task declaration
Next task will fill implementation/tests per row and move coverage gate to strict mode so that any untracked/unimplemented row blocks CI.
