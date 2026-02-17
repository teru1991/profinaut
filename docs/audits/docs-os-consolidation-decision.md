# DOC-OS Consolidation Decision

| Artifact | Existing candidate(s) | Proposed canonical path | Keep/Merge/Replace | Notes (why) |
|---|---|---|---|---|
| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/parallel-task-safety.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/SSOT/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |

## Consolidation notes
- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
- `docs/README.md` remains a general docs hub; `docs/SSOT/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
