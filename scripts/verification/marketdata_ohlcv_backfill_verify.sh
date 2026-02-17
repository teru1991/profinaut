#!/usr/bin/env bash
set -euo pipefail

DB_PATH="$(mktemp -u /tmp/marketdata-ohlcv-backfill-XXXXXX.sqlite3)"
STATE_PATH="$(mktemp -u /tmp/marketdata-ohlcv-cursor-XXXXXX.json)"

PYTHONPATH=/workspace/profinaut python - <<PY
import json
import sqlite3
from services.marketdata.app.backfill import BackfillPage, run_backfill_ohlcv

class Pager:
    def __init__(self):
        self.pages = {
            1: [
                {"openTime": "2026-02-16T00:00:00Z", "open": 100, "high": 101, "low": 99, "close": 100.5, "volume": 10},
                {"openTime": "2026-02-16T00:01:00Z", "open": 100.5, "high": 102, "low": 100, "close": 101, "volume": 12},
            ],
            2: [],
        }
    def __call__(self, _symbol, _tf, page):
        candles = self.pages.get(page, [])
        return BackfillPage(candles=candles, has_more=(page == 1))

summary = run_backfill_ohlcv(
    venue="gmo",
    market="spot",
    tf="1m",
    from_ts="2026-02-16T00:00:00Z",
    to_ts="2026-02-16T00:03:00Z",
    db_dsn="sqlite:///${DB_PATH}",
    max_pages_per_run=3,
    symbol="BTC_JPY",
    fetch_page=Pager(),
    cursor_file="${STATE_PATH}",
)
print(json.dumps(summary.__dict__, separators=(",", ":"), ensure_ascii=False))
conn = sqlite3.connect("${DB_PATH}")
count = conn.execute("SELECT COUNT(*) FROM md_ohlcv").fetchone()[0]
if count <= 0:
    raise SystemExit("no md_ohlcv rows written")
print(f"Backfill verification passed: md_ohlcv rows={count}")
PY
