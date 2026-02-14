# simple_mm spec

- Path: `bots/simple_mm/main.py`
- Goal: shortest E2E verification for `ticker -> paper order intent -> order/fills log`
- Safety invariant: in degraded or SAFE_MODE, no new order is submitted
  - If controlplane is unreachable, block new orders
  - If controlplane capabilities is `status=degraded`, block new orders
  - If ticker is stale, block new orders
  - Unknown exchange/symbol is refused by default
- Logging: lines include `run_id`, `bot_id`, `state`, `decision`, `idempotency_key` (and `order_id` when available)
- One-command run:

```bash
SAFE_MODE=0 \
MARKETDATA_BASE_URL=http://127.0.0.1:8081 \
CONTROLPLANE_BASE_URL=http://127.0.0.1:8000 \
EXECUTION_BASE_URL=http://127.0.0.1:8001 \
python bots/simple_mm/main.py
```

- Optional env:
  - `ALLOWED_EXCHANGES` (default: `gmo`)
  - `ALLOWED_SYMBOLS` (default: `BTC_JPY`)
  - `MAX_TICKER_AGE_SECONDS` (default: `30`)
