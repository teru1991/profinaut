# UCEL Crate Graph + Catalog Type Placement (UCEL-HUB-ROUTER-001A)

## Workspace crates (name / path / role)
- `ucel-core` (`ucel/crates/ucel-core`): canonical shared models/errors/ops.
- `ucel-transport` (`ucel/crates/ucel-transport`): transport abstractions, retry policy helpers, ws buffering.
- `ucel-registry` (`ucel/crates/ucel-registry`): catalog loading/validation and now Hub entry/registry resolver.
- `ucel-testkit` (`ucel/crates/ucel-testkit`): shared testing harness.
- `ucel-cex-*` crates (`ucel/crates/ucel-cex-*`): exchange adapters and venue-specific transformations.
- `ucel-chain-ethereum` (`ucel/crates/ucel-chain-ethereum`): chain adapter.
- `ucel-ir` (`ucel/crates/ucel-ir`): IR ingestion domain.

## Dependency direction graph (high-level)
- `ucel-core` is foundational.
- `ucel-transport -> ucel-core`.
- `ucel-registry -> ucel-core`, and for Hub retry reuse `ucel-registry -> ucel-transport`.
- Most `ucel-cex-* -> ucel-core + ucel-transport` (some also `-> ucel-testkit` and `ucel-cex-bittrade -> ucel-registry`).
- `ucel-testkit -> ucel-core + ucel-transport + ucel-registry`.
- `ucel-ir` currently independent of the CEX stack.

## Catalog specs location (data)
- SSOT catalog data is in `docs/exchanges/*/catalog.json`.
- `ucel-registry` already loads/validates these catalogs via `ExchangeCatalog` and `CatalogEntry`.

## Catalog spec TYPES location
- Canonical reusable catalog row type is `ucel_registry::CatalogEntry`.
- For Hub clarity, `EndpointSpec` and `WsChannelSpec` are aliases to `CatalogEntry` (single canonical type source in `ucel-registry`).
- Existing exchange crates contain many duplicated local `EndpointSpec/WsChannelSpec` definitions; these remain unchanged to minimize churn.

## Risk notes
- Existing duplicated per-exchange spec struct definitions increase long-term drift risk.
- `ucel-cex-bittrade` depends on `ucel-registry`, so new dependencies in registry must avoid introducing reverse edges back into cex crates.
- Hub construction must avoid a giant hard-coded match to stay maintainable.

## Decision A — Hub placement
- **Place Hub in `ucel-registry` as `ucel_registry::hub`**.
- Rationale: registry already owns catalog parsing/validation (SSOT bridge), avoids introducing a new top-level facade crate, and keeps dependency direction acyclic.

## Decision B — Registry construction strategy
- **Use provider-list construction backed by SSOT catalog JSON includes** (compile-time `include_str!` list).
- Rationale: minimal churn now, keeps catalog source-of-truth in `docs/exchanges/*/catalog.json`, avoids fragile codegen/build.rs in this iteration, and no hand-maintained giant key match table.
