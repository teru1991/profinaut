from __future__ import annotations

import logging
import uuid
import contextvars
from typing import Any, Callable

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware

from libs.observability.logging import build_log_event, emit_json

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
    correlation = get_request_correlation_context() or {}
    if request_id and "run_id" not in correlation:
        correlation = {**correlation, "run_id": request_id}

    payload_fields = {"request_id": request_id, **_redact(fields)}
    payload_fields = _inject_correlation_fields(payload_fields)
    payload_fields.pop("run_id", None)
    payload_fields.pop("instance_id", None)
    payload_fields.pop("trace_id", None)
    payload_fields.pop("op", None)
    payload_fields.pop("schema_version", None)

    log_event = build_log_event(
        level="INFO",
        msg=event,
        logger="observability.audit",
        service=service,
        op=(correlation.get("op") if isinstance(correlation, dict) else None) or event,
        corr=correlation,
        fields=payload_fields,
    )
    emit_json(log_event)


def obs_emit(
    level: str,
    msg: str,
    logger: str,
    service: str,
    op: str,
    fields: dict[str, Any] | None = None,
    reason_code: str | None = None,
) -> None:
    corr = get_request_correlation_context()
    evt = build_log_event(
        level=level,
        msg=msg,
        logger=logger,
        service=service,
        op=op,
        corr=corr,
        fields=fields,
        reason_code=reason_code,
    )
    emit_json(evt)
