# README_AI (SSOT entrypoint for operators)

This page is the must-read entrypoint for AI/human operators before starting work.

## Must-read links (in order)

1. This file: `docs/context/README_AI.md` (AI constitution / entrypoint)
2. Runtime status SSOT: `docs/status/status.json`
3. Human-readable status guidance (non-SSOT redirect): `docs/status/CURRENT_STATUS.md`
4. Tech context hub (non-SSOT links-only): `docs/context/TECH_CONTEXT.md`
5. Prompt templates (non-SSOT, copy/paste): `docs/context/AI_PROMPTS.md`
6. Handoff SSOT: `docs/handoff/HANDOFF.json`
7. Decision baseline: `docs/decisions/decisions.md`
8. Trace SSOT: `docs/status/trace-index.json`
9. Task Generation SSOT:
   - `docs/rules/task-generation-policy-v3-enforced.md`
   - `docs/rules/task-generation-rules-using-docs-os.md`
10. Rules / Safety references:
   - `docs/rules/task-generation-policy.md` (compatibility stub)
   - `docs/specs/crosscut/safety_interlock_spec.md`
   - `docs/specs/crosscut/parallel_task_safety_spec.md`
11. Before PR / Merge checklist: `docs/runbooks/pr-preflight.md`

## SSOT boundaries (important)

- **Canonical AI entrypoint:** `docs/context/README_AI.md`
- **Status SSOT:** `docs/status/status.json`
- **`docs/status/CURRENT_STATUS.md` is a summary only (non-SSOT).**
- **`docs/context/TECH_CONTEXT.md` is a links hub only (non-SSOT).**
- **`docs/context/AI_PROMPTS.md` is an execution prompt library (non-SSOT).**
- Detailed specifications must remain in their existing canonical docs (specs/workplan/contracts/runbooks/status artifacts).

## Magic Prompt (universal, shortest)

Copy/paste this into any AI before starting:

> Read in order: `docs/context/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` → `docs/decisions/decisions.md`.
> Treat `docs/status/status.json` as SSOT and check `active_task`, `open_prs`, `locks_held` before planning.
> If stopping/pause/credit-out before completion, you MUST update `docs/handoff/HANDOFF.json`.
> Use `docs/status/trace-index.json` as trace link SSOT.
> In task cards always declare Allowed/Forbidden paths and Required LOCKS; if LOCK conflicts exist, stop and return.


## PR LOCK declaration (required in PR descriptions)

When opening or updating a PR, include a short lock block in the PR description:

```
Required Locks:
- LOCK:<name>

Releases Locks:
- LOCK:<name>
```

If lock ownership is unknown, state `LOCK status: unknown` and do not proceed with conflicting work until resolved in `docs/status/status.json`.

## Mandatory operating reminders

- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
- **Single active task:** only one active task is tracked in `docs/status/status.json`.
- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
- **Trace SSOT:** `docs/status/trace-index.json` are the canonical place for PR/commit/evidence links.
- **Decisions are binding:** use `docs/decisions/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
- **Preflight gate for task issuance:** if preflight cannot be satisfied, do **not** issue implementation tasks; issue a docs-only remediation task first.
