# ARCH_UCEL_INVOKER

## Audit summary
- Invoker placement: `ucel-registry::invoker` to reuse catalog loading/validation and avoid cyclic deps.
- Catalog SSOT: `docs/exchanges/<venue>/catalog.json` parsed via `ucel_registry::ExchangeCatalog`/`CatalogEntry`.
- Coverage SSOT: `ucel/coverage/*.yaml` auto-discovered at runtime/tests.

## Decision
- **Registry strategy:** Provider-less dynamic discovery from coverage + catalog files (build-time fixed tableなし).
- Rationale: coverage files already enumerate target venues/ids; dynamic scan automatically includes newly added coverage venues and enforces strict gate.
