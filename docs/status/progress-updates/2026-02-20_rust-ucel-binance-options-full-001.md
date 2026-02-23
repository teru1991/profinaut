# RUST-UCEL-BINANCE-OPTIONS-FULL-001 progress

## Catalog evidence snapshot
- Source SSOT: `docs/exchanges/binance-options/catalog.json`
- Counted rows:
  - REST: 8
  - WS: 6
  - Total tracked IDs: 14

## Op mapping SSOT rule
- Mapping is centralized in `ucel/crates/ucel-registry/src/lib.rs` (`map_operation_by_id`).
- Every `binance-options` catalog row is mapped by `id` without inference from payload samples.
- `requires_auth` is mechanically derived from `visibility == private` only (`op_meta_from_entry`).

## Coverage gate design
- Added `ucel/coverage/binance-options.yaml` with all 14 catalog IDs.
- Each entry has `implemented/tested` booleans.
- `strict: false` for this task, so gate emits warn-only via `run_coverage_gate`.
- Warn-only explicitly detects unimplemented / untested rows and reports both lists.

## Perf baseline policy (for subsequent implementation tasks)
- Typed deserialize only (`serde_json::from_slice` into structs); no `serde_json::Value` as primary decode path.
- Request/response bodies flow through `bytes::Bytes` to avoid unnecessary copies.
- WS consumers must use bounded channels with backpressure (`tokio::sync::mpsc::channel(capacity)`).

## Next task declaration
- Next task will fill implementation/tests for all 14 IDs and flip the coverage gate to strict mode.
