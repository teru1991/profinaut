# Task Generation Rules using Docs Development OS (SSOT)

This document defines **enforced** task-generation rules for this repository.
Any generated task MUST comply with Docs Development OS artifacts:

- `docs/SSOT/README_AI.md`
- `docs/status/status.json`
- `docs/handoff/HANDOFF.json`
- `docs/decisions/decisions.md`
- `docs/status/trace-index.json`
- LOCK discipline from `docs/rules/parallel-development-safety.md`

## 1) Enforced pre-read order

Task generation must load context in this order before proposing any implementation task:

1. `docs/SSOT/README_AI.md`
2. `docs/status/status.json`
3. `docs/handoff/HANDOFF.json`
4. `docs/decisions/decisions.md`
5. `docs/status/trace-index.json`
6. `docs/runbooks/pr-preflight.md`

If any artifact is missing/contradictory/stale, do not issue implementation tasks.
Issue a **docs-only consistency-fix task** first.

## 2) Mandatory runtime checks before task issuance

A task generator MUST verify from `docs/status/status.json`:

- `active_task`
- `open_prs` (if present in schema/workflow)
- `locks_held` (if present in schema/workflow)
- owner/state recency (`owner`, `state`, `last_updated`)

If lock state cannot be validated, emit a docs-only task to restore status observability.

## 3) Task card must include hard constraints

Every generated task card MUST include:

- Task ID / Title / Scope / Execution mode
- Allowed paths (`ONLY`)
- Forbidden paths (`MUST NOT touch`)
- Required Locks
- Depends-on
- Deliverables
- Verification steps (commands)
- Rollback plan
- Acceptance criteria

Missing any of the above means task is invalid and should not be issued.

## 4) LOCK conflict handling (hard stop)

If required lock overlaps with another active/open workstream:

- Stop issuance of implementation task.
- Return conflict report + replan options (split scope, re-sequence, or wait).
- Never auto-widen scope to "just proceed".

## 5) Preflight gate before implementation tasks

Implementation task issuance is allowed only if PR preflight is satisfiable (`docs/runbooks/pr-preflight.md`):

- base can be made up-to-date,
- required locks can be non-conflicting,
- allowed/forbidden path boundary is clear,
- progress evidence path is defined (`status.json` or `progress-updates`),
- handoff update path is defined for interruption.

If preflight cannot be satisfied, issue **docs-only remediation task** (not implementation).

## 6) Credit-out / stop / handoff enforcement

When work may stop before completion, generated tasks MUST require handoff updates:

- `what_done`
- `what_next`
- `errors`
- `commands_next`

No handoff plan => task generation is incomplete.

## 7) Evidence and trace enforcement

A task generator MUST require that:

- progress claims are backed by `docs/status/status.json` updates and/or `docs/status/progress-updates/*`,
- PR/commit/run links are registered in `docs/status/trace-index.json`.

Narrative-only progress without evidence is non-compliant.

## 8) Decision and assumption discipline

If requirements are ambiguous or changed, generator MUST:

- require decision/assumption update in `docs/decisions/decisions.md` (and/or `docs/assumptions.md` where applicable),
- block irreversible implementation tasks until decision baseline is explicit.

## 9) SSOT boundary rule

This document governs generation behavior only.
Do not duplicate detailed product/contract specifications here; link canonical specs instead.
