# Verification: C-B-OBSSEC-003

## 1) Changed files
- (see) `git diff --name-only`

## 2) What/Why
- Enforced JSON limits at venue WS parse entry points by inserting `InboundJsonGuard::enforce(raw)` before JSON parsing.
- Standardized transport-side decode-error telemetry helper API (`record_decode_error`, `inbound_violation`) and used it on transport frame-size violation paths.
- Added startup/runtime endpoint validation in registry hubs:
  - WS endpoints validated with allowlist + wss/https scheme enforcement.
  - REST base endpoints validated with allowlist + strict https for REST.
- Added CEX-side endpoint guard calls in required venue crates (coincheck/bitbank/bitflyer/bybit/htx/sbivc/upbit) so override/spec URLs are validated before use.
- Added registry allowlist tests and finalized crosscut observability/support-bundle docs references.

## 3) Self-check results
- Allowed-path check: OK
- Tests:
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-transport -p ucel-registry` : OK
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-okx -p ucel-cex-bybit -p ucel-cex-bitflyer -p ucel-cex-bitbank -p ucel-cex-upbit -p ucel-cex-htx` : OK
- Secrets scan:
  - `git diff | rg -n "(Bearer |Authorization:|api_key=|secret=|signature=)"` -> no secrets

## 4) History check evidence (required)
- Reviewed:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git log --merges --oneline -n 30`
  - `git show HEAD`
- Last-touch checks reviewed:
  - transport ws/adapter: latest foundation commit `32dc557`
  - registry hub ws/rest: `ae692ad`
  - cex bitflyer/bybit lib: `48dbd54`
- Blame summaries reviewed:
  - registry ws hub selection/subscribe loop kept intact; validation inserted near endpoint resolution.
  - transport ws connection reconnect/read/write loops kept intact; only guard/telemetry hooks added.
- Remote note: `origin/<default-branch>` is unavailable in this environment; local merge-base used:
  - `git merge-base HEAD work`.

## 5) “No unguarded path” checklist
- [x] Venue classify_inbound: guard.enforce(raw) inserted in all listed ws.rs.
- [x] registry hub ws/rest: endpoint validate invoked on resolved URLs.
- [x] cex overrides/spec URLs: validate invoked in required cex lib.rs call paths.
- [x] ws max frame bytes: enforced in transport inbound handling with fixed WS protocol violation message.
