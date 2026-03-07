from __future__ import annotations

import json

from fastapi import FastAPI, Request
from fastapi.testclient import TestClient

from contracts.observability.contract_constants import HEADER_REQUEST_ID, HEADER_RUN_ID, HEADER_TRACE_ID
from libs.observability.correlation import current_correlation
from libs.observability.core import install_standard_error_handlers
from libs.observability.middleware import install_correlation_middleware


def test_middleware_injects_headers_and_emits_start_finish_logs(capsys):
    app = FastAPI()
    install_correlation_middleware(app, component="testsvc", source="services.testsvc", strict=True)
    install_standard_error_handlers(app, component="testsvc", source="services.testsvc")

    @app.get("/ping")
    def ping(request: Request):
        assert getattr(request.state, "correlation", None) is not None
        return {"ok": True}

    client = TestClient(app)
    response = client.get("/ping")
    assert response.status_code == 200
    assert HEADER_REQUEST_ID in response.headers
    assert HEADER_TRACE_ID in response.headers
    assert HEADER_RUN_ID in response.headers

    logs = [json.loads(line) for line in capsys.readouterr().out.strip().splitlines() if line.strip()]
    assert [evt["message"] for evt in logs] == ["request_started", "request_finished"]
    for evt in logs:
        assert evt["request_id"] == response.headers[HEADER_REQUEST_ID]
        assert evt["trace_id"] == response.headers[HEADER_TRACE_ID]
        assert evt["run_id"] == response.headers[HEADER_RUN_ID]
    assert "duration_ms" in logs[-1]["fields"]


def test_exception_path_keeps_correlation_and_context_is_reset(capsys):
    app = FastAPI()
    install_correlation_middleware(app, component="testsvc", source="services.testsvc", strict=True)
    install_standard_error_handlers(app, component="testsvc", source="services.testsvc")

    @app.get("/boom")
    def boom():
        raise RuntimeError("boom")

    client = TestClient(app, raise_server_exceptions=False)
    response = client.get("/boom")
    assert response.status_code == 500
    assert HEADER_REQUEST_ID in response.headers

    logs = [json.loads(line) for line in capsys.readouterr().out.strip().splitlines() if line.strip()]
    assert any(evt["message"] in {"request_failed", "unhandled_exception", "request_finished"} for evt in logs)
    assert current_correlation() is None
