# UCEL-TRANSPORT-STABILITY-002 Verification

## 1) Changed files
- docs/runbooks/transport/ws_health_support_bundle.md
- docs/status/trace-index.json
- services/marketdata-rs/Cargo.lock
- services/marketdata-rs/ucel-ws-subscriber/Cargo.toml
- services/marketdata-rs/ucel-ws-subscriber/src/adapters.rs
- services/marketdata-rs/ucel-ws-subscriber/src/config.rs
- services/marketdata-rs/ucel-ws-subscriber/src/http.rs
- services/marketdata-rs/ucel-ws-subscriber/src/lib.rs
- services/marketdata-rs/ucel-ws-subscriber/src/main.rs
- services/marketdata-rs/ucel-ws-subscriber/src/state.rs
- services/marketdata-rs/ucel-ws-subscriber/src/supervisor.rs
- services/marketdata-rs/ucel-ws-subscriber/tests/http_endpoints.rs
- ucel/crates/ucel-cex-bitget/Cargo.toml
- ucel/crates/ucel-cex-bitget/src/symbols.rs
- ucel/crates/ucel-cex-bitget/src/ws.rs
- ucel/crates/ucel-cex-okx/Cargo.toml

## 2) What / Why
Added operational HTTP endpoints to `ucel-ws-subscriber` so transport health and support bundles are exposed externally (`/healthz`, `/support_bundle`). Added shared runtime state (`AppState`) containing transport metrics/event-ring/health/rules snapshot and wired supervisor updates to produce periodic health snapshots. Added endpoint integration test to assert JSON contract and bundle version. Added operational runbook for health and support-bundle usage. Updated dependency wiring and minimal compile fixes in adapter crates required to build ws-subscriber tests in this workspace.

## 3) Self-check
- Allowed-path check: changed paths are under `docs/**`, `services/**`, and `ucel/crates/**` only.
- Tests added/updated:
  - `services/marketdata-rs/ucel-ws-subscriber/tests/http_endpoints.rs` (new)
- Build/Unit tests:
  - `cd services/marketdata-rs && cargo test -p ucel-ws-subscriber` => PASS
  - `cd ucel && cargo test -p ucel-transport -p ucel-subscription-store` => PASS
- trace-index json check:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan:
  - `rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" docs/runbooks/transport/ws_health_support_bundle.md services/marketdata-rs/ucel-ws-subscriber/src` => no secrets added.
