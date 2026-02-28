# UCEL-WS-RL-BUCKET-OBS-001 Verification

## 1) Changed files
```text
docs/status/trace-index.json
ucel/crates/ucel-transport/src/ws/connection.rs
ucel/crates/ucel-transport/src/ws/limiter.rs
docs/verification/UCEL-WS-RL-BUCKET-OBS-001.md
```

## 2) What / Why
- Reworked WS outbound limiter wiring in `connection.rs` to share one `Arc<Mutex<WsRateLimiter>>` between writer and drip paths.
- Updated bucket defaults to Control/Private/Public with conservative `private` and faster `control` throughput.
- Replaced dropped `outq.push` outcomes with explicit `PushOutcome` matching and warning logs for dropped/spilled overflow events.
- Added a minimal penalty hook by calling `apply_penalty()` when websocket read error text indicates rate pressure.
- Recorded task artifacts and verification evidence in trace-index SSOT entry.

## 3) Self-check results
- `cargo test -p ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `cargo fmt --package ucel-transport` (run from `/workspace/profinaut/ucel`): **PASS**
- `python -m json.tool docs/status/trace-index.json > /dev/null`: **PASS**
- `git diff --name-only | awk '!/^(ucel\/crates\/ucel-transport\/src\/ws\/|docs\/)/ {print}'`: **PASS** (no output)
- Link existence check (`docs/` refs in touched docs): **PASS**
  - `docs/verification/UCEL-WS-RL-BUCKET-OBS-001.md` has no `docs/` path references.
