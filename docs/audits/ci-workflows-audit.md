# CI Workflow Audit (TASK: CI-010)

## 0) Docs OS Preflight

- Read in required order:
  1. `docs/SSOT/README_AI.md`
  2. `docs/status/status.json`
  3. `docs/handoff/HANDOFF.json`
  4. `docs/decisions/decisions.md`
  5. `docs/status/trace-index.json`
  6. `docs/runbooks/pr-preflight.md`
- Preflight result: **PASS (after docs normalization)**
  - `status.json` now includes required keys: `base_branch`, `active_task`, `open_prs`, `locks_held`, `next_actions`, `last_updated`.
  - `active_task` alignment confirmed: `CI-010` in both `status.json` and `docs/handoff/HANDOFF.json`.
  - LOCK check: `open_prs` is empty, and no competing PR holds `LOCK:ci`.

---

## 1) Workflow inventory (all `.github/workflows/**`)

| File | `name` | Trigger (`on`) | Jobs summary |
|---|---|---|---|
| `.github/workflows/ci.yml` | `CI` | `push`, `pull_request` with path filters | `contracts` (OpenAPI+schema validation), `dashboard-api-tests`, `sdk-python-tests`, `web-build`, `docker-compose-smoke`, `baseline` |
| `.github/workflows/chaos-injection.yml` | `chaos-injection` | `pull_request`/`push` with paths for chaos tests + `workflow_dispatch` | `chaos-tests` (pytest for `tests/chaos`) |
| `.github/workflows/gemini-review.yml` | `Gemini PR Review (with GitHub Suggestions)` | `issue_comment` (`/gemini review`) | `review` (trusted requester guard, fork guard, secret guard, diff fetch, Gemini API review comment) |
| `.github/workflows/security-supply-chain.yml` | `Security Supply Chain Guardrails` | `pull_request`, `workflow_dispatch` | `supply-chain-guardrails` (SBOM, Trivy scan, artifact guard) |
| `.github/workflows/codeql.yml` | `CodeQL` | `pull_request`, `push(main)`, `schedule` | `analyze` matrix (`python`, `javascript-typescript`) |
| `.github/workflows/dependency-review.yml` | `Dependency Review` | `pull_request` with dependency file path filters | `dependency-review` (`actions/dependency-review-action`) |
| `.github/workflows/secret-scan.yml` | `Secret Scan` | `pull_request`, `push(main)`, `schedule` | `gitleaks` (leak detection) |

---

## 2) Cross-cutting configuration audit

### permissions

- **Explicitly set (good):**
  - `ci.yml`: `contents: read`
  - `gemini-review.yml`: `contents: read`, `pull-requests: write`, `issues: write` (minimum needed for PR comments)
  - `security-supply-chain.yml`: `contents: read`
  - `codeql.yml`: `actions: read`, `contents: read`, `security-events: write` (required for SARIF upload)
  - `dependency-review.yml`: `contents: read`
  - `secret-scan.yml`: `contents: read`
- **Not explicitly set:**
  - `chaos-injection.yml` (uses default token permissions; candidate for future tightening)

### concurrency

- Set: `ci.yml`, `gemini-review.yml`, `codeql.yml`, `dependency-review.yml`, `secret-scan.yml`
- Not set: `chaos-injection.yml`, `security-supply-chain.yml`

### paths / paths-ignore optimization

- Set: `ci.yml`, `chaos-injection.yml`, `dependency-review.yml`, `codeql.yml`
- Not set: `security-supply-chain.yml`, `gemini-review.yml`, `secret-scan.yml` (acceptable depending on intent; security scans may intentionally run broadly)

### Execution conditions

- PR+push baseline CI: `ci.yml`
- PR-only guardrail: `security-supply-chain.yml`, `dependency-review.yml`
- Manual dispatch: `chaos-injection.yml`, `security-supply-chain.yml`
- Scheduled nightly/periodic: `codeql.yml`, `secret-scan.yml`
- Comment-driven review bot: `gemini-review.yml`

### Cache strategy

- No `actions/cache` usage detected.
- Current workflows prioritize simplicity over build acceleration.

### Failure notification

- No explicit Slack/email notification jobs.
- Failures rely on native GitHub Checks status.

---

## 3) Required CI set fit/gap judgment (○/△/×)

| Required set | Status | Evidence |
|---|---|---|
| Contracts SSOT validation (OpenAPI/JSON Schema + breaking-change detection mindset) | △ | `ci.yml` has OpenAPI lint + JSON Schema validation, but no explicit breaking-change gate tool/job. |
| Backend CI (lint/typecheck/test as applicable) | △ | `dashboard-api-tests` and `sdk-python-tests` run tests, but dedicated lint/typecheck jobs are absent. |
| Frontend CI (lint/typecheck/build as applicable) | △ | `web-build` covers build only; no explicit lint/typecheck workflow stage. |
| Docker/Compose smoke (at least build) | ○ | `ci.yml` `docker-compose-smoke` runs `docker compose config` and `docker compose build`. |
| Security: CodeQL | ○ | `codeql.yml` matrix scan for Python and JS/TS. |
| Security: Dependency Review (PR dep change detection) | ○ | `dependency-review.yml` on dependency-manifest path changes. |
| Secrets policy (leak detection/prevention) | ○ | `secret-scan.yml` uses gitleaks on PR/push/schedule; `gemini-review.yml` includes fork/secret guards. |

### Strongly recommended set

| Recommended set | Status | Evidence |
|---|---|---|
| Nightly schedule for build/test/light smoke | △ | Schedule exists for security (`codeql`, `secret-scan`) but not for general build/test smoke. |
| Path-based optimization | ○ | Present in core `ci.yml` and targeted workflows. |
| PR template/required checks operational alignment in runbooks | △ | `docs/runbooks/pr-preflight.md` has generic checklist, but no explicit required-check list tied to current workflow names. |


---

## 4) CI-010-B follow-up hardening (ci-security-guardrails-tuning)

### Implemented changes

1. Added `.github/dependency-review.yml` and wired `actions/dependency-review-action` to use `with: config-file: .github/dependency-review.yml`.
   - Policy now enforces `fail-on-severity: high` for PR dependency changes.
2. Updated `security-supply-chain.yml` trigger/behavior split:
   - `pull_request`: Trivy runs with `continue-on-error: true` (warn-only behavior).
   - `schedule` and `workflow_dispatch`: Trivy remains blocking (`exit-code: '1'`) for HIGH/CRITICAL findings.
3. Hardened `scripts/security/artifact_guard.sh` for zero-diff stability:
   - If no changed files are detected, it exits successfully with an explicit pass message.

### Verification notes

- Local static validation confirmed workflow branches and config linkage are present.
- PR runtime validation target:
  - `Dependency Review` should fail when dependency updates include HIGH+ severity advisories.
  - `Security Supply Chain Guardrails` should not fail entire PR checks solely due to Trivy findings.
- Schedule/runtime validation target:
  - Trigger via `workflow_dispatch` as schedule proxy; blocking Trivy path (`if: github.event_name != 'pull_request'`) must fail on HIGH/CRITICAL.

### Rollback ready

- Revert `security-supply-chain.yml` to single Trivy step without `continue-on-error` for PR.
- Remove `.github/dependency-review.yml` and `config-file` binding from dependency-review workflow.
