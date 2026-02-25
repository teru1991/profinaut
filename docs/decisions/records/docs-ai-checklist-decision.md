# DOC-AI-CHECKLIST Consolidation Decision

| Artifact | Existing candidate(s) | Proposed canonical path | Action (keep/merge/stub) | Notes |
|---|---|---|---|---|
| PR preflight checklist | No dedicated checklist/preflight runbook found in `docs/runbooks/` | `docs/runbooks/pr-preflight.md` | choose one | Checklist is operational procedure, so runbooks location is preferred. |
| Merge decision template | No dedicated merge-decision template found; partial reminders in `docs/context/README_AI.md` | `docs/runbooks/pr-preflight.md` | merge | Keep one practical checklist/template doc to avoid duplication. |
| Rollback prevention rules | Existing safety/parallel policy docs (`docs/specs/crosscut/safety_interlock_spec.md`, `docs/specs/crosscut/parallel_task_safety_spec.md`) | `docs/runbooks/pr-preflight.md` (with references) | reference | Keep concise runbook guidance and link policy sources rather than duplicating policy text. |
| Multi-AI handoff requirements | `docs/context/README_AI.md`, `docs/status/HANDOFF.json`, `docs/handoff/HANDOFF.json` | `docs/context/README_AI.md` + `docs/runbooks/pr-preflight.md` link/reference | reference | README_AI stays canonical entrypoint; runbook references handoff obligations for PR/merge flow. |

## Consolidation rules applied

- Canonical operations checklist lives in `docs/runbooks/pr-preflight.md`.
- `docs/context/README_AI.md` remains AI entrypoint and links this checklist as pre-PR must-read.
- No duplicate checklist docs were found requiring stub conversion.
