# DOC-AI-PACK Consolidation Decision

| Artifact | Existing candidate(s) | Proposed canonical path | Action (keep/merge/stub) | Notes |
|---|---|---|---|---|
| AI entry (constitution) | `docs/SSOT/README_AI.md` | `docs/SSOT/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
| Tech context (links + constraints) | No dedicated tech-context hub found in `docs/` | `docs/SSOT/TECH_CONTEXT.md` | create/merge | Links-only navigation hub to existing specs/runbooks/status artifacts; no full-schema duplication. |
| Any root-level AI files | None detected | none | avoid | Do not create root-level `AI_READ_ME.md` or equivalent SSOT duplicates. |

## Consolidation rules applied

- Keep `docs/SSOT/README_AI.md` as the single AI constitution entrypoint.
- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
- Keep `docs/SSOT/TECH_CONTEXT.md` as a link hub (not a new canonical source of detailed specs).
- No duplicate-intent files requiring stub conversion were found during the scan.
