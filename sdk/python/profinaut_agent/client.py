from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

import requests

from .models import CommandAck


class ControlPlaneClient:
    def __init__(self, base_url: str, timeout_seconds: float = 5.0) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds

    def send_heartbeat(self, heartbeat: dict[str, Any]) -> None:
        response = requests.post(f"{self.base_url}/ingest/heartbeat", json=heartbeat, timeout=self.timeout_seconds)
        response.raise_for_status()

    def send_ack(self, ack: CommandAck) -> None:
        response = requests.post(
            f"{self.base_url}/commands/{ack.command_id}/ack",
            json=ack.as_dict(),
            timeout=self.timeout_seconds,
        )
        response.raise_for_status()

    def place_order(self, order: dict[str, Any]) -> None:
        response = requests.post(
            f"{self.base_url}/orders/place",
            json=order,
            timeout=self.timeout_seconds,
        )
        response.raise_for_status()


def build_heartbeat(instance_id: str, bot_id: str, runtime_mode: str, exchange: str, symbol: str, version: str) -> dict[str, Any]:
    return {
        "instance_id": instance_id,
        "bot_id": bot_id,
        "runtime_mode": runtime_mode,
        "exchange": exchange,
        "symbol": symbol,
        "version": version,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "metadata": {},
    }
