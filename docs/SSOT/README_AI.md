# README_AI (SSOT entrypoint for operators)

This page is the must-read entrypoint for AI/human operators before starting work.

## Must-read links (in order)

1. This file: `docs/SSOT/README_AI.md` (AI constitution / entrypoint)
2. Human-readable status snapshot (non-SSOT): `docs/status/CURRENT_STATUS.md`
3. Runtime status SSOT: `docs/status/status.json`
4. Tech context hub (non-SSOT links-only): `docs/SSOT/TECH_CONTEXT.md`
5. Prompt templates (non-SSOT, copy/paste): `docs/SSOT/AI_PROMPTS.md`
6. Handoff state: `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json`
7. Decision baseline: `docs/status/decisions.md`
8. Trace SSOT: `docs/status/trace-index.md` and `docs/status/trace-index.json`
9. Task Generation SSOT:
   - `docs/rules/task-generation-policy-v3-enforced.md`
   - `docs/rules/task-generation-rules-using-docs-os.md`
10. Rules / Safety references:
   - `docs/rules/task-generation-policy.md` (compatibility stub)
   - `docs/rules/parallel-development-safety.md`
   - `docs/specs/parallel-task-safety.md`
11. Before PR / Merge checklist: `docs/runbooks/pr-preflight.md`

## SSOT boundaries (important)

- **Canonical AI entrypoint:** `docs/SSOT/README_AI.md`
- **Status SSOT:** `docs/status/status.json`
- **`docs/status/CURRENT_STATUS.md` is a summary only (non-SSOT).**
- **`docs/SSOT/TECH_CONTEXT.md` is a links hub only (non-SSOT).**
- **`docs/SSOT/AI_PROMPTS.md` is an execution prompt library (non-SSOT).**
- Detailed specifications must remain in their existing canonical docs (specs/workplan/contracts/runbooks/status artifacts).

## Magic Prompt (universal, shortest)

Copy/paste this into any AI before starting:

> Read in order: `docs/SSOT/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
> Treat `docs/status/status.json` as SSOT and check `active_task`, `open_prs`, `locks_held` before planning.
> If stopping/pause/credit-out before completion, you MUST update `docs/handoff/HANDOFF.json` (and keep `docs/status/HANDOFF.json` aligned if used).
> Use `docs/status/trace-index.json` as trace link SSOT.
> In task cards always declare Allowed/Forbidden paths and Required LOCKS; if LOCK conflicts exist, stop and return.

## Mandatory operating reminders

- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
- **Single active task:** only one active task is tracked in `docs/status/status.json`.
- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
- **Trace SSOT:** `docs/status/trace-index.md` / `docs/status/trace-index.json` are the canonical place for PR/commit/evidence links.
- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
- **Preflight gate for task issuance:** if preflight cannot be satisfied, do **not** issue implementation tasks; issue a docs-only remediation task first.
