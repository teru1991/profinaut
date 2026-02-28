# UCEL-WS-NACK-RETRYAFTER-PENALTY-001 Verification

## 1) Changed files
```text
docs/status/trace-index.json
ucel/crates/ucel-transport/src/ws/adapter.rs
ucel/crates/ucel-transport/src/ws/connection.rs
docs/verification/UCEL-WS-NACK-RETRYAFTER-PENALTY-001.md
```

## 2) What / Why
- Added `retry_after_ms: Option<u64>` to `InboundClass::Nack` so adapters can pass retry hints without changing existing classification flows.
- Introduced `looks_like_rate_limited()` in WS connection to detect rate-limit style Nack reasons in a conservative, transport-local way.
- Changed inbound handling to receive shared `Arc<Mutex<WsRateLimiter>>`, then auto-apply limiter penalties from Nack (`retry_after_ms` preferred, otherwise 500ms fallback for rate-limit-like reasons).
- Penalty priority is derived from `op_id` (`classify_op_id_priority`) and logged with `warn!` for SSOT observability.
- Kept writer/drip limiter acquisition via shared lock-based limiter path to ensure applied penalties propagate to outbound permit allocation.

## 3) Self-check results
- `cargo test -p ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo fmt --package ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `python -m json.tool docs/status/trace-index.json > /dev/null`: **PASS**
- `git diff --name-only | awk '!/^(ucel\/crates\/ucel-transport\/src\/ws\/|docs\/)/ {print}'`: **PASS** (no output)
- Link existence check (`docs/` refs in touched docs): **none**
