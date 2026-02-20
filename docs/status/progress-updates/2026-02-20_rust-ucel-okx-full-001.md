# RUST-UCEL-OKX-FULL-001 Progress Update

## Scope
- Task ID: RUST-UCEL-OKX-FULL-001
- Title: okx Full-Coverage 基盤SSOT化

## Catalog evidence (SSOT source)
- Source catalog: `docs/exchanges/okx/catalog.json`
- REST rows: 4
- WS rows: 3
- Total tracked rows: 7

## Mapping rule (single source of truth)
- `requires_auth` is determined mechanically from `visibility` only:
  - `visibility=private` => `requires_auth=true`
  - `visibility=public` => `requires_auth=false`
- `op` mapping is centralized in one rule function (`map_okx_op_name`) based on id pattern:
  - `okx.*.ws.*` + private => `SubscribeExecutionEvents`
  - `okx.*.ws.*` + public => `SubscribeTicker`
  - `okx.*.rest.*` + private => `PlaceOrder`
  - public rest/default => `FetchStatus`

## Coverage gate design
- Added `ucel/coverage/okx.yaml` enumerating all 7 catalog ids.
- Manifest state for this task:
  - `strict: false` (warn-only)
  - every row starts as `implemented: false`, `tested: false`
- Expected behavior:
  - warn when gaps exist (no silent pass)
  - strict enablement planned in Task3 after implementation/tests are filled.

## Contract test index coverage
- Added OKX-specific catalog index check path so all catalog ids can be registered and validated.
- Intended invariant: every catalog row must have contract test registration coverage.

## Next task declaration
- 次タスクで、catalog 全行を実装・テストで埋め、coverage gate を strict に移行する。
