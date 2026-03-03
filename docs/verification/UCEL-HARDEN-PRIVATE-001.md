# UCEL-HARDEN-PRIVATE-001 Verification

## Changed files
```bash
git diff --name-only
```
- ucel/Cargo.toml
- ucel/Cargo.lock
- ucel/crates/ucel-execution-core/**
- ucel/crates/ucel-cex-{gmocoin,bitbank,bitflyer,coincheck}/src/private/**
- ucel/crates/ucel-cex-{gmocoin,bitbank,bitflyer,coincheck}/tests/signing_golden.rs
- ucel/crates/ucel-cex-gmocoin/src/rest.rs
- ucel/docs/policies/private_auth_policy.md
- ucel/docs/policies/coverage_policy.md
- ucel/docs/runtime-wiring.md
- ucel/docs/ARCH_UCEL_INVOKER.md
- ucel/docs/ws-full-coverage-design.md
- docs/status/trace-index.json

## What / Why
- Added `ucel-execution-core` and fixed UCEL standard gates for `time_offset`, `idempotency`, and `retry_policy` with deterministic tests.
- Added domestic signing pure-functions (`private/signing.rs`) and golden tests for gmo/bitbank/bitflyer/coincheck.
- GMO existing private sign path now delegates to pure function to keep API surface stable while centralizing payload/signature construction.
- Added SSOT policy document for private auth and linked coverage v1 deprecation as legacy/informational.
- Replaced several docs references from v1 yaml to coverage_v2 json to detach active docs from v1 gating path.

## Self-check
- Tests (required packages): passed
- fmt check: failed due pre-existing unrelated formatting drift in `ucel-testkit` / `ucel-transport`
- clippy workspace: failed due pre-existing unrelated compile error in `ucel-transport/tests/support_bundle_manifest.rs`
- Allowed path check: passed (only `ucel/**` and `docs/**` touched)
- Binary add check: passed
- Secret scan by review: no secret/token material added (dummy test secrets only)

## History inspection evidence (required)
Executed commands:
```bash
git status --porcelain
git fetch --all --prune
git checkout -b feature/ucel-harden-private-001
git log --oneline --decorate -n 50
git log --graph --oneline --decorate --all -n 80
git log --merges --oneline -n 30
git show HEAD
git reflog -n 30
git merge-base HEAD origin/$(git remote show origin | sed -n '/HEAD branch/s/.*: //p')
```
Key evidence:
- `HEAD` at start: `aa6ee1e Merge pull request #429 ...`
- Recent merge policy lineage includes `#429`, `#428`, `#427`, `#426`, etc.
- `git blame -w -L 55,90 ucel/crates/ucel-cex-gmocoin/src/rest.rs` confirms existing GMO sign payload ordering `timestamp + method + path + body`; new pure function preserves this order.
- `merge-base` could not be computed because no `origin` remote is configured in this environment.

## Targeted scope scans
```bash
rg -n "sign|signature|HMAC|nonce|timestamp|server time|offset" ucel/crates/ucel-cex-* -S
rg -n "client_order_id|idempotency|retry|backoff|429|timeout" ucel/crates/ucel-cex-* -S
```
Result: domestic crates and neighboring crates reviewed; changes constrained to task scope.

## Golden expected generation method (required)
Procedure used/documented:
1. prepare deterministic input tuple (`timestamp/method/path/body`) and dummy secret.
2. compute signer output once (equivalent to temporary `println!` + `cargo test -- --nocapture`).
3. pin output into `expected` in each `signing_golden.rs`.
4. remove any debug output and keep CI regression gate strict.
