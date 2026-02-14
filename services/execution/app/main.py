import logging
from datetime import datetime, timezone

from fastapi import FastAPI, HTTPException

from .config import get_settings
from .schemas import CapabilitiesResponse, HealthResponse, Order, OrderIntent
from .storage import get_storage

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger("execution")

app = FastAPI(title="Profinaut Execution Service", version="0.1.0")


@app.get("/healthz", response_model=HealthResponse)
def get_healthz() -> HealthResponse:
    return HealthResponse(status="ok", timestamp=datetime.now(timezone.utc))


@app.get("/capabilities", response_model=CapabilitiesResponse)
def get_capabilities() -> CapabilitiesResponse:
    settings = get_settings()
    return CapabilitiesResponse(
        service="execution",
        version=settings.service_version,
        status="ok",
        features=["paper_execution"],
        degraded_reason=None,
        generated_at=datetime.now(timezone.utc),
    )


@app.post("/execution/order-intents", status_code=201, response_model=Order)
def post_order_intent(intent: OrderIntent) -> Order:
    settings = get_settings()
    storage = get_storage()

    # Log the request with all required fields
    logger.info(
        "Received order intent",
        extra={
            "idempotency_key": intent.idempotency_key,
            "exchange": intent.exchange,
            "symbol": intent.symbol,
            "side": intent.side,
            "qty": intent.qty,
            "type": intent.type,
        },
    )

    # Check if symbol is allowed (safe default: reject unknown symbols)
    allowed_symbols = settings.get_allowed_symbols()
    if not allowed_symbols or intent.symbol not in allowed_symbols:
        logger.warning(
            "Symbol not in allowlist - rejecting order",
            extra={
                "idempotency_key": intent.idempotency_key,
                "exchange": intent.exchange,
                "symbol": intent.symbol,
            },
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
            extra={
                "idempotency_key": intent.idempotency_key,
                "exchange": intent.exchange,
                "symbol": intent.symbol,
            },
        )
        raise HTTPException(
            status_code=400,
            detail=f"Exchange '{intent.exchange}' is not allowed. Configure ALLOWED_EXCHANGES to enable.",
        )

    # Validate LIMIT order has limit_price
    if intent.type == "LIMIT" and intent.limit_price is None:
        logger.warning(
            "LIMIT order missing limit_price",
            extra={
                "idempotency_key": intent.idempotency_key,
                "exchange": intent.exchange,
                "symbol": intent.symbol,
            },
        )
        raise HTTPException(status_code=400, detail="LIMIT orders must specify limit_price")

    # Create order (handles idempotency check)
    order = storage.create_order(intent)

    if order is None:
        # Duplicate idempotency_key
        logger.warning(
            "Duplicate idempotency_key rejected",
            extra={
                "idempotency_key": intent.idempotency_key,
                "exchange": intent.exchange,
                "symbol": intent.symbol,
            },
        )
        raise HTTPException(status_code=409, detail="Duplicate idempotency_key")

    # Log successful order creation
    logger.info(
        "Order created successfully",
        extra={
            "idempotency_key": intent.idempotency_key,
            "order_id": order.order_id,
            "exchange": order.exchange,
            "symbol": order.symbol,
            "side": order.side,
            "qty": order.qty,
            "status": order.status,
        },
    )

    return order
