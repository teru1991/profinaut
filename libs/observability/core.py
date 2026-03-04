from __future__ import annotations

import json
import logging
import uuid
import contextvars
from datetime import UTC, datetime
from typing import Any, Callable

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware

LOGGER = logging.getLogger("observability")
SENSITIVE_KEYS = {"api_key", "apikey", "secret", "token", "password", "passphrase"}
_obs_correlation_ctx: contextvars.ContextVar[dict[str, Any] | None] = contextvars.ContextVar(
    "obs_correlation", default=None
)


def _redact(data: dict[str, Any]) -> dict[str, Any]:
    redacted: dict[str, Any] = {}
    for key, value in data.items():
        if any(s in key.lower() for s in SENSITIVE_KEYS):
            redacted[key] = "***"
        else:
            redacted[key] = value
    return redacted


def extract_request_id(request: Request) -> str:
    header_value = request.headers.get("x-request-id", "").strip()
    if header_value:
        return header_value
    return str(uuid.uuid4())


def request_id_middleware() -> type[BaseHTTPMiddleware]:
    class RequestIDMiddleware(BaseHTTPMiddleware):
        async def dispatch(self, request: Request, call_next: Callable):
            request_id = extract_request_id(request)
            request.state.request_id = request_id
            response = await call_next(request)
            response.headers["x-request-id"] = request_id
            return response

    return RequestIDMiddleware


def error_envelope(*, code: str, message: str, request_id: str, details: dict[str, Any] | None = None) -> dict[str, Any]:
    return {
        "code": code,
        "message": message,
        "details": _redact(details or {}),
        "request_id": request_id,
    }


def set_request_correlation_context(corr: dict[str, Any] | None) -> None:
    _obs_correlation_ctx.set(corr)


def get_request_correlation_context() -> dict[str, Any] | None:
    return _obs_correlation_ctx.get()


def _inject_correlation_fields(payload: dict[str, Any]) -> dict[str, Any]:
    corr = get_request_correlation_context()
    if not corr:
        return payload
    for key in ["run_id", "instance_id", "trace_id", "op", "schema_version"]:
        if key in corr and key not in payload:
            payload[key] = corr[key]
    return payload


def audit_event(*, service: str, event: str, request_id: str | None = None, **fields: Any) -> None:
    payload = {
        "ts_utc": datetime.now(UTC).isoformat(),
        "service": service,
        "event": event,
        "request_id": request_id,
        **_redact(fields),
    }
    payload = _inject_correlation_fields(payload)
    LOGGER.info(json.dumps(payload, ensure_ascii=False))
