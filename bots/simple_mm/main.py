import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from uuid import uuid4


class BotError(Exception):
    pass


def now_utc_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def log_event(level: str, event: str, run_id: str, bot_id: str, **fields: object) -> None:
    payload = {
        "ts_utc": now_utc_iso(),
        "level": level,
        "event": event,
        "run_id": run_id,
        "bot_id": bot_id,
        **fields,
    }
    print(json.dumps(payload, ensure_ascii=False))


def env_bool(name: str, default: bool = False) -> bool:
    raw = os.getenv(name)
    if raw is None:
        return default
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def http_json(method: str, url: str, body: dict | None = None, timeout: float = 5.0) -> tuple[int, dict]:
    data = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"accept": "application/json", "content-type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw_body = response.read().decode("utf-8")
            payload = json.loads(raw_body) if raw_body else {}
            return response.status, payload
    except urllib.error.HTTPError as exc:
        raw_body = exc.read().decode("utf-8")
        detail = raw_body
        try:
            detail = json.loads(raw_body)
        except json.JSONDecodeError:
            pass
        raise BotError(f"HTTP {exc.code} for {method} {url}: {detail}") from exc
    except (urllib.error.URLError, TimeoutError) as exc:
        raise BotError(f"request failed for {method} {url}: {exc}") from exc


def fetch_ticker(marketdata_base_url: str, exchange: str, symbol: str) -> dict:
    query = urllib.parse.urlencode({"exchange": exchange, "symbol": symbol})
    status, payload = http_json("GET", f"{marketdata_base_url}/ticker/latest?{query}")
    if status != 200:
        raise BotError(f"unexpected ticker status: {status}")
    return payload


def fetch_execution_capabilities(execution_base_url: str) -> dict:
    status, payload = http_json("GET", f"{execution_base_url}/capabilities")
    if status != 200:
        raise BotError(f"unexpected capabilities status: {status}")
    return payload


def submit_order_intent(execution_base_url: str, intent: dict) -> dict:
    status, payload = http_json("POST", f"{execution_base_url}/execution/order-intents", body=intent)
    if status != 201:
        raise BotError(f"unexpected order-intents status: {status}")
    return payload


def should_block_new_order(safe_mode: bool, ticker: dict, capabilities: dict) -> tuple[bool, str | None]:
    if safe_mode:
        return True, "SAFE_MODE"
    if bool(ticker.get("degraded")):
        return True, str(ticker.get("degraded_reason") or "MARKETDATA_DEGRADED")
    if capabilities.get("status") == "degraded":
        return True, str(capabilities.get("degraded_reason") or "EXECUTION_DEGRADED")
    return False, None


def run() -> int:
    run_id = str(uuid4())
    bot_id = os.getenv("BOT_ID", "simple-mm")
    safe_mode = env_bool("SAFE_MODE", default=False)

    marketdata_base_url = os.getenv("MARKETDATA_BASE_URL", "http://127.0.0.1:8081")
    execution_base_url = os.getenv("EXECUTION_BASE_URL", "http://127.0.0.1:8001")

    market_exchange = os.getenv("MARKETDATA_EXCHANGE", "gmo")
    market_symbol = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")

    order_exchange = os.getenv("ORDER_EXCHANGE", market_exchange)
    order_symbol = os.getenv("ORDER_SYMBOL", market_symbol)
    order_side = os.getenv("ORDER_SIDE", "BUY")
    order_qty = float(os.getenv("ORDER_QTY", "0.001"))

    log_event("INFO", "bot_start", run_id, bot_id, safe_mode=safe_mode)

    if safe_mode:
        log_event("WARN", "new_order_blocked", run_id, bot_id, reason="SAFE_MODE")
        return 0

    try:
        ticker = fetch_ticker(marketdata_base_url, market_exchange, market_symbol)
        log_event("INFO", "ticker_fetched", run_id, bot_id, ticker=ticker)

        capabilities = fetch_execution_capabilities(execution_base_url)
        log_event("INFO", "execution_capabilities", run_id, bot_id, capabilities=capabilities)

        blocked, reason = should_block_new_order(safe_mode, ticker, capabilities)
        if blocked:
            log_event("WARN", "new_order_blocked", run_id, bot_id, reason=reason)
            return 0

        intent = {
            "idempotency_key": f"{bot_id}-{run_id}",
            "exchange": order_exchange,
            "symbol": order_symbol,
            "side": order_side,
            "qty": order_qty,
            "type": "MARKET",
            "client_ts_utc": now_utc_iso(),
        }
        order = submit_order_intent(execution_base_url, intent)

        fills = []
        if float(order.get("filled_qty", 0.0)) > 0:
            fills.append({"qty": order.get("filled_qty")})

        log_event("INFO", "order_result", run_id, bot_id, order=order, fills=fills)
        return 0
    except (BotError, ValueError) as exc:
        log_event("ERROR", "bot_error", run_id, bot_id, error=str(exc))
        return 1


if __name__ == "__main__":
    sys.exit(run())
