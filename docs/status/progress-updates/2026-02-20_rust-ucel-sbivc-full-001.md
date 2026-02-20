# RUST-UCEL-SBIVC-FULL-001 Progress Update

## Catalog ingestion counts (SSOT source: `docs/exchanges/sbivc/catalog.json`)
- REST rows: 0
- WS rows: 0
- Total contract-target rows (REST+WS): 0

## OpName SSOT mapping rule (single source)
1. `ucel-registry::map_operation` routes all `sbivc.*` ids through one function: `map_sbivc_operation`.
2. `requires_auth` is mechanically derived from `visibility=private` via `op_meta_from_entry` and catalog validation rejects conflicting explicit `requires_auth`.
3. Mapping fallback is deterministic (`operation` literal table first, then `id` pattern), with `NOT_SUPPORTED` reserved for future strict mapping when unknown ids are introduced.

## Coverage gate design (sbivc)
- Added `ucel/coverage/sbivc.yaml` with `strict: false` and `entries: []` (current catalog has no REST/WS rows).
- `ucel-testkit` gate remains warn-capable in non-strict mode and explicitly verifies warn behavior for unimplemented entries.
- Contract index check for sbivc is enabled so every catalog id must be test-registered once rows appear.

## Perf baseline compliance
- Typed deserialization is preserved (`serde` structs for catalog/manifest); no `serde_json::Value` in the sbivc SSOT rail.
- Existing transport layer already uses `bytes::Bytes` and bounded WS channels (`tokio::sync::mpsc`) for backpressure behavior.

## Next task declaration
- 次タスク（Task 2/3）で sbivc catalog に REST/WS 行が追加された時点で、全行を `implemented/tested=true` に段階的に埋め、取り逃がしゼロ運用へ移行する。
