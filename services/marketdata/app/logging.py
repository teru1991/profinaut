from __future__ import annotations

from collections.abc import Mapping
from typing import Any

_SENSITIVE_KEYS = {"payload", "payload_json", "payload_jsonb", "headers", "headers_json", "raw_body"}
_SECRET_KEYWORDS = ("secret", "token", "password", "passphrase", "authorization", "api_key", "apikey")


def _is_secret_key(key: str) -> bool:
    normalized = key.strip().lower()
    if normalized == "key_id":
        return False
    return any(word in normalized for word in _SECRET_KEYWORDS)


def scrub_sensitive_fields(fields: Mapping[str, Any]) -> dict[str, Any]:
    """Best-effort guard to avoid logging large/sensitive payload bodies."""
    sanitized: dict[str, Any] = {}
    for key, value in fields.items():
        if key in _SENSITIVE_KEYS or _is_secret_key(key):
            sanitized[key] = "<redacted>"
        else:
            sanitized[key] = value
    return sanitized
