# README_AI (SSOT entrypoint for operators)

This page is the must-read entrypoint for AI/human operators before starting work.

## Must-read links (in order)

1. This file: `docs/SSOT/README_AI.md`
2. Runtime status: `docs/status/status.json`
3. Handoff state: `docs/status/HANDOFF.json`
4. Decision baseline: `docs/status/decisions.md`
5. Rules SSOT:
   - `docs/rules/task-generation-policy.md`
   - `docs/rules/parallel-development-safety.md`

## Mandatory operating reminders

- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
- **Single active task:** only one active task is tracked in `docs/status/status.json`.
- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
