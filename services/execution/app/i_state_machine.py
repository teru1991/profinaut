from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, Optional

OrderState = Literal[
    "PENDING_ACK",
    "LIVE",
    "PARTIALLY_FILLED",
    "FILLED",
    "CANCELED",
    "REJECTED",
    "UNKNOWN",
]


@dataclass
class OrderModel:
    client_order_id: str
    venue_order_id: Optional[str]
    state: OrderState
    filled_qty: str
    last_reason: Optional[str] = None


def apply_event(order: OrderModel, event: dict) -> OrderModel:
    kind = event.get("kind")
    if kind == "ORDER_SUBMITTED":
        order.state = "PENDING_ACK"
    elif kind == "ORDER_ACK":
        order.venue_order_id = event.get("venue_order_id")
        order.state = "LIVE"
    elif kind == "ORDER_REJECT":
        order.state = "REJECTED"
        order.last_reason = event.get("reason")
    elif kind == "ORDER_FILL":
        order.filled_qty = event.get("filled_qty", order.filled_qty)
        order.state = "FILLED" if event.get("is_final") else "PARTIALLY_FILLED"
    elif kind == "ORDER_CANCELED":
        order.state = "CANCELED"
    elif kind == "ORDER_UNKNOWN":
        order.state = "UNKNOWN"
    return order
