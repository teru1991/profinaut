# CI Workflow Gap Plan (TASK: CI-010)

## Goal
不足/弱点（×/△）を最小変更で埋め、`1PR=1scope` と `LOCK:ci` 運用を維持しつつ CI の安全性を底上げする。

## Gap summary

1. **Security coverage gaps (resolved in this task):**
   - CodeQL absent → added `codeql.yml`.
   - Dependency Review absent → added `dependency-review.yml`.
   - Secret leak scanning policy was implicit only → added `secret-scan.yml`.
2. **Docker/Compose smoke gap (resolved in this task):**
   - CI lacked compose build smoke → added `docker-compose-smoke` job in `ci.yml`.
3. **Operational quality gaps (remaining, follow-up recommended):**
   - Contracts breaking-change explicit detector is not yet introduced.
   - Backend/frontend lint/typecheck stages are still partial.
   - Nightly general smoke (non-security) remains absent.

---

## Implemented minimal additions/modifications

### 1) `.github/workflows/ci.yml` (existing workflow extension)
- **Purpose:** Keep one baseline CI SSOT while reducing waste and improving safety.
- **Changes:**
  - Add `permissions: contents: read`.
  - Add `concurrency` per workflow/ref.
  - Add `paths` filters for PR/push to avoid irrelevant runs.
  - Add `docker-compose-smoke` job (`docker compose config` + `docker compose build`).
- **Why minimal:** no existing job behavior removed; only protective controls and smoke validation added.

### 2) `.github/workflows/codeql.yml` (new)
- **Purpose:** mandatory static security analysis baseline.
- **Trigger:** `pull_request`, `push(main)`, `schedule`.
- **Target paths:** `services/**`, `apps/**`, `sdk/**`.
- **Permissions:** `actions: read`, `contents: read`, `security-events: write` (minimum needed).

### 3) `.github/workflows/dependency-review.yml` (new)
- **Purpose:** PR dependency risk gate.
- **Trigger:** PR only, manifest/lockfile path changes.
- **Permissions:** `contents: read`.

### 4) `.github/workflows/secret-scan.yml` (new)
- **Purpose:** leak detection baseline for secrets.
- **Trigger:** PR, `push(main)`, schedule.
- **Permissions:** `contents: read`.

---

## LOCK:ci operational safety

- Single scope maintained: this task modifies only CI workflows + audit docs.
- Lock discipline:
  - Preflight checked `open_prs`/`locks_held` first.
  - Task assumes exclusive `LOCK:ci` and avoids touching runtime app/service code.
- Rollback:
  - Revert workflow commits if CI regression occurs.
  - Keep docs audit/gap-plan as diagnosis artifact even if workflow rollback is needed.

---

## Integration and SSOT anti-duplication policy

- Prefer extending `ci.yml` for baseline checks instead of spawning multiple overlapping CI files.
- Keep new workflows purpose-specific (`codeql`, `dependency-review`, `secret-scan`) to avoid rule overlap.
- Avoid duplicated triggers by constraining with `paths` where feasible.

---

## Follow-up proposals (separate PRs)

1. Add explicit contracts breaking-change gate (e.g., OpenAPI diff against base branch) as a dedicated contracts job.
2. Add backend/frontend lint/typecheck jobs where toolchains are already present.
3. Add nightly lightweight end-to-end smoke beyond security-only schedule.
4. Define required checks list in runbook (workflow/job names) for merge governance.

