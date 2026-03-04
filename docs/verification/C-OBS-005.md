# C-OBS-005 Verification

## 1) Changed files

```
docs/policy/observability_budget.toml
docs/specs/observability/cardinality_budget.md
docs/status/trace-index.json
docs/verification/C-OBS-005.md
libs/observability/audit.py
libs/observability/budget.py
libs/observability/cardinality.py
libs/observability/http_contracts.py
libs/observability/logging.py
libs/observability/metrics.py
libs/observability/middleware.py
scripts/ci/observability_budget_lint.py
tests/test_observability_budget_state_and_healthz.py
tests/test_observability_cardinality_guard.py
tests/test_observability_metrics_drops_on_budget.py
```

## 2) What / Why

- Added observability budget SSOT (`docs/policy/observability_budget.toml`) and C-17/C-16 behavior spec (`docs/specs/observability/cardinality_budget.md`).
- Added runtime budget/cardinality modules (`budget.py`, `cardinality.py`) and metrics integration (`metrics.py`) to pre-check and suppress cardinality explosions.
- Added audit path (`audit.py`) for `cardinality_violation` and `budget_exceeded` events with simple rate limiting.
- Extended logging runtime guard (`logging.py`) for field-count, event-size, and unique-field-key budget constraints with `truncate/drop` behavior and budget-state updates.
- Extended healthz/capabilities builders (`http_contracts.py`) so budget exceed always appears as degraded observability state.
- Hooked request middleware (`middleware.py`) into metric observation path to ensure runtime guard is exercised on all HTTP traffic.
- Added CI lint (`scripts/ci/observability_budget_lint.py`) and regression tests for guard/drop/degraded behavior.

## 3) Self-check results

- Allowed-path check: **OK** (only `docs/`, `libs/`, `tests/`, `scripts/` changed).
- Tests:
  - `PROFINAUT_OBS_BUDGET_STRICT=1 pytest -q tests/test_observability_cardinality_guard.py tests/test_observability_budget_state_and_healthz.py tests/test_observability_metrics_drops_on_budget.py tests/test_observability_contract_*.py tests/test_observability_log_contract_schema.py tests/test_observability_logging_required_keys.py tests/test_observability_middleware_injects_headers_and_logs.py` => **PASS (19 passed)**.
  - `python scripts/ci/observability_budget_lint.py` => **PASS** (includes strict pytest subset).
  - `pytest -q` => **FAIL (pre-existing unrelated baseline issues: `worker` import, missing `cryptography`, unrelated syntax error in `dashboard_api/main.py`)**.
- trace-index check:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => **OK**.
- secrets scan:
  - added-lines scan did not detect new secret-like strings.

## 4) History review evidence (required)

### Commands executed

- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git show HEAD`
- `git log -n 1 -- libs/observability/metrics.py`
- `git log -n 1 -- libs/observability/metrics_guard.py`
- `git log -n 1 -- libs/observability/logging.py`
- `git log -n 1 -- docs/policy/observability_slo_alerts.toml`
- `git blame -w libs/observability/metrics_guard.py`
- `git blame -w libs/observability/logging.py`
- `git reflog -n 30`
- `git merge-base HEAD work`
- `git branch -vv`
- `git log --merges --oneline -n 30`

### Findings / conclusions

- Pre-task base commit was `06bd63f` (`Introduce observability contracts...`).
- `libs/observability/metrics.py` and `libs/observability/metrics_guard.py` did not exist in base, so there was no prior runtime cardinality enforcement path.
- Existing forbidden key enforcement existed in logging redaction (`forbidden_keys.toml` + `logging.py`), but no prior budget/cardinality suppression for metrics/log field cardinality.
- No direct revert signal found around this observability area in recent merge history (`#443/#442/#441`); therefore added guard logic via new modules + minimal integration points.
- Existing budget SSOT for observability-specific cardinality was absent; this task adds dedicated policy and lint.

## 5) Mandatory gap completion

- The provided template noted an `os` import omission in budget implementation; this task includes `import os` in `libs/observability/budget.py` and verifies strict-mode paths in tests/lint.
- Services already route healthz/capabilities through `libs/observability/http_contracts.py` from the previous observability work, so budget degradation injection in `http_contracts.py` now applies uniformly without broad service refactor.
