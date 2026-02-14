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


class DeadmanSwitch:
    """Dead man's switch that latches SAFE_MODE after prolonged control-plane failures.

    Defaults are safe-by-default for control-plane outages:
    - If control-plane is unreachable for `timeout_seconds`, SAFE_MODE is latched.
    - Once latched, SAFE_MODE persists until `recovery_successes_required` consecutive
      successful control-plane checks are observed.
    """

    def __init__(self, timeout_seconds: float, recovery_successes_required: int) -> None:
        self.timeout_seconds = timeout_seconds
        self.recovery_successes_required = max(1, recovery_successes_required)
        self.unreachable_since_monotonic: float | None = None
        self.latched_safe_mode: bool = False
        self._recovery_successes: int = 0

    def note_failure(self, now_monotonic: float) -> tuple[bool, str | None]:
        if self.unreachable_since_monotonic is None:
            self.unreachable_since_monotonic = now_monotonic
            return self.latched_safe_mode, None

        if (now_monotonic - self.unreachable_since_monotonic) >= self.timeout_seconds:
            self.latched_safe_mode = True
            self._recovery_successes = 0
            return True, "CONTROLPLANE_UNREACHABLE"
        return self.latched_safe_mode, None

    def note_success(self) -> tuple[bool, str | None]:
        self.unreachable_since_monotonic = None
        if self.latched_safe_mode:
            self._recovery_successes += 1
            if self._recovery_successes >= self.recovery_successes_required:
                self.latched_safe_mode = False
                self._recovery_successes = 0
                return False, "DEADMAN_RECOVERED"
        return self.latched_safe_mode, None


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


def ticker_is_degraded(ticker: dict) -> tuple[bool, str | None]:
    if bool(ticker.get("stale")):
        return True, "STALE_TICKER"
    if ticker.get("degraded_reason"):
        return True, str(ticker.get("degraded_reason"))
    quality = ticker.get("quality") if isinstance(ticker.get("quality"), dict) else {}
    if quality.get("status") not in (None, "OK"):
        return True, str(quality.get("status"))
    return False, None


def should_block_new_order(
    forced_safe_mode: bool,
    deadman_safe_mode: bool,
    execution_mode: str,
    live_enabled: bool,
    ticker: dict,
    capabilities: dict,
) -> tuple[bool, str | None]:
    if forced_safe_mode or deadman_safe_mode:
        return True, "SAFE_MODE"

    ticker_degraded, ticker_reason = ticker_is_degraded(ticker)
    if ticker_degraded:
        return True, ticker_reason or "MARKETDATA_DEGRADED"

    if capabilities.get("status") == "degraded":
        return True, str(capabilities.get("degraded_reason") or "EXECUTION_DEGRADED")

    if execution_mode == "live" and not live_enabled:
        return True, "LIVE_DISABLED"

    return False, None


