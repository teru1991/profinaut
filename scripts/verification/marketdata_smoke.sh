#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:18080}"
DB_DSN="${DB_DSN:-sqlite:////tmp/marketdata-smoke.sqlite3}"
BRONZE_FS_ROOT="${BRONZE_FS_ROOT:-/tmp/marketdata-smoke-bronze}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing command: $1" >&2; exit 2; }
}

need_cmd curl
need_cmd python

health_json="$(curl -fsS "$BASE_URL/healthz")"
caps_json="$(curl -fsS "$BASE_URL/capabilities")"

echo "healthz: $health_json"
echo "capabilities: $caps_json"

trade_resp="$(curl -fsS -X POST "$BASE_URL/raw/ingest" -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_trade.json)"
trade_dup_resp="$(curl -fsS -X POST "$BASE_URL/raw/ingest" -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_trade.json)"
unknown_resp="$(curl -fsS -X POST "$BASE_URL/raw/ingest" -H 'Content-Type: application/json' --data @scripts/verification/sample_raw_unknown.json)"

echo "ingest trade: $trade_resp"
echo "ingest trade duplicate: $trade_dup_resp"
echo "ingest unknown: $unknown_resp"

python - <<'PY'
import json
import os
import sqlite3
from pathlib import Path

db_dsn = os.environ.get("DB_DSN", "sqlite:////tmp/marketdata-smoke.sqlite3")
if not db_dsn.startswith("sqlite:///"):
    raise SystemExit("DB_DSN must be sqlite:/// for smoke script")
db_path = db_dsn.removeprefix("sqlite:///")
bronze_root = Path(os.environ.get("BRONZE_FS_ROOT", "/tmp/marketdata-smoke-bronze"))
conn = sqlite3.connect(db_path)

for table in ["raw_ingest_meta", "md_trades", "md_events_json"]:
    c = conn.execute(f"select count(*) from {table}").fetchone()[0]
    print(f"{table}_count={c}")

latest = conn.execute("select raw_msg_id, object_key, payload_hash from raw_ingest_meta order by received_ts desc limit 1").fetchone()
print("latest_meta=", latest)
if latest is None:
    raise SystemExit("no raw_ingest_meta rows")

obj = latest[1]
if obj is None:
    raise SystemExit("object_key missing")
fp = bronze_root / obj
print("bronze_path=", fp)
if not fp.exists():
    raise SystemExit(f"bronze object missing: {fp}")
PY

ticker_json="$(curl -fsS "$BASE_URL/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy")"
ohlcv_json="$(curl -sS "$BASE_URL/ohlcv/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy&timeframe=1m")"

echo "ticker latest: $ticker_json"
echo "ohlcv latest: $ohlcv_json"

echo "smoke script completed"
