from __future__ import annotations

import logging
import uuid
from typing import Any, Callable

from fastapi import HTTPException, Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from pydantic import ValidationError
from starlette.middleware.base import BaseHTTPMiddleware

from libs.observability.correlation import CorrelationContext, current_correlation, set_current_correlation, to_response_headers
from libs.observability.logging import build_log_event, emit_json

LOGGER = logging.getLogger("observability")
SENSITIVE_KEYS = {"api_key", "apikey", "secret", "token", "password", "passphrase"}

_ERROR_MAP: dict[int, dict[str, Any]] = {
    400: {"code": "PLATFORM_BAD_REQUEST", "reason_code": "REQUEST_INVALID", "kind": "client_error", "severity": "warn", "retryable": False},
    401: {"code": "PLATFORM_UNAUTHORIZED", "reason_code": "AUTH_REQUIRED", "kind": "auth_error", "severity": "warn", "retryable": False},
    403: {"code": "PLATFORM_FORBIDDEN", "reason_code": "PERMISSION_DENIED", "kind": "permission_error", "severity": "warn", "retryable": False},
    404: {"code": "PLATFORM_NOT_FOUND", "reason_code": "RESOURCE_NOT_FOUND", "kind": "client_error", "severity": "info", "retryable": False},
    409: {"code": "PLATFORM_CONFLICT", "reason_code": "STATE_CONFLICT", "kind": "conflict_error", "severity": "warn", "retryable": False},
    422: {"code": "PLATFORM_VALIDATION_ERROR", "reason_code": "REQUEST_BODY_INVALID", "kind": "validation_error", "severity": "warn", "retryable": False},
    429: {"code": "PLATFORM_RATE_LIMITED", "reason_code": "TOO_MANY_REQUESTS", "kind": "rate_limit_error", "severity": "warn", "retryable": True},
    503: {"code": "PLATFORM_UPSTREAM_UNAVAILABLE", "reason_code": "UPSTREAM_UNAVAILABLE", "kind": "unavailable_error", "severity": "error", "retryable": True},
    504: {"code": "PLATFORM_UPSTREAM_TIMEOUT", "reason_code": "UPSTREAM_TIMEOUT", "kind": "timeout_error", "severity": "error", "retryable": True},
}


def _redact(data: dict[str, Any]) -> dict[str, Any]:
    redacted: dict[str, Any] = {}
    for key, value in data.items():
        redacted[key] = "***" if any(s in key.lower() for s in SENSITIVE_KEYS) else value
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
    return {"code": code, "message": message, "details": _redact(details or {}), "request_id": request_id}


def set_request_correlation_context(corr: dict[str, Any] | None) -> None:
    if corr is None:
        set_current_correlation(None)
        return
    try:
        ctx = CorrelationContext(
            trace_id=str(corr.get("trace_id") or f"trace-{uuid.uuid4()}"),
            run_id=str(corr.get("run_id") or f"run-{uuid.uuid4()}"),
            component=str(corr.get("component") or "unknown"),
            source=str(corr.get("source") or "unknown"),
            schema_version=str(corr.get("schema_version") or "obs.correlation.v1"),
            op=str(corr.get("op") or "unknown"),
            emitted_at=str(corr.get("emitted_at") or ""),
            instance_id=str(corr.get("instance_id") or "unknown"),
            request_id=str(corr.get("request_id")) if corr.get("request_id") else None,
            event_id=str(corr.get("event_id") or corr.get("event_uid")) if (corr.get("event_id") or corr.get("event_uid")) else None,
        )
        set_current_correlation(ctx)
    except Exception:
        set_current_correlation(None)


def get_request_correlation_context() -> dict[str, Any] | None:
    ctx = current_correlation()
    return ctx.to_dict() if ctx else None


def _inject_correlation_fields(payload: dict[str, Any]) -> dict[str, Any]:
    corr = get_request_correlation_context()
    if not corr:
        return payload
    for key in ["run_id", "trace_id", "request_id", "event_id", "schema_version", "component", "source"]:
        if key in corr and key not in payload:
            payload[key] = corr[key]
    return payload


def _extract_correlation_value(request: Request | None, key: str) -> str | None:
    if request is not None:
        corr = getattr(request.state, "correlation", None)
        if corr is not None and hasattr(corr, key):
            value = getattr(corr, key)
            if isinstance(value, str) and value:
                return value
        from_state = getattr(request.state, key, None)
        if isinstance(from_state, str) and from_state:
            return from_state
        header_key = key.replace("_", "-")
        from_header = request.headers.get(f"x-{header_key}", "").strip()
        if from_header:
            return from_header

    corr_ctx = current_correlation()
    if corr_ctx is not None and hasattr(corr_ctx, key):
        value = getattr(corr_ctx, key)
        return value if isinstance(value, str) and value else None
    return None


