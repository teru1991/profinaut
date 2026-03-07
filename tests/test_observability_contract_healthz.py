import json
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator, RefResolver


BASE = Path("docs/contracts/observability")


def _load_schema(name: str) -> dict:
    return json.loads((BASE / name).read_text(encoding="utf-8"))


def _validator(name: str) -> Draft202012Validator:
    schema = _load_schema(name)
    correlation = _load_schema("correlation.schema.json")
    resolver = RefResolver(
        base_uri=(BASE.resolve().as_uri() + "/"),
        referrer=schema,
        store={
            "https://profinaut.local/contracts/observability/correlation.schema.json": correlation,
            "correlation.schema.json": correlation,
        },
    )
    return Draft202012Validator(schema, resolver=resolver)


@pytest.mark.parametrize("status", ["OK", "DEGRADED", "FAILED", "UNKNOWN"])
def test_healthz_schema_accepts_all_statuses(status: str):
    validator = _validator("healthz.schema.json")
    sample = {
        "schema_version": "obs.healthz.v1",
        "status": status,
        "checks": [
            {
                "name": "self",
                "status": status,
                "reason_code": "OK" if status == "OK" else "UNKNOWN",
                "summary": "status sample",
                "observed_at": "2026-01-01T00:00:00Z",
                "details": {},
            }
        ],
        "correlation": {
            "schema_version": "obs.correlation.v1",
            "trace_id": "trace-0000000000000001",
            "run_id": "run-0000000000000001",
            "component": "marketdata",
            "source": "services.marketdata",
            "request_id": "req-0000000000000001",
            "instance_id": "instance-0000000000000001",
            "op": "healthz",
            "emitted_at": "2026-01-01T00:00:00Z",
        },
    }
    assert sorted(validator.iter_errors(sample), key=lambda err: list(err.path)) == []


def test_healthz_schema_rejects_extra_fields():
    validator = _validator("healthz.schema.json")
    bad = {
        "schema_version": "obs.healthz.v1",
        "status": "OK",
        "checks": [
            {
                "name": "self",
                "status": "OK",
                "reason_code": "OK",
                "summary": "ok",
                "observed_at": "2026-01-01T00:00:00Z",
                "details": {},
                "oops": 1,
            }
        ],
        "correlation": {
            "schema_version": "obs.correlation.v1",
            "trace_id": "trace-0000000000000001",
            "run_id": "run-0000000000000001",
            "component": "marketdata",
            "source": "services.marketdata",
            "request_id": "req-0000000000000001",
            "instance_id": "instance-0000000000000001",
            "op": "healthz",
            "emitted_at": "2026-01-01T00:00:00Z",
        },
    }
    assert list(validator.iter_errors(bad))


def test_unknown_checks_aggregate_to_unknown():
    from fastapi import FastAPI, Request
    from libs.observability.contracts import HealthCheck, HealthStatus
    from libs.observability.http_contracts import build_healthz_response

    app = FastAPI()

    @app.get("/h")
    def endpoint(request: Request):
        checks = [
            HealthCheck(
                name="dep",
                status=HealthStatus.UNKNOWN,
                reason_code="NOT_IMPLEMENTED",
                summary="missing dep",
                observed_at="2026-01-01T00:00:00Z",
            )
        ]
        body, _ = build_healthz_response(request, checks)
        return body

    from fastapi.testclient import TestClient

    response = TestClient(app).get("/h")
    assert response.json()["status"] == "UNKNOWN"
    assert response.json()["checks"][0]["reason_code"] == "NOT_IMPLEMENTED"
