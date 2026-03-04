from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from fastapi.testclient import TestClient

from libs.observability import budget
from libs.observability.contracts import HealthCheck, HealthStatus
from libs.observability.http_contracts import build_healthz_response


def test_budget_exceeded_adds_health_check(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_BUDGET_STRICT", "0")
    budget.reset_for_tests()
    budget.mark_metrics_exceeded()
    try:
        app = FastAPI()

        @app.get("/healthz")
        def healthz(request: Request):
            chk = HealthCheck(
                name="self",
                status=HealthStatus.OK,
                reason_code="OK",
                summary="ok",
                observed_at="2026-01-01T00:00:00Z",
                details={},
            )
            body, headers = build_healthz_response(request, [chk])
            resp = JSONResponse(content=body)
            for k, v in headers.items():
                resp.headers[k] = v
            return resp

        response = TestClient(app).get("/healthz")
        checks = response.json()["checks"]
        names = [item["name"] for item in checks]
        assert "observability_budget" in names
    finally:
        budget.reset_for_tests()
