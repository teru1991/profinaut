from __future__ import annotations

from collections.abc import Mapping
import re
from typing import Any

_SECRET_KEY_RE = re.compile(r"(?i)(token|secret|private[_-]?key|api[_-]?key|password)")
_LONG_HEX_RE = re.compile(r"\b[a-fA-F0-9]{32,}\b")
_ADDRESS_LIKE_RE = re.compile(r"\b(?:0x)?[A-Za-z0-9]{20,}\b")


def _mask(text: str) -> str:
    if len(text) <= 6:
        return "***"
    return f"{text[:3]}***{text[-3:]}"


def redact(value: Any) -> Any:
    if isinstance(value, Mapping):
        out: dict[str, Any] = {}
        for k, v in value.items():
            if _SECRET_KEY_RE.search(str(k)):
                out[str(k)] = "***REDACTED***"
            else:
                out[str(k)] = redact(v)
        return out
    if isinstance(value, list):
        return [redact(v) for v in value]
    if isinstance(value, tuple):
        return tuple(redact(v) for v in value)
    if isinstance(value, str):
        if _LONG_HEX_RE.search(value) or _ADDRESS_LIKE_RE.search(value):
            return _mask(value)
        return value
    return value
