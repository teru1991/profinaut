from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from fastapi.testclient import TestClient

from contracts.observability.contract_constants import (
    HEADER_INSTANCE_ID,
    HEADER_OBS_SCHEMA,
    HEADER_OP,
    HEADER_RUN_ID,
)
from libs.observability.contracts import HealthCheck, HealthStatus
from libs.observability.http_contracts import build_capabilities_response, build_healthz_response


def test_headers_present_and_run_id_preserved():
    app = FastAPI()

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

    client = TestClient(app)
    run_id = "00000000-0000-0000-0000-0000000000aa"

    health = client.get("/healthz", headers={HEADER_RUN_ID: run_id})
    assert health.headers[HEADER_RUN_ID] == run_id
    assert HEADER_INSTANCE_ID in health.headers
    assert health.headers[HEADER_OP] == "healthz"
    assert HEADER_OBS_SCHEMA in health.headers

    cap = client.get("/capabilities")
    assert HEADER_RUN_ID in cap.headers
    assert HEADER_INSTANCE_ID in cap.headers
    assert cap.headers[HEADER_OP] == "capabilities"
    assert HEADER_OBS_SCHEMA in cap.headers
