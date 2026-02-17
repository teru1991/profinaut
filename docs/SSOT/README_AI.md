# README_AI (Canonical Docs Development OS Entry)

This is the **canonical SSOT entrypoint** for AI agents and humans operating this repository's docs workflow.

## Must-read order
1. North Star (What): `docs/workplan/ultimate-gold-implementation-feature-list.md`
2. Now/Progress (Now): `docs/status/ultimate-gold-progress-check.md`
3. Machine-readable current status: `docs/status/status.json`
4. Traceability SSOT: `docs/status/trace-index.json`
5. Rules SSOT: `docs/specs/parallel-task-safety.md`
6. Runbooks: `docs/runbooks/**`
7. Verification evidence: `docs/verification/**`
8. Progress evidence logs: `docs/status/progress-updates/*`
9. Handoff protocol: `docs/handoff/HANDOFF.json`
10. Decision log: `docs/decisions/decisions.md`

## Canonical SSOT declarations
- **North Star (What)** → `docs/workplan/ultimate-gold-implementation-feature-list.md`
- **Now/Progress (Now)** → `docs/status/ultimate-gold-progress-check.md`
- **Evidence** → `docs/status/progress-updates/*`
- **Rules SSOT** → `docs/specs/parallel-task-safety.md`
- **Runbooks** → `docs/runbooks/**`
- **Verification** → `docs/verification/**`
- **Credit-out / Stop protocol** → update `docs/handoff/HANDOFF.json` before stopping
- **Decision logging** → append to `docs/decisions/decisions.md`
- **Traceability SSOT** → `docs/status/trace-index.json`

## Update policy (mandatory)
- You MUST NOT claim progress unless it is reflected in `docs/status/status.json` or `docs/status/progress-updates/*`.
- When an AI agent stops, hands off, or is replaced, updating `docs/handoff/HANDOFF.json` is mandatory.
- PR URLs, commits, and requirement/evidence link mappings are SSOT in `docs/status/trace-index.json`; do not maintain separate competing link lists in other docs.
