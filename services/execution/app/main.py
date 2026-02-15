import logging
from datetime import datetime, timedelta, timezone
import os
import sys
from pathlib import Path

from fastapi import Depends, FastAPI, HTTPException, Query, Request
from fastapi.responses import JSONResponse

from .auth import require_execution_token
from .config import Settings, get_settings
from .live import GmoLiveExecutor, LiveRateLimitError, LiveTimeoutError
from .policy_gate import PolicyAction, PolicyGateInput, PolicyGateResult, evaluate_policy_gate
from .schemas import CapabilitiesResponse, FillsHistoryResponse, HealthResponse, OrdersHistoryResponse, Order, OrderIntent
from .storage import get_storage

# Add libs to path for observability module
_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.append(str(_REPO_ROOT))

from libs.observability import audit_event, error_envelope, request_id_middleware

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger("execution")

app = FastAPI(title="Profinaut Execution Service", version="0.1.0")
app.add_middleware(request_id_middleware())

_live_backoff_until_utc: datetime | None = None
_degraded_reason: str | None = None


@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    code = "HTTP_ERROR"
    message = str(exc.detail)
    details: dict[str, object] = {}

    if isinstance(exc.detail, dict):
        code = str(exc.detail.get("error") or code)
        message = str(exc.detail.get("message") or message)

    return JSONResponse(
        status_code=exc.status_code,
        content=error_envelope(code=code, message=message, details=details, request_id=request_id),
    )


@app.exception_handler(Exception)
async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    return JSONResponse(
        status_code=500,
        content=error_envelope(
            code="INTERNAL_ERROR",
            message="Unexpected error",
            details={},
            request_id=request_id,
        ),
    )




def _resolve_safe_mode() -> tuple[str, str | None]:
    settings = get_settings()
    configured_mode = settings.get_safe_mode()
    now = datetime.now(timezone.utc)
    is_live_degraded = _live_backoff_until_utc is not None and now < _live_backoff_until_utc

    if configured_mode in {"SAFE_MODE", "HALTED"}:
        reason = settings.execution_degraded_reason or f"Execution mode is {configured_mode}"
        return configured_mode, reason
    if configured_mode == "DEGRADED" or is_live_degraded:
        reason = _degraded_reason or settings.execution_degraded_reason or "Execution service degraded"
        return "DEGRADED", reason
    return "NORMAL", None


def _allowed_actions_for_mode(mode: str) -> list[str]:
    if mode in {"SAFE_MODE", "HALTED", "DEGRADED"}:
        return ["CANCEL"]
    return ["NEW_ORDER", "CANCEL"]


def _policy_error_message(result: PolicyGateResult, safe_mode: str, degraded_reason: str | None) -> str:
    if result.reason_code == "SAFE_MODE_BLOCKED":
        return f"New order placement is blocked while mode={safe_mode}. {degraded_reason or 'Execution is in safety mode.'}"
    if result.reason_code == "LIVE_DISABLED":
        return "Live execution is disabled by feature flag"
    if result.reason_code == "DRY_RUN_ONLY":
        return "Live execution is in dry-run mode. Set EXECUTION_LIVE_MODE=live to enable upstream calls."
    if result.reason_code == "LIVE_DEGRADED":
        return f"Live execution degraded: {_degraded_reason}"
    return f"Execution blocked by policy gate ({result.reason_code})"


def _enforce_policy_gate(*, action: PolicyAction, exchange: str) -> PolicyGateResult:
    settings = get_settings()
    safe_mode, degraded_reason = _resolve_safe_mode()
    result = evaluate_policy_gate(
        PolicyGateInput(
            action=action,
            exchange=exchange,
            safe_mode=safe_mode,
            live_enabled=settings.execution_live_enabled,
            live_mode=settings.execution_live_mode,
            live_backoff_until_utc=_live_backoff_until_utc,
            degraded_reason=degraded_reason,
        )
    )

    if result.decision == "ALLOW":
        return result

    status_code = 403
    if result.decision == "THROTTLE":
        status_code = 503
    raise HTTPException(
        status_code=status_code,
        detail=_error_payload(result.reason_code, _policy_error_message(result, safe_mode, degraded_reason)),
    )


def _log_context(
    *,
    idempotency_key: str,
    exchange: str,
    symbol: str,
    status: str,
    order_id: str | None = None,
) -> dict[str, str | None]:
    return {
        "idempotency_key": idempotency_key,
        "order_id": order_id,
        "exchange": exchange,
        "symbol": symbol,
        "status": status,
    }


