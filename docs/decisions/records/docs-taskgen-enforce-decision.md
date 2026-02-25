# DOC-TASKGEN-ENFORCE Consolidation Decision

| Artifact | Existing candidate(s) | Proposed canonical path | Action (keep/merge/stub) | Notes |
|---|---|---|---|---|
| Enforced rules (Docs OS) | `docs/rules/task-generation-policy.md`, `docs/rules/parallel-development-safety.md` | `docs/rules/task-generation-rules-using-docs-os.md` | create/merge | New focused canonical rule set for Docs OS enforcement (locks/handoff/trace/preflight gate). |
| Task-gen policy v3 (docs OS enforced) | `docs/rules/task-generation-policy.md` | `docs/rules/task-generation-policy-v3-enforced.md` | create/merge | v3 canonical policy with strict issuance criteria and task-card schema. |
| Existing task-gen docs | `docs/rules/task-generation-policy.md`, `docs/specs/crosscut/parallel_task_safety_spec.md` | `docs/rules/task-generation-policy-v3-enforced.md` + `docs/rules/task-generation-rules-using-docs-os.md` | merge/stub | Keep compatibility links but remove duplicate SSOT intent by converting old policy to stub that points to new canonicals. |
| README_AI links | `docs/context/README_AI.md` | `docs/context/README_AI.md` | update | Add must-read links under Task Generation SSOT and explicit "no preflight => docs-only task" rule. |

## Consolidation rules applied

- One-topic-one-canonical: task generation enforcement lives in two non-overlapping canonical docs (rules and policy-v3).
- Previous `task-generation-policy.md` is retained as compatibility stub only to avoid link breakage.
- `README_AI.md` remains the entrypoint and links all canonical task-generation docs.
