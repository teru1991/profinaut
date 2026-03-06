# UCEL-PRIVATE-WS-006 Verification

## 1) Changed files (`git diff --name-only`)
- docs/status/trace-index.json
- docs/specs/ucel/private_ws_surface_v1.md
- docs/verification/UCEL-PRIVATE-WS-006.md
- ucel/docs/policies/private_auth_policy.md
- ucel/docs/exchanges/private_ws_matrix.md
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-core/src/private_ws.rs
- ucel/crates/ucel-transport/src/ws/mod.rs
- ucel/crates/ucel-transport/src/ws/private_runtime.rs
- ucel/crates/ucel-transport/src/ws/session.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-sdk/src/private_ws.rs
- ucel/crates/ucel-registry/src/hub/errors.rs
- ucel/crates/ucel-registry/src/hub/ws.rs
- ucel/crates/ucel-registry/src/policy.rs
- ucel/crates/ucel-ws-rules/src/lib.rs
- ucel/crates/ucel-ws-rules/src/private_rules.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/src/private_ws.rs
- ucel/crates/ucel-testkit/tests/private_ws_auth_and_ack.rs
- ucel/crates/ucel-testkit/tests/private_ws_reauth_resume.rs
- ucel/crates/ucel-testkit/tests/private_ws_policy_gate.rs
- ucel/crates/ucel-testkit/tests/private_ws_event_normalization.rs
- ucel/crates/ucel-testkit/tests/private_ws_deadletter.rs

## 2) What / Why
- Added private WS SSOT spec and matrix to define canonical channels, lifecycle, ACK modes, and normalized reject classes.
- Added `ucel-core` canonical private WS model and transition/retry helpers for shared semantics.
- Added transport private WS session/runtime primitives and SDK facade surface.
- Added registry fail-fast private WS policy/auth gate path and channel mapping helper.
- Added ws-rules private rule view and private WS focused testkit/tests for ACK, reauth/resume, policy gate, normalization, and deadletter outcomes.
- Updated trace-index task entry for `UCEL-PRIVATE-WS-006` with branch/artifacts/evidence.

## 3) Self-check results
- Allowed-path check: **Needs care** (working tree has pre-existing out-of-scope `services/marketdata-rs/Cargo.lock`; excluded from commit).
- Tests added/updated OK:
  - `private_ws_auth_and_ack`
  - `private_ws_reauth_resume`
  - `private_ws_policy_gate`
  - `private_ws_event_normalization`
  - `private_ws_deadletter`
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-core -p ucel-transport -p ucel-registry -p ucel-sdk -p ucel-ws-rules -p ucel-testkit`:
    - PASS for new/updated private WS tests and most suites.
    - Known unrelated FAIL: `support_bundle_manifest_fixture_is_sane` (missing `/workspace/profinaut/fixtures/support_bundle/manifest.json`).
- `python -m json.tool docs/status/trace-index.json > /dev/null`: OK.
- Secrets scan: checked touched private WS docs/code for token/secret/signature leakage patterns; no raw secrets added.
- docsリンク存在チェック（今回触った docs 内の `docs/` 参照）: N/A (no new `docs/` inline path references in touched docs).

## 4) 履歴確認の証拠（必須）
### 4.1 git log / merges / merge-base
- Executed and archived to `/tmp/ucel_private_ws_hist.txt`:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git reflog -n 30`
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
- `origin/master` is not configured in this environment, therefore:
  - `git merge-base HEAD origin/master` could not be resolved.
  - conflict-check commands using `origin/master` were recorded as unavailable.

### 4.2 対象 venue の private WS 実装棚卸し
- Executed inventory queries and archived to `/tmp/ucel_private_ws_inventory.txt`:
  - private ws/auth/ack/session related grep over venue + transport + ws-rules.
  - registry/docs/coverage private visibility and requires_auth search.
  - planner/store/journal/transport reconnect/deadletter/inflight search.
- Result summary:
  - bitbank/bitflyer/coincheck/gmocoin/bittrade に private WS に関する実装断片あり。
  - sbivc/upbit は policy 上 private 有効化の確証不足または public_only 扱い。

### 4.3 policy allowlist / blocked 根拠
- `ucel/docs/policies/venue_access_policy.md` および `docs/specs/ucel/venue_access_policy_v1.md` を確認。
- JP resident baseline:
  - allow: bitbank/bitflyer/coincheck/gmocoin
  - blocked/public_only: sbivc
  - upbit は明示 private 許可なし（fail-closed）

### 4.4 login/auth/ack/reauth/resubscribe 設計根拠
- `private_ws_surface_v1.md` で lifecycle, ack_mode, auth patterns, retry/reconnect を先に定義。
- `ucel-core` と `ucel-transport` へ共通 state/reject/session primitives を実装。
- explicit ACK / implicit observation を `PrivateWsAckMode` と runtime hook で分離。

### 4.5 event normalization / deadletter / gap handling 根拠
- `CanonicalPrivateWsEvent` + reject/outcome classes を `ucel-core` に追加。
- deadletter/retryability を `PrivateWsRejectClass::as_outcome` で固定。
- testkit tests で unknown event non-panic、ack timeout/subscription reject/gap detected の outcome を回帰固定。

### 4.6 不足分を補う追加実装
- 既存に private WS 共通 canonical/runtime/surface が不足していたため、下記を具体実装:
  - core canonical model (`private_ws.rs`)
  - transport private runtime/session
  - sdk private ws facade
  - registry private ws fail-fast policy/auth gate
  - ws-rules private rule view
  - private ws integration-style tests群
