# TECH_CONTEXT (Navigation hub for AI/operators)

This page is a **links-and-constraints hub** to help operators navigate quickly.
It is **not** a full technical SSOT and must not duplicate long specs.

> Principle: **Link to canonical docs; do not copy entire schemas/specs here.**

## Must-read (start here)

- AI entrypoint: `docs/SSOT/README_AI.md`
- Parallel safety rules: `docs/specs/parallel-task-safety.md` and `docs/rules/parallel-development-safety.md`
- Runtime status SSOT: `docs/status/status.json`
- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
- Decisions baseline: `docs/status/decisions.md` (and `docs/decisions/decisions.md` if referenced by task)
- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)

## Architecture pointers (code locations)

- Services: `services/`
- Applications/UI: `apps/`
- Contracts/API schemas: `contracts/`

> These paths are implementation sources. Consult task-specific docs before editing.

## Data / DB / protocol pointers

- Workplan: `docs/workplan/ultimate-gold-implementation-feature-list.md`
- Market/data behavior specs: `docs/specs/ui-marketdata.md`, `docs/specs/execution.md`, `docs/specs/execution-gmo.md`
- Bot/control specs: `docs/specs/controlplane-bots.md`, `docs/specs/simple-bot.md`, `docs/specs/ui-bots.md`
- Verification evidence: `docs/verification/marketdata-data-platform-smoke.md`, `docs/verification/marketdata-data-platform-smoke-results.md`

## Environment / secrets / operations policy

- Supply chain and security runbook: `docs/runbooks/supply-chain-security.md`
- Local marketdata runbooks: `docs/runbooks/marketdata-local.md`, `docs/runbooks/marketdata-replay.md`
- E2E operational guides: `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md`

## Constraints reminder

- `TECH_CONTEXT.md` is a map, not a replacement for source specs.
- Do **not** paste full schemas, contract definitions, or long procedures here.
- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
