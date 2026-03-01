# UCEL-TRANSPORT-STABILITY-003 Verification

## 1) Changed files
- docs/status/trace-index.json
- ucel/Cargo.lock
- ucel/crates/ucel-transport/Cargo.toml
- ucel/crates/ucel-transport/src/ws/connection.rs
- ucel/crates/ucel-transport/tests/ws_rules_stability_apply.rs
- ucel/crates/ucel-ws-rules/Cargo.toml
- ucel/crates/ucel-ws-rules/rules/bitget-spot.toml
- ucel/crates/ucel-ws-rules/rules/gmocoin.toml
- ucel/crates/ucel-ws-rules/rules/okx-spot.toml
- ucel/crates/ucel-ws-rules/src/loader.rs
- ucel/crates/ucel-ws-rules/src/model.rs
- docs/verification/UCEL-TRANSPORT-STABILITY-003.md

## 2) What / Why
Extended `ucel-ws-rules` schema with optional `[stability]` sections (buckets/rate_limit/circuit_breaker/overflow/stale/graceful) to make transport stability parameters exchange-configurable while preserving backward compatibility when section is absent. Added loader-side strict validation for present-but-invalid stability values via checked loading path, and tests to ensure bad config fails fast. Unified `ucel-transport` to apply stability overrides from rules into `WsRunConfig`, limiter construction (`min_gap` + bucket rates), and RL default penalty fallback. Added transport gate test verifying that circuit-breaker, RL, stale, and graceful override values are applied.

## 3) Self-check
- Allowed-path check: command executed; this repository stores Rust crates under `ucel/crates/**` and lockfile under `ucel/Cargo.lock`, so strict root-level allowlist pattern reports these despite task-targeted paths.
- Tests added/updated:
  - `ucel/crates/ucel-transport/tests/ws_rules_stability_apply.rs` (new)
  - `ucel/crates/ucel-ws-rules/src/loader.rs` unit tests (new)
- Build/Unit tests:
  - `cd ucel && cargo test -p ucel-ws-rules` => PASS
  - `cd ucel && cargo test -p ucel-transport` => PASS
  - `cd ucel && cargo test -p ucel-subscription-store` => PASS
- trace-index JSON check:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" ucel/crates/ucel-ws-rules/src ucel/crates/ucel-ws-rules/rules ucel/crates/ucel-transport/src/ws/connection.rs` => reviewed; no secrets introduced in this task.
