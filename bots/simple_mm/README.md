# simple_mm bot

Minimal E2E bot for **ticker → paper order intent → order/fills log**.

## One-command run

```bash
python bots/simple_mm/main.py
```

## Required services

- Controlplane: `GET /capabilities`
- Marketdata: `GET /ticker/latest`
- Execution: `GET /capabilities`, `POST /execution/order-intents`

## Safety rule

When either condition is true, this bot **never** submits a new order intent:

- `SAFE_MODE=1`
- controlplane capabilities is unreachable
- controlplane capabilities has `status=degraded`
- marketdata ticker has `degraded=true`
- execution capabilities has `status=degraded`

## Optional env

- `BOT_ID` (default: `simple-mm`)
- `SAFE_MODE` (`0/1`, default: `0`)
- `MARKETDATA_BASE_URL` (default: `http://127.0.0.1:8081`)
- `CONTROL_PLANE_BASE_URL` (preferred; default: `http://127.0.0.1:8000`)
- `CONTROLPLANE_BASE_URL` (legacy fallback)
- `EXECUTION_BASE_URL` (default: `http://127.0.0.1:8001`)
- `MARKETDATA_EXCHANGE` (default: `gmo`)
- `MARKETDATA_SYMBOL` (default: `BTC_JPY`)
- `ORDER_EXCHANGE` (default: same as `MARKETDATA_EXCHANGE`)
- `ORDER_SYMBOL` (default: same as `MARKETDATA_SYMBOL`)
- `ORDER_SIDE` (default: `BUY`)
- `ORDER_QTY` (default: `0.001`)
- `COMMAND_POLL_INTERVAL_SEC` (default: `2`)

Logs are JSON lines and include `run_id`, `bot_id`, `state`, `decision` (and `order_id` on `order_result`).


## Command polling

The bot polls control-plane commands via:

- `GET /commands?target_bot_id=<BOT_ID>&status=pending`

Supported commands:

- `PAUSE`: stop placing new orders (logs `reason=PAUSED`)
- `RESUME`: resume order placement when not blocked by SAFE_MODE/degraded gates

Each processed command is acked back to control plane:

- `POST /commands/{id}/ack` with `{ ok, reason, ts }`
