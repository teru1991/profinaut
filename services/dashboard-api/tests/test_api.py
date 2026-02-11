from datetime import datetime, timedelta, timezone


def test_healthz(client):
    response = client.get("/healthz")
    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] == "ok"


def test_heartbeat_upsert_and_bots_list(client):
    now = datetime.now(timezone.utc).isoformat()
    heartbeat = {
        "instance_id": "inst-1",
        "bot_id": "bot-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now,
        "metadata": {"region": "local"},
    }

    r1 = client.post("/ingest/heartbeat", json=heartbeat)
    assert r1.status_code == 202

    heartbeat["version"] = "1.0.1"
    heartbeat["timestamp"] = (datetime.now(timezone.utc) + timedelta(seconds=2)).isoformat()
    r2 = client.post("/ingest/heartbeat", json=heartbeat)
    assert r2.status_code == 202

    unauthorized = client.get("/bots")
    assert unauthorized.status_code == 401

    authorized = client.get("/bots?page=1&page_size=10", headers={"X-Admin-Token": "test-admin-token"})
    assert authorized.status_code == 200
    data = authorized.json()
    assert data["total"] == 1
    assert data["items"][0]["version"] == "1.0.1"


def test_module_crud_with_auth(client):
    module = {
        "module_id": "11111111-1111-1111-1111-111111111111",
        "name": "indices-watch",
        "description": "Indices monitor",
        "enabled": True,
        "execution_mode": "MANUAL_AND_SCHEDULED",
        "schedule_cron": "*/5 * * * *",
        "config": {"symbols": ["BTCUSDT"]},
        "created_at": datetime.now(timezone.utc).isoformat(),
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }

    unauth = client.post("/modules", json=module)
    assert unauth.status_code == 401

    created = client.post("/modules", json=module, headers={"X-Admin-Token": "test-admin-token"})
    assert created.status_code == 201

    listed = client.get("/modules", headers={"X-Admin-Token": "test-admin-token"})
    assert listed.status_code == 200
    assert listed.json()["total"] == 1

    fetched = client.get(
        "/modules/11111111-1111-1111-1111-111111111111",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert fetched.status_code == 200

    deleted = client.delete(
        "/modules/11111111-1111-1111-1111-111111111111",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert deleted.status_code == 204
