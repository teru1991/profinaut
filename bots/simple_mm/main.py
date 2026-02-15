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


def fetch_pending_commands(controlplane_base_url: str, bot_id: str) -> list[dict]:
    query = urllib.parse.urlencode({"target_bot_id": bot_id, "status": "pending"})
    status, payload = http_json("GET", f"{controlplane_base_url}/commands?{query}")
    if status != 200:
        raise BotError(f"unexpected commands status: {status}")
    if not isinstance(payload, list):
        raise BotError("unexpected commands payload shape")
    return [item for item in payload if isinstance(item, dict)]


def send_command_ack(controlplane_base_url: str, command_id: str, ok: bool, reason: str | None) -> None:
    ack_payload = {"ok": ok, "reason": reason, "ts": now_utc_iso()}
    status, _ = http_json("POST", f"{controlplane_base_url}/commands/{command_id}/ack", body=ack_payload)
    if status not in {200, 201, 202}:
        raise BotError(f"unexpected command ack status: {status}")


def process_commands(controlplane_base_url: str, bot_id: str, run_id: str, paused: bool) -> bool:
    commands = fetch_pending_commands(controlplane_base_url, bot_id)
    if not commands:
        return paused

    for command in commands:
        command_id = str(command.get("id") or "")
        command_type = str(command.get("type") or "").strip().upper()

        if not command_id:
            log_event(
                "WARN",
                "command_skipped",
                run_id,
                bot_id,
                reason="MISSING_COMMAND_ID",
                command=command,
                state="DEGRADED",
                decision="SKIP_ORDER",
            )
            continue

        if command_type in ("PAUSE", "RESUME"):
            paused = (command_type == "PAUSE")
            send_command_ack(controlplane_base_url, command_id, ok=True, reason=None)
            log_event(
                "INFO",
                "command_applied",
                run_id,
                bot_id,
                command_id=command_id,
                command_type=command_type,
                paused=paused,
                state="PAUSED" if paused else "RUNNING",
                decision="SKIP_ORDER" if paused else "CHECK",
            )
        else:
            reason = f"Unsupported command type '{command_type or 'UNKNOWN'}'"
            send_command_ack(controlplane_base_url, command_id, ok=False, reason=reason)
            log_event(
                "WARN",
                "command_rejected",
                run_id,
                bot_id,
                command_id=command_id,
                command_type=command_type,
                reason=reason,
                state="DEGRADED",
                decision="SKIP_ORDER",
            )

    return paused


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
    controlplane_base_url = os.getenv("CONTROL_PLANE_BASE_URL") or os.getenv("CONTROLPLANE_BASE_URL", "http://127.0.0.1:8000")
    execution_base_url = os.getenv("EXECUTION_BASE_URL", "http://127.0.0.1:8001")

    market_exchange = os.getenv("MARKETDATA_EXCHANGE", "gmo")
    market_symbol = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")

    order_exchange = os.getenv("ORDER_EXCHANGE", market_exchange)
    order_symbol = os.getenv("ORDER_SYMBOL", market_symbol)
    order_side = os.getenv("ORDER_SIDE", "BUY")
    order_qty = float(os.getenv("ORDER_QTY", "0.001"))

    loop_interval_seconds = float(os.getenv("BOT_LOOP_INTERVAL_SECONDS", "2"))
    command_poll_interval_seconds = float(os.getenv("COMMAND_POLL_INTERVAL_SEC", "2"))
    max_loops = int(os.getenv("BOT_MAX_LOOPS", "1"))

    deadman = DeadmanSwitch(
        timeout_seconds=float(os.getenv("DEADMAN_TIMEOUT_SECONDS", "10")),
        recovery_successes_required=int(os.getenv("DEADMAN_RECOVERY_SUCCESSES", "2")),
    )
    paused_by_command = False
    last_command_poll_monotonic: float | None = None

    log_event(
        "INFO",
        "bot_start",
        run_id,
        bot_id,
        forced_safe_mode=forced_safe_mode,
        execution_mode=execution_mode,
        live_enabled=live_enabled,
        control_plane_base_url=controlplane_base_url,
        command_poll_interval_seconds=command_poll_interval_seconds,
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
                    deadman_safe_mode=deadman_safe_mode,
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
                    state="SAFE_MODE" if deadman_safe_mode else "DEGRADED",
                    decision="SKIP_ORDER",
                )
                if deadman_safe_mode:
                    log_event(
                        "WARN",
                        "new_order_blocked",
                        run_id,
                        bot_id,
                        reason="SAFE_MODE",
                        block_source="DEADMAN_SWITCH",
                        state="SAFE_MODE",
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
                    block_source="CONTROLPLANE_STATUS",
                    state="DEGRADED",
                    decision="SKIP_ORDER",
                )
                if iteration < max_loops - 1:
                    time.sleep(loop_interval_seconds)
                continue


            now_mono = time.monotonic()
            if last_command_poll_monotonic is None or (now_mono - last_command_poll_monotonic) >= command_poll_interval_seconds:
                paused_by_command = process_commands(
                    controlplane_base_url=controlplane_base_url,
                    bot_id=bot_id,
                    run_id=run_id,
                    paused=paused_by_command,
                )
                last_command_poll_monotonic = now_mono

            if paused_by_command:
                log_event(
                    "WARN",
                    "new_order_blocked",
                    run_id,
                    bot_id,
                    reason="PAUSED",
                    block_source="COMMAND",
                    state="PAUSED",
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
                block_state = "SAFE_MODE" if reason == "SAFE_MODE" else "DEGRADED"
                block_source = "DEADMAN_SWITCH" if reason == "SAFE_MODE" else "SERVICE_GATING"
                log_event(
                    "WARN",
                    "new_order_blocked",
                    run_id,
                    bot_id,
                    reason=reason,
                    block_source=block_source,
                    state=block_state,
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
