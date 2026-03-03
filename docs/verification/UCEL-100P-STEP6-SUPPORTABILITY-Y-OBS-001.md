# UCEL-100P-STEP6-SUPPORTABILITY-Y-OBS-001 Verification

## Changed files
- docs/specs/ucel/observability_spec_v1.md
- docs/specs/ucel/support_bundle_spec_v1.md
- ucel/crates/ucel-transport/src/obs/{mod.rs,logging.rs,metrics.rs,trace.rs}
- ucel/crates/ucel-transport/src/support_bundle.rs
- ucel/crates/ucel-transport/src/ws/connection.rs
- ucel/crates/ucel-transport/src/http/mod.rs
- ucel/crates/ucel-transport/src/lib.rs
- ucel/crates/ucel-registry/src/support_bundle.rs
- ucel/crates/ucel-registry/src/lib.rs
- ucel/crates/ucel-sdk/src/support_bundle.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-testkit/tests/support_bundle_redaction.rs
- ucel/crates/ucel-testkit/Cargo.toml
- docs/status/trace-index.json (task entry only)

## What / Why
- Standardized UCEL observability contract with required keys and fixed metric names.
- Enforced required fields at logging context creation and added context-aware logging helpers.
- Added connection/op trace span helpers.
- Added transport/registry/sdk support bundle generators with secret-safe structure.
- Added redaction regression test to ensure banned secret tokens are absent.

## Self-check results
- `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check`: **failed** due pre-existing formatting drift in unrelated files.
- `cargo clippy --manifest-path ucel/Cargo.toml --workspace --all-targets -- -D warnings`: **failed** due pre-existing clippy issue (`derivable_impls`) in `ucel-transport/src/ws/adapter.rs`.
- `cargo test --manifest-path ucel/Cargo.toml --workspace --all-targets`: **failed** due pre-existing missing imports in `ucel-cex-sbivc` tests.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-testkit --test support_bundle_redaction`: **passed**.

## History checks evidence (required)
Executed:
- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git show HEAD --stat --oneline`
- `git reflog -n 30`
- `git branch -vv`
- `git log --merges --oneline -n 30`
- `git merge-base HEAD origin/master` (not available in this environment: no `origin` remote configured)
- `git blame -w ucel/crates/ucel-transport/src/obs/logging.rs | head -n 20`
- `git blame -w ucel/crates/ucel-registry/src/hub/mod.rs | head -n 20`
- `git blame -w ucel/crates/ucel-sdk/src/lib.rs | head -n 20`

Findings summary:
- Step5 merge commit (`#420`) already established transport resilience counters and state transitions.
- Current changes extend that baseline into a normalized observability/supportability contract without conflicting prior intent.

## Spec alignment summary
- `observability_spec_v1`: required log/trace keys + metric names + span naming/fields documented.
- `support_bundle_spec_v1`: deterministic JSON shape, explicit redaction deny-list, and SDK single-call generation API documented.
