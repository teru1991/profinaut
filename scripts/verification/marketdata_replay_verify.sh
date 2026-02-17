#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${DB_DSN:-}" ]]; then
  DB_PATH="$(mktemp -u /tmp/marketdata-replay-verify-XXXXXX.sqlite3)"
  DB_DSN="sqlite:///${DB_PATH}"
else
  DB_DSN="sqlite:///${DB_PATH}"
  [[ "${DB_DSN}" == sqlite:///* ]] || { echo "DB_DSN must be sqlite:///..." >&2; exit 2; }
  DB_PATH="${DB_DSN#sqlite:///}"
fi

BRONZE_ROOT="${BRONZE_ROOT:-/tmp/marketdata-replay-verify-bronze}"
FROM_TS="${FROM_TS:-2026-02-16T00:00:00Z}"
TO_TS="${TO_TS:-2026-02-16T01:00:00Z}"

rm -f "${DB_PATH}"
rm -rf "${BRONZE_ROOT}"

mkdir -p "${BRONZE_ROOT}/bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00"
cat > "${BRONZE_ROOT}/bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00/part-0001.jsonl" <<'JSONL'
{"raw_msg_id":"01ARZ3NDEKTSV4RRFFQ69G5FAV","tenant_id":"tenant-a","source_type":"WS_PUBLIC","venue_id":"gmo","market_id":"spot","received_ts":"2026-02-16T00:10:00Z","payload_json":{"price":100,"qty":1.25,"side":"buy"}}
{"raw_msg_id":"01ARZ3NDEKTSV4RRFFQ69G5FAW","tenant_id":"tenant-a","source_type":"WS_PUBLIC","venue_id":"gmo","market_id":"spot","received_ts":"2026-02-16T00:11:00Z","payload_json":{"mystery":true}}
JSONL

before_count=0

PYTHONPATH=/workspace/profinaut REPLAY_NOW_TS=2026-02-16T00:59:59Z python -m services.marketdata.app.cli replay \
  --from-ts "${FROM_TS}" \
  --to-ts "${TO_TS}" \
  --db-dsn "${DB_DSN}" \
  --bronze-root "${BRONZE_ROOT}" \
  --venue gmo \
  --market spot \
  --write

after_count="$(python - <<PY
import sqlite3
conn = sqlite3.connect("${DB_PATH}")
row = conn.execute("SELECT (SELECT COUNT(*) FROM md_trades) + (SELECT COUNT(*) FROM md_events_json)").fetchone()
print(int(row[0]))
PY
)"

if (( after_count <= before_count )); then
  echo "Replay verification failed: silver/event row count did not increase (${before_count} -> ${after_count})" >&2
  exit 1
fi

echo "Replay verification passed: silver/event row count increased (${before_count} -> ${after_count})"
