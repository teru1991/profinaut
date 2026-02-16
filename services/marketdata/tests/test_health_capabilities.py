from __future__ import annotations

from fastapi.testclient import TestClient

from services.marketdata.app import main


async def _idle_poller() -> None:
    return None


def test_capabilities_safe_defaults(monkeypatch) -> None:
    monkeypatch.delenv("DB_DSN", raising=False)
    monkeypatch.delenv("OBJECT_STORE_BACKEND", raising=False)
    monkeypatch.delenv("SILVER_ENABLED", raising=False)
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        response = client.get("/capabilities")

    assert response.status_code == 200
    payload = response.json()
    assert payload["ingest_raw_enabled"] is False
    assert payload["silver_enabled"] is False
    assert payload["storage_backend"] is None
    assert payload["db_enabled"] is False
    assert payload["degraded"] is True
    assert payload["degraded_reasons"] == ["STORAGE_NOT_CONFIGURED", "DB_NOT_CONFIGURED"]


def test_capabilities_enabled_when_db_and_storage_configured(monkeypatch) -> None:
    monkeypatch.setenv("DB_DSN", "postgresql://user:pass@db:5432/profinaut")
    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("SILVER_ENABLED", "1")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        response = client.get("/capabilities")

    assert response.status_code == 200
    payload = response.json()
    assert payload["ingest_raw_enabled"] is True
    assert payload["silver_enabled"] is True
    assert payload["storage_backend"] == "fs"
    assert payload["db_enabled"] is True
    assert payload["degraded"] is False
    assert payload["degraded_reasons"] == []


def test_healthz_returns_degraded_without_dependencies(monkeypatch) -> None:
    monkeypatch.delenv("DB_DSN", raising=False)
    monkeypatch.delenv("OBJECT_STORE_BACKEND", raising=False)
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        response = client.get("/healthz")

    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] == "degraded"
    assert isinstance(payload["checks"], list)
    assert {check["name"] for check in payload["checks"]} == {"object_store", "db"}
    assert isinstance(payload["ts"], str)
