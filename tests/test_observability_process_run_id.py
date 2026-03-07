from __future__ import annotations

from fastapi import FastAPI
from fastapi.testclient import TestClient

from contracts.observability.contract_constants import HEADER_REQUEST_ID, HEADER_RUN_ID
from libs.observability.middleware import install_correlation_middleware


def _app(*, trusted: bool) -> FastAPI:
    app = FastAPI()
    install_correlation_middleware(
        app,
        component="testsvc",
        source="services.testsvc",
        strict=True,
        trusted_run_id_override=trusted,
    )

    @app.get("/ping")
    def ping() -> dict[str, bool]:
        return {"ok": True}

    return app


def test_process_run_id_stable_and_request_id_changes() -> None:
    client = TestClient(_app(trusted=False))
    first = client.get("/ping")
    second = client.get("/ping")

    assert first.headers[HEADER_RUN_ID] == second.headers[HEADER_RUN_ID]
    assert first.headers[HEADER_REQUEST_ID] != second.headers[HEADER_REQUEST_ID]


def test_untrusted_inbound_run_id_is_ignored() -> None:
    client = TestClient(_app(trusted=False))
    baseline = client.get("/ping")
    injected = client.get("/ping", headers={HEADER_RUN_ID: "run-external-override"})
    assert injected.headers[HEADER_RUN_ID] == baseline.headers[HEADER_RUN_ID]


def test_trusted_valid_inbound_run_id_is_accepted() -> None:
    client = TestClient(_app(trusted=True))
    response = client.get("/ping", headers={HEADER_RUN_ID: "run-trusted-0001"})
    assert response.headers[HEADER_RUN_ID] == "run-trusted-0001"


def test_invalid_inbound_run_id_falls_back_to_process_run_id() -> None:
    client = TestClient(_app(trusted=True))
    baseline = client.get("/ping")
    response = client.get("/ping", headers={HEADER_RUN_ID: "bad"})
    assert response.headers[HEADER_RUN_ID] == baseline.headers[HEADER_RUN_ID]
