# Data Platform How-to (DP-DB-005-FINAL)

## Quickstart (single command E2E)
```bash
python -m services.marketdata.app.cli dataplat-e2e --seed 11 --rate 50 --duration 5
```

Expected tail:
- JSON summary (includes `deterministic_digest`)
- `PASS`

## Determinism rule
Run the same command twice with the same seed; `deterministic_digest` must match.

## Workflow
1. **Mock feed → Bronze**: harness generates deterministic trade/ticker/orderbook events and injects via raw ingest.
2. **Bronze → Silver**: normalizer writes trades/BBA/OHLCV/events.
3. **Silver → Gold**: `materialize_gold` runs.
4. **Gold → Serving/API**: `/markets/*` endpoints are queried and latency sampled.
5. **Hardening**: restart, object-store-down, clickhouse-down, valkey-down simulations are validated.

## Perf smoke knobs
- `--rate`: target events per second.
- `--duration`: sustained seconds.

Recommended smoke:
```bash
python -m services.marketdata.app.cli dataplat-e2e --seed 17 --rate 150 --duration 20
```
Check:
- `throughput_eps`
- `bronze_p95_ms`
- `api_hit_p95_ms`
- `queue_depth_stable=true`

## Operator rules
- Keep `seed` fixed for regression comparisons.
- Treat `objectstore_degraded=false` during failure simulation as test failure.
- Treat `clickhouse_degraded_safe=false` or `valkey_degraded_safe=false` as serving-hardening failure.
