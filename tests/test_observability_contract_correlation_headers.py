from __future__ import annotations

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse, PlainTextResponse
from fastapi.testclient import TestClient

from contracts.observability.contract_constants import (
    HEADER_REQUEST_ID,
    HEADER_RUN_ID,
    HEADER_SCHEMA_VERSION,
    HEADER_TRACE_ID,
)
from libs.observability.contracts import HealthCheck, HealthStatus
from libs.observability.core import install_standard_error_handlers
from libs.observability.http_contracts import build_capabilities_response, build_healthz_response
from libs.observability.middleware import install_correlation_middleware


def _build_app() -> FastAPI:
    app = FastAPI()
    install_correlation_middleware(app, component="testsvc", source="services.testsvc", strict=True)
    install_standard_error_handlers(app, component="testsvc", source="services.testsvc")

    @app.get("/healthz")
    def healthz(request: Request) -> JSONResponse:
        body, headers = build_healthz_response(
            request,
            [
                HealthCheck(
                    name="self",
                    status=HealthStatus.OK,
                    reason_code="OK",
                    summary="ok",
                    observed_at="2026-01-01T00:00:00Z",
                    details={},
                )
            ],
        )
        return JSONResponse(content=body, headers=headers)

    @app.get("/capabilities")
    def capabilities(request: Request) -> JSONResponse:
        body, headers = build_capabilities_response(request, [])
        return JSONResponse(content=body, headers=headers)

    @app.get("/metrics")
    def metrics() -> PlainTextResponse:
        return PlainTextResponse("ok 1\n")

    @app.get("/error")
    def error() -> None:
        raise RuntimeError("boom")

    return app


def _assert_headers(response) -> None:
    assert HEADER_REQUEST_ID in response.headers
    assert HEADER_TRACE_ID in response.headers
    assert HEADER_RUN_ID in response.headers
    assert HEADER_SCHEMA_VERSION in response.headers


def test_correlation_headers_present_on_healthz_capabilities_metrics_and_error() -> None:
    client = TestClient(_build_app(), raise_server_exceptions=False)

    health = client.get("/healthz")
    _assert_headers(health)

    caps = client.get("/capabilities")
    _assert_headers(caps)

    metrics = client.get("/metrics")
    _assert_headers(metrics)

    error = client.get("/error")
    _assert_headers(error)
    body = error.json()
    assert body["error"]["context"]["request_id"] == error.headers[HEADER_REQUEST_ID]
    assert body["error"]["context"]["trace_id"] == error.headers[HEADER_TRACE_ID]
    assert body["error"]["context"]["run_id"] == error.headers[HEADER_RUN_ID]
