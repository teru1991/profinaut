from __future__ import annotations

from typing import Any

from libs.observability.redaction import sanitize


def sanitize_health_check_details(details: dict[str, Any]) -> tuple[dict[str, Any], int, list[str]]:
    sanitized, violations = sanitize(details)
    keys = sorted({str(v.get("key")) for v in violations if v.get("kind") == "key" and v.get("key")})
    if isinstance(sanitized, dict):
        return sanitized, len(violations), keys
    return {"sanitized": True}, len(violations), keys


def sanitize_capability_reasons(reasons: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], int, list[str]]:
    sanitized, violations = sanitize(reasons)
    keys = sorted({str(v.get("key")) for v in violations if v.get("kind") == "key" and v.get("key")})
    if isinstance(sanitized, list):
        return sanitized, len(violations), keys
    return [], len(violations), keys
