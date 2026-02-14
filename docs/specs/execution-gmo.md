# Execution GMO Live (Minimal)

## Scope (W2-060)

This spec defines the minimum live execution behavior for GMO:

- place order (`POST /execution/order-intents`)
- cancel order (`POST /execution/orders/{order_id}/cancel`)
- map `idempotency_key -> client_order_id` for live requests
- gate live execution behind explicit `EXECUTION_LIVE_ENABLED=true`
- keep API keys out of control-plane models/storage
- degrade + backoff when GMO returns `429` or when request timeout occurs

## Safety Defaults

- Live execution is **disabled by default**.
- When disabled, GMO order requests are rejected.
- API keys are read from runtime environment only:
  - `GMO_API_KEY`
  - `GMO_API_SECRET`

## Degrade / Backoff

- On GMO `429` or timeout, service enters degraded live mode.
- During backoff window (`LIVE_BACKOFF_SECONDS`, default `30`), new live requests are rejected.
- Capabilities endpoint reports degraded status while backoff is active.
