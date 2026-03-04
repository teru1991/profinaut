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
        "ts": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "msg": "hello",
        "logger": "test",
        "service": "execution",
        "op": "healthz",
        "run_id": "00000000-0000-0000-0000-000000000001",
        "instance_id": "00000000-0000-0000-0000-000000000002",
        "trace_id": None,
        "fields": {"k": "v"},
    }
    assert list(validator.iter_errors(sample)) == []


def test_log_event_schema_rejects_extra():
    schema = _load_schema("log_event.schema.json")
    validator = Draft202012Validator(schema)
    bad = {
        "schema_version": "obs.log_event.v1",
        "ts": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "msg": "hello",
        "logger": "test",
        "service": "execution",
        "op": "healthz",
        "run_id": "00000000-0000-0000-0000-000000000001",
        "instance_id": "00000000-0000-0000-0000-000000000002",
        "oops": 1,
    }
    assert list(validator.iter_errors(bad))
