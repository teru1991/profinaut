#!/usr/bin/env bash
set -euo pipefail

CONTROLPLANE_BASE_URL="${CONTROLPLANE_BASE_URL:-http://127.0.0.1:8000}"
EXECUTION_BASE_URL="${EXECUTION_BASE_URL:-http://127.0.0.1:8002}"
MARKETDATA_BASE_URL="${MARKETDATA_BASE_URL:-http://127.0.0.1:8001}"
SMOKE_TIMEOUT_SECONDS="${SMOKE_TIMEOUT_SECONDS:-5}"
SYMBOL="${SMOKE_MARKET_SYMBOL:-BTC_JPY}"

fail() {
  echo "[smoke][fail] $1" >&2
  exit 1
}

step() {
  echo "[smoke][step] $1"
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

need_cmd curl
need_cmd python

step "Health checks"
curl -fsS -m "$SMOKE_TIMEOUT_SECONDS" "$CONTROLPLANE_BASE_URL/healthz" >/dev/null || fail "control-plane /healthz unreachable: $CONTROLPLANE_BASE_URL/healthz"
curl -fsS -m "$SMOKE_TIMEOUT_SECONDS" "$EXECUTION_BASE_URL/healthz" >/dev/null || fail "execution /healthz unreachable: $EXECUTION_BASE_URL/healthz"
curl -fsS -m "$SMOKE_TIMEOUT_SECONDS" "$MARKETDATA_BASE_URL/healthz" >/dev/null || fail "marketdata /healthz unreachable: $MARKETDATA_BASE_URL/healthz"
echo "[smoke][ok] health endpoints are reachable"

step "Ticker latest"
ticker_payload="$(curl -sS -m "$SMOKE_TIMEOUT_SECONDS" "$MARKETDATA_BASE_URL/ticker/latest?symbol=$SYMBOL")" || fail "ticker request failed"
python - <<'PY' "$ticker_payload" || fail "ticker validation failed (expected symbol/price fields and non-degraded result)"
import json,sys
p=json.loads(sys.argv[1])
required=["symbol","bid","ask","last"]
missing=[k for k in required if k not in p]
if missing:
    raise SystemExit(f"ticker missing fields: {missing}")
if p.get("degraded_reason"):
    raise SystemExit(f"ticker degraded: {p.get('degraded_reason')}")
if p.get("stale") is True:
    raise SystemExit("ticker stale")
print(f"[smoke][ok] ticker symbol={p.get('symbol')} last={p.get('last')}")
PY

step "Create order intent"
idempotency_key="smoke-$(date +%s)"
order_payload="{\"idempotency_key\":\"$idempotency_key\",\"exchange\":\"binance\",\"symbol\":\"BTC/USDT\",\"side\":\"BUY\",\"qty\":0.001,\"type\":\"MARKET\"}"
order_response="$(curl -sS -m "$SMOKE_TIMEOUT_SECONDS" -X POST "$EXECUTION_BASE_URL/execution/order-intents" -H 'content-type: application/json' -d "$order_payload")" || fail "order intent request failed"

order_id="$(python - <<'PY' "$order_response"
import json,sys
p=json.loads(sys.argv[1])
print(p.get("order_id", ""))
PY
)"

[[ -n "$order_id" ]] || fail "order intent did not return order_id. Response: $order_response"
echo "[smoke][ok] order created order_id=$order_id"

step "Fill order"
fill_response="$(curl -sS -m "$SMOKE_TIMEOUT_SECONDS" -X POST "$EXECUTION_BASE_URL/execution/orders/$order_id/fill")" || fail "fill request failed for order_id=$order_id"
python - <<'PY' "$fill_response" || fail "fill validation failed (expected status=FILLED)"
import json,sys
p=json.loads(sys.argv[1])
if p.get("status") != "FILLED":
    raise SystemExit(f"unexpected status: {p.get('status')}")
print(f"[smoke][ok] order filled status={p.get('status')} filled_qty={p.get('filled_qty')}")
PY

echo "[smoke][success] paper e2e smoke completed"
