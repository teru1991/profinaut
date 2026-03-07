from __future__ import annotations

import pytest

from libs.observability.logging import build_log_event, validate_log_event


def _corr() -> dict[str, str]:
    return {
        "trace_id": "trace-0000000000000001",
        "run_id": "run-0000000000000001",
        "request_id": "req-0000000000000001",
        "component": "execution",
        "source": "services.execution",
    }


def test_build_log_event_contains_required_keys() -> None:
    evt = build_log_event(
        level="INFO",
        msg="hello",
        logger="obs.test",
        service="services.execution",
        op="http_request",
        corr=_corr(),
        fields={"path": "/healthz", "method": "GET"},
        strict=True,
    )
    for key in ["timestamp", "level", "message", "component", "trace_id", "run_id", "schema_version", "request_id"]:
        assert key in evt


def test_validator_fails_when_component_missing_in_strict_mode() -> None:
    bad = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "x",
        "trace_id": "trace-1",
        "run_id": "run-1",
    }
    with pytest.raises(ValueError):
        validate_log_event(bad, strict=True)


def test_validator_fails_when_trace_or_run_missing() -> None:
    bad_trace = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "x",
        "component": "execution",
        "run_id": "run-1",
    }
    with pytest.raises(ValueError):
        validate_log_event(bad_trace, strict=True)

    bad_run = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "x",
        "component": "execution",
        "trace_id": "trace-1",
    }
    with pytest.raises(ValueError):
        validate_log_event(bad_run, strict=True)


def test_request_scoped_log_requires_request_id() -> None:
    evt = {
        "schema_version": "obs.log_event.v1",
        "timestamp": "2026-01-01T00:00:00Z",
        "level": "INFO",
        "message": "x",
        "component": "execution",
        "trace_id": "trace-1",
        "run_id": "run-1",
    }
    with pytest.raises(ValueError):
        validate_log_event(evt, strict=True, request_scoped=True)


def test_event_log_keeps_event_id_and_error_codes() -> None:
    evt = build_log_event(
        level="ERROR",
        msg="event failed",
        logger="obs.test",
        service="services.execution",
        op="event",
        corr={**_corr(), "event_id": "evt-0000000000000001"},
        reason_code="UNHANDLED_EXCEPTION",
        error_code="PLATFORM_INTERNAL_ERROR",
        strict=True,
    )
    assert evt["event_id"] == "evt-0000000000000001"
    assert evt["error_code"] == "PLATFORM_INTERNAL_ERROR"
    assert evt["reason_code"] == "UNHANDLED_EXCEPTION"
