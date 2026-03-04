from __future__ import annotations

import os
from typing import Any

from app.config import Settings
from app.live import GmoLiveExecutor


def send_live_payload(settings: Settings, payload: dict[str, Any]) -> dict[str, Any]:
    exchange = str(payload.get("exchange", ""))
    if exchange != "gmo":
        raise ValueError(f"unsupported exchange: {exchange}")

    live = GmoLiveExecutor(
        base_url=settings.gmo_api_base_url,
        timeout_seconds=settings.gmo_request_timeout_seconds,
        api_key=os.getenv("GMO_API_KEY", ""),
        api_secret=os.getenv("GMO_API_SECRET", ""),
    )
    op = str(payload.get("op", ""))
    if op == "new_order":
        placed = live.place_order(
            symbol=str(payload["symbol"]),
            side=str(payload["side"]),
            qty=float(payload["qty"]),
            order_type=str(payload["order_type"]),
            limit_price=payload.get("limit_price"),
            client_order_id=str(payload["client_order_id"]),
        )
        return {"order_id": placed.order_id}
    if op == "cancel":
        live.cancel_order(order_id=str(payload["order_id"]), client_order_id=str(payload["client_order_id"]))
        return {"canceled": True}

    raise ValueError(f"unknown op: {op}")
