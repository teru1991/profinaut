# README_AI (SSOT entrypoint for operators)

This page is the must-read entrypoint for AI/human operators before starting work.

## Must-read links (in order)

1. This file: `docs/SSOT/README_AI.md` (AI constitution / entrypoint)
2. Human-readable status snapshot (non-SSOT): `docs/status/CURRENT_STATUS.md`
3. Runtime status SSOT: `docs/status/status.json`
4. Tech context hub (non-SSOT links-only): `docs/SSOT/TECH_CONTEXT.md`
5. Handoff state: `docs/status/HANDOFF.json`
6. Decision baseline: `docs/status/decisions.md`
7. Trace SSOT: `docs/status/trace-index.md`
8. Rules SSOT:
   - `docs/rules/task-generation-policy.md`
   - `docs/rules/parallel-development-safety.md`
   - `docs/specs/parallel-task-safety.md`

## SSOT boundaries (important)

- **Canonical AI entrypoint:** `docs/SSOT/README_AI.md`
- **Status SSOT:** `docs/status/status.json`
- **`docs/status/CURRENT_STATUS.md` is a summary only (non-SSOT).**
- **`docs/SSOT/TECH_CONTEXT.md` is a links hub only (non-SSOT).**
- Detailed specifications must remain in their existing canonical docs (specs/workplan/contracts/runbooks/status artifacts).

## Mandatory operating reminders

- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
- **Single active task:** only one active task is tracked in `docs/status/status.json`.
- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
