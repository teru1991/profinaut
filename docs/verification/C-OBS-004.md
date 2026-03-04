# C-OBS-004 Verification

## 1) Changed files

```
docs/contracts/observability/metrics_catalog.snapshot.txt
docs/specs/observability/metrics_catalog.md
docs/status/trace-index.json
docs/verification/C-OBS-004.md
libs/observability/metrics.py
libs/observability/metrics_catalog.py
libs/observability/metrics_guard.py
libs/observability/middleware.py
scripts/ci/observability_metrics_check.py
scripts/ci/observability_metrics_snapshot.py
services/dashboard-api/app/main.py
services/execution/app/main.py
services/marketdata/app/main.py
tests/test_observability_metrics_guard_labels.py
tests/test_observability_metrics_required.py
tests/test_observability_metrics_snapshot_gate.py
```

## 2) What / Why

- Added C-3/C-5-3 SSOT metrics catalog documentation and a machine-checked snapshot baseline.
- Implemented reusable observability metrics modules (`metrics_catalog`, `metrics_guard`, `metrics`) to enforce naming/label policy and required metrics.
- Added execution `/metrics` endpoint (previously missing), and aligned marketdata/dashboard-api to expose required shared metrics via common helper.
- Wired request middleware to observe HTTP request counters/histograms in the shared metrics registry.
- Added snapshot gate + tests so catalog-breaking changes fail CI.

## 3) Self-check results

- Allowed-path check: **OK** (no changes outside docs/libs/services/tests/scripts).
- Metrics tests: **OK**
  - `pytest -q tests/test_observability_metrics_required.py tests/test_observability_metrics_guard_labels.py tests/test_observability_metrics_snapshot_gate.py`
- Snapshot gate: **OK**
  - `python scripts/ci/observability_metrics_snapshot.py`
  - `python scripts/ci/observability_metrics_check.py`
- Full repo tests: **FAIL (baseline unrelated issues)**
  - `pytest -q` fails with pre-existing issues (`worker` import missing, `dashboard_api/main.py` syntax error, missing `cryptography`, missing `jsonschema` in environment).
- trace-index JSON validation: **OK**
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- secrets scan: **OK**
  - Added-lines scan for secret-like patterns produced no new leaks.

## 4) History review evidence (required)

### Commands executed

- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git show HEAD`
- `git log -n 1 -- services/marketdata/app/main.py`
- `git log -n 1 -- services/execution/app/main.py`
- `git log -n 1 -- services/dashboard-api/app/main.py`
- `git log -n 1 -- libs/observability/*`
- `git blame -w services/marketdata/app/main.py`
- `git blame -w services/execution/app/main.py`
- `git blame -w services/dashboard-api/app/main.py`
- `git reflog -n 30`
- `git merge-base HEAD work`
- `git branch -vv`
- `git log --merges --oneline -n 30`

### Findings / judgments

- Pre-task tip was `6aadad0` (observability contracts/middleware/logging baseline).
- `/metrics` implementation difference confirmed:
  - marketdata had `/metrics` endpoint.
  - dashboard-api did not expose `/metrics`.
  - execution did not expose `/metrics`.
- `prometheus_client` already exists in runtime dependencies (`pyproject.toml`), so no dependency change was required.
- Recent merge/reflog history shows no explicit `/metrics` revert event in the recent window; to reduce conflict risk this task localized to endpoint additions/delegation and new observability helper modules.

## 5) Additional implementation added after review

- Added snapshot gate script with CI-safe `--accept` behavior (forbidden in CI).
- Added strict label-policy enforcement (allowed+forbidden labels) and regression test for forbidden labels.
- Added middleware-level HTTP metric observation hook to guarantee `profinaut_http_requests_total` and duration histogram are updated across services.
