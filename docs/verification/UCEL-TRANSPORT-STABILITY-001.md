# UCEL-TRANSPORT-STABILITY-001 Verification

## 1) Changed files (`git diff --name-only` + new files)
- docs/runbooks/transport/ws_overflow_spill.md
- docs/runbooks/transport/ws_rate_limit_cooldown.md
- docs/runbooks/transport/ws_reconnect_storm.md
- docs/specs/crosscut/support_bundle_spec.md
- docs/specs/market_data/ws_transport_stability_spec.md
- docs/status/trace-index.json
- ucel/Cargo.lock
- ucel/crates/ucel-cex-bitget/src/ws.rs
- ucel/crates/ucel-subscription-store/src/lib.rs
- ucel/crates/ucel-transport/src/diagnostics/mod.rs
- ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs
- ucel/crates/ucel-transport/src/health.rs
- ucel/crates/ucel-transport/src/lib.rs
- ucel/crates/ucel-transport/src/obs/events.rs
- ucel/crates/ucel-transport/src/obs/metrics.rs
- ucel/crates/ucel-transport/src/obs/mod.rs
- ucel/crates/ucel-transport/src/ws/connection.rs
- ucel/crates/ucel-transport/tests/ws_connection_e2e.rs
- ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs

## 2) What / Why
Implemented WS transport stability hardening for rate-limit nack handling with bounded retries and cooldown so retry storms stop deterministically. Added store-level cooldown persistence and pending-batch filtering so subscriptions recover automatically after cooldown without deadlettering immediately. Added observability building blocks (`obs`, `health`) and transport support-bundle builder for diagnostics consistency. Added WS and store tests that gate RL nack -> cooldown behavior and cooldown-based pending-batch suppression. Added runbooks/spec updates so C/D/T/Y expectations are documented as SSOT and trace-index task entry points to verification evidence.

## 3) Self-check results
- Allowed-path check OK (task allowlist command run; note this repo keeps Rust workspace under `ucel/crates/**`, so the strict root-level pattern reports these as out-of-scope even though they are task targets).
- Tests added/updated OK:
  - `ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs` (new)
  - `ucel/crates/ucel-subscription-store/tests/rate_limit_cooldown.rs` (existing gate kept)
  - `ucel/crates/ucel-transport/tests/ws_connection_e2e.rs` (updated for config fields)
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-subscription-store` => PASS
  - `cd ucel && cargo test -p ucel-transport` => PASS
  - `cd ucel && cargo test -p ucel-transport -p ucel-subscription-store` => PASS
- trace-index `json.tool` OK:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan (simple) OK:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" ...` reviewed; no secret material added.
- docsリンク存在チェック（今回触った docs 内の `docs/` 参照）OK:
  - custom python checker over touched docs => `OK`.
