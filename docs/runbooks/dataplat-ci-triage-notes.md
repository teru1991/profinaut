# dataplat CI triage notes (FIX-DATAPLATFORM-DB-ERRATA-001)

## failing workflow names
- GitHub Checks data for DP-DB-001..005 PRs was not available in this environment.
- Local CI-equivalent triage was executed against `services/marketdata/**`, `infra/**`, and `docker-compose.yml`.

## failing jobs/steps (local reproduction)
- `pytest -q services/marketdata/tests` collection failures when `PYTHONPATH` is not set (`ModuleNotFoundError: No module named 'services'`).
- `PYTHONPATH=. pytest -q services/marketdata/tests` logic failures:
  - `/healthz` and `/capabilities` response-contract mismatches.
  - quality-gate counters not incrementing expected aggregate metric.
  - trade dedupe key not stable/populated, causing duplicate inserts.
  - orderbook state contamination across test runs causing false gap/degraded reason.
  - session recorder test expected row access by column-name, connection returned tuple rows.
  - UCEL GMO compatibility test expected `process_orderbook_delta` and adapter metrics/state shim.

## first error lines + stack traces
- `E   ModuleNotFoundError: No module named 'services'`
- `E   KeyError: 'ingest_raw_enabled'`
- `E   AssertionError: assert 2 == 1` (trade dedupe)
- `E   AttributeError: 'GmoPublicMarketDataAdapter' object has no attribute 'process_orderbook_delta'`
- `E   TypeError: tuple indices must be integers or slices, not str`

## reproduction commands used locally
- `pytest -q services/marketdata/tests`
- `PYTHONPATH=. pytest -q services/marketdata/tests`
- `PYTHONPATH=. pytest -q services/marketdata/tests/test_quality_gates.py::test_orderbook_crossed_marks_degraded_data_invalid`
- `PYTHONPATH=. pytest -q services/marketdata/tests/test_trade_dedup.py`
- `PYTHONPATH=. pytest -q services/marketdata/tests/test_ucel_testkit_dod_gates_v114.py::test_orderbook_gap_to_resync_gate`
- `docker compose config`

## fixes applied
- Restored backward-compatible health/capabilities fields and dependency-state logic.
- Added DB DSN compatibility (`DB_DSN` fallback).
- Added stable trade source message keys for dedupe when `source_msg_key` is absent.
- Counted quality-gate anomalies in normalization aggregate metrics.
- Scoped in-memory orderbook state by repository instance to avoid cross-run contamination.
- Enabled sqlite row name access (`sqlite3.Row`) in repository wrapper.
- Added compatibility shim around GMO UCEL adapter (`metrics`, `_orderbook_state`, and `process_orderbook_delta`).

## remaining known risks / follow-up
- Local environment lacks `duckdb`, so full silver iceberg recompute test cannot pass locally without installing runtime dependency.
- Local environment has pre-existing unrelated git modifications outside allowed task scope; these were not altered.
