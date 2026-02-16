# MarketData replay runbook (Bronze -> Silver/events)

Rebuild Silver tables from Bronze JSONL parts for a time window.

## Command

From repository root:

```bash
PYTHONPATH=/workspace/profinaut python -m services.marketdata.app.replay \
  --from 2026-02-16T00:00:00Z \
  --to 2026-02-16T01:00:00Z \
  --db-dsn sqlite:///./data/marketdata/replay.sqlite3 \
  --bronze-root ./data/bronze \
  --parser-version v0.2
```

Optional filters:

```bash
  --venue gmo \
  --source_type WS_PUBLIC
```

Dry-run (count only, no DB writes):

```bash
PYTHONPATH=/workspace/profinaut python -m services.marketdata.app.replay \
  --from 2026-02-16T00:00:00Z \
  --to 2026-02-16T01:00:00Z \
  --db-dsn sqlite:///./data/marketdata/replay.sqlite3 \
  --bronze-root ./data/bronze \
  --dry_run
```

## Output

The command prints a single JSON summary with:

- `read_count`
- `silver_count`
- `events_count`
- `skipped_count`
- `parser_version`
- filters and window info

## Notes

- Replay is designed to be idempotent-safe for trade/ohlcv where unique constraints + `INSERT OR IGNORE` apply.
- `md_best_bid_ask` and `md_events_json` are append-oriented in v0.1.
- Ensure `--from` / `--to` are RFC3339 UTC timestamps.
