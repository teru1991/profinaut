# DOC-AI-PROMPT Consolidation Decision

| Topic | Existing candidate(s) | Canonical location | Action (keep/merge/stub) | Notes |
|---|---|---|---|---|
| Magic prompt (must-read OS) | `docs/context/README_AI.md` mandatory entrypoint; no dedicated magic-prompt block found | `docs/context/README_AI.md` | merge | Keep single canonical onboarding page and add concise universal magic prompt there. |
| Role prompts (Planner/Builder/Verifier/Reviewer) | No dedicated role-prompt file found | `docs/context/AI_PROMPTS.md` | choose one | Create focused prompts file to avoid over-growing README_AI; README_AI links to it. |
| Stop/Credit-out/Handoff text | Existing stop protocol in `docs/context/README_AI.md` | `docs/context/README_AI.md` + mirrored in `docs/context/AI_PROMPTS.md` | merge | Keep wording consistent with OS: stop/pause/handoff requires HANDOFF update before exit. |

## Rules applied

- Canonical prompt locations are fixed to two non-conflicting roles:
  - `README_AI.md` for universal short magic prompt and governance boundary.
  - `AI_PROMPTS.md` for detailed role-specific copy/paste templates.
- No duplicate-intent prompt docs were detected in current `docs/**` scan, so no stub conversion required.
