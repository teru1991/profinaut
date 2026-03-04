import json

from fastapi import FastAPI
from fastapi.testclient import TestClient

from contracts.observability.contract_constants import HEADER_INSTANCE_ID, HEADER_OP, HEADER_RUN_ID
from libs.observability.middleware import ObservabilityMiddleware


def test_middleware_sets_headers_and_emits_json_log(capsys, monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_LOG_STRICT", "1")

    app = FastAPI()
    app.add_middleware(ObservabilityMiddleware, service_name="testsvc")

    @app.get("/ping")
    def ping():
        return {"ok": True}

    client = TestClient(app)
    response = client.get("/ping")
    assert response.status_code == 200
    assert HEADER_RUN_ID in response.headers
    assert HEADER_INSTANCE_ID in response.headers
    assert HEADER_OP in response.headers

    out = capsys.readouterr().out.strip().splitlines()
    assert out
    evt = json.loads(out[-1])
    assert evt["schema_version"] == "obs.log_event.v1"
    assert evt["service"] == "testsvc"
    assert evt["op"] == "http_request"
    assert "run_id" in evt and "instance_id" in evt
    assert evt["fields"]["path"] == "/ping"
