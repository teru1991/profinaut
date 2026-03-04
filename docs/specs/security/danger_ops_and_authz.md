# Dangerous Ops & AuthZ Contract (Domain B / Step4)

## Goals
- Deny-by-default authorization.
- Dangerous operations require:
  - explicit policy allow
  - step-up auth
  - challenge/confirm token bound to (op, scope, actor, session, expiry)
  - audit health must be OK (audit down => deny)

## Definitions
- Operation: string identifier of an action (e.g. start_live, rotate_secret).
- Scope: structured string (e.g. bot:<name>:<env>, venue:<name>:<kind>).

## Contracts
1) Authz:
- If no allow entry matches, operation MUST be denied.

2) Dangerous ops gate:
- For dangerous ops, if no token present => return challenge (no execution).
- confirm() must:
  - require step_up
  - require audit health OK
  - validate token signature + binding + expiry
  - reject on mismatch (fail-closed)

3) Audit health:
- audit write failures must mark audit health down.
- dangerous ops must refuse when audit health down.

4) Dual-run lease:
- If lease conflict detected for same scope => refuse (fail-closed).

## Tests
- tests/test_danger_ops_gate.py
- tests/test_dual_run_lease_guard.py
