from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum

import requests


class Severity(StrEnum):
    INFO = "INFO"
    WARNING = "WARNING"
    CRITICAL = "CRITICAL"
    AUDIT = "AUDIT"


@dataclass(slots=True)
class NotificationEvent:
    severity: Severity
    title: str
    message: str
    timestamp: datetime
    metadata: dict


class NotificationRouter:
    def __init__(self, discord_webhook_url: str | None) -> None:
        self.discord_webhook_url = discord_webhook_url

    def route(self, event: NotificationEvent) -> bool:
        # Severity routing skeleton (extend with channel-specific routing later)
        if event.severity in {Severity.INFO, Severity.WARNING, Severity.AUDIT}:
            return self._send_discord(event)
        if event.severity == Severity.CRITICAL:
            return self._send_discord(event)
        return False

    def _send_discord(self, event: NotificationEvent) -> bool:
        if not self.discord_webhook_url:
            return False

        payload = {
            "content": (
                f"[{event.severity.value}] {event.title}\n"
                f"{event.message}\n"
                f"time={event.timestamp.isoformat()}"
            )
        }
        response = requests.post(self.discord_webhook_url, json=payload, timeout=5)
        return 200 <= response.status_code < 300
