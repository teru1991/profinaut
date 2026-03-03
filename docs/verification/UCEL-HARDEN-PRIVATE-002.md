# UCEL-HARDEN-PRIVATE-002 Verification

## 1) Changed files
```bash
git diff --name-only
```
- services/marketdata-rs/symbol-master/src/app.rs
- services/marketdata-rs/symbol-master/src/config.rs
- services/marketdata-rs/symbol-master/src/lib.rs
- services/marketdata-rs/symbol-master/src/resync_loop.rs
- services/marketdata-rs/symbol-master/src/snapshot.rs
- services/marketdata-rs/symbol-master/src/store_bridge.rs
- services/marketdata-rs/symbol-master/tests/resync_e2e_mock.rs
- services/marketdata-rs/symbol-master/Cargo.toml
- services/marketdata-rs/Cargo.lock
- ucel/crates/ucel-cex-gmocoin/src/private/mod.rs
- ucel/crates/ucel-cex-gmocoin/src/private/request_builders.rs
- ucel/crates/ucel-cex-bitbank/src/private/mod.rs
- ucel/crates/ucel-cex-bitbank/src/private/request_builders.rs
- ucel/crates/ucel-cex-bitflyer/src/private/mod.rs
- ucel/crates/ucel-cex-bitflyer/src/private/request_builders.rs
- ucel/crates/ucel-cex-coincheck/src/private/mod.rs
- ucel/crates/ucel-cex-coincheck/src/private/request_builders.rs
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/src/http_mock.rs
- ucel/crates/ucel-testkit/tests/private_request_shape_domestic.rs
- ucel/docs/policies/private_auth_policy.md
- ucel/Cargo.lock
- docs/status/trace-index.json
- docs/verification/UCEL-HARDEN-PRIVATE-002.md

## 2) What / Why
- Implemented symbol-master resync execution chain: `hint -> snapshot fetch -> SymbolStore apply_snapshot -> checkpoint JSONL append`.
- Added `snapshot_url()` config extraction to avoid schema break and allow per-exchange URL injection via `params.snapshot_url`.
- Added dedicated `snapshot.rs` and `store_bridge.rs` to keep resync-loop diff minimal and local.
- Added integration proof (`resync_e2e_mock`) that a lagged hint triggers real snapshot fetch and checkpoint file output.
- Added domestic private request-shape builders (gmo/bitbank/bitflyer/coincheck) and testkit integration gate using wiremock helper.
- Policy was extended with explicit requirement for mock request-shape gates without real keys.

## 3) Self-check
- Allowed-path: pass (changes only in `services/**`, `ucel/**`, `docs/**`, lockfiles)
- Binary add: pass
- Secrets: pass (dummy values only)
- symbol-master tests: pass
- domestic exchange tests: pass
- ucel-testkit tests: partial fail due pre-existing SSOT gate failures unrelated to this task
- fmt check: fail due pre-existing unrelated formatting drift (`ucel-testkit/src/normalize.rs`, `ucel-transport/*`)
- clippy: fail due pre-existing unrelated compile issue in `ucel-transport/tests/support_bundle_manifest.rs`

## 4) History inspection evidence (required)
Commands executed:
```bash
git status --porcelain
git fetch --all --prune
git checkout -b feature/ucel-harden-private-002
git log --oneline --decorate -n 50
git log --graph --oneline --decorate --all -n 80
git log --merges --oneline -n 30
git show HEAD
git reflog -n 30
git merge-base HEAD origin/$(git remote show origin | sed -n '/HEAD branch/s/.*: //p')
git log --oneline -n 30 -- services/marketdata-rs/symbol-master/src/resync_loop.rs
git log --oneline -n 30 -- services/marketdata-rs/symbol-master/src/app.rs
git log --oneline -n 30 -- ucel/crates/ucel-symbol-store/src/lib.rs
git log --oneline -n 30 -- ucel/crates/ucel-symbol-adapter/src/resync.rs
git blame -w services/marketdata-rs/symbol-master/src/resync_loop.rs
rg -n "fetch_snapshot|SnapshotProvider|symbols|market meta" services/ ucel/ -S
```
Evidence notes:
- baseline commit before this task: `d1520db`.
- symbol-master resync files were introduced in `0ce9c9d` and intentionally kept as localized extension points.
- `origin` remote is not configured in this environment, so `merge-base` with `origin/*` is not computable.

## 5) No-placeholder proof
- `store_bridge::apply_snapshot_to_store` is implemented (not placeholder), parses canonical snapshot payload and applies to `SymbolStore`.
- `resync_e2e_mock` proves end-to-end path: after `ResyncHint::Lagged`, store gets instruments and checkpoint JSONL includes exchange id.