@app.get("/healthz", response_model=HealthResponse)
def get_healthz() -> HealthResponse:
    return HealthResponse(status="ok", timestamp=datetime.now(timezone.utc))


@app.get("/capabilities", response_model=CapabilitiesResponse)
def get_capabilities() -> CapabilitiesResponse:
    settings = get_settings()
    now = datetime.now(timezone.utc)
    safe_mode, degraded_reason = _resolve_safe_mode()
    features = ["paper_execution"]
    if settings.execution_live_enabled:
        features.append("live_execution")
        if not settings.is_live_mode():
            features.append("live_dry_run")
    return CapabilitiesResponse(
        service="execution",
        version=settings.service_version,
        status="degraded" if safe_mode == "DEGRADED" else "ok",
        safe_mode=safe_mode,
        allowed_actions=_allowed_actions_for_mode(safe_mode),
        features=features,
        degraded_reason=degraded_reason,
        generated_at=now,
    )


def _mark_live_degraded(reason: str) -> None:
    global _live_backoff_until_utc, _degraded_reason
    settings = get_settings()
    _degraded_reason = reason
    _live_backoff_until_utc = datetime.now(timezone.utc).replace(microsecond=0) + timedelta(
        seconds=settings.live_backoff_seconds
    )


def _error_payload(code: str, message: str) -> dict[str, str]:
    return {"error": code, "message": message}


def _get_live_executor(settings: Settings) -> GmoLiveExecutor:
    api_key = os.getenv("GMO_API_KEY", "")
    api_secret = os.getenv("GMO_API_SECRET", "")
    if not settings.gmo_api_base_url or not api_key or not api_secret:
        raise HTTPException(
            status_code=503,
            detail=_error_payload("LIVE_NOT_CONFIGURED", "GMO live execution is not configured"),
        )
    return GmoLiveExecutor(
        base_url=settings.gmo_api_base_url,
        timeout_seconds=settings.gmo_request_timeout_seconds,
        api_key=api_key,
        api_secret=api_secret,
    )


@app.post("/execution/order-intents", status_code=201, response_model=Order)
def post_order_intent(intent: OrderIntent) -> Order:
    settings = get_settings()
    storage = get_storage()

    # Log the request with all required fields
    logger.info(
        "Received order intent",
        extra=_log_context(
            idempotency_key=intent.idempotency_key,
            exchange=intent.exchange,
            symbol=intent.symbol,
            status="RECEIVED",
        ),
    )

    _enforce_policy_gate(action="ORDER_INTENT", exchange=intent.exchange)

    # Check if symbol is allowed (safe default: reject unknown symbols)
    allowed_symbols = settings.get_allowed_symbols()
    if not allowed_symbols or intent.symbol not in allowed_symbols:
        logger.warning(
            "Symbol not in allowlist - rejecting order",
            extra=_log_context(
                idempotency_key=intent.idempotency_key,
                exchange=intent.exchange,
                symbol=intent.symbol,
                status="REJECTED",
            ),
        )
        raise HTTPException(
            status_code=400,
            detail=f"Symbol '{intent.symbol}' is not allowed. Configure ALLOWED_SYMBOLS to enable.",
        )

    # Check if exchange is allowed
    allowed_exchanges = settings.get_allowed_exchanges()
    if not allowed_exchanges or intent.exchange not in allowed_exchanges:
        logger.warning(
            "Exchange not in allowlist - rejecting order",
            extra=_log_context(
                idempotency_key=intent.idempotency_key,
                exchange=intent.exchange,
                symbol=intent.symbol,
                status="REJECTED",
            ),
        )
        raise HTTPException(
            status_code=400,
            detail=f"Exchange '{intent.exchange}' is not allowed. Configure ALLOWED_EXCHANGES to enable.",
        )

    # Validate LIMIT order has limit_price
    if intent.type == "LIMIT" and intent.limit_price is None:
        logger.warning(
            "LIMIT order missing limit_price",
            extra=_log_context(
                idempotency_key=intent.idempotency_key,
                exchange=intent.exchange,
                symbol=intent.symbol,
                status="REJECTED",
            ),
        )
        raise HTTPException(status_code=400, detail="LIMIT orders must specify limit_price")

    # Idempotency pre-check to avoid duplicate upstream live placement side effects.
    existing_order = storage.get_order_by_idempotency_key(intent.idempotency_key)
    if existing_order is not None:
        logger.warning(
            "Duplicate idempotency_key rejected",
            extra=_log_context(
                idempotency_key=intent.idempotency_key,
                order_id=existing_order.order_id,
                exchange=intent.exchange,
                symbol=intent.symbol,
                status="REJECTED",
            ),
        )
        raise HTTPException(status_code=409, detail="Duplicate idempotency_key")

    # Create order (handles idempotency check)
    if intent.exchange == "gmo":
        live = _get_live_executor(settings)
        client_order_id = GmoLiveExecutor.build_client_order_id(intent.idempotency_key)
        try:
            placed = live.place_order(
                symbol=intent.symbol,
                side=intent.side,
                qty=intent.qty,
                order_type=intent.type,
                limit_price=intent.limit_price,
                client_order_id=client_order_id,
            )
        except (LiveRateLimitError, LiveTimeoutError) as exc:
            _mark_live_degraded(str(exc))
            raise HTTPException(status_code=503, detail=str(exc)) from exc
        order = storage.create_order(intent, order_id=placed.order_id, client_order_id=client_order_id)
    else:
        order = storage.create_order(intent)

    if order is None:
        raise HTTPException(status_code=409, detail="Duplicate idempotency_key")

    # Log successful order creation
    logger.info(
        "Order created successfully",
        extra=_log_context(
            idempotency_key=intent.idempotency_key,
            order_id=order.order_id,
            exchange=order.exchange,
            symbol=order.symbol,
            status=order.status,
        ),
    )

    return order


