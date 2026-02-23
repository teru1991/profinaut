# PR Preflight & Merge Safety Checklist (Multi-AI)

Purpose: prevent PR/merge accidents (stale PR merge, LOCK collision, SSOT drift, docs-state mismatch) with a single reusable procedure.

Related canonical docs:
- AI entrypoint: `docs/SSOT/README_AI.md`
- Runtime status SSOT: `docs/status/status.json`
- Handoff: `docs/handoff/HANDOFF.json`
- Decisions: `docs/decisions/decisions.md`
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
- [ ] `docs/decisions/decisions.md` updated if assumptions/spec baseline changed.

---

## 4) Rollback / Conflict Handling (short guidance)

- If an old PR would rollback newer behavior, **do not merge**; rebase/recreate on latest base.
- If conflicts are large, split by scope/LOCK and merge in dependency order.
- If spec/assumptions changed mid-flight, record explicit entry in `docs/decisions/decisions.md` before merge.

---

## 5) CI workflow correctness audit (read-only; no workflow edits)

When CI fails, first confirm whether the workflow definition is behaving as intended **without editing** `.github/workflows/**` in this task.

### 5.1 Audit checklist (copy/paste)

- [ ] Identify which workflow ran (`Workflow name`, file path, trigger event).
- [ ] Confirm trigger/event context is expected (`pull_request`, `push`, `workflow_dispatch`, branch/path filter conditions).
- [ ] Confirm job-level and workflow-level `permissions` are minimally sufficient (no over-permission, no missing read/write required for steps).
- [ ] Confirm `concurrency` behavior is expected (group key and `cancel-in-progress` are not unintentionally cancelling required runs).
- [ ] Confirm matrix/conditional execution (`if`, `needs`) is not skipping required validation.
- [ ] Confirm affected file paths in PR actually match expected `paths` / `paths-ignore` conditions.

### 5.2 CI failure evidence capture template

Use the template below in PR comment/review request so triage can be done without re-running guesswork:

```md
## CI Failure Evidence
- Workflow: <name> (`.github/workflows/<file>.yml`)
- Event/Ref: <pull_request|push|workflow_dispatch> / <branch-or-ref>
- Job: <job-id-or-name>
- Failed step: <step-name>
- Run URL: <github actions run url>

### Error log (raw)
```text
<paste full error section from first failure line through stack trace / command output end>
```

### Reproduction hints
- Diff scope: <changed paths>
- Suspected trigger condition: <paths/if/needs/concurrency/permissions>
- Local reproduction command (if any): `<command>`
```

---

## 6) File creation/edit rules to reduce CI failures

### 6.1 Before editing

- [ ] Declare task `Allowed paths (ONLY)` and `Forbidden paths (MUST NOT touch)` first.
- [ ] Verify actual edit targets with `git diff --name-only` stay inside allowed scope.
- [ ] Keep `1PR=1scope` (split unrelated fixes into follow-up tasks).

### 6.2 Before commit

- [ ] Run formatting/lint/typecheck commands required by the touched area (project-specific defaults).
- [ ] Normalize line endings to `LF` unless target file explicitly requires otherwise.
- [ ] Avoid accidental generated artifacts (`dist`, `.next`, `node_modules`, `__pycache__`, `*.pyc`).
- [ ] Follow existing naming conventions in the touched directory (do not introduce ad-hoc patterns).

### 6.3 PR operation guardrails

- [ ] Keep PR in Draft until dependency/lock constraints are clear.
- [ ] Move Draft -> Ready only after local checks + CI evidence are attached.
- [ ] If CI fails, attach the evidence template from section 5.2 before requesting help.

---

## 7) Gemini-assisted CI review protocol (on-demand only)

Use Gemini review only when CI failure root cause is unclear or repeated. Do **not** auto-trigger on every PR.

### 7.1 Review request template (copy/paste)

```md
Please review this CI failure and propose a minimal fix.

Context:
- Task/PR scope: <scope>
- Changed paths: <paths>
- Workflow/job/step: <workflow> / <job> / <step>
- Run URL: <url>

Logs:
```text
<paste failure log>
```

Reproduction:
- Local command tried: <command or none>
- Result: <output summary>

Required output format:
1) Cause hypothesis (most likely -> second likely)
2) Minimal diff fix proposal
3) Impact/risk scope
4) GitHub suggestion blocks if possible
```

### 7.2 Required Gemini output format

Ask Gemini to always return:

1. **Cause hypothesis** (most likely -> second likely)
2. **Minimal diff fix proposal**
3. **Impact/risk scope** (CI/jobs/files/runtime impact)
4. **GitHub suggestion blocks** (if possible)

### 7.3 Noise control rules

- Trigger Gemini only for blocking or repeated CI failures.
- Close the loop by posting whether the proposal fixed CI.
- If proposal is rejected, record why (false positive / scope mismatch / unsafe change).

---

## Update history

- 2026-02-17: Added CI workflow read-only audit checklist, CI-safe editing checklist, and Gemini-assisted CI review protocol.
