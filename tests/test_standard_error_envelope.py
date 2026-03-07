from __future__ import annotations

from fastapi import Body, FastAPI, HTTPException
from fastapi.testclient import TestClient

from libs.observability.core import install_standard_error_handlers
from services.marketdata.app.main import _poller


def _app_for(component: str, source: str) -> FastAPI:
    app = FastAPI()
    install_standard_error_handlers(app, component=component, source=source)

    @app.get("/http")
    async def http_error() -> None:
        raise HTTPException(status_code=404, detail={"code": "RESOURCE_NOT_FOUND", "message": "missing"})

    @app.post("/validation")
    async def validation_error(payload: dict[str, int] = Body(...)) -> dict[str, int]:
        return payload

    @app.get("/boom")
    async def boom() -> None:
        raise RuntimeError("boom")

    return app


def _assert_canonical_envelope(body: dict, *, component: str, source: str) -> None:
    assert "error" in body
    err = body["error"]
    for key in ["code", "reason_code", "kind", "severity", "retryable", "source", "context"]:
        assert key in err
    assert err["source"] == source
    assert err["context"]["component"] == component


def test_handlers_return_same_shape_for_three_service_components() -> None:
    targets = [
        ("execution", "services.execution"),
        ("marketdata", "services.marketdata"),
        ("dashboard-api", "services.dashboard_api"),
    ]
    for component, source in targets:
        client = TestClient(_app_for(component, source), raise_server_exceptions=False)

        http_response = client.get("/http")
        assert http_response.status_code == 404
        _assert_canonical_envelope(http_response.json(), component=component, source=source)

        validation_response = client.post("/validation", json={"invalid": "x"})
        assert validation_response.status_code == 422
        _assert_canonical_envelope(validation_response.json(), component=component, source=source)
        assert validation_response.json()["error"]["kind"] == "validation_error"

        boom_response = client.get("/boom")
        assert boom_response.status_code == 500
        _assert_canonical_envelope(boom_response.json(), component=component, source=source)
        assert boom_response.json()["error"]["kind"] == "internal_error"


def test_context_ids_are_included_from_headers() -> None:
    client = TestClient(_app_for("execution", "services.execution"), raise_server_exceptions=False)
    response = client.get(
        "/http",
        headers={"x-request-id": "req-1", "x-trace-id": "trace-1", "x-run-id": "run-1"},
    )
    context = response.json()["error"]["context"]
    assert context["request_id"] == "req-1"
    assert context["trace_id"] == "trace-1"
    assert context["run_id"] == "run-1"


def test_install_fails_fast_without_component_or_source() -> None:
    app = FastAPI()
    try:
        install_standard_error_handlers(app, component="", source="services.execution")
        assert False, "expected ValueError"
    except ValueError:
        pass


def test_marketdata_degraded_payload_keeps_top_level_and_adds_standard_error() -> None:
    payload = _poller._degraded_payload(
        symbol="BTC_JPY",
        reason="UPSTREAM_UNREACHABLE",
        code="TICKER_NOT_READY",
        message="Ticker not ready",
    )
    for key in ["symbol", "stale", "degraded_reason", "error"]:
        assert key in payload
    err = payload["error"]
    for key in ["code", "reason_code", "kind", "severity", "retryable", "source", "context"]:
        assert key in err
    assert err["context"]["component"] == "marketdata"
