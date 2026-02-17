# Parallel Development Safety (SSOT)

This document is the canonical source for safe parallel work execution and lock governance.

## 1. Purpose

Prevent fatal merge/process accidents during concurrent delivery by enforcing clear scope boundaries and lock declarations.

## 2. Core rules

### 2.1 1PR = 1 scope

- Every PR must have one explicit scope.
- If additional scope appears, split into separate task/PR.

### 2.2 Depends-on + Draft rule

- If task B depends on task A, B stays Draft until A merges.
- `Depends-on` must be explicit in task metadata and PR description.

### 2.3 Allowed/Forbidden path enforcement

- Edit only paths listed as allowed by the task card.
- Respect forbidden paths without exception.
- Unknown path access is treated as safety violation.

## 3. LOCK policy

### 3.1 LOCK / semi-LOCK behavior

- **LOCK area**: only one active PR may touch that area.
- **Semi-LOCK area**: parallel PRs allowed only with zero file overlap and explicit coordination.
- On overlap risk, serialize changes.

### 3.2 Default lock-sensitive areas

Unless a task says otherwise, treat the following as lock-sensitive:

- `contracts/**`
- `ci/**` and `.github/**`
- `infra/**`
- lockfiles
- shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs/indexes)

### 3.3 Required Locks declaration

Task cards must declare `Required Locks` explicitly. Examples:

- `Required Locks: docs/SSOT/**`
- `Required Locks: none` (only when truly lock-free)

## 4. Shared-file policy

- Shared or high-risk files must have narrowly-scoped edits only.
- Do not combine unrelated content changes in shared docs/hubs.
- Prefer additive edits and explicit backlinks.

## 5. Conflict handling

When cross-scope edits become necessary during execution:

1. keep current PR constrained to declared scope,
2. create follow-up task/PR for extra scope,
3. connect with `Depends-on` and follow-up references.

## 6. Quick checklist

- [ ] One scope only (`1PR=1scope`)
- [ ] Allowed/Forbidden paths respected
- [ ] Depends-on and Draft constraints respected
- [ ] Required Locks declared
- [ ] No overlap in LOCK/semi-LOCK areas
- [ ] Revert/rollback path is clear
