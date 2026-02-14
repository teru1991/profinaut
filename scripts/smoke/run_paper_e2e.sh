#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CONTROLPLANE_BASE_URL="${CONTROLPLANE_BASE_URL:-http://127.0.0.1:8000}"
EXECUTION_BASE_URL="${EXECUTION_BASE_URL:-http://127.0.0.1:8001}"
MARKETDATA_BASE_URL="${MARKETDATA_BASE_URL:-http://127.0.0.1:8081}"
SMOKE_TIMEOUT_SECONDS="${SMOKE_TIMEOUT_SECONDS:-3}"
SMOKE_RETRIES="${SMOKE_RETRIES:-20}"
SMOKE_RETRY_SLEEP_SECONDS="${SMOKE_RETRY_SLEEP_SECONDS:-1}"
SMOKE_AUTO_START="${SMOKE_AUTO_START:-0}"

if [[ "$SMOKE_AUTO_START" == "1" ]]; then
  "$ROOT_DIR/scripts/smoke/start_stack.sh"
fi

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[smoke][error] required command '$1' is missing" >&2
    exit 1
  fi
}

need_cmd curl
need_cmd python

wait_http_ok() {
  local url="$1"
  local name="$2"
  local i
  for ((i=1; i<=SMOKE_RETRIES; i++)); do
    if curl -fsS --max-time "$SMOKE_TIMEOUT_SECONDS" "$url" >/dev/null 2>&1; then
      echo "[smoke][ok] $name reachable: $url"
      return 0
    fi
    sleep "$SMOKE_RETRY_SLEEP_SECONDS"
  done
  echo "[smoke][error] $name is not reachable at $url after ${SMOKE_RETRIES} retries" >&2
  return 1
}

echo "[smoke] waiting for service health endpoints"
wait_http_ok "$CONTROLPLANE_BASE_URL/healthz" "control-plane" || exit 1
wait_http_ok "$EXECUTION_BASE_URL/healthz" "execution" || exit 1
wait_http_ok "$MARKETDATA_BASE_URL/healthz" "marketdata" || exit 1

echo "[smoke] checking capabilities"
exec_caps="$(curl -fsS --max-time "$SMOKE_TIMEOUT_SECONDS" "$EXECUTION_BASE_URL/capabilities")" || {
  echo "[smoke][error] failed to fetch execution capabilities" >&2
  exit 1
}
python - <<'PY' "$exec_caps"
import json,sys
caps=json.loads(sys.argv[1])
features=caps.get("features",[])
status=caps.get("status")
print(f"[smoke][ok] execution status={status} features={features}")
PY

# Try canonical marketdata query first; fallback to non-query route.
echo "[smoke] fetching ticker"
ticker_payload=""
if ticker_payload="$(curl -fsS --max-time "$SMOKE_TIMEOUT_SECONDS" "$MARKETDATA_BASE_URL/ticker/latest?exchange=gmo&symbol=BTC_JPY" 2>/dev/null)"; then
  :
elif ticker_payload="$(curl -fsS --max-time "$SMOKE_TIMEOUT_SECONDS" "$MARKETDATA_BASE_URL/ticker/latest" 2>/dev/null)"; then
  :
else
  echo "[smoke][error] failed to fetch marketdata ticker from $MARKETDATA_BASE_URL/ticker/latest" >&2
  exit 1
fi

python - <<'PY' "$ticker_payload"
import json,sys
p=json.loads(sys.argv[1])
required=["symbol","bid","ask","last"]
missing=[k for k in required if k not in p]
if missing:
    raise SystemExit(f"[smoke][error] ticker missing fields: {missing}")
if p.get("stale") is True or p.get("degraded_reason"):
    raise SystemExit(f"[smoke][error] ticker is degraded/stale: stale={p.get('stale')} reason={p.get('degraded_reason')}")
print(f"[smoke][ok] ticker symbol={p.get('symbol')} bid={p.get('bid')} ask={p.get('ask')} last={p.get('last')}")
PY

idempotency_key="smoke-paper-$(date +%s)"
order_payload="$(cat <<JSON
{
  "idempotency_key": "$idempotency_key",
  "exchange": "binance",
  "symbol": "BTC/USDT",
  "side": "BUY",
  "qty": 0.001,
  "type": "MARKET"
}
JSON
)"

echo "[smoke] submitting paper order-intent"
order_response="$(curl -fsS --max-time "$SMOKE_TIMEOUT_SECONDS" -X POST "$EXECUTION_BASE_URL/execution/order-intents" -H 'content-type: application/json' -d "$order_payload")" || {
  echo "[smoke][error] paper order-intent request failed. Check execution logs and ALLOWED_* env settings." >&2
  exit 1
}

python - <<'PY' "$order_response"
import json,sys
order=json.loads(sys.argv[1])
order_id=order.get("order_id")
status=order.get("status")
if not order_id:
    raise SystemExit("[smoke][error] execution response missing order_id")
if status not in {"NEW","PARTIALLY_FILLED","FILLED"}:
    raise SystemExit(f"[smoke][error] unexpected paper order status: {status}")
print(f"[smoke][ok] paper order accepted order_id={order_id} status={status}")
PY

echo "[smoke][success] paper E2E smoke completed (ticker -> order-intent -> paper order acceptance)"
