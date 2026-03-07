# UCEL-DOMESTIC-PUBLIC-WS-009C Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/domestic_public_ws_runtime_v1.md
- docs/specs/ucel/domestic_public_ws_surface_v1.md
- docs/status/trace-index.json
- docs/verification/UCEL-DOMESTIC-PUBLIC-WS-009C.md
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-core/src/public_ws.rs
- ucel/crates/ucel-registry/src/hub/ws.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-sdk/src/public_ws.rs
- ucel/crates/ucel-testkit/src/domestic_public_ws.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ws_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ws_docs_drift.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ws_integrity.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ws_runtime.rs
- ucel/crates/ucel-transport/src/ws/integrity.rs
- ucel/crates/ucel-transport/src/ws/mod.rs
- ucel/crates/ucel-transport/src/ws/public_runtime.rs
- ucel/docs/exchanges/domestic_public_ws_integrity_policy.md
- ucel/docs/exchanges/domestic_public_ws_matrix.md
- ucel/docs/exchanges/domestic_public_ws_runtime_matrix.md
- ucel/examples/domestic_public_ws_preview.rs
- ucel/fixtures/domestic_public_ws/cases.json

## 2) What / Why
- Added canonical domestic public WS surface/types in `ucel-core` and exported them so SDK/runtime can use one typed model.
- Added runtime readiness and integrity primitives in `ucel-transport/ws` to cover ack/implicit-ready/immediate-active style transitions and integrity failure mapping.
- Added inventory-backed registry helper for pending vendor public WS extensions (009E handoff visibility).
- Added SDK facade (`DomesticPublicWsFacade`) and preview output so domestic WS support/pending channels are visible.
- Added spec docs, exchange WS matrices, and integrity policy docs to prevent docs drift.
- Added testkit helpers + 4 WS gate tests (contract/runtime/integrity/docs drift) with fixtures.

## 3) Self-check results
- Allowed-path check: OK for this task scope (note: pre-existing unrelated dirty file exists at `services/marketdata-rs/Cargo.lock` before this task and was not included in commit).
- Tests added/updated:
  - `domestic_public_ws_contract_matrix`
  - `domestic_public_ws_runtime`
  - `domestic_public_ws_integrity`
  - `domestic_public_ws_docs_drift`
- Build / unit test command results:
  - `cd ucel && cargo test -p ucel-core -p ucel-transport -p ucel-registry -p ucel-sdk -p ucel-ws-rules -p ucel-subscription-planner -p ucel-subscription-store -p ucel-journal --lib` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ws_contract_matrix --test domestic_public_ws_runtime --test domestic_public_ws_integrity --test domestic_public_ws_docs_drift` ✅
- Trace-index JSON tool: `python -m json.tool docs/status/trace-index.json > /dev/null` ✅
- Secrets scan (light): no suspicious task-added secret strings detected in changed files (`rg -n "AKIA|BEGIN PRIVATE KEY|SECRET|TOKEN"` over changed paths).
- docs link existence check: task-added docs do not include broken `docs/` relative links.

## 4) 履歴確認の証拠
### Git history / merge evidence
- `git log --oneline --decorate -n 50` reviewed.
- `git log --graph --oneline --decorate --all -n 80` reviewed.
- latest commit context reviewed: `0c4c6dd Add JP domestic public API inventory, vendor public REST extension surface, docs and CI gates`.
- `git reflog -n 30` reviewed (branch movement + latest baseline commit confirmed).
- `git log --merges --oneline -n 30` reviewed (recent PR merge chain around #464/#463/#462 confirmed).
- `git merge-base HEAD origin/master` failed in this environment (`origin/master` missing), recorded as environment limitation.

### Blame / hotspot evidence
- `git blame -w ucel/coverage_v2/domestic_public/jp_public_inventory.json` reviewed to confirm inventory baseline provenance.
- `git blame -w ucel/crates/ucel-transport/src/ws/public_runtime.rs` reviewed to localize runtime edits.
- `git blame -w ucel/crates/ucel-registry/src/hub/ws.rs` reviewed to localize registry routing/helper edits.
- `git blame -w ucel/crates/ucel-ws-rules/src/public_rules.rs` reviewed to confirm existing ack/integrity rule posture.

### 現状棚卸しと分類根拠
- Inventory (`api_kind=ws`) contains 29 domestic WS entries: 19 `canonical_core`, 10 `vendor_public_extension`, all currently `implemented`.
- This task implemented canonical WS surface/runtime primitives and explicitly surfaced vendor extension WS channels through registry helper + runtime/docs matrices as `pending_009e` (not silently dropped, not reclassified to `not_supported`).
- Canonical core/extended type definitions were introduced in `ucel-core` and wired to SDK/runtime; extended channels are typed and available for future inventory rows while current inventory has no `canonical_extended` WS rows.
- Runtime requirements (ack/readiness/integrity/deadletter) are fixed in typed code paths plus policy docs/tests to keep drift detectable.

### 既存実装再利用・抽出
- Reused existing `PublicWsAckMode`, `PublicWsIntegrityMode`, and `PublicWsReasonCode` from `market_data` for backward compatibility.
- Reused existing `PublicWsSession` runtime skeleton, adding localized domestic-ready transitions and deadletter mapping in the same module (minimal conflict scope).
- Reused inventory include pattern used by REST extension helper to add WS pending helper in registry.

### 追加実装が必要だった不足点
- Gap identified: no explicit domestic WS canonical surface module, no pending vendor WS visibility helper, and no WS-specific docs drift gates for 009C outputs.
- Added concrete implementations (core/runtime/registry/sdk/docs/tests/fixtures) in this task to close those gaps.
