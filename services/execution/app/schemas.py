from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class StrictBaseModel(BaseModel):
    model_config = ConfigDict(extra="forbid")


class HealthResponse(StrictBaseModel):
    status: str
    timestamp: datetime


class CapabilitiesResponse(StrictBaseModel):
    service: str = "execution"
    version: str
    status: Literal["ok", "degraded"] = "ok"
    features: list[str]
    degraded_reason: str | None = None
    generated_at: datetime


class OrderIntent(StrictBaseModel):
    idempotency_key: str = Field(..., min_length=1)
    exchange: str = Field(..., min_length=1)
    symbol: str = Field(..., min_length=1)
    side: Literal["BUY", "SELL"]
    qty: float = Field(..., gt=0)
    type: Literal["MARKET", "LIMIT"]
    limit_price: float | None = Field(None, gt=0)
    client_ts_utc: datetime | None = None


class Order(StrictBaseModel):
    order_id: str = Field(..., min_length=1)
    status: Literal["ACCEPTED", "FILLED", "CANCELED", "REJECTED"]
    accepted_ts_utc: datetime
    exchange: str = Field(..., min_length=1)
    symbol: str = Field(..., min_length=1)
    side: Literal["BUY", "SELL"]
    qty: float = Field(..., gt=0)
    filled_qty: float = Field(..., ge=0)


class Fill(StrictBaseModel):
    fill_id: str = Field(..., min_length=1)
    order_id: str = Field(..., min_length=1)
    symbol: str = Field(..., min_length=1)
    side: Literal["BUY", "SELL"]
    qty: float = Field(..., gt=0)
    ts_utc: datetime


class OrdersHistoryResponse(StrictBaseModel):
    items: list[Order]
    page: int = Field(..., ge=1)
    page_size: int = Field(..., ge=1)
    total: int = Field(..., ge=0)


class FillsHistoryResponse(StrictBaseModel):
    items: list[Fill]
    page: int = Field(..., ge=1)
    page_size: int = Field(..., ge=1)
    total: int = Field(..., ge=0)
