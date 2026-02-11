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

    assert client.post("/ingest/heartbeat", json=heartbeat).status_code == 202
    heartbeat["version"] = "1.0.1"
    heartbeat["timestamp"] = (datetime.now(timezone.utc) + timedelta(seconds=2)).isoformat()
    assert client.post("/ingest/heartbeat", json=heartbeat).status_code == 202

    assert client.get("/bots").status_code == 401
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

    assert client.post("/modules", json=module).status_code == 401
    assert client.post("/modules", json=module, headers={"X-Admin-Token": "test-admin-token"}).status_code == 201
    assert client.get("/modules", headers={"X-Admin-Token": "test-admin-token"}).json()["total"] == 1


def test_command_end_to_end_and_audit_persistence(client):
    now = datetime.now(timezone.utc)

    # create instance via heartbeat
    hb = {
        "instance_id": "inst-cmd",
        "bot_id": "bot-cmd",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "ETHUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    cmd = {
        "command_id": "22222222-2222-2222-2222-222222222222",
        "instance_id": "inst-cmd",
        "command_type": "SAFE_MODE",
        "issued_at": now.isoformat(),
        "expires_at": (now + timedelta(minutes=1)).isoformat(),
        "payload": {},
    }

    create_res = client.post("/commands", json=cmd, headers={"X-Admin-Token": "test-admin-token"})
    assert create_res.status_code == 202

    pending = client.get("/instances/inst-cmd/commands/pending")
    assert pending.status_code == 200
    assert len(pending.json()) == 1

    ack = {
        "command_id": cmd["command_id"],
        "instance_id": "inst-cmd",
        "status": "COMPLETED",
        "reason": None,
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    ack_res = client.post(f"/commands/{cmd['command_id']}/ack", json=ack)
    assert ack_res.status_code == 202

    pending_after = client.get("/instances/inst-cmd/commands/pending")
    assert pending_after.status_code == 200
    assert len(pending_after.json()) == 0

    audit = client.get("/audit/logs", headers={"X-Admin-Token": "test-admin-token"})
    assert audit.status_code == 200
    actions = [item["action"] for item in audit.json()["items"]]
    assert "COMMAND_CREATE" in actions
    assert "COMMAND_ACK" in actions


def test_expired_command_not_delivered(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-expired",
        "bot_id": "bot-expired",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    expired = {
        "command_id": "33333333-3333-3333-3333-333333333333",
        "instance_id": "inst-expired",
        "command_type": "STOP",
        "issued_at": now.isoformat(),
        "expires_at": (now + timedelta(seconds=5)).isoformat(),
        "payload": {},
    }

    assert client.post("/commands", json=expired, headers={"X-Admin-Token": "test-admin-token"}).status_code == 202

    # simulate late poll past TTL
    late = client.get("/instances/inst-expired/commands/pending")
    assert late.status_code == 200
    assert len(late.json()) == 1

    # ack as rejected expired from agent
    ack = {
        "command_id": expired["command_id"],
        "instance_id": "inst-expired",
        "status": "REJECTED_EXPIRED",
        "reason": "Command TTL expired",
        "timestamp": (now + timedelta(minutes=2)).isoformat(),
    }
    assert client.post(f"/commands/{expired['command_id']}/ack", json=ack).status_code == 202

    no_more = client.get("/instances/inst-expired/commands/pending")
    assert no_more.status_code == 200
    assert len(no_more.json()) == 0
