import json
import os
import sys
import time
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
        "state": fields.pop("state", None),
        "decision": fields.pop("decision", None),
        "idempotency_key": fields.pop("idempotency_key", None),
        "order_id": fields.pop("order_id", None),
        **fields,
    }
    print(json.dumps(payload, ensure_ascii=False))


def env_bool(name: str, default: bool = False) -> bool:
    raw = os.getenv(name)
    if raw is None:
        return default
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def http_json(
    method: str,
    url: str,
    body: dict | None = None,
    timeout: float = 5.0,
    retries: int = 2,
    backoff_seconds: float = 0.2,
) -> tuple[int, dict]:
    data = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"accept": "application/json", "content-type": "application/json"},
    )
    for attempt in range(retries + 1):
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
            if exc.code in {429, 500, 502, 503, 504} and attempt < retries:
                time.sleep(backoff_seconds * (2**attempt))
                continue
            raise BotError(f"HTTP {exc.code} for {method} {url}: {detail}") from exc
        except (urllib.error.URLError, TimeoutError) as exc:
            if attempt < retries:
                time.sleep(backoff_seconds * (2**attempt))
                continue
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


def fetch_controlplane_capabilities(controlplane_base_url: str) -> dict:
    status, payload = http_json("GET", f"{controlplane_base_url}/capabilities")
    if status != 200:
        raise BotError(f"unexpected controlplane capabilities status: {status}")
    return payload


def submit_order_intent(execution_base_url: str, intent: dict) -> dict:
    status, payload = http_json("POST", f"{execution_base_url}/execution/order-intents", body=intent)
    if status != 201:
        raise BotError(f"unexpected order-intents status: {status}")
    return payload


def should_block_new_order(
    safe_mode: bool,
    ticker: dict,
    capabilities: dict,
    max_ticker_age_seconds: float = 30.0,
) -> tuple[bool, str | None]:
    if safe_mode:
        return True, "SAFE_MODE"
    ts_utc = ticker.get("ts_utc")
    if not isinstance(ts_utc, str):
        return True, "TICKER_TS_MISSING"
    try:
        ticker_ts = datetime.fromisoformat(ts_utc.replace("Z", "+00:00"))
    except ValueError:
        return True, "TICKER_TS_INVALID"
    age = (datetime.now(timezone.utc) - ticker_ts).total_seconds()
    if age > max_ticker_age_seconds:
        return True, "TICKER_STALE"
    if bool(ticker.get("degraded")):
        return True, str(ticker.get("degraded_reason") or "MARKETDATA_DEGRADED")
    if capabilities.get("status") == "degraded":
        return True, str(capabilities.get("degraded_reason") or "EXECUTION_DEGRADED")
    return False, None


