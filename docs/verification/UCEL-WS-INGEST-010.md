# UCEL-WS-INGEST-010 Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/ws_ingest_runtime_v1.md
- docs/status/trace-index.json
- docs/verification/UCEL-WS-INGEST-010.md
- ucel/Cargo.lock
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-core/src/ws_ingest.rs
- ucel/crates/ucel-transport/src/ws/mod.rs
- ucel/crates/ucel-transport/src/ws/public_runtime.rs
- ucel/crates/ucel-transport/src/ws/private_runtime.rs
- ucel/crates/ucel-transport/src/ws/supervisor.rs
- ucel/crates/ucel-transport/src/ws/restart.rs
- ucel/crates/ucel-transport/src/ws/backoff.rs
- ucel/crates/ucel-ws-rules/src/lib.rs
- ucel/crates/ucel-ws-rules/src/runtime_policy.rs
- ucel/crates/ucel-subscription-planner/src/lib.rs
- ucel/crates/ucel-subscription-planner/src/plan.rs
- ucel/crates/ucel-subscription-planner/src/replan.rs
- ucel/crates/ucel-subscription-store/Cargo.toml
- ucel/crates/ucel-subscription-store/src/lib.rs
- ucel/crates/ucel-subscription-store/src/state.rs
- ucel/crates/ucel-subscription-store/src/persistence.rs
- ucel/crates/ucel-subscription-store/src/resume.rs
- ucel/crates/ucel-journal/Cargo.toml
- ucel/crates/ucel-journal/src/lib.rs
- ucel/crates/ucel-journal/src/events.rs
- ucel/crates/ucel-journal/src/writer.rs
- ucel/crates/ucel-journal/src/replay.rs
- ucel/crates/ucel-sdk/src/market_data.rs
- ucel/crates/ucel-sdk/src/private_ws.rs
- ucel/crates/ucel-registry/src/hub/ws.rs
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/src/ws_ingest.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_state_machine.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_resume_after_restart.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_deadletter_and_replay.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_heartbeat_and_stall.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_checksum_gap.rs
- ucel/crates/ucel-testkit/tests/ws_ingest_rate_limit_and_backoff.rs
- ucel/docs/marketdata/ws_ingest_policy.md
- ucel/docs/marketdata/ws_deadletter_policy.md
- ucel/docs/marketdata/ws_resume_policy.md
- ucel/docs/exchanges/ws_ingest_matrix.md
- ucel/examples/ws_ingest_preview.rs
- ucel/examples/ws_ingest_resume_preview.rs
- ucel/fixtures/ws_ingest/README.md

## 2) What / Why
- Added canonical WS ingest runtime spec/docs for lifecycle, journal-first, deadletter, and resume.
- Added core ingest state-machine domain model and directive helpers.
- Added planner/store/journal split modules to formalize desired plan, durable runtime state, and append-only evidence.
- Added transport supervisor/restart/backoff hooks to connect runtime failures to journal-first state transitions.
- Added runtime policy view in ws-rules and SDK/registry ingest preview hooks.
- Added six ingest-focused tests covering lifecycle transitions, resume-after-restart, deadletter/replay, heartbeat stall, integrity mismatch, and backoff/rate-limit behavior.

## 3) Self-check results
- Allowed-path check OK
  - allowlist awk check over staged files returned empty.
- Tests added/updated OK
  - ws_ingest_state_machine
  - ws_ingest_resume_after_restart
  - ws_ingest_deadletter_and_replay
  - ws_ingest_heartbeat_and_stall
  - ws_ingest_checksum_gap
  - ws_ingest_rate_limit_and_backoff
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-transport --lib` => OK
  - `cd ucel && cargo test -p ucel-ws-rules --lib` => OK
  - `cd ucel && cargo test -p ucel-subscription-planner --lib` => OK
  - `cd ucel && cargo test -p ucel-subscription-store --lib` => OK
  - `cd ucel && cargo test -p ucel-journal --lib` => OK
  - `cd ucel && cargo test -p ucel-sdk --lib` => OK
  - `cd ucel && cargo test -p ucel-registry --lib` => OK
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_state_machine -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_resume_after_restart -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_deadletter_and_replay -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_heartbeat_and_stall -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_checksum_gap -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test ws_ingest_rate_limit_and_backoff -- --nocapture` => 1 passed
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => OK.
- Secrets scan
  - `rg -n "AKIA|SECRET_KEY|BEGIN PRIVATE KEY|token|secret|signature" docs/specs/ucel/ws_ingest_runtime_v1.md ucel/docs/marketdata/ws_* docs/verification/UCEL-WS-INGEST-010.md` => no secret material (policy text only).
- docsリンク存在チェック（今回触った docs 内の `docs/` 参照）
  - new docs do not introduce `docs/` internal links.

## 4) 履歴確認の証拠
- `git log --oneline --decorate -n 30` / `git log --graph --oneline --decorate --all -n 40`
  - latest chain: `44cd0a3` (public adapter scaffolding) on top of `2d83fec` merge (#461 private ws scaffolding).
- `git show 44cd0a3 --stat`, `git show 1f72be0 --stat`, `git show 9a8fb42 --stat`
  - confirms preceding tasks established private/public scaffolding and policy gating used as base.
- blame evidence (`git blame -w` on ws transport/rules/planner/store/journal/registry/sdk)
  - transport/ws and ws-rules are hotspots; this task kept changes localized by adding new modules (`supervisor`, `restart`, `backoff`, `runtime_policy`) rather than rewriting existing runtime files.
- `git reflog -n 20`, `git branch -vv`, `git log --merges --oneline -n 20`
  - confirms branch ancestry and recent merge train.
- `git merge-base HEAD origin/master` and origin-based conflict checks
  - not executable in this environment because `origin/master` ref is absent; recorded as environment limitation.

### Responsibility decisions (planner/store/journal)
- planner = desired stream plan generation + replan intent.
- store = durable runtime state and checkpoint association.
- journal = append-only transition/failure evidence with sanitized detail.

### public/private ingest runtime current-state and design decisions
- Public/private runtime now emit typed failure signals mapped to ingest failure classes.
- Supervisor applies journal-first transition recording then durable store update.
- Deadletter/resume/backoff directives are derived from typed failure and integrity mode.

### Added work for identified gaps
- Added explicit core ingest lifecycle and transition guards to avoid ad-hoc state drift.
- Added replay/resume primitives and dedicated tests to enforce restart behavior and deadletter traceability.
