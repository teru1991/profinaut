from datetime import datetime, timedelta, timezone


def test_healthz(client):
    response = client.get("/healthz")
    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] == "ok"




def test_capabilities(client):
    response = client.get("/capabilities")
    assert response.status_code == 200
    payload = response.json()
    assert payload["service"] == "dashboard-api"
    assert payload["version"]
    assert payload["status"] in {"ok", "degraded"}
    assert isinstance(payload["features"], list)
    assert payload["generated_at"]

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
    assert data["items"][0]["state"] == "RUNNING"
    assert data["items"][0]["degraded"] is False
    assert data["items"][0]["degraded_reason"] is None


def test_bots_empty_list_returns_200_with_structure(client):
    """Verify /bots returns 200 with expected envelope structure even when empty."""
    response = client.get("/bots?page=1&page_size=50", headers={"X-Admin-Token": "test-admin-token"})
    assert response.status_code == 200
    
    data = response.json()
    # Validate envelope structure
    assert "page" in data
    assert "page_size" in data
    assert "total" in data
    assert "items" in data
    
    # Validate values
    assert data["page"] == 1
    assert data["page_size"] == 50
    assert data["total"] == 0
    assert data["items"] == []


def test_bots_last_seen_utc_iso_format(client):
    """Verify last_seen is UTC ISO format when present, null when absent."""
    # Create a bot with heartbeat
    now = datetime.now(timezone.utc)
    heartbeat = {
        "instance_id": "inst-utc-test",
        "bot_id": "bot-utc-test",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "ETHUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=heartbeat).status_code == 202
    
    # Query bots
    response = client.get("/bots?page=1&page_size=10", headers={"X-Admin-Token": "test-admin-token"})
    assert response.status_code == 200
    
    data = response.json()
    assert data["total"] >= 1
    
    # Find our bot
    bot = next((b for b in data["items"] if b["bot_id"] == "bot-utc-test"), None)
    assert bot is not None
    
    # Validate last_seen is a string (ISO format)
    assert bot["last_seen"] is not None
    assert isinstance(bot["last_seen"], str)
    
    # Parse and verify it's a valid ISO timestamp with timezone
    parsed_time = datetime.fromisoformat(bot["last_seen"].replace("Z", "+00:00"))
    assert parsed_time.tzinfo is not None, "last_seen must include timezone information"
    
    # Verify the timestamp is close to when we sent it (within 5 seconds)
    time_diff = abs((parsed_time - now).total_seconds())
    assert time_diff < 5, f"last_seen timestamp differs by {time_diff} seconds"


def test_bots_null_last_seen_for_bot_without_heartbeat(client, db_session):
    """Verify last_seen is null for bots that never sent heartbeat."""
    from app.models import Bot
    
    # Directly insert a bot without creating an instance or bot_status
    bot = Bot(
        bot_id="bot-no-heartbeat",
        name="Never Sent Heartbeat",
        strategy_name="test_strategy",
    )
    db_session.add(bot)
    db_session.commit()
    
    # Query bots
    response = client.get("/bots?page=1&page_size=50", headers={"X-Admin-Token": "test-admin-token"})
    assert response.status_code == 200
    
    data = response.json()
    
    # Find our bot
    bot_data = next((b for b in data["items"] if b["bot_id"] == "bot-no-heartbeat"), None)
    assert bot_data is not None
    
    # Verify last_seen is null
    assert bot_data["last_seen"] is None
    assert bot_data["instance_id"] is None
    assert bot_data["version"] is None
    assert bot_data["state"] == "UNKNOWN"
    assert bot_data["degraded"] is False  # No heartbeat = not degraded, just unknown


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


def test_execution_quality_ingest_and_summary(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-eq-1",
        "bot_id": "bot-eq-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    assert client.post(
        "/ingest/execution-quality",
        json={
            "instance_id": "inst-eq-1",
            "symbol": "BTCUSDT",
            "slippage_bps": 2.0,
            "latency_ms": 120.0,
            "fill_ratio": 0.95,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202
    assert client.post(
        "/ingest/execution-quality",
        json={
            "instance_id": "inst-eq-1",
            "symbol": "BTCUSDT",
            "slippage_bps": 4.0,
            "latency_ms": 80.0,
            "fill_ratio": 0.90,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202

    summary = client.get("/analytics/execution-quality?symbol=BTCUSDT", headers={"X-Admin-Token": "test-admin-token"})
    assert summary.status_code == 200
    body = summary.json()
    assert body["samples"] == 2
    assert body["avg_slippage_bps"] == 3.0
    assert body["avg_latency_ms"] == 100.0
    assert body["avg_fill_ratio"] == 0.925


def test_module_run_trigger_and_status_update(client):
    now = datetime.now(timezone.utc).isoformat()
    module = {
        "module_id": "44444444-4444-4444-4444-444444444444",
        "name": "alert-rules",
        "description": "Alert rules module",
        "enabled": True,
        "execution_mode": "MANUAL_AND_SCHEDULED",
        "schedule_cron": "*/10 * * * *",
        "config": {},
        "created_at": now,
        "updated_at": now,
    }
    assert client.post("/modules", json=module, headers={"X-Admin-Token": "test-admin-token"}).status_code == 201

    trigger = client.post(
        "/modules/44444444-4444-4444-4444-444444444444/run",
        json={"trigger_type": "MANUAL", "summary": {"reason": "operator"}},
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert trigger.status_code == 202
    run_id = trigger.json()["run_id"]
    assert trigger.json()["status"] == "QUEUED"

    updated = client.patch(
        f"/module-runs/{run_id}",
        json={"status": "SUCCEEDED", "summary": {"processed": 3}},
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert updated.status_code == 200
    body = updated.json()
    assert body["status"] == "SUCCEEDED"
    assert body["ended_at"] is not None

    listed = client.get(
        "/module-runs?module_id=44444444-4444-4444-4444-444444444444&status=SUCCEEDED",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert listed.status_code == 200
    assert listed.json()["total"] == 1


def test_module_run_cancel_and_stats(client):
    now = datetime.now(timezone.utc).isoformat()
    module = {
        "module_id": "55555555-5555-5555-5555-555555555555",
        "name": "indices-watch",
        "description": "indices",
        "enabled": True,
        "execution_mode": "MANUAL_AND_SCHEDULED",
        "schedule_cron": "*/5 * * * *",
        "config": {},
        "created_at": now,
        "updated_at": now,
    }
    assert client.post("/modules", json=module, headers={"X-Admin-Token": "test-admin-token"}).status_code == 201

    trigger = client.post(
        "/modules/55555555-5555-5555-5555-555555555555/run",
        json={"trigger_type": "MANUAL"},
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert trigger.status_code == 202
    run_id = trigger.json()["run_id"]

    cancel = client.post(f"/module-runs/{run_id}/cancel", headers={"X-Admin-Token": "test-admin-token"})
    assert cancel.status_code == 200
    assert cancel.json()["status"] == "CANCELED"

    stats = client.get("/module-runs/stats?module_id=55555555-5555-5555-5555-555555555555", headers={"X-Admin-Token": "test-admin-token"})
    assert stats.status_code == 200
    body = stats.json()
    assert body["total_runs"] == 1
    assert body["active_runs"] == 0
    assert body["status_counts"]["CANCELED"] == 1


def test_module_run_stuck_check_creates_warning_alert(client, monkeypatch):
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

    now = datetime.now(timezone.utc).isoformat()
    module_id = "66666666-6666-6666-6666-666666666666"
    module = {
        "module_id": module_id,
        "name": "stuck-watch",
        "description": "stuck run detector test",
        "enabled": True,
        "execution_mode": "MANUAL_AND_SCHEDULED",
        "schedule_cron": "*/5 * * * *",
        "config": {},
        "created_at": now,
        "updated_at": now,
    }
    assert client.post("/modules", json=module, headers={"X-Admin-Token": "test-admin-token"}).status_code == 201

    trigger = client.post(
        f"/modules/{module_id}/run",
        json={"trigger_type": "MANUAL"},
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert trigger.status_code == 202
    run_id = trigger.json()["run_id"]

    # move checker clock forward so run appears stale
    class FutureDateTime:
        @staticmethod
        def now(tz=None):
            return datetime.now(timezone.utc) + timedelta(hours=1)

    monkeypatch.setattr("app.main.datetime", FutureDateTime)

    res = client.post("/alerts/module-runs/stuck-check?stale_after_seconds=60", headers={"X-Admin-Token": "test-admin-token"})
    assert res.status_code == 200
    body = res.json()
    assert body["stuck_runs"] == 1
    assert body["alerts_created"] == 1
    assert sent["count"] == 1

    # second run should deduplicate OPEN alert
    res2 = client.post("/alerts/module-runs/stuck-check?stale_after_seconds=60", headers={"X-Admin-Token": "test-admin-token"})
    assert res2.status_code == 200
    assert res2.json()["alerts_created"] == 0


def test_equity_drawdown_summary(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-dd-1",
        "bot_id": "bot-dd-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    equities = [1000.0, 1200.0, 900.0, 1100.0]
    for i, eq in enumerate(equities):
        ts = (now + timedelta(minutes=i)).isoformat()
        assert client.post(
            "/ingest/metrics",
            json={
                "instance_id": "inst-dd-1",
                "symbol": "BTCUSDT",
                "metric_type": "equity",
                "value": eq,
                "timestamp": ts,
            },
        ).status_code == 202

    res = client.get("/analytics/equity-drawdown?symbol=BTCUSDT", headers={"X-Admin-Token": "test-admin-token"})
    assert res.status_code == 200
    body = res.json()
    assert body["samples"] == 4
    assert body["peak_equity"] == 1200.0
    assert body["latest_equity"] == 1100.0
    assert body["max_drawdown_abs"] == 300.0
    assert round(body["max_drawdown_pct"], 6) == 0.25
    assert round(body["current_drawdown_pct"], 6) == round(100.0 / 1200.0, 6)


def test_module_run_performance_summary(client):
    now = datetime.now(timezone.utc)
    module_id = "mod-perf-1"
    create = client.post(
        "/modules",
        headers={"X-Admin-Token": "test-admin-token"},
        json={
            "module_id": module_id,
            "name": "Perf Module",
            "description": "test",
            "enabled": True,
            "execution_mode": "MANUAL",
            "schedule_cron": None,
            "config": {},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        },
    )
    assert create.status_code == 201

    durations = [10, 20, 40]
    statuses = ["SUCCEEDED", "FAILED", "CANCELED"]
    for dur, status in zip(durations, statuses):
        trig = client.post(
            f"/modules/{module_id}/run",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"trigger_type": "MANUAL", "summary": {}},
        )
        assert trig.status_code == 202
        run = trig.json()
        started_at = datetime.fromisoformat(run["started_at"].replace("Z", "+00:00"))
        ended_at = started_at + timedelta(seconds=dur)

        upd = client.patch(
            f"/module-runs/{run['run_id']}",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"status": status, "summary": {}, "ended_at": ended_at.isoformat()},
        )
        assert upd.status_code == 200

    res = client.get(
        f"/analytics/module-runs/performance?module_id={module_id}",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert res.status_code == 200
    body = res.json()
    assert body["total_runs"] == 3
    assert body["completed_runs"] == 3
    assert round(body["success_rate"], 6) == round(1 / 3, 6)
    assert round(body["avg_duration_seconds"], 6) == round(sum(durations) / 3, 6)
    assert body["p95_duration_seconds"] == 40


def test_module_run_failure_rate_summary(client):
    now = datetime.now(timezone.utc)
    module_id = "mod-fail-1"
    create = client.post(
        "/modules",
        headers={"X-Admin-Token": "test-admin-token"},
        json={
            "module_id": module_id,
            "name": "Failure Module",
            "description": "test",
            "enabled": True,
            "execution_mode": "MANUAL",
            "schedule_cron": None,
            "config": {},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        },
    )
    assert create.status_code == 201

    statuses = ["SUCCEEDED", "FAILED", "SUCCEEDED", "FAILED", "CANCELED"]
    for i, status in enumerate(statuses):
        trig = client.post(
            f"/modules/{module_id}/run",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"trigger_type": "MANUAL", "summary": {}},
        )
        assert trig.status_code == 202
        run = trig.json()
        started_at = datetime.fromisoformat(run["started_at"].replace("Z", "+00:00"))
        ended_at = started_at + timedelta(seconds=10 + i)

        upd = client.patch(
            f"/module-runs/{run['run_id']}",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"status": status, "summary": {}, "ended_at": ended_at.isoformat()},
        )
        assert upd.status_code == 200

    res = client.get(
        f"/analytics/module-runs/failure-rate?module_id={module_id}&window_size=5",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert res.status_code == 200
    body = res.json()
    assert body["total_completed"] == 5
    assert body["failed_runs"] == 2
    assert round(body["failure_rate"], 6) == 0.4
    assert body["window_size_used"] == 5


def test_module_run_throughput_summary(client):
    now = datetime.now(timezone.utc)
    module_id = "mod-throughput-1"
    create = client.post(
        "/modules",
        headers={"X-Admin-Token": "test-admin-token"},
        json={
            "module_id": module_id,
            "name": "Throughput Module",
            "description": "test",
            "enabled": True,
            "execution_mode": "MANUAL",
            "schedule_cron": None,
            "config": {},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        },
    )
    assert create.status_code == 201

    for _ in range(6):
        trig = client.post(
            f"/modules/{module_id}/run",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"trigger_type": "MANUAL", "summary": {}},
        )
        assert trig.status_code == 202

    res = client.get(
        f"/analytics/module-runs/throughput?module_id={module_id}&window_hours=3",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert res.status_code == 200
    body = res.json()
    assert body["window_hours"] == 3
    assert body["total_runs"] == 6
    assert round(body["runs_per_hour"], 6) == 2.0


def test_module_run_active_age_summary(client):
    now = datetime.now(timezone.utc)
    module_id = "mod-active-age-1"
    create = client.post(
        "/modules",
        headers={"X-Admin-Token": "test-admin-token"},
        json={
            "module_id": module_id,
            "name": "ActiveAge Module",
            "description": "test",
            "enabled": True,
            "execution_mode": "MANUAL",
            "schedule_cron": None,
            "config": {},
            "created_at": now.isoformat(),
            "updated_at": now.isoformat(),
        },
    )
    assert create.status_code == 201

    for _ in range(3):
        trig = client.post(
            f"/modules/{module_id}/run",
            headers={"X-Admin-Token": "test-admin-token"},
            json={"trigger_type": "MANUAL", "summary": {}},
        )
        assert trig.status_code == 202

    # Close one run; leave two active
    runs = client.get(
        f"/module-runs?module_id={module_id}",
        headers={"X-Admin-Token": "test-admin-token"},
    ).json()["items"]
    done_id = runs[0]["run_id"]
    upd = client.patch(
        f"/module-runs/{done_id}",
        headers={"X-Admin-Token": "test-admin-token"},
        json={"status": "SUCCEEDED", "summary": {}, "ended_at": (datetime.now(timezone.utc) + timedelta(seconds=1)).isoformat()},
    )
    assert upd.status_code == 200

    res = client.get(
        f"/analytics/module-runs/active-age?module_id={module_id}",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert res.status_code == 200
    body = res.json()
    assert body["active_runs"] == 2
    assert body["oldest_active_seconds"] >= 0.0
    assert body["avg_active_seconds"] >= 0.0


def test_indices_ingest_and_latest_summary(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-idx-1",
        "bot_id": "bot-idx-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    points = [
        {"index_name": "BTC_INDEX", "value": 50000.0, "timestamp": (now + timedelta(minutes=0)).isoformat()},
        {"index_name": "BTC_INDEX", "value": 50100.0, "timestamp": (now + timedelta(minutes=1)).isoformat()},
        {"index_name": "ETH_INDEX", "value": 3000.0, "timestamp": (now + timedelta(minutes=2)).isoformat()},
    ]
    for p in points:
        assert client.post(
            "/ingest/indices",
            json={"instance_id": "inst-idx-1", **p},
        ).status_code == 202

    res = client.get("/analytics/indices/latest", headers={"X-Admin-Token": "test-admin-token"})
    assert res.status_code == 200
    body = res.json()
    by_name = {item["index_name"]: item for item in body["items"]}
    assert by_name["BTC_INDEX"]["value"] == 50100.0
    assert by_name["ETH_INDEX"]["value"] == 3000.0

    btc_only = client.get("/analytics/indices/latest?index_name=BTC_INDEX", headers={"X-Admin-Token": "test-admin-token"})
    assert btc_only.status_code == 200
    assert len(btc_only.json()["items"]) == 1
    assert btc_only.json()["items"][0]["index_name"] == "BTC_INDEX"


def test_resource_ingest_and_latest_summary(client):
    now = datetime.now(timezone.utc)
    hb = {
        "instance_id": "inst-res-1",
        "bot_id": "bot-res-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb).status_code == 202

    assert client.post(
        "/ingest/resource",
        json={
            "instance_id": "inst-res-1",
            "cpu_pct": 31.5,
            "memory_pct": 62.25,
            "timestamp": now.isoformat(),
        },
    ).status_code == 202

    res = client.get("/analytics/resource/latest?instance_id=inst-res-1", headers={"X-Admin-Token": "test-admin-token"})
    assert res.status_code == 200
    body = res.json()
    assert body["instance_id"] == "inst-res-1"
    assert body["latest_cpu_pct"] == 31.5
    assert body["latest_memory_pct"] == 62.25


def test_resource_window_summary(client):
    now = datetime.now(timezone.utc)

    hb_1 = {
        "instance_id": "inst-resw-1",
        "bot_id": "bot-resw-1",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "BTCUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    hb_2 = {
        "instance_id": "inst-resw-2",
        "bot_id": "bot-resw-2",
        "runtime_mode": "PAPER",
        "exchange": "BINANCE",
        "symbol": "ETHUSDT",
        "version": "1.0.0",
        "timestamp": now.isoformat(),
        "metadata": {},
    }
    assert client.post("/ingest/heartbeat", json=hb_1).status_code == 202
    assert client.post("/ingest/heartbeat", json=hb_2).status_code == 202

    samples = [
        ("inst-resw-1", 20.0, 40.0, now - timedelta(hours=2)),
        ("inst-resw-1", 30.0, 50.0, now - timedelta(minutes=30)),
        ("inst-resw-2", 40.0, 60.0, now - timedelta(minutes=10)),
        ("inst-resw-1", 10.0, 30.0, now - timedelta(hours=30)),  # outside default 24h window
    ]

    for instance_id, cpu, mem, ts in samples:
        assert client.post(
            "/ingest/resource",
            json={
                "instance_id": instance_id,
                "cpu_pct": cpu,
                "memory_pct": mem,
                "timestamp": ts.isoformat(),
            },
        ).status_code == 202

    summary_all = client.get(
        "/analytics/resource/window",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert summary_all.status_code == 200
    body_all = summary_all.json()
    assert body_all["window_hours"] == 24
    assert body_all["cpu_samples"] == 3
    assert body_all["memory_samples"] == 3
    assert body_all["avg_cpu_pct"] == 30.0
    assert body_all["max_cpu_pct"] == 40.0
    assert body_all["avg_memory_pct"] == 50.0
    assert body_all["max_memory_pct"] == 60.0

    summary_filtered = client.get(
        "/analytics/resource/window?instance_id=inst-resw-1&window_hours=3",
        headers={"X-Admin-Token": "test-admin-token"},
    )
    assert summary_filtered.status_code == 200
    body_filtered = summary_filtered.json()
    assert body_filtered["instance_id"] == "inst-resw-1"
    assert body_filtered["window_hours"] == 3
    assert body_filtered["cpu_samples"] == 2
    assert body_filtered["memory_samples"] == 2
    assert body_filtered["avg_cpu_pct"] == 25.0
    assert body_filtered["max_cpu_pct"] == 30.0
    assert body_filtered["avg_memory_pct"] == 45.0
    assert body_filtered["max_memory_pct"] == 50.0