def run() -> int:
    run_id = str(uuid4())
    bot_id = os.getenv("BOT_ID", "simple-mm")

    forced_safe_mode = env_bool("SAFE_MODE", default=False)
    execution_mode = os.getenv("EXECUTION_MODE", "paper").strip().lower() or "paper"
    live_enabled = env_bool("EXECUTION_LIVE_ENABLED", default=False)

    marketdata_base_url = os.getenv("MARKETDATA_BASE_URL", "http://127.0.0.1:8081")
    controlplane_base_url = os.getenv("CONTROLPLANE_BASE_URL", "http://127.0.0.1:8000")
    execution_base_url = os.getenv("EXECUTION_BASE_URL", "http://127.0.0.1:8001")

    market_exchange = os.getenv("MARKETDATA_EXCHANGE", "gmo")
    market_symbol = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")

    order_exchange = os.getenv("ORDER_EXCHANGE", market_exchange)
    order_symbol = os.getenv("ORDER_SYMBOL", market_symbol)
    order_side = os.getenv("ORDER_SIDE", "BUY")
    order_qty = float(os.getenv("ORDER_QTY", "0.001"))

    loop_interval_seconds = float(os.getenv("BOT_LOOP_INTERVAL_SECONDS", "2"))
    max_loops = int(os.getenv("BOT_MAX_LOOPS", "1"))

    deadman = DeadmanSwitch(
        timeout_seconds=float(os.getenv("DEADMAN_TIMEOUT_SECONDS", "10")),
        recovery_successes_required=int(os.getenv("DEADMAN_RECOVERY_SUCCESSES", "2")),
    )

    log_event(
        "INFO",
        "bot_start",
        run_id,
        bot_id,
        forced_safe_mode=forced_safe_mode,
        execution_mode=execution_mode,
        live_enabled=live_enabled,
        state="STARTING",
        decision="BOOT",
    )

    if execution_mode not in {"paper", "live"}:
        log_event(
            "ERROR",
            "bot_error",
            run_id,
            bot_id,
            error=f"Unsupported EXECUTION_MODE '{execution_mode}'",
            state="ERROR",
            decision="SKIP_ORDER",
        )
        return 1

    try:
        for iteration in range(max_loops):
            deadman_safe_mode = False

            try:
                controlplane = fetch_controlplane_capabilities(controlplane_base_url)
                deadman_safe_mode, recovery_event = deadman.note_success()
                if recovery_event == "DEADMAN_RECOVERED":
                    log_event(
                        "INFO",
                        "deadman_recovered",
                        run_id,
                        bot_id,
                        state="RUNNING",
                        decision="CHECK",
                    )
                log_event(
                    "INFO",
                    "controlplane_capabilities",
                    run_id,
                    bot_id,
                    controlplane=controlplane,
                    state="RUNNING",
                )
            except BotError:
                deadman_safe_mode, reason = deadman.note_failure(time.monotonic())
                log_event(
                    "WARN",
                    "controlplane_unreachable",
                    run_id,
                    bot_id,
                    reason=reason or "CONTROLPLANE_CHECK_FAILED",
                    deadman_safe_mode=deadman_safe_mode,
                    state="DEGRADED",
                    decision="SKIP_ORDER",
                )
                if iteration < max_loops - 1:
                    time.sleep(loop_interval_seconds)
                continue

            if controlplane.get("status") == "degraded":
                log_event(
                    "WARN",
                    "new_order_blocked",
                    run_id,
                    bot_id,
                    reason=str(controlplane.get("degraded_reason") or "CONTROLPLANE_DEGRADED"),
                    state="DEGRADED",
                    decision="SKIP_ORDER",
                )
                if iteration < max_loops - 1:
                    time.sleep(loop_interval_seconds)
                continue

            ticker = fetch_ticker(marketdata_base_url, market_exchange, market_symbol)
            log_event("INFO", "ticker_fetched", run_id, bot_id, ticker=ticker, state="RUNNING")

            capabilities = fetch_execution_capabilities(execution_base_url)
            log_event("INFO", "execution_capabilities", run_id, bot_id, capabilities=capabilities, state="RUNNING")

            blocked, reason = should_block_new_order(
                forced_safe_mode=forced_safe_mode,
                deadman_safe_mode=deadman_safe_mode,
                execution_mode=execution_mode,
                live_enabled=live_enabled,
                ticker=ticker,
                capabilities=capabilities,
            )
            if blocked:
                log_event(
                    "WARN",
                    "new_order_blocked",
                    run_id,
                    bot_id,
                    reason=reason,
                    state="SAFE_MODE" if reason == "SAFE_MODE" else "DEGRADED",
                    decision="SKIP_ORDER",
                )
                if iteration < max_loops - 1:
                    time.sleep(loop_interval_seconds)
                continue

            intent = {
                "idempotency_key": f"{bot_id}-{run_id}-{iteration}",
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
            )

            if iteration < max_loops - 1:
                time.sleep(loop_interval_seconds)

        return 0
    except (BotError, ValueError) as exc:
        log_event("ERROR", "bot_error", run_id, bot_id, error=str(exc), state="ERROR", decision="SKIP_ORDER")
        return 1


if __name__ == "__main__":
    sys.exit(run())
