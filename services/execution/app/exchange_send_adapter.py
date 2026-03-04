from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

from app.config import Settings
from app.exchange_gateway import send_live_payload


@dataclass(frozen=True)
class Adapter:
    send_func: Callable[[dict[str, Any]], Any]


def build_sender(settings: Settings) -> Adapter:
    return Adapter(send_func=lambda payload: send_live_payload(settings, payload))
