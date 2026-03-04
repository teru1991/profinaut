import pytest

from libs.observability.logging import build_log_event


def test_strict_mode_missing_corr_fails(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_LOG_STRICT", "1")
    with pytest.raises(ValueError):
        build_log_event(
            level="INFO",
            msg="x",
            logger="t",
            service="s",
            op="op",
            corr=None,
            fields={},
        )


def test_strict_mode_with_corr_ok(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_LOG_STRICT", "1")
    corr = {
        "run_id": "00000000-0000-0000-0000-000000000001",
        "instance_id": "00000000-0000-0000-0000-000000000002",
        "trace_id": None,
        "op": "http_request",
        "schema_version": "obs.correlation.v1",
    }
    evt = build_log_event(
        level="INFO",
        msg="x",
        logger="t",
        service="s",
        op="op",
        corr=corr,
        fields={"hello": "world"},
    )
    assert evt["run_id"] == corr["run_id"]
    assert evt["instance_id"] == corr["instance_id"]
    assert evt["schema_version"] == "obs.log_event.v1"


def test_forbidden_key_masked(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_LOG_STRICT", "1")
    corr = {
        "run_id": "00000000-0000-0000-0000-000000000001",
        "instance_id": "00000000-0000-0000-0000-000000000002",
    }
    evt = build_log_event(
        level="INFO",
        msg="x",
        logger="t",
        service="s",
        op="op",
        corr=corr,
        fields={"token": "abc", "safe": "ok"},
    )
    assert evt["fields"]["token"] == "***"
    assert evt["fields"]["safe"] == "ok"
