# UCEL SSOT Catalog Contract v1

## Contract intent
`docs/exchanges/<venue>/catalog.json` is the canonical source of what UCEL supports per venue.

## Required consistency
- Every `coverage.entries[].id` must exist in the venue catalog.
- Catalog may include more than currently implemented entries during staged rollout, but tracked coverage ids must never drift from catalog ids.

## Current structure (authoritative)
- Existing arrays (for example `rest_endpoints`, `ws_channels`, `fix_feeds`, `data_feeds`) remain authoritative.
- Any object field named `id` is considered a contract id for gate matching.

## Naming rules
- Keep dotted ids stable (e.g., `domain.visibility.transport.operation`).
- Renaming existing ids is a breaking change and is disallowed unless alias migration is added.

## Breaking-change handling
- Prefer additive ids.
- If rename is unavoidable, keep old id as alias during migration and update coverage + tests in the same PR.