def build_error_context(
    *,
    component: str,
    request: Request | None = None,
    status_code: int | None = None,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    context: dict[str, Any] = {"component": component}
    for key in ["request_id", "trace_id", "run_id", "event_id"]:
        value = _extract_correlation_value(request, key)
        if value:
            context[key] = value
    if request is not None:
        context["path"] = request.url.path
        context["method"] = request.method
    if status_code is not None:
        context["status_code"] = status_code
    if extra:
        context.update(extra)
    return context


def _safe_message(exc: Exception, fallback: str) -> str:
    if isinstance(exc, HTTPException):
        detail = exc.detail
        if isinstance(detail, dict):
            message = detail.get("message")
            return str(message) if message else fallback
        if isinstance(detail, str):
            return detail
    return fallback


def classify_exception(
    exc: Exception,
    *,
    status_code: int | None = None,
    source: str,
    component: str,
    request: Request | None = None,
) -> dict[str, Any]:
    status = exc.status_code if isinstance(exc, HTTPException) else status_code

    if isinstance(exc, (RequestValidationError, ValidationError)):
        return {
            "code": "PLATFORM_VALIDATION_ERROR",
            "reason_code": "REQUEST_BODY_INVALID",
            "kind": "validation_error",
            "severity": "warn",
            "retryable": False,
            "source": source,
            "message": "request validation failed",
            "details": {"error_count": len(getattr(exc, "errors", lambda: [])() or [])},
            "context": build_error_context(component=component, request=request, status_code=status or 422),
        }

    if isinstance(exc, HTTPException):
        mapped = dict(_ERROR_MAP.get(status or 500, _ERROR_MAP[400 if (status or 500) < 500 else 503]))
        detail = exc.detail if isinstance(exc.detail, dict) else {}
        if isinstance(detail, dict) and detail.get("code"):
            mapped["code"] = str(detail["code"])
        if isinstance(detail, dict) and detail.get("reason_code"):
            mapped["reason_code"] = str(detail["reason_code"])

        details = _redact(dict(detail.get("details") or {})) if isinstance(detail, dict) else {}
        mapped.update(
            {
                "source": source,
                "message": _safe_message(exc, "request failed"),
                "details": details,
                "context": build_error_context(component=component, request=request, status_code=status),
            }
        )
        return mapped

    if isinstance(exc, TimeoutError):
        return {
            "code": "PLATFORM_UPSTREAM_TIMEOUT",
            "reason_code": "UPSTREAM_TIMEOUT",
            "kind": "timeout_error",
            "severity": "error",
            "retryable": True,
            "source": source,
            "message": "upstream request timed out",
            "context": build_error_context(component=component, request=request, status_code=status or 504),
        }

    if isinstance(exc, (ConnectionError, OSError)):
        return {
            "code": "PLATFORM_UPSTREAM_UNAVAILABLE",
            "reason_code": "UPSTREAM_UNAVAILABLE",
            "kind": "dependency_error",
            "severity": "error",
            "retryable": True,
            "source": source,
            "message": "upstream dependency unavailable",
            "context": build_error_context(component=component, request=request, status_code=status or 503),
        }

    return {
        "code": "PLATFORM_INTERNAL_ERROR",
        "reason_code": "UNHANDLED_EXCEPTION",
        "kind": "internal_error",
        "severity": "critical",
        "retryable": False,
        "source": source,
        "message": "internal server error",
        "context": build_error_context(component=component, request=request, status_code=status or 500),
    }


def build_error_envelope(error: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(error)
    details = normalized.get("details")
    if isinstance(details, dict):
        normalized["details"] = _redact(details)
    normalized.setdefault("schema_version", "1.0")
    normalized.setdefault("contract_version", "error-envelope.v1")
    return {"error": normalized}


def install_standard_error_handlers(app: Any, *, component: str, source: str) -> None:
    if not component or not source:
        raise ValueError("component and source are required")

    @app.exception_handler(HTTPException)
    async def _http_exc_handler(request: Request, exc: HTTPException) -> JSONResponse:
        error = classify_exception(exc, status_code=exc.status_code, source=source, component=component, request=request)
        response = JSONResponse(status_code=exc.status_code, content=build_error_envelope(error))
        corr = getattr(request.state, "correlation", None)
        if corr is not None:
            for key, value in to_response_headers(corr).items():
                response.headers.setdefault(key, value)
        return response

    @app.exception_handler(RequestValidationError)
    async def _validation_exc_handler(request: Request, exc: RequestValidationError) -> JSONResponse:
        error = classify_exception(exc, status_code=422, source=source, component=component, request=request)
        response = JSONResponse(status_code=422, content=build_error_envelope(error))
        corr = getattr(request.state, "correlation", None)
        if corr is not None:
            for key, value in to_response_headers(corr).items():
                response.headers.setdefault(key, value)
        return response

    @app.exception_handler(Exception)
    async def _generic_exc_handler(request: Request, exc: Exception) -> JSONResponse:
        LOGGER.exception("unhandled_exception", extra={"component": component, "source": source})
        error = classify_exception(exc, status_code=500, source=source, component=component, request=request)
        emit_json(
            build_log_event(
                level="ERROR",
                msg="unhandled_exception",
                logger="observability.error",
                service=source,
                op="http_request",
                corr=get_request_correlation_context(),
                strict=False,
                error_code=error.get("code"),
                reason_code=error.get("reason_code"),
            )
        )
        response = JSONResponse(status_code=500, content=build_error_envelope(error))
        corr_ctx = getattr(request.state, "correlation", None)
        if corr_ctx is not None:
            for key, value in to_response_headers(corr_ctx).items():
                response.headers.setdefault(key, value)
        return response


def audit_event(*, service: str, event: str, request_id: str | None = None, **fields: Any) -> None:
    correlation = get_request_correlation_context() or {}
    payload_fields = {"request_id": request_id, **_redact(fields)}
    payload_fields = _inject_correlation_fields(payload_fields)

    log_event = build_log_event(
        level="INFO",
        msg=event,
        logger="observability.audit",
        service=service,
        op=(correlation.get("op") if isinstance(correlation, dict) else None) or event,
        corr=correlation,
        fields=payload_fields,
        strict=False,
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
        strict=False,
    )
    emit_json(evt)
