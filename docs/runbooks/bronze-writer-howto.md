# Bronze Writer How-to

## Prerequisites
- Python environment with project dependencies installed.
- Writable local directories for Bronze object files and sqlite metadata.

## Run locally
```bash
export OBJECT_STORE_BACKEND=fs
export DB_DSN=sqlite:///./data/marketdata.sqlite3
export BRONZE_FS_ROOT=./data/bronze
export BRONZE_IDEMPOTENCY_DB=./data/bronze/idempotency.sqlite3
PYTHONPATH=. uvicorn services.marketdata.app.main:app --host 0.0.0.0 --port 8081
```

## Send a sample BronzeRecord
```bash
curl -sS -X POST http://127.0.0.1:8081/raw/ingest \
  -H 'content-type: application/json' \
  -d '{
    "tenant_id": "tenant-a",
    "source_type": "WS_PUBLIC",
    "received_ts": "2026-02-16T00:00:01Z",
    "event_ts": "2026-02-16T00:00:01Z",
    "venue_id": "gmo",
    "market_id": "BTC_JPY",
    "stream_name": "trade",
    "source_event_id": "evt-1",
    "payload_json": {"symbol": "BTC_JPY", "price": "100", "qty": "0.1", "side": "buy"}
  }' | jq
```

## Verify object keys and gzip contents
```bash
find ./data/bronze -name '*.jsonl.gz' -print
python - <<'PY'
import gzip, json
from pathlib import Path
for p in Path('data/bronze').rglob('*.jsonl.gz'):
    print(f'FILE={p}')
    for line in gzip.decompress(p.read_bytes()).decode('utf-8').splitlines():
        if line.strip():
            record=json.loads(line)
            print(record['idempotency_key'], record['meta']['raw_ref'])
PY
```

Expected layout:
- `bronze/<asset>/<venue>/dt=YYYY-MM-DD/hh=HH/part-xxxxx.jsonl.gz`

## Verify idempotency and restart behavior
1. POST the same payload twice with identical `idempotency_key`.
2. Confirm second response contains `object_key: dedupe://dropped`.
3. Restart service and POST same payload again.
4. Confirm it is still dropped.

## Health and metrics
```bash
curl -sS http://127.0.0.1:8081/healthz | jq
curl -sS http://127.0.0.1:8081/metrics
```

Important writer fields:
- `degraded`, `degraded_reason`, `spool_bytes`, `queue_depth`, `circuit_open` in health payload.
- `ingest_total`, `persisted_total`, `reject_secret_total`, `reject_schema_total`, `dedupe_drop_total`, `write_latency_ms`, `spool_bytes`, `queue_depth` in metrics.
