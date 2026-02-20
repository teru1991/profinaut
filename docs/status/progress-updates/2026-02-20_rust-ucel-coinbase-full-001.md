# RUST-UCEL-COINBASE-FULL-001

## Catalog coverage counts (SSOT source: `docs/exchanges/coinbase/catalog.json`)
- REST rows: 7
- WS rows: 8
- Total tracked rows for contract/gate: 15

## OpName SSOT mapping rule (single-source rule in `ucel-registry`)
- REST rows (`*.rest.*`) map to `OpName::FetchStatus` for current reference/discoverability scope.
- Private WS rows (`*.private.ws.*`) map to `OpName::SubscribeOrderEvents`.
- Public WS rows (`*.public.ws.*`) map to `OpName::SubscribeTicker`.
- `requires_auth` is mechanically derived from `visibility == private`.
- When `visibility` field is absent, visibility is deterministically resolved from ID segments (`.public.` / `.private.`), then used for auth decision.

## Coverage gate design (Task1: warn-only)
- Added `ucel/coverage/coinbase.yaml` with all 15 REST/WS IDs from the catalog.
- Each entry has `implemented` and `tested` fields.
- `strict: false` in this task so gaps are surfaced as warn-only (`CoverageGateResult::WarnOnly`).

## Perf baseline policy (carry-forward for Coinbase implementation)
- Use typed deserialization (no `serde_json::Value` for protocol payload paths).
- Use `bytes::Bytes` and avoid unnecessary copies in transport path.
- WS runtime must use bounded channels and explicit backpressure.

## Next task declaration
- Next task will fill implementation/tests across all 15 rows and switch coverage gate to strict mode with zero gaps.