@app.get("/orders", response_model=OrdersHistoryResponse)
def list_orders(page: int = Query(default=1, ge=1), page_size: int = Query(default=50, ge=1, le=200)) -> OrdersHistoryResponse:
    storage = get_storage()
    items, total = storage.list_orders(page=page, page_size=page_size)
    return OrdersHistoryResponse(items=items, page=page, page_size=page_size, total=total)


@app.get("/fills", response_model=FillsHistoryResponse)
def list_fills(page: int = Query(default=1, ge=1), page_size: int = Query(default=50, ge=1, le=200)) -> FillsHistoryResponse:
    storage = get_storage()
    items, total = storage.list_fills(page=page, page_size=page_size)
    return FillsHistoryResponse(items=items, page=page, page_size=page_size, total=total)


@app.post("/execution/orders/{order_id}/cancel", response_model=Order)
def cancel_order(order_id: str, _actor: str = Depends(require_execution_token)) -> Order:
    settings = get_settings()
    storage = get_storage()
    order = storage.get_order(order_id)
    if order is None:
        raise HTTPException(status_code=404, detail="Order not found")

    if order.exchange == "gmo":
        _enforce_policy_gate(action="CANCEL", exchange=order.exchange)
        live = _get_live_executor(settings)
        client_order_id = storage.get_client_order_id_by_order_id(order_id)
        if client_order_id is None:
            raise HTTPException(status_code=409, detail="Missing client_order_id mapping")
        try:
            live.cancel_order(order_id=order_id, client_order_id=client_order_id)
        except (LiveRateLimitError, LiveTimeoutError) as exc:
            _mark_live_degraded(str(exc))
            raise HTTPException(status_code=503, detail=str(exc)) from exc

    canceled = storage.cancel_order(order_id)
    if canceled is None:
        raise HTTPException(status_code=404, detail="Order not found")
    if canceled.status != "CANCELED":
        raise HTTPException(status_code=409, detail=f"Order cannot be canceled from status {order.status}")
    return canceled


@app.post("/execution/orders/{order_id}/fill", response_model=Order)
def fill_order(order_id: str) -> Order:
    storage = get_storage()
    order = storage.get_order(order_id)
    if order is None:
        raise HTTPException(status_code=404, detail="Order not found")
    filled = storage.fill_order(order_id)
    if filled is None:
        raise HTTPException(status_code=404, detail="Order not found")
    if filled.status != "FILLED":
        raise HTTPException(status_code=409, detail=f"Order cannot be filled from status {order.status}")
    return filled


@app.post("/execution/orders/{order_id}/reject", response_model=Order)
def reject_order(order_id: str) -> Order:
    storage = get_storage()
    order = storage.get_order(order_id)
    if order is None:
        raise HTTPException(status_code=404, detail="Order not found")
    rejected = storage.reject_order(order_id)
    if rejected is None:
        raise HTTPException(status_code=404, detail="Order not found")
    if rejected.status != "REJECTED":
        raise HTTPException(status_code=409, detail=f"Order cannot be rejected from status {order.status}")
    return rejected
