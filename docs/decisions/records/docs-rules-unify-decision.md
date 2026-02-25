# DOC-RULES-UNIFY-001 Consolidation Decision

## Canonicalization summary

- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
- Anti-duplication action: `docs/specs/crosscut/parallel_task_safety_spec.md` is retained as a **stub** that redirects to canonical docs.

## Topic table

| Topic | Existing doc(s) | Proposed canonical doc | Action (keep/merge/stub) | Notes |
|---|---|---|---|---|
| Parallel safety / 1PR=1scope / Allowed/Forbidden | `docs/specs/crosscut/parallel_task_safety_spec.md` | `docs/specs/crosscut/safety_interlock_spec.md` | merge + stub old | Preserve prior safety intent and checklist; move to rules namespace. |
| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
| Task card required fields | `docs/specs/crosscut/parallel_task_safety_spec.md` section 4.1 | `docs/rules/task-generation-policy.md` | merge + extend | Keep minimum fields and add multi-AI runtime governance fields. |

## Link routing policy

- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
- `docs/README.md` points to the new canonical rules docs.
- Non-canonical legacy document remains as short stub to prevent link rot.
