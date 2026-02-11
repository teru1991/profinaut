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


def test_heartbeat_loss_triggers_critical_alert_and_webhook(client, monkeypatch):
    sent = {"count": 0}

    class FakeResponse:
        status_code = 204

    def fake_post(url, json, timeout):
        sent["count"] += 1
        return FakeResponse()

    monkeypatch.setenv("DISCORD_WEBHOOK_URL", "https://discord.example/webhook")
    from app.config import get_settings
    get_settings.cache_clear()

    monkeypatch.setattr("app.notifications.requests.post", fake_post)

    old = (datetime.now(timezone.utc) - timedelta(minutes=5)).isoformat()
    hb = {
        "instance_id": "inst-stale",
        "bot_id": "bot-stale",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": old,
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    res = client.post("/alerts/heartbeat-check?stale_after_seconds=60", headers={"X-Admin-Token": "test-admin-token"})
    assert res.status_code == 200
    payload = res.json()
    assert payload["alerts_created"] == 1
    assert sent["count"] == 1

    # second check should not duplicate OPEN alert
    res2 = client.post("/alerts/heartbeat-check?stale_after_seconds=60", headers={"X-Admin-Token": "test-admin-token"})
    assert res2.status_code == 200
    assert res2.json()["alerts_created"] == 0


def test_metrics_positions_and_portfolio_exposure(client):
    now = datetime.now(timezone.utc)
    hb1 = {
        "instance_id": "inst-port-1",
        "bot_id": "bot-port-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    hb2 = {
        "instance_id": "inst-port-2",
        "bot_id": "bot-port-2",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "ETHUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb1).status_code == 202
    assert client.post("/ingest/heartbeat", json=hb2).status_code == 202

    assert (
        client.post(
            "/ingest/metrics",
            json={
                "instance_id": "inst-port-1",
                "symbol": "BTCUSDT",
                "metric_type": "equity",
                "value": 1234.5,
                "timestamp": now.isoformat(),
            },
        ).status_code
        == 202
    )

    assert (
        client.post(
            "/ingest/positions",
            json={
                "instance_id": "inst-port-1",
                "symbol": "BTCUSDT",
                "net_exposure": 100.0,
                "gross_exposure": 150.0,
                "updated_at": now.isoformat(),
            },
        ).status_code
        == 202
    )
    assert (
        client.post(
            "/ingest/positions",
            json={
                "instance_id": "inst-port-2",
                "symbol": "ETHUSDT",
                "net_exposure": -40.0,
                "gross_exposure": 60.0,
                "updated_at": now.isoformat(),
            },
        ).status_code
        == 202
    )

    summary = client.get("/portfolio/exposure", headers={"X-Admin-Token": "test-admin-token"})
    assert summary.status_code == 200
    body = summary.json()
    assert body["total_net_exposure"] == 60.0
    assert body["total_gross_exposure"] == 210.0
    assert body["key_metrics"]["latest_equity"] == 1234.5
    assert body["key_metrics"]["tracked_positions"] == 2


def test_reconcile_persistence_and_mismatch_alert(client, monkeypatch):
    sent = {"count": 0}

    class FakeResponse:
        status_code = 204

    def fake_post(url, json, timeout):
        sent["count"] += 1
        return FakeResponse()

    monkeypatch.setenv("DISCORD_WEBHOOK_URL", "https://discord.example/webhook")
    from app.config import get_settings

    get_settings.cache_clear()
    monkeypatch.setattr("app.notifications.requests.post", fake_post)

    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-recon-1",
        "bot_id": "bot-recon-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    reconcile = {
        "instance_id": "inst-recon-1",
        "exchange_equity": 1000.0,
        "internal_equity": 970.0,
        "difference": 30.0,
        "status": "MISMATCH",
        "timestamp": now.isoformat(),
    }

    post_res = client.post("/reconcile", json=reconcile, headers={"X-Admin-Token": "test-admin-token"})
    assert post_res.status_code == 202
    assert post_res.json()["reconcile_id"]

    listed = client.get("/reconcile/results?status=MISMATCH", headers={"X-Admin-Token": "test-admin-token"})
    assert listed.status_code == 200
    body = listed.json()
    assert body["total"] == 1
    assert body["items"][0]["instance_id"] == "inst-recon-1"
    assert sent["count"] == 1


def test_net_pnl_formula_with_costs(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-net-1",
        "bot_id": "bot-net-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    assert client.post(
        "/ingest/metrics",
        json={
            "instance_id": "inst-net-1",
            "symbol": "BTCUSDT",
            "metric_type": "realized_pnl",
            "value": 120.0,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202
    assert client.post(
        "/ingest/metrics",
        json={
            "instance_id": "inst-net-1",
            "symbol": "BTCUSDT",
            "metric_type": "unrealized_pnl",
            "value": 30.0,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202

    assert client.post(
        "/ingest/costs",
        json={
            "instance_id": "inst-net-1",
            "symbol": "BTCUSDT",
            "cost_type": "FEE",
            "amount": 10.0,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202
    assert client.post(
        "/ingest/costs",
        json={
            "instance_id": "inst-net-1",
            "symbol": "BTCUSDT",
            "cost_type": "FUNDING",
            "amount": 5.0,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202

    summary = client.get("/analytics/net-pnl?symbol=BTCUSDT", headers={"X-Admin-Token": "test-admin-token"})
    assert summary.status_code == 200
    body = summary.json()
    assert body["realized"] == 120.0
    assert body["unrealized"] == 30.0
    assert body["fees"] == 10.0
    assert body["funding"] == 5.0
    assert body["net_pnl"] == 145.0
