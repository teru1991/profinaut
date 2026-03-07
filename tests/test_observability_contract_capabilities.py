import json
from pathlib import Path

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


def test_capabilities_schema_accepts_not_implemented():
    validator = _validator("capabilities.schema.json")
    sample = {
        "schema_version": "obs.capabilities.v1",
        "features": [
            {
                "name": "execution.place_order",
                "state": "NOT_IMPLEMENTED",
                "reasons": [{"code": "NOT_IMPLEMENTED", "message": "not wired yet"}],
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
            "op": "capabilities",
            "emitted_at": "2026-01-01T00:00:00Z",
        },
    }
    assert sorted(validator.iter_errors(sample), key=lambda err: list(err.path)) == []
