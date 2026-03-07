from __future__ import annotations

import time
from typing import Any

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import Response

from contracts.observability.contract_constants import SCHEMA_VERSION_CORRELATION
from libs.observability.correlation import (
    bind_correlation,
    ensure_request_correlation,
    reset_correlation,
    to_response_headers,
)
from libs.observability.logging import build_log_event, emit_json
from libs.observability.metrics import observe_http_request


class CorrelationMiddleware(BaseHTTPMiddleware):
    def __init__(
        self,
        app: Any,
        *,
        component: str,
        source: str,
        strict: bool = True,
        trusted_run_id_override: bool = False,
    ) -> None:
        super().__init__(app)
        self.component = component
        self.source = source
        self.strict = strict
        self.trusted_run_id_override = trusted_run_id_override

    async def dispatch(self, request: Request, call_next):
        started = time.perf_counter()
        ctx = ensure_request_correlation(
            request.headers,
            component=self.component,
            source=self.source,
            op="http_request",
            trusted_run_id_override=self.trusted_run_id_override,
        )

        request.state.request_id = ctx.request_id
        request.state.trace_id = ctx.trace_id
        request.state.run_id = ctx.run_id
        request.state.correlation = ctx

        token = bind_correlation(ctx)
        emit_json(
            build_log_event(
                level="INFO",
                msg="request_started",
                logger="obs.middleware",
                service=self.source,
                op="http_request",
                corr=ctx.to_dict(),
                fields={"path": request.url.path, "method": request.method},
                strict=self.strict,
            )
        )

        try:
            response: Response = await call_next(request)
            duration_ms = int((time.perf_counter() - started) * 1000)
            observe_http_request(
                service=self.component,
                path=request.url.path,
                method=request.method,
                status=str(int(response.status_code)),
            )
            for key, value in to_response_headers(ctx, [SCHEMA_VERSION_CORRELATION]).items():
                response.headers.setdefault(key, value)

            message = "request_failed" if int(response.status_code) >= 500 else "request_finished"
            level = "ERROR" if int(response.status_code) >= 500 else "INFO"
            emit_json(
                build_log_event(
                    level=level,
                    msg=message,
                    logger="obs.middleware",
                    service=self.source,
                    op="http_request",
                    corr=ctx.to_dict(),
                    fields={
                        "path": request.url.path,
                        "method": request.method,
                        "status_code": int(response.status_code),
                        "duration_ms": duration_ms,
                    },
                    strict=self.strict,
                    error_code="PLATFORM_INTERNAL_ERROR" if int(response.status_code) >= 500 else None,
                    reason_code="UNHANDLED_EXCEPTION" if int(response.status_code) >= 500 else None,
                )
            )
            return response
        finally:
            reset_correlation(token)


class ObservabilityMiddleware(CorrelationMiddleware):
    def __init__(self, app: Any, service_name: str) -> None:
        super().__init__(app, component=service_name or "unknown_service", source=service_name or "unknown_service")


def install_correlation_middleware(
    app: Any,
    *,
    component: str,
    source: str,
    strict: bool = True,
    trusted_run_id_override: bool = False,
) -> None:
    app.add_middleware(
        CorrelationMiddleware,
        component=component,
        source=source,
        strict=strict,
        trusted_run_id_override=trusted_run_id_override,
    )
