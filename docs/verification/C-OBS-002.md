# C-OBS-002 Verification

## 1) Changed files

```
docs/contracts/observability/log_event.schema.json
docs/specs/observability/README.md
docs/status/trace-index.json
docs/verification/C-OBS-002.md
libs/observability/__init__.py
libs/observability/core.py
libs/observability/logging.py
libs/observability/middleware.py
scripts/ci/observability_logging_check.py
services/dashboard-api/app/main.py
services/execution/app/main.py
services/marketdata/app/main.py
tests/test_observability_log_contract_schema.py
tests/test_observability_logging_required_keys.py
tests/test_observability_middleware_injects_headers_and_logs.py
```

## 2) What / Why

- Added C-2 structured log schema (`obs.log_event.v1`) and SSOT README updates to define required keys and strict behavior.
- Added centralized logging helpers (`build_log_event`, `validate_required_keys`, `redact_fields`, `emit_json`) with strict mode (`PROFINAUT_OBS_LOG_STRICT=1`) and forbidden key masking from `docs/policy/forbidden_keys.toml`.
- Added `ObservabilityMiddleware` to generate request correlation, inject response correlation headers, emit one JSON request summary log, and clear context each request.
- Integrated `audit_event` in `libs/observability/core.py` with contract-based JSON emission while preserving existing function signatures and call sites.
- Added schema/strict/middleware tests and CI helper script to make contract regressions fail fast.

## 3) Self-check results

- Allowed-path check: **OK** (`git diff --name-only` + allowlist awk check yielded empty output).
- Tests:
  - `PROFINAUT_OBS_LOG_STRICT=1 pytest -q tests/test_observability_log_contract_schema.py tests/test_observability_logging_required_keys.py tests/test_observability_middleware_injects_headers_and_logs.py tests/test_observability_contract_*.py` => **PASS (14 passed)**.
  - `python scripts/ci/observability_logging_check.py` => **PASS (6 passed)**.
  - `python scripts/ci/observability_contract_check.py` => **PASS (8 passed)**.
  - `PROFINAUT_OBS_LOG_STRICT=1 pytest -q` => **FAIL** due to pre-existing repository baseline issues outside this task (`worker` import, missing `cryptography`, unrelated syntax error in `dashboard_api/main.py`).
- `python -m json.tool docs/status/trace-index.json > /dev/null` => **OK**.
- Secrets scan:
  - heuristic full-file scan flagged legacy `api_secret` usage in existing execution code (not added by this task).
  - added-lines-only scan (`git diff --unified=0` on `+` lines) found **no new secret-like strings**.

## 4) History review evidence (required)

### Commands executed

- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git show HEAD`
- `git log -n 1 -- libs/observability/core.py`
- `git log -n 1 -- libs/observability/correlation.py`
- `git log -n 1 -- libs/observability/http_contracts.py`
- `git log -n 1 -- services/marketdata/app/main.py`
- `git log -n 1 -- services/execution/app/main.py`
- `git log -n 1 -- services/dashboard-api/app/main.py`
- `git blame -w libs/observability/core.py`
- `git blame -w services/marketdata/app/main.py`
- `git reflog -n 30`
- `git merge-base HEAD work`
- `git branch -vv`
- `git log --merges --oneline -n 30`

### Findings / constraints

- Pre-task tip was `c7f1679` (C-OBS-001), and target observability files were last touched in that commit.
- Existing logging mode was mixed (JSON from `libs/observability/core.py` plus text logger formatters in service modules), so required key enforcement was still missing in practice.
- Blame/commit history showed no strict key gate implementation and no immediate revert in this exact scope; therefore this task added strict checks and middleware-level invariant points with minimal localized diffs.
- Conflict minimization applied: only added middleware blocks near existing `app.add_middleware(...)`, and kept core function signatures unchanged.

## 5) Additional concrete implementation due to identified gaps

- Added middleware-wide correlation propagation and request JSON emission to cover all endpoints uniformly.
- Added strict-mode regression tests that fail on required-key omissions.
- Added forbidden-key masking behavior and corresponding regression test (`token` -> `***`).