def run() -> int:
    run_id = str(uuid4())
    bot_id = os.getenv("BOT_ID", "simple-mm")
    idempotency_key = f"{bot_id}-{run_id}"
    safe_mode = env_bool("SAFE_MODE", default=False)

    marketdata_base_url = os.getenv("MARKETDATA_BASE_URL", "http://127.0.0.1:8081")
    controlplane_base_url = os.getenv("CONTROLPLANE_BASE_URL", "http://127.0.0.1:8000")
    execution_base_url = os.getenv("EXECUTION_BASE_URL", "http://127.0.0.1:8001")

    market_exchange = os.getenv("MARKETDATA_EXCHANGE", "gmo")
    market_symbol = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")

    order_exchange = os.getenv("ORDER_EXCHANGE", market_exchange)
    order_symbol = os.getenv("ORDER_SYMBOL", market_symbol)
    order_side = os.getenv("ORDER_SIDE", "BUY")
    order_qty = float(os.getenv("ORDER_QTY", "0.001"))
    max_ticker_age_seconds = float(os.getenv("MAX_TICKER_AGE_SECONDS", "30"))
    allowed_exchanges = {value.strip() for value in os.getenv("ALLOWED_EXCHANGES", "gmo").split(",") if value.strip()}
    allowed_symbols = {value.strip() for value in os.getenv("ALLOWED_SYMBOLS", "BTC_JPY").split(",") if value.strip()}

    log_event(
        "INFO",
        "bot_start",
        run_id,
        bot_id,
        safe_mode=safe_mode,
        state="STARTING",
        decision="BOOT",
        idempotency_key=idempotency_key,
    )

    if safe_mode:
        log_event(
            "WARN",
            "new_order_blocked",
            run_id,
            bot_id,
            reason="SAFE_MODE",
            state="SAFE_MODE",
            decision="SKIP_ORDER",
            idempotency_key=idempotency_key,
        )
        return 0

    if (
        market_exchange not in allowed_exchanges
        or order_exchange not in allowed_exchanges
        or market_symbol not in allowed_symbols
        or order_symbol not in allowed_symbols
    ):
        log_event(
            "WARN",
            "new_order_blocked",
            run_id,
            bot_id,
            reason="UNKNOWN_EXCHANGE_OR_SYMBOL",
            state="SAFE",
            decision="SKIP_ORDER",
            idempotency_key=idempotency_key,
        )
        return 0

    try:
        try:
            controlplane = fetch_controlplane_capabilities(controlplane_base_url)
            log_event(
                "INFO",
                "controlplane_capabilities",
                run_id,
                bot_id,
                controlplane=controlplane,
                state="RUNNING",
                decision="CHECK_CONTROLPLANE",
                idempotency_key=idempotency_key,
            )
        except BotError:
            log_event(
                "WARN",
                "new_order_blocked",
                run_id,
                bot_id,
                reason="CONTROLPLANE_UNREACHABLE",
                state="DEGRADED",
                decision="SKIP_ORDER",
                idempotency_key=idempotency_key,
            )
            return 0

        if controlplane.get("status") == "degraded":
            log_event(
                "WARN",
                "new_order_blocked",
                run_id,
                bot_id,
                reason=str(controlplane.get("degraded_reason") or "CONTROLPLANE_DEGRADED"),
                state="DEGRADED",
                decision="SKIP_ORDER",
                idempotency_key=idempotency_key,
            )
            return 0

        ticker = fetch_ticker(marketdata_base_url, market_exchange, market_symbol)
        log_event(
            "INFO",
            "ticker_fetched",
            run_id,
            bot_id,
            ticker=ticker,
            state="RUNNING",
            decision="CHECK_MARKETDATA",
            idempotency_key=idempotency_key,
        )

        capabilities = fetch_execution_capabilities(execution_base_url)
        log_event(
            "INFO",
            "execution_capabilities",
            run_id,
            bot_id,
            capabilities=capabilities,
            state="RUNNING",
            decision="CHECK_EXECUTION",
            idempotency_key=idempotency_key,
        )

        blocked, reason = should_block_new_order(
            safe_mode,
            ticker,
            capabilities,
            max_ticker_age_seconds=max_ticker_age_seconds,
        )
        if blocked:
            log_event(
                "WARN",
                "new_order_blocked",
                run_id,
                bot_id,
                reason=reason,
                state="DEGRADED",
                decision="SKIP_ORDER",
                idempotency_key=idempotency_key,
            )
            return 0

        intent = {
            "idempotency_key": idempotency_key,
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

        log_event(
            "INFO",
            "order_result",
            run_id,
            bot_id,
            order=order,
            order_id=order.get("order_id"),
            fills=fills,
            state="RUNNING",
            decision="PLACE_ORDER",
            idempotency_key=idempotency_key,
        )
        return 0
    except (BotError, ValueError) as exc:
        log_event(
            "ERROR",
            "bot_error",
            run_id,
            bot_id,
            error=str(exc),
            state="ERROR",
            decision="SKIP_ORDER",
            idempotency_key=idempotency_key,
        )
        return 1


if __name__ == "__main__":
    sys.exit(run())
