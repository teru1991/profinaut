from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from fastapi.testclient import TestClient

from libs.observability.contracts import HealthCheck, HealthStatus
from libs.observability.http_contracts import build_healthz_response
from libs.observability.http_sanitize import sanitize_health_check_details


def test_healthz_details_sanitized_no_bearer_leak():
    app = FastAPI()

    @app.get("/healthz")
    def healthz(request: Request):
        details = {"authorization": "Bearer abc.def.ghi", "ok": "x"}
        details_sanitized, violation_count, _keys = sanitize_health_check_details(details)

        status = HealthStatus.OK
        reason = "OK"
        summary = "ok"
        if violation_count > 0:
            status = HealthStatus.DEGRADED
            reason = "REDACTION_VIOLATION"
            summary = "ok (sanitized)"

        check = HealthCheck(
            name="self",
            status=status,
            reason_code=reason,
            summary=summary,
            observed_at="2026-01-01T00:00:00Z",
            details=details_sanitized,
        )
        body, headers = build_healthz_response(request, [check])
        response = JSONResponse(content=body)
        for key, value in headers.items():
            response.headers[key] = value
        return response

    client = TestClient(app)
    response = client.get("/healthz")
    text = response.text
    assert "Bearer" not in text
    assert "abc.def.ghi" not in text
    payload = response.json()
    assert payload["checks"][0]["status"] in ["DEGRADED", "OK"]
