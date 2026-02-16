from __future__ import annotations

from collections.abc import Mapping
from typing import Any

_SENSITIVE_KEYS = {"payload", "payload_json", "payload_jsonb", "headers", "headers_json", "raw_body"}


def scrub_sensitive_fields(fields: Mapping[str, Any]) -> dict[str, Any]:
    """Best-effort guard to avoid logging large/sensitive payload bodies."""
    sanitized: dict[str, Any] = {}
    for key, value in fields.items():
        if key in _SENSITIVE_KEYS:
            sanitized[key] = "<redacted>"
        else:
            sanitized[key] = value
    return sanitized
