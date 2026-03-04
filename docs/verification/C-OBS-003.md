# C-OBS-003 Verification

## 1) Changed files

```
docs/specs/observability/data_classification.md
docs/specs/observability/README.md
docs/policy/redaction.toml
docs/status/trace-index.json
docs/verification/C-OBS-003.md
libs/observability/redaction.py
libs/observability/http_sanitize.py
libs/observability/audit.py
libs/observability/logging.py
services/marketdata/app/main.py
services/execution/app/main.py
services/dashboard-api/app/main.py
tests/test_observability_redaction_policy.py
tests/test_observability_redaction_deep.py
tests/test_observability_healthz_details_sanitized.py
scripts/ci/observability_redaction_lint.py
```

## 2) What / Why

- Added classification SSOT and runtime redaction policy to mechanize C-15/C-0/C-2 expectations for observability outputs.
- Added deep redaction engine with key classification, nested sanitization, value-pattern masking, and max-depth/max-keys guards.
- Integrated redaction into structured logging path so log fields are sanitized before emission and redaction violations are surfaced by reason code/count without exposing values.
- Added healthz/capabilities sanitization hooks and audit event emission path (`audit:redaction_violation`) as degradation foundation when sanitized violations are detected.
- Added CI lint and regression tests to lock policy loading, deep sanitization, and healthz no-leak behavior.

## 3) Self-check results

- Allowed-path check: **OK** (`git diff --name-only` allowlist filter returned empty).
- Test commands:
  - `pytest -q tests/test_observability_redaction_policy.py tests/test_observability_redaction_deep.py tests/test_observability_healthz_details_sanitized.py tests/test_observability_log_contract_schema.py tests/test_observability_logging_required_keys.py tests/test_observability_middleware_injects_headers_and_logs.py tests/test_observability_contract_*.py` => **PASS (19 passed)**.
  - `python scripts/ci/observability_redaction_lint.py` => **PASS**.
  - `python scripts/ci/observability_contract_check.py` => **PASS (8 passed)**.
  - `python scripts/ci/observability_logging_check.py` => **PASS (6 passed)**.
  - `pytest -q` => **FAIL (pre-existing baseline issues outside this task: missing `worker`, missing `cryptography`, existing syntax error in `dashboard_api/main.py`)**.
- `python -m json.tool docs/status/trace-index.json > /dev/null` => **OK**.
- Secrets scan:
  - Added-lines scan on `git diff --unified=0` shows no raw secret-like newly introduced output values.

## 4) History review evidence (required)

### Commands executed

- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git show HEAD`
- `git log -n 1 -- docs/policy/forbidden_keys.toml`
- `git log -n 1 -- libs/observability/logging.py`
- `git log -n 1 -- libs/observability/core.py`
- `git log -n 1 -- services/marketdata/app/main.py`
- `git blame -w docs/policy/forbidden_keys.toml`
- `git blame -w libs/observability/logging.py`
- `git reflog -n 30`
- `git merge-base HEAD work`
- `git branch -vv`
- `git log --merges --oneline -n 30`

### Findings / conclusions

- Branch tip before implementation was `c6d78a1` (previous observability contract/middleware rollout).
- `docs/policy/forbidden_keys.toml` originates from `7f6c2775` and explicitly states consistent application to logs, audit events, support bundle outputs, and crash payloads; this is evidence of prior incident-prevention intent.
- Existing redaction behavior before this task was shallow, key-exact masking in `libs/observability/logging.py::redact_fields` and key-substring masking in `libs/observability/core.py::_redact`, without deep traversal/value pattern checks.
- Existing support bundle design appears in policy/spec references (`docs/policy/retention.toml`, `docs/policy/support_bundle_triggers.toml`, `docs/policy/ucel_golden/support_bundle.toml`), but no Python runtime sanitizer bridge existed here; this task adds reusable redaction primitives to serve that future integration.
- No immediate revert churn in this precise path; minimal localized changes were applied to reduce conflict risks.

## 5) Conflict-minimizing approach

- Redaction logic concentrated in new files (`redaction.py`, `http_sanitize.py`, `audit.py`) and existing modules only touched at call points.
- Service edits kept local to healthz/capabilities post-construction sanitization blocks.
- `docs/status/trace-index.json` updated only for `tasks["C-OBS-003"]`.
