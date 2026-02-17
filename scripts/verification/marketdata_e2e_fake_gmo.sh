#!/usr/bin/env bash
set -euo pipefail

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "missing command: $1" >&2; exit 2; }; }
need_cmd python
need_cmd curl
need_cmd sqlite3

FAKE_PORT="${FAKE_PORT:-19091}"
SVC_PORT="${SVC_PORT:-18081}"
DB_PATH="${DB_PATH:-/tmp/marketdata-e2e-fake-gmo.sqlite3}"
BRONZE_ROOT="${BRONZE_ROOT:-/tmp/marketdata-e2e-fake-gmo-bronze}"

rm -f "$DB_PATH"
rm -rf "$BRONZE_ROOT"

cleanup() {
  set +e
  [[ -n "${SVC_PID:-}" ]] && kill "$SVC_PID" >/dev/null 2>&1
  [[ -n "${FAKE_PID:-}" ]] && kill "$FAKE_PID" >/dev/null 2>&1
}
trap cleanup EXIT

PYTHONPATH=/workspace/profinaut python services/marketdata/tests/fake_gmo_server.py --port "$FAKE_PORT" &
FAKE_PID=$!

for _ in $(seq 1 50); do
  if curl -fsS "http://127.0.0.1:${FAKE_PORT}/public/v1/ticker?symbol=BTC_JPY" >/dev/null 2>/dev/null; then
    break
  fi
  sleep 0.1
done

PYTHONPATH=/workspace/profinaut \
OBJECT_STORE_BACKEND=fs \
DB_DSN="sqlite:///$DB_PATH" \
BRONZE_FS_ROOT="$BRONZE_ROOT" \
SILVER_ENABLED=1 \
GMO_REST_ENABLED=0 \
GMO_WS_ENABLED=1 \
GMO_WS_URL="ws://127.0.0.1:${FAKE_PORT}/ws/public/v1" \
GMO_MARKETDATA_BASE_URL="http://127.0.0.1:${FAKE_PORT}/public/v1" \
MARKETDATA_SYMBOL=BTC_JPY \
MARKETDATA_MARKET_ID=spot \
GMO_WS_CHANNELS=ticker,trades,orderbooks \
python -m uvicorn services.marketdata.app.main:app --host 127.0.0.1 --port "$SVC_PORT" --log-level warning &
SVC_PID=$!

for _ in $(seq 1 80); do
  if curl -fsS "http://127.0.0.1:${SVC_PORT}/healthz" >/dev/null 2>/dev/null; then
    break
  fi
  sleep 0.1
done

found=0
for _ in $(seq 1 80); do
  body="$(curl -fsS "http://127.0.0.1:${SVC_PORT}/orderbook/bbo/latest?venue_id=gmo&market_id=spot" || true)"
  if [[ "$body" == *'"found":true'* && "$body" == *'"bid"'* && "$body" == *'"ask"'* ]]; then
    found=1
    echo "bbo_latest=$body"
    break
  fi
  sleep 0.1
done

if [[ "$found" -ne 1 ]]; then
  echo "E2E failed: expected found=true bbo response" >&2
  exit 1
fi

raw_count="$(sqlite3 "$DB_PATH" 'select count(*) from raw_ingest_meta;')"
if [[ "$raw_count" -lt 1 ]]; then
  echo "E2E failed: expected raw_ingest_meta rows >= 1" >&2
  exit 1
fi

echo "raw_ingest_meta_count=$raw_count"
echo "E2E fake GMO PASS"
