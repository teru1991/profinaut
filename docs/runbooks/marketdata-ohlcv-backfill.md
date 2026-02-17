# MarketData OHLCV backfill runbook (DP-032A)

## Purpose
Backfill historical OHLCV candles from GMO REST into `md_ohlcv` with bounded pages and resumable cursor.

## Command

```bash
PYTHONPATH=/workspace/profinaut python -m services.marketdata.app.cli backfill ohlcv \
  --venue gmo \
  --market spot \
  --tf 1m \
  --from 2026-02-16T00:00:00Z \
  --to 2026-02-16T06:00:00Z \
  --db-dsn sqlite:////tmp/marketdata-backfill.sqlite3 \
  --max-pages-per-run 5
```

## Safety / bounded execution
- Bounded by `--max-pages-per-run` each invocation.
- Automatic retry/backoff is applied for upstream `429` and transient URL errors.

## Resume behavior
- Cursor is persisted to `OHLCV_BACKFILL_CURSOR_FILE` (default: `services/marketdata/.state/ohlcv_backfill_cursor.json`).
- Re-run the same command to resume from the saved page.
- On completion, cursor entry for the exact `(venue, market, tf, from, to)` window is removed.

## Idempotency
- `md_ohlcv` has unique key `(venue_id, market_id, instrument_id, timeframe, open_ts)`.
- Re-running the same window will not duplicate existing candles.

## Verification

```bash
scripts/verification/marketdata_ohlcv_backfill_verify.sh
```

Expected tail output:

```text
verify=PASS
```
