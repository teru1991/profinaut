# Paper E2E verification runbook

This runbook validates the paper trading flow end-to-end, including MarketData GMO polling behavior and execution idempotency/lifecycle transitions.

## Prerequisites

- Python dependencies installed for both services.
- `ALLOWED_SYMBOLS`/`ALLOWED_EXCHANGES` configured for execution service (defaults in tests are enough for `BTC/USDT` on `binance`).

## One-command smoke run

Use the existing smoke script:

```bash
bash scripts/smoke/run_paper_e2e.sh
```

Expected result:

- script exits with status `0`
- `/healthz` checks succeed
- order creation succeeds once and duplicate idempotency attempt is rejected

If this fails, use the manual verification steps below to isolate MarketData vs execution issues.

## Minimal-step manual verification

### 1) Start services

Terminal A (MarketData):

```bash
uvicorn services.marketdata.app.main:app --host 0.0.0.0 --port 8001
```

Terminal B (Execution):

```bash
ALLOWED_SYMBOLS=BTC/USDT,ETH/USDT ALLOWED_EXCHANGES=binance,coinbase uvicorn services.execution.app.main:app --host 0.0.0.0 --port 8002
```

### 2) Verify MarketData health/capabilities/latest

```bash
curl -s http://localhost:8001/healthz
curl -s http://localhost:8001/capabilities
curl -s "http://localhost:8001/ticker/latest?symbol=BTC_JPY"
```

Expected output patterns:

- `healthz`: `{"status":"ok"}`
- `capabilities`: includes `service=marketdata` and status `ok|degraded`
- `ticker/latest`: stable shape including keys:
  - `symbol, ts, bid, ask, last, mid, source, quality, stale, degraded_reason`
- If GMO is unreachable, endpoint returns degraded structured payload (`error.code=TICKER_NOT_READY`, `degraded_reason=UPSTREAM_ERROR`) instead of crashing.

### 3) Create paper order (accepted)

```bash
curl -s -X POST http://localhost:8002/execution/order-intents \
  -H 'content-type: application/json' \
  -d '{"idempotency_key":"rbk-1","exchange":"binance","symbol":"BTC/USDT","side":"BUY","qty":0.01,"type":"MARKET"}'
```

Expected:

- HTTP `201`
- response contains `order_id`
- `status` is `ACCEPTED`

### 4) Verify idempotency duplicate prevention

Run the same request again with the same `idempotency_key` (`rbk-1`).

Expected:

- HTTP `409`
- duplicate creation prevented

### 5) Validate lifecycle transitions

Fill flow:

```bash
curl -s -X POST http://localhost:8002/execution/orders/<ORDER_ID>/fill
```

Expected:

- HTTP `200`
- `status=FILLED`
- `filled_qty` equals requested `qty`

Reject flow (on a fresh accepted order):

```bash
curl -s -X POST http://localhost:8002/execution/orders/<ORDER_ID>/reject
```

Expected:

- HTTP `200`
- `status=REJECTED`

Cancel flow (on a fresh accepted order):

```bash
curl -s -X POST http://localhost:8002/execution/orders/<ORDER_ID>/cancel
```

Expected:

- HTTP `200`
- `status=CANCELED`

## Troubleshooting

- **MarketData returns stale/degraded**:
  - Check network reachability to GMO public API.
  - Review MarketData logs for `gmo_poll_failure` and `marketdata_state_transition` events.
  - Adjust `MARKETDATA_*` env vars (poll interval, stale threshold, backoff) if needed.
- **Execution rejects symbols/exchanges**:
  - Ensure `ALLOWED_SYMBOLS` and `ALLOWED_EXCHANGES` include your payload values.
- **Unexpected 409 on lifecycle endpoint**:
  - Terminal states are guarded; create a new order and retry the desired transition.
