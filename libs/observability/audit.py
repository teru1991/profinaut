from __future__ import annotations

from typing import Any

from libs.observability.core import get_request_correlation_context
from libs.observability.logging import build_log_event, emit_json


def emit_audit_event(event_type: str, fields: dict[str, Any], *, service: str = "unknown") -> None:
    corr = get_request_correlation_context() or {}
    event = build_log_event(
        level="WARN",
        msg=f"audit:{event_type}",
        logger="obs.audit",
        service=service,
        op="audit",
        corr=corr,
        fields=fields,
        reason_code="AUDIT",
    )
    emit_json(event)
