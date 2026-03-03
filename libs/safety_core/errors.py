from __future__ import annotations

from typing import Any

from libs.safety_core.redaction import safe_str


class SecError(Exception):
    def __init__(self, code: str, message: str, details: dict[str, Any] | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details

    def __str__(self) -> str:
        base = f"{self.code}: {self.message}"
        if self.details:
            return base + " " + safe_str(self.details)
        return base


def err(code: str, message: str, **details: Any) -> SecError:
    return SecError(code=code, message=message, details=details or None)


E_SECRETREF_PARSE = "E_SECRETREF_PARSE"
E_SECRETREF_INVALID = "E_SECRETREF_INVALID"
E_SECRET_PROVIDER_DISABLED = "E_SECRET_PROVIDER_DISABLED"
E_SECRET_PROVIDER_NOT_CONFIGURED = "E_SECRET_PROVIDER_NOT_CONFIGURED"
E_SECRET_REGISTRY_DENY = "E_SECRET_REGISTRY_DENY"
E_SECRET_ENV_FORBIDDEN = "E_SECRET_ENV_FORBIDDEN"
E_SECRET_RESOLVE_FAILED = "E_SECRET_RESOLVE_FAILED"
