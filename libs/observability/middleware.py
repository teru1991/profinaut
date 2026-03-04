from __future__ import annotations

import time

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import Response

from contracts.observability.contract_constants import SCHEMA_VERSION_CORRELATION
from libs.observability.correlation import make_correlation, set_correlation_response_headers
from libs.observability.core import set_request_correlation_context
from libs.observability.logging import build_log_event, emit_json
from libs.observability.metrics import observe_http_request


class ObservabilityMiddleware(BaseHTTPMiddleware):
    def __init__(self, app, service_name: str) -> None:
        super().__init__(app)
        self.service_name = service_name or "unknown_service"

    async def dispatch(self, request: Request, call_next):
        started = time.perf_counter()
        corr = make_correlation(op="http_request", request_headers=dict(request.headers))
        set_request_correlation_context(corr)
        try:
            response: Response = await call_next(request)
            set_correlation_response_headers(response.headers, corr, [SCHEMA_VERSION_CORRELATION])

            duration_ms = int((time.perf_counter() - started) * 1000)
            observe_http_request(
                service=self.service_name,
                path=request.url.path,
                method=request.method,
                status=str(int(response.status_code)),
            )

            event = build_log_event(
                level="INFO",
                msg="http_request",
                logger="obs.middleware",
                service=self.service_name,
                op="http_request",
                corr=corr,
                fields={
                    "method": request.method,
                    "path": request.url.path,
                    "status": int(response.status_code),
                    "duration_ms": duration_ms,
                },
            )
            emit_json(event)
            return response
        finally:
            set_request_correlation_context(None)
