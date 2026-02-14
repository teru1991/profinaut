from datetime import datetime
from typing import Literal

from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    status: str
    timestamp: datetime


class CapabilitiesResponse(BaseModel):
    service: str = "execution"
    version: str
    status: Literal["ok", "degraded"] = "ok"
    features: list[str]
    degraded_reason: str | None = None
    generated_at: datetime


class OrderIntent(BaseModel):
    idempotency_key: str = Field(..., min_length=1)
    exchange: str = Field(..., min_length=1)
    symbol: str = Field(..., min_length=1)
    side: Literal["BUY", "SELL"]
    qty: float = Field(..., gt=0)
    type: Literal["MARKET", "LIMIT"]
    limit_price: float | None = Field(None, gt=0)
    client_ts_utc: datetime | None = None


class Order(BaseModel):
    order_id: str = Field(..., min_length=1)
    status: Literal["NEW", "PARTIALLY_FILLED", "FILLED", "CANCELED", "REJECTED", "EXPIRED"]
    accepted_ts_utc: datetime
    exchange: str = Field(..., min_length=1)
    symbol: str = Field(..., min_length=1)
    side: Literal["BUY", "SELL"]
    qty: float = Field(..., gt=0)
    filled_qty: float = Field(..., ge=0)
