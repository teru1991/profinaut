# UCEL-TRANSPORT-STABILITY-003 Verification

## 1) Changed files
- docs/status/trace-index.json
- docs/verification/UCEL-TRANSPORT-STABILITY-003.md
- ucel/Cargo.lock
- ucel/crates/ucel-transport/Cargo.toml
- ucel/crates/ucel-transport/src/ws/connection.rs
- ucel/crates/ucel-transport/tests/ws_connection_e2e.rs
- ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs
- ucel/crates/ucel-transport/tests/ws_rules_stability_apply.rs
- ucel/crates/ucel-ws-rules/Cargo.toml
- ucel/crates/ucel-ws-rules/rules/bitget-spot.toml
- ucel/crates/ucel-ws-rules/rules/bybit-spot.toml
- ucel/crates/ucel-ws-rules/rules/gmocoin.toml
- ucel/crates/ucel-ws-rules/src/loader.rs
- ucel/crates/ucel-ws-rules/src/model.rs

## 2) What / Why
Added backward-compatible `[stability]` schema to `ucel-ws-rules` so WS stability parameters (bucket, RL, breaker, overflow, stale, graceful) can be controlled by TOML as SSOT. Added loader validation in strict mode that rejects invalid stability values to fail fast on broken configs. Updated transport WS connection to apply rules-based stability overrides into `WsRunConfig`, and to prefer stability bucket/min-gap values for rate limiter construction plus configurable RL fallback penalty. Added gate test to verify rules-to-config override behavior and updated existing tests for new config field. Added sample stability blocks to representative exchange TOML files.

## 3) Self-check
- Allowed-path check: only `docs/**` and `ucel/crates/**` and `ucel/Cargo.lock` changed (repo places Rust workspace under `ucel/`).
- Tests added/updated:
  - Added `ucel/crates/ucel-transport/tests/ws_rules_stability_apply.rs`
  - Updated `ucel/crates/ucel-transport/tests/ws_connection_e2e.rs`
  - Updated `ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs`
- Commands run:
  - `cd ucel && cargo test -p ucel-ws-rules` => PASS
  - `cd ucel && cargo test -p ucel-transport` => PASS
  - `cd ucel && cargo test -p ucel-subscription-store` => PASS
- trace-index JSON check:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" docs/verification/UCEL-TRANSPORT-STABILITY-003.md ucel/crates/ucel-ws-rules/rules ucel/crates/ucel-ws-rules/src ucel/crates/ucel-transport/src/ws/connection.rs` => no secrets added.
