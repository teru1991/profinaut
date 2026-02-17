# MarketData Replay CLI verification (DP-027A)

## Command

```bash
PYTHONPATH=/workspace/profinaut REPLAY_NOW_TS=2026-02-16T00:59:59Z \
python -m services.marketdata.app.cli replay \
  --from-ts 2026-02-16T00:00:00Z \
  --to-ts 2026-02-16T01:00:00Z \
  --db-dsn sqlite:////tmp/marketdata-replay-verify.sqlite3 \
  --bronze-root /tmp/marketdata-replay-verify-bronze \
  --venue gmo \
  --market spot \
  --write
```

## Automated evidence

```bash
scripts/verification/marketdata_replay_verify.sh
```

Expected output includes:

- replay JSON summary with `read_count`, `silver_count`, `events_count`
- `Replay verification passed: silver/event row count increased (...)`
