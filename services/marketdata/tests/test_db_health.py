from __future__ import annotations

from fastapi.testclient import TestClient

from services.marketdata.app.main import app


def test_healthz_includes_db_latency_when_db_ok(monkeypatch) -> None:
    from services.marketdata.app import main as app_main

    monkeypatch.setattr(app_main._db_checker, "ping", lambda: (True, 12.34, None))

    with TestClient(app) as client:
        response = client.get("/healthz")

    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "ok"
    assert body["db_ok"] is True
    assert isinstance(body["db_latency_ms"], float)


def test_capabilities_reports_db_unreachable_in_degraded_reasons(monkeypatch) -> None:
    from services.marketdata.app import main as app_main

    monkeypatch.setattr(app_main._db_checker, "ping", lambda: (False, None, "DB_UNREACHABLE"))

    with TestClient(app) as client:
        response = client.get("/capabilities")

    assert response.status_code == 200
    body = response.json()
    assert body["db_ok"] is False
    assert body["db_latency_ms"] is None
    assert "DB_UNREACHABLE" in body["degraded_reasons"]
    assert body["status"] == "degraded"
