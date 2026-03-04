from __future__ import annotations

import time

_LAST_EVENT_AT: dict[str, float] = {}


def emit_audit_event(event_name: str, *, service: str, details: dict[str, object] | None = None, min_interval_s: int = 60) -> None:
    now = time.time()
    last = _LAST_EVENT_AT.get(event_name, 0.0)
    if now - last < min_interval_s:
        return
    _LAST_EVENT_AT[event_name] = now

    from libs.observability.core import audit_event

    audit_event(service=service, event=event_name, details=details or {})
