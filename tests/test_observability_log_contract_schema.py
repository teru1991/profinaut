import json
from pathlib import Path

from jsonschema import Draft202012Validator


def _load_schema(name: str) -> dict:
    path = Path("docs/contracts/observability") / name
    return json.loads(path.read_text(encoding="utf-8"))


def test_log_event_schema_accepts_minimal():
    schema = _load_schema("log_event.schema.json")
    validator = Draft202012Validator(schema)
    sample = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "hello",
        "component": "execution",
        "source": "services.execution",
        "logger_name": "test",
        "op": "healthz",
        "run_id": "run-0000000000000001",
        "trace_id": "trace-0000000000000001",
        "request_id": "req-0000000000000001",
        "fields": {"k": "v"},
    }
    assert list(validator.iter_errors(sample)) == []


def test_log_event_schema_rejects_missing_required():
    schema = _load_schema("log_event.schema.json")
    validator = Draft202012Validator(schema)
    bad = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "hello",
        "component": "execution",
        "run_id": "run-0000000000000001",
    }
    assert list(validator.iter_errors(bad))
