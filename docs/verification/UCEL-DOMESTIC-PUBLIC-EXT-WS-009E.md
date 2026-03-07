# UCEL-DOMESTIC-PUBLIC-EXT-WS-009E Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/domestic_public_ext_ws_surface_v1.md
- docs/specs/ucel/domestic_public_ext_ws_schema_policy_v1.md
- docs/specs/ucel/domestic_public_ext_ws_runtime_policy_v1.md
- ucel/docs/exchanges/domestic_public_ws_extension_matrix.md
- ucel/docs/exchanges/domestic_public_ws_extension_schema_matrix.md
- ucel/docs/exchanges/domestic_public_ws_extension_runtime_matrix.md
- ucel/docs/exchanges/domestic_public_ws_extension_usage.md
- ucel/crates/ucel-core/src/public_ws_ext.rs
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-transport/src/ws/ext_runtime.rs
- ucel/crates/ucel-transport/src/ws/mod.rs
- ucel/crates/ucel-sdk/src/public_ws_ext.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-registry/src/hub/ws.rs
- ucel/crates/ucel-ws-rules/src/ext_public_rules.rs
- ucel/crates/ucel-ws-rules/src/lib.rs
- ucel/crates/ucel-testkit/src/domestic_public_ws_ext.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_ws_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_ws_schemas.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_ws_runtime.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_ws_docs_drift.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_ws_compat.rs
- ucel/examples/domestic_public_ext_ws_preview.rs
- ucel/fixtures/domestic_public_ext_ws/cases.json
- docs/status/trace-index.json
- docs/verification/UCEL-DOMESTIC-PUBLIC-EXT-WS-009E.md

## 2) What / Why
- Added a typed vendor public WS extension model (`public_ws_ext`) with mandatory schema/category/payload/runtime/metadata fields.
- Added extension runtime primitives (`ext_runtime`) to formalize readiness and resume semantics.
- Added registry + SDK extension routing surfaces (`list_vendor_public_ws_extension_operation_ids`, `subscribe_vendor_public_typed`, extension facade methods).
- Added extension WS spec/docs/matrices/usage docs and fixture-backed tests to gate inventory/schema/runtime/docs compatibility drift.
- Kept 009C canonical WS surface/runtime untouched and layered extension behavior in separate modules.

## 3) Self-check results
- Allowed-path check: OK for task-added files (note: pre-existing unrelated dirty `services/marketdata-rs/Cargo.lock` remained untouched).
- Tests added/updated:
  - domestic_public_ext_ws_contract_matrix
  - domestic_public_ext_ws_schemas
  - domestic_public_ext_ws_runtime
  - domestic_public_ext_ws_docs_drift
  - domestic_public_ext_ws_compat
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ext_ws_contract_matrix --test domestic_public_ext_ws_schemas --test domestic_public_ext_ws_runtime --test domestic_public_ext_ws_docs_drift --test domestic_public_ext_ws_compat` ✅
  - `cd ucel && cargo test -p ucel-core -p ucel-transport -p ucel-registry -p ucel-sdk -p ucel-ws-rules -p ucel-subscription-planner -p ucel-subscription-store -p ucel-journal --lib` ✅
- trace-index json.tool: `python -m json.tool docs/status/trace-index.json > /dev/null` ✅
- Secrets scan: `rg -n "AKIA|BEGIN PRIVATE KEY|SECRET|TOKEN"` over changed task files showed no secrets.
- docsリンク存在チェック: task-added docs include no broken `docs/` references.

## 4) ★履歴確認の証拠
- Reviewed:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show <latest_sha>`
  - `git reflog -n 30`
  - `git log --merges --oneline -n 30`
- `git merge-base HEAD origin/master` failed due missing `origin/master` in this environment (recorded limitation).
- Blame evidence reviewed:
  - `ucel/coverage_v2/domestic_public/jp_public_inventory.json`
  - `ucel/crates/ucel-core/src/public_ws.rs`
  - `ucel/crates/ucel-sdk/src/public_ws.rs`
  - `ucel/crates/ucel-transport/src/ws/public_runtime.rs`
  - `ucel/crates/ucel-transport/src/ws/integrity.rs`
  - `ucel/crates/ucel-registry/src/hub/ws.rs`
- Inventory-to-implementation evidence:
  - vendor public WS extension entries = 10 (bitbank 2, bitflyer 6, bittrade 2; coincheck/gmocoin/sbivc 0).
  - Added operation specs for all 10 entries and synchronized docs schema/runtime matrices + fixture cases + registry listing gate.
- Design basis:
  - category/schema/payload/runtime modes fixed in `VendorPublicWsOperationSpec`.
  - metadata completeness, payload type shape, and runtime mode constraints validated in core helpers.
  - runtime policy separated via `ext_runtime` and `ext_public_rules` for conflict-minimized extension layering.
- Additional implementation due gaps:
  - Added missing typed extension envelope builder and extension route methods to avoid raw passthrough fallback.
  - Added compatibility gates (version ordering + invalid mode combinations) to fail fast on schema/runtime contract breaks.
