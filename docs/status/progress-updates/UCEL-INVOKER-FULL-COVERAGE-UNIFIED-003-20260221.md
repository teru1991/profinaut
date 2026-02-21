# UCEL-INVOKER-FULL-COVERAGE-UNIFIED-003 progress (2026-02-21)

- Preflight passed: status/handoff consistency and no open PR lock conflicts.
- Ownership acquired with locks `LOCK: ucel/**` and `LOCK: shared-docs`.

## Implementation
- Added `ucel_registry::invoker` module with unified VenueId/OperationId/MarketSymbol, SymbolCodec, binder, REST/WS executor and list APIs.
- Added strict coverage registry auto-discovery from `ucel/coverage/*.yaml` and synthetic fallback spec for coverage-only IDs.
- Added fixtures/docs/examples for invoker usage and architecture decision.

## Verification
- `cd ucel && cargo fmt --check` => FAIL (pre-existing workspace formatting diffs in unrelated crates).
- `cd ucel && cargo clippy --all-targets --all-features -- -D warnings` => FAIL (pre-existing unresolved imports/warnings in other crates).
- `cd ucel && cargo test --all-features` => FAIL (pre-existing unresolved test imports in other crates).
- `cd ucel && cargo test -p ucel-registry --all-features` => PASS.
- `cd ucel && cargo run --example invoker_list` => PASS.
