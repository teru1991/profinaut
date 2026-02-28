# UCEL-TRANSPORT-STABILITY-EVENTS-METRICS-001 Verification

## 1) Changed files
```text
docs/status/trace-index.json
ucel/crates/ucel-transport/src/lib.rs
ucel/crates/ucel-transport/src/stability/events.rs
ucel/crates/ucel-transport/src/stability/metrics.rs
ucel/crates/ucel-transport/src/stability/mod.rs
ucel/crates/ucel-transport/src/ws/connection.rs
docs/verification/UCEL-TRANSPORT-STABILITY-EVENTS-METRICS-001.md
```

## 2) What / Why
- Added `stability` SSOT modules with a fixed `TransportStabilityEvent` enum and structured in-memory metrics (counters/gauges).
- Introduced `StabilityHub::emit()` as a single entrypoint that emits structured logs (`event_kind`, reason/state/outcome/priority/attempt) and updates metrics.
- Wired WS connection flow to emit events for reconnect attempts, breaker wait state, stale requeue, overflow outcomes, RL penalty/cooldown, connection state, and shutdown phases.
- Added gauge updates for `active_conn`, `outq_len`, and `wal_queue_len` at reachable update points.
- Exported `pub mod stability` and updated trace-index task entry.

## 3) Self-check results
- `cargo test -p ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo fmt --package ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `python -m json.tool docs/status/trace-index.json > /dev/null`: **PASS**
- `git diff --name-only | awk '!/^(ucel\/crates\/ucel-transport\/|docs\/)/ {print}'`: **PASS** (no output)
- Link existence check (`docs/` refs in touched docs): **none**
