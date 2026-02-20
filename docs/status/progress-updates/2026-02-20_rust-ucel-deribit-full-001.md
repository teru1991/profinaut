# RUST-UCEL-DERIBIT-FULL-001 progress update

- Catalog source (SSOT): `docs/exchanges/deribit/catalog.json`
- Catalog counts:
  - REST (`rpc_http_methods`): 9
  - WS RPC (`rpc_ws_methods`): 10
  - WS subscriptions (`ws_subscriptions`): 9
  - Coverage target for this task (REST + WS): 28 rows

## Mapping rule (single source)

- Deribit op-name mapping is fixed in one place: `ucel/crates/ucel-registry/src/deribit.rs`.
- `requires_auth` is determined mechanically from visibility encoded in id:
  - `*.private.*` => `requires_auth = true`
  - `*.public.*` => `requires_auth = false`
- No per-endpoint guessing is used.

## Coverage gate design

- Added `ucel/coverage/deribit.yaml` and enumerated all 28 REST/WS ids from catalog.
- Gate mode is `strict: false` in this task (warn-only).
- Gaps (`implemented=false` / `tested=false`) are intentionally visible in this task.

## Next task declaration

- Next task will fill implementation and tests for all rows and switch deribit coverage gate to strict mode (`strict: true`) with zero gaps.
