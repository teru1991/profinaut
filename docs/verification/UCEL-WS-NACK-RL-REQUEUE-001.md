# UCEL-WS-NACK-RL-REQUEUE-001 Verification

## 1) Changed files
```text
docs/status/trace-index.json
ucel/crates/ucel-subscription-store/src/lib.rs
ucel/crates/ucel-transport/src/ws/connection.rs
docs/verification/UCEL-WS-NACK-RL-REQUEUE-001.md
```

## 2) What / Why
- Added `requeue_key_to_pending(key, now)` to `SubscriptionStore` so a single subscription can be moved back to `pending` without schema changes.
- Updated WS Nack handling to branch by rate-limit reason: RL Nack requeues to `pending` (recovery-first), non-RL Nack still goes to `deadletter`.
- Kept limiter penalty application integrated with shared `Arc<Mutex<WsRateLimiter>>` path so penalty reflects into writer/drip permit scheduling.
- Retained existing Nack observability with explicit `warn!` logs for both RL-requeue and non-RL-deadletter decisions.
- Updated trace-index with task artifacts and verification evidence.

## 3) Self-check results
- `cargo test -p ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo test -p ucel-subscription-store` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo fmt --package ucel-subscription-store --package ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `python -m json.tool docs/status/trace-index.json > /dev/null`: **PASS**
- `git diff --name-only | awk '!/^(ucel\/crates\/ucel-(subscription-store|transport)\/|docs\/)/ {print}'`: **PASS** (no output)
- Link existence check (`docs/` refs in touched docs): **none**
