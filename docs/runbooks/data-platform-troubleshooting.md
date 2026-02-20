# Data Platform Troubleshooting

## 1) E2E prints FAIL
Run:
```bash
python -m services.marketdata.app.cli dataplat-e2e --seed 11 --rate 50 --duration 5
```
Inspect summary booleans:
- `restart_no_growth`
- `objectstore_degraded`
- `objectstore_spool_bounded`
- `clickhouse_degraded_safe`
- `valkey_degraded_safe`
- `queue_depth_stable`

## 2) Restart safety regression
Symptom: `restart_no_growth=false`.
- Cause: idempotency DB reset/wrong path.
- Fix: ensure `BRONZE_IDEMPOTENCY_DB` is persistent across writer restart.

## 3) Object store failure unsafe
Symptom: `objectstore_degraded=false` or `objectstore_spool_bounded=false`.
- Cause: writer health not entering degraded mode or spool cap misconfigured.
- Fix: verify bronze writer health and spool cap (`spool_bytes` must stay bounded).

## 4) ClickHouse down causes API outage
Symptom: `clickhouse_degraded_safe=false`.
- Expected behavior: API returns from sqlite gold fallback.
- Fix: verify read fallback path in `/markets/ticker/latest` and `/markets/bba/latest`.

## 5) Valkey down causes API outage
Symptom: `valkey_degraded_safe=false`.
- Expected behavior: cache failure triggers direct backend read, not 503.
- Fix: validate cache-exception fallback branch in serving endpoints.

## 6) Perf smoke unstable queue
Symptom: `queue_depth_stable=false`.
- Cause: ingest rate too high or backend writes stalling.
- Fix: lower `--rate` to baseline, then increase gradually and record stable boundary.
