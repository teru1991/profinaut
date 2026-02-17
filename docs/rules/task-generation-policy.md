# Task Generation Policy (SSOT)

This document is the canonical source for task-card structure and multi-AI execution governance.

## 1. Mandatory read order at task start

At the beginning of work, every operator/agent must read in this order:

1. `docs/SSOT/README_AI.md`
2. `docs/status/status.json`
3. `docs/status/HANDOFF.json`
4. `docs/status/decisions.md`

If any file is missing, stale, or contradictory, pause and resolve documentation consistency before implementation.

## 2. One active task rule

- Exactly **one active task** is allowed at a time.
- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
- Starting a new task requires setting previous active task to completed/blocked/aborted first.

## 3. Task card required fields

Every task request must include, at minimum:

- `TASK ID`
- `Title`
- `Scope`
- `Execution mode`
- `Depends-on`
- `LOCK`
- `Required Locks`
- Allowed paths (`ONLY`)
- Forbidden paths (`MUST NOT touch`)
- Deliverables
- Acceptance criteria
- Verification steps
- Rollback plan

## 4. Multi-AI / Credit-out / Handoff protocol

### 4.1 Multi-AI execution baseline

- Multi-AI collaboration is allowed only when all collaborators use the same task card and scope.
- Canonical runtime status must be updated through `docs/status/status.json` only.
- Handoff responsibility is explicit: the current owner must leave resumable state.

### 4.2 Credit-out (forced stop) rule

When an operator must stop early (time, token, context, incident, rate limits), that stop is treated as **Credit-out** and requires a handoff update before exiting.

### 4.3 Mandatory HANDOFF update on stop/switch

Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:

- `what_done`
- `what_next`
- `errors`
- `commands_next`

Without this update, the task is considered not safely handed over.

## 5. Progress claim evidence rule

- Any claim like “progress advanced”, “task moved forward”, or “phase completed” must be accompanied by:
  - an update to `docs/status/status.json`, **or**
  - a new entry under `docs/status/progress-updates/`.
- Narrative-only progress claims outside these logs are non-authoritative.

## 6. Decision fixation policy

- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
- If work depends on a decision, reference its decision entry ID/date in task notes.
- Do not silently override a recorded decision; record superseding decision explicitly.

## 7. Traceability policy

- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
- Avoid scattering authoritative trace links across unrelated docs.
- Other docs may include convenience links, but must treat trace-index as canonical.

## 8. Allowed/forbidden path discipline

- Unknown path is dangerous: if path is not explicitly allowed in task card, do not edit it.
- “Incidental edits” are prohibited.
- Split out extra work into follow-up tasks instead of broadening scope ad hoc.
