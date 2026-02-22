# Task Generation Policy v3 (Docs OS Enforced) (SSOT)

This is the canonical v3 policy for generating executable tasks in this repository.
It operationalizes Docs Development OS as hard gate conditions.

## 0. Policy objective

Generate tasks that are:

- scope-safe (single-purpose, explicit boundaries),
- lock-safe (no conflicting lock claims),
- handoff-safe (credit-out recoverable),
- traceable (status/trace evidence mandatory),
- merge-safe (preflight-aware).

## 1. Generation pipeline (required)

1. **Context load**: read `README_AI` -> `status.json` -> `HANDOFF` -> `decisions` -> `trace-index.json` -> `pr-preflight`.
2. **Feasibility check**: ensure locks/path constraints/preflight can be satisfied.
3. **Task card synth**: emit strict schema fields (Section 2).
4. **Safety gate**: reject issuance if conflict/ambiguity unresolved.
5. **Output**: one task = one scope; include verification + rollback.

## 2. Required task-card schema

Every generated task must contain all fields:

- `TASK ID`
- `Title`
- `Scope`
- `Execution mode`
- `Depends-on`
- `Required Locks`
- `Allowed paths (ONLY)`
- `Forbidden paths (MUST NOT touch)`
- `Deliverables`
- `Verification steps`
- `Rollback plan`
- `Acceptance criteria`

Optional additions (recommended): risk notes, non-goals, open questions.

## 3. Issuance hard-fail conditions

Do not generate implementation tasks when any is true:

- lock conflict unresolved,
- allowed/forbidden path boundary unclear,
- active task state is contradictory or stale and unresolved,
- handoff path is not defined for interruption,
- preflight cannot be met,
- decision baseline needed but missing.

In these cases, generate a **docs-only remediation task** first.

## 4. Multi-AI / credit-out requirements

Each generated task must be resumable by another operator.
Therefore include explicit stop protocol requirements:

- on pause/stop, update `docs/handoff/HANDOFF.json`,
- include `what_done`, `what_next`, `errors`, `commands_next`,
- preserve command-level reproducibility.

## 5. Evidence requirements for progress claims

A task is not "advanced" unless evidence is updated in at least one:

- `docs/status/status.json`
- `docs/status/progress-updates/*`

And trace links should be present in:

- `docs/status/trace-index.json`

## 6. Merge safety coupling (preflight-aware generation)

Generated tasks must be merge-ready in design:

- explicit dependency ordering (`Depends-on`),
- explicit lock ownership (`Required Locks`),
- rollback-safe sequencing (older branch must not overwrite newer intent),
- CI/check commands included in verification steps.

## 7. Canonical references

- Enforcement rules: `docs/rules/task-generation-rules-using-docs-os.md`
- Parallel lock policy: `docs/rules/parallel-development-safety.md`
- Entry and runtime: `docs/SSOT/README_AI.md`, `docs/status/status.json`
- Handoff: `docs/handoff/HANDOFF.json`
- Decisions: `docs/decisions/decisions.md`
- Trace: `docs/status/trace-index.json`
- PR safety: `docs/runbooks/pr-preflight.md`
