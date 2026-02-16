# Parallel Task Safety Spec (SSOT)

This document is the source-of-truth for safe parallel documentation and implementation work in Profinaut.

## 1. Purpose
Prevent fatal merge/process accidents during parallel delivery by making task boundaries explicit and enforceable.

Safety alignment:
- Profinaut Ultimate Gold Spec emphasizes safety-first operation and auditable process.
- Unknown path is dangerous: if a file/path is not explicitly allowed by a task, do not touch it.

## 2. Core operating rules

### 2.1 1PR = 1scope
- Every PR must have exactly one explicit scope.
- Scope must map to an isolated task objective.
- If work grows across multiple scopes, split into separate PRs.

### 2.2 Depends-on + Draft rule
- If task B depends on task A, task B PR must be opened as Draft until A is merged.
- `Depends-on` must be written in task metadata and PR description.
- Do not silently bundle dependency work from another scope.

### 2.3 LOCK / semi-LOCK operation
- LOCK area: one active PR at a time for that area.
- Semi-LOCK area: parallel PRs are allowed only if file-level overlap is zero.
- If overlap risk exists, serialize changes.

### 2.4 Shared-file policy
- Shared/high-risk files (e.g., root `README.md`, docs hubs, central specs) require strict scope isolation.
- Do not mix unrelated edits in shared files.
- Prefer short additive edits with clear headings and backlinks.

### 2.5 Conflict resolution rule
- If execution reveals cross-scope edits are necessary, stop and split work:
  1. keep current PR limited to declared scope,
  2. create follow-up task/PR for the extra scope,
  3. link both directions (`Depends-on` / "follow-up").

## 3. Contracts safety rule (reference)
- Contracts are additive-only when changed.
- Contract changes should run in dedicated contract-focused scope/PR.
- Non-contract tasks must reference this rule and avoid opportunistic contract edits.

## 4. Profinaut開発：Codexアジャイル「タスク生成」方針

### 4.1 Task definition minimum
Each task should define at least:
- `TASK ID`
- `Title`
- `Scope`
- `Execution mode` (e.g., SINGLE-RUN)
- `Depends-on`
- `LOCK`
- Allowed paths (ONLY)
- Forbidden paths (MUST NOT touch)
- Deliverables / Acceptance criteria / Verification / Rollback plan

### 4.2 Unknown path is dangerous
- 未知パス（タスクで許可されていないパス）は危険扱いにする。
- "ついで編集"は禁止。
- 迷った場合は、現在PRでは触らず次タスクへ分離する。

### 4.3 Parallel-safe task slicing
- Prefer vertical slices with minimal file overlap.
- Use docs hubs/spec hubs to avoid duplication.
- Keep one task small enough for deterministic review and rollback.

## 5. PR checklist (quick)
- [ ] One scope only.
- [ ] Allowed paths only.
- [ ] Depends-on respected (Draft if blocked).
- [ ] No cross-scope shared-file drift.
- [ ] Rollback is possible via PR revert.

## 6. Examples
- Good: "docs-navigation" PR edits only `README.md`, `docs/README.md`, and one docs spec file.
- Bad: same PR edits README + services implementation + contracts because they were "related".

## 7. Change control
- Update this spec only via dedicated docs-safety/governance scope.
- Cross-reference from task templates and docs hub when process changes.
