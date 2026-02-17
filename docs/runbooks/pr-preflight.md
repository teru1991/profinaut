# PR Preflight & Merge Safety Checklist (Multi-AI)

Purpose: prevent PR/merge accidents (stale PR merge, LOCK collision, SSOT drift, docs-state mismatch) with a single reusable procedure.

Related canonical docs:
- AI entrypoint: `docs/SSOT/README_AI.md`
- Runtime status SSOT: `docs/status/status.json`
- Handoff: `docs/handoff/HANDOFF.json` and `docs/status/HANDOFF.json`
- Decisions: `docs/status/decisions.md`
- Trace index SSOT: `docs/status/trace-index.json`
- Safety policy: `docs/rules/parallel-development-safety.md`, `docs/specs/parallel-task-safety.md`

---

## 1) PR Preflight Checklist (before opening/updating PR)

- [ ] Base branch is up-to-date (fetch/rebase completed and no stale base divergence).
- [ ] No `Required Locks` conflict with open PRs touching same lock scope.
- [ ] `git diff --name-only` changes are only in declared **Allowed paths**.
- [ ] No changes in **Forbidden paths**.
- [ ] If `Depends-on` PR exists and dependency is unmerged, keep this PR in Draft.
- [ ] Scope is single-purpose ("1 scope") and does not mix unrelated concerns.
- [ ] Progress evidence updated:
  - [ ] either `docs/status/status.json` updated, or
  - [ ] `docs/status/progress-updates/*` appended.
- [ ] If task is incomplete / handoff needed:
  - [ ] update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.

---

## 2) PR Description Template (copy/paste)

```md
## Task
- Task ID: <TASK-ID>
- Scope: <one-scope summary>

## Dependencies
- Depends-on:
  - <PR/Issue link or "none">

## Locks
- Required Locks:
  - <lock-key-1>
  - <lock-key-2 or "none">

## Changed paths summary
- <path-group 1>: <what changed>
- <path-group 2>: <what changed>

## Verification steps
- `<command 1>`
- `<command 2>`
- `<command 3>`

## Rollback plan
- Revert this PR commit(s) and re-apply via fresh branch on latest base.

## Notes / Risks
- <known risks, assumptions, residual concerns>
```

---

## 3) Merge Decision Checklist (before merge)

- [ ] Required CI checks are green.
- [ ] Branch is up-to-date with target base at merge time.
- [ ] No lock collision: no other open PR currently claims same `Required Locks`.
- [ ] All dependency PRs (`Depends-on`) are already merged.
- [ ] No rollback regression risk from stale PR ordering (older PR would not overwrite newer intent).
- [ ] PR link/evidence is registered in `docs/status/trace-index.json` if required by team flow.
- [ ] `docs/status/decisions.md` updated if assumptions/spec baseline changed.

---

## 4) Rollback / Conflict Handling (short guidance)

- If an old PR would rollback newer behavior, **do not merge**; rebase/recreate on latest base.
- If conflicts are large, split by scope/LOCK and merge in dependency order.
- If spec/assumptions changed mid-flight, record explicit entry in `docs/status/decisions.md` before merge.
