from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Any, Literal, Optional

Lane = Literal["LANE0_CANCEL", "LANE0_FLATTEN", "LANE1_REPLACE", "LANE2_NEW"]
OutboxStatus = Literal["PENDING", "INFLIGHT", "SENT", "FAILED", "BLOCKED"]

OrderSide = Literal["BUY", "SELL"]
OrderType = Literal["LIMIT", "MARKET"]
TimeInForce = Literal["GTC", "IOC", "FOK"]


@dataclass(frozen=True)
class OrderIntentRecord:
    intent_id: str
    ts_utc: datetime
    venue: str
    symbol: str
    side: OrderSide
    order_type: OrderType
    qty: str
    price: Optional[str]
    tif: TimeInForce
    client_order_id: str
    dedupe_key: str
    lane: Lane


@dataclass(frozen=True)
class GateEvidence:
    decision: str
    reason_code: str
    evidence: dict[str, Any]


@dataclass(frozen=True)
class OutboxItem:
    outbox_id: str
    lane: Lane
    venue: str
    symbol: str
    payload_json: str
    status: OutboxStatus
    dedupe_key: str
    attempt: int
    next_attempt_at_utc: datetime
