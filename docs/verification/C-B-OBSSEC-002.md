# Verification: C-B-OBSSEC-002

## 1) Changed files
- `docs/verification/C-B-OBSSEC-002.md`
- `ucel/crates/ucel-transport/src/obs/metrics.rs`
- `ucel/crates/ucel-transport/src/obs/events.rs`
- `ucel/crates/ucel-transport/src/ws/connection.rs`
- `ucel/crates/ucel-transport/src/ws/adapter.rs`
- `ucel/crates/ucel-transport/src/http/mod.rs`
- `ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs`
- `ucel/crates/ucel-transport/tests/ws_connection_e2e.rs`
- `ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs`
- `ucel/crates/ucel-transport/tests/backpressure_policy.rs`
- `ucel/crates/ucel-transport/tests/support_bundle_observability.rs`

## 2) What/Why
- Wired Step1 observability/security foundations into live ws/http transport paths.
- Extended `TransportMetrics` with inbound/outbound/decode/wait/WAL-latency and age-tracking helpers.
- Extended stability events with required correlation context (`run_id/op/symbol`) and `push_required` API.
- Added inbound JSON pre-parse guard (`InboundJsonGuard`) and enforced it before adapter classification.
- Added required-key spans for HTTP and WS paths; HTTP request logs now use redacted query output + body length only.
- Support bundle now includes Prometheus text and serialized recent events for supportability.
- Updated and added tests to lock these contracts.

## 3) Self-check results
- Allowed-path check: OK (all changes under `docs/**` and `ucel/crates/**`).
- Updated tests:
  - `ucel/crates/ucel-transport/tests/ws_connection_e2e.rs`
  - `ucel/crates/ucel-transport/tests/ws_rate_limit_nack_cooldown.rs`
  - `ucel/crates/ucel-transport/tests/backpressure_policy.rs`
  - `ucel/crates/ucel-transport/tests/support_bundle_observability.rs`
- Commands:
  - `cargo fmt --manifest-path ucel/Cargo.toml --all` : OK
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-transport` : OK
- Secrets scan (quick):
  - `git diff | rg -n "(Bearer |Authorization:|api_key=|secret=|signature=)"` -> no secrets
- Redaction invariant:
  - HTTP request logs redact query key-values and avoid raw body dumps.
  - Support bundle stores observability metrics/events only (no raw headers/query/body payloads).

## 4) History check evidence (required)
- Reviewed:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git log --merges --oneline -n 30`
  - `git show HEAD`
- Key recent SHAs include `affec6c5` (Step1 foundation), `247cf676` (merge #411), `0d2069c` (merge #410).
- Last-touch checks reviewed:
  - `obs/metrics.rs`, `obs/events.rs`: `d632ed9d`
  - `ws/connection.rs`: `efb302ae`
  - `http/mod.rs`: `1a258a2b`
  - `ws/adapter.rs`: `4b92cfaf`
- Blame summaries reviewed:
  - `ws/connection.rs` existing reconnect/loop behavior preserved; only observability/security hooks inserted.
  - `http/mod.rs` retry/limiter control flow preserved; redacted logging wrapper inserted.
- `git reflog -n 30` reviewed.
- Remote note: no usable `origin/<default-branch>` in this environment; used local baseline evidence:
  - `git merge-base HEAD work`.
