# Data Platform Ready Check (READY FOR WS PARALLEL INGEST)

Mark **ALL** items ✅ before parallel WS collectors begin.

## Functional chain
- [ ] Single command E2E passes: `python -m services.marketdata.app.cli dataplat-e2e --seed 11 --rate 50 --duration 5`
- [ ] Bronze/Silver/Gold row counts are non-zero in summary.
- [ ] Serving API checks pass in summary (`clickhouse_degraded_safe`, `valkey_degraded_safe`).

## Hardening
- [ ] Restart safety: `restart_no_growth=true`.
- [ ] Object store failure safety: `objectstore_degraded=true` and `objectstore_spool_bounded=true`.
- [ ] Full backend outage degrades cleanly: `api_unavailable_status=503`.

## Performance smoke
- [ ] Throughput measured (`throughput_eps > 0`).
- [ ] p95 ingest latency measured (`bronze_p95_ms > 0`).
- [ ] p95 API hit/miss latency measured.
- [ ] Queue remains stable (`queue_depth_stable=true`).

## Determinism
- [ ] Two runs with same seed produce same `deterministic_digest`.

## Runbook completeness
- [ ] `data-platform-howto.md` available.
- [ ] `data-platform-samples.md` available.
- [ ] `data-platform-troubleshooting.md` available.
- [ ] This ready-check file available.
