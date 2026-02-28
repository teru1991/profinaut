# UCEL-SUBSTORE-RL-COOLDOWN-BREAKER-001 Verification

## 1) Changed files
```text
docs/status/trace-index.json
ucel/crates/ucel-subscription-store/src/lib.rs
ucel/crates/ucel-subscription-store/tests/rate_limit_cooldown.rs
ucel/crates/ucel-transport/src/ws/connection.rs
docs/verification/UCEL-SUBSTORE-RL-COOLDOWN-BREAKER-001.md
```

## 2) What / Why
- Added backward-compatible `rate_limit_until` support in subscription-store schema (create-time column + safe ALTER migration).
- Updated `next_pending_batch` to skip pending rows while cooldown is active (`rate_limit_until > now`).
- Added store APIs for cooldown application and attempts lookup to support local RL breaker behavior.
- Updated WS Nack(rate-limit) handling to enforce attempts ceiling, apply pending+cooldown backoff when under threshold, and deadletter when threshold exceeded.
- Non-rate-limit Nack behavior remains deadletter, and RL penalty application remains shared-limiter scoped.

## 3) Self-check results
- `cargo test -p ucel-subscription-store` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo test -p ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo fmt --package ucel-subscription-store --package ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `python -m json.tool docs/status/trace-index.json > /dev/null`: **PASS**
- `git diff --name-only | awk '!/^(ucel\/crates\/ucel-(subscription-store|transport)\/|docs\/)/ {print}'`: **PASS** (no output)
- Link existence check (`docs/` refs in touched docs): **none**
