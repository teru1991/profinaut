from __future__ import annotations

import sqlite3
from pathlib import Path

from fastapi.testclient import TestClient

from services.marketdata.app import main
from services.marketdata.app.metrics import ingest_metrics


async def _idle_poller() -> None:
    return None


def test_trade_payload_routes_to_md_trades_when_silver_enabled(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setenv("SILVER_ENABLED", "1")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:01Z",
                "payload_json": {"price": 100.0, "qty": 2.5, "side": "buy"},
                "venue_id": "gmo",
                "market_id": "spot",
                "instrument_id": "btc_jpy",
                "source_msg_key": "trade-1",
            },
        )

    assert resp.status_code == 200
    body = resp.json()
    assert body["normalized_target"] == "md_trades"

    conn = sqlite3.connect(db_file)
    row = conn.execute("SELECT raw_msg_id, price, qty, side FROM md_trades").fetchone()
    assert row is not None
    assert row[0] == body["raw_msg_id"]
    assert row[1] == 100.0
    assert row[2] == 2.5
    assert row[3] == "buy"


def test_unknown_payload_routes_to_md_events_json(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setenv("SILVER_ENABLED", "1")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:01Z",
                "payload_json": {"mystery": "value", "nested": {"x": 1}},
                "venue_id": "gmo",
                "market_id": "spot",
            },
        )

    assert resp.status_code == 200
    body = resp.json()
    assert body["normalized_target"] == "md_events_json"
    assert isinstance(body["normalized_event_type"], str)
    assert body["normalized_event_type"].startswith("venue.")

    conn = sqlite3.connect(db_file)
    row = conn.execute("SELECT raw_msg_id, event_type FROM md_events_json").fetchone()
    assert row is not None
    assert row[0] == body["raw_msg_id"]
    assert row[1] == body["normalized_event_type"]
