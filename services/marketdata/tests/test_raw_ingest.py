from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from fastapi.testclient import TestClient

from services.marketdata.app import main
from services.marketdata.app.metrics import ingest_metrics
from services.marketdata.app.routes.raw_ingest import _canonical_payload_hash, _derive_event_ts_quality


async def _idle_poller() -> None:
    return None


def test_payload_hash_is_deterministic() -> None:
    a = {"b": 1, "a": {"z": 2, "x": [3, 2, 1]}}
    b = {"a": {"x": [3, 2, 1], "z": 2}, "b": 1}
    assert _canonical_payload_hash(a) == _canonical_payload_hash(b)


def test_lag_quality_when_event_ts_missing() -> None:
    event_ts_quality, lag_ms = _derive_event_ts_quality(None, "2026-02-16T00:00:00Z")
    assert event_ts_quality == "missing"
    assert lag_ms is None


def test_ingest_returns_503_when_storage_not_configured(monkeypatch) -> None:
    ingest_metrics.reset_for_tests()
    monkeypatch.delenv("OBJECT_STORE_BACKEND", raising=False)
    monkeypatch.setenv("DB_DSN", "sqlite:///:memory:")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "t1",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:00Z",
                "payload_json": {"x": 1},
            },
        )

    assert resp.status_code == 503
    assert resp.json() == {"error": "INGEST_DISABLED", "reason": "STORAGE_NOT_CONFIGURED"}


def test_raw_ingest_stores_bronze_and_db(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    req = {
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "received_ts": "2026-02-16T00:00:01Z",
        "payload_json": {"symbol": "BTC_JPY", "price": 123.45},
        "venue_id": "gmo",
        "market_id": "spot",
    }

    with TestClient(main.app) as client:
        resp = client.post("/raw/ingest", json=req)

    assert resp.status_code == 200
    body = resp.json()
    assert body["stored"] is True
    assert body["dup_suspect"] is False
    assert body["degraded"] is False
    assert len(body["raw_msg_id"]) == 26
    assert body["object_key"].startswith("bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00/part-")

    files = list(bronze_root.rglob("*.jsonl"))
    assert len(files) == 1
    line = files[0].read_text(encoding="utf-8").strip().splitlines()[0]
    stored = json.loads(line)
    assert stored["tenant_id"] == "tenant-a"
    assert stored["payload_json"] == req["payload_json"]
    assert stored["payload_hash"] == _canonical_payload_hash(req["payload_json"])
    assert stored["quality_json"]["dup_suspect"] is False
    assert stored["quality_json"]["event_ts_quality"] == "missing"

    conn = sqlite3.connect(db_file)
    row = conn.execute("SELECT raw_msg_id, object_key, payload_hash, quality_json FROM raw_ingest_meta").fetchone()
    assert row is not None
    assert row[0] == body["raw_msg_id"]
    assert row[1] == body["object_key"]
    assert row[2] == _canonical_payload_hash(req["payload_json"])
    assert json.loads(row[3])["dup_suspect"] is False


def test_raw_ingest_marks_dup_suspect_and_updates_capabilities_stats(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    payload = {"symbol": "BTC_JPY", "price": 100}

    with TestClient(main.app) as client:
        first = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:01Z",
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": "spot",
                "source_msg_key": "abc-1",
            },
        )
        second = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:01:01Z",
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": "spot",
                "source_msg_key": "abc-1",
            },
        )
        caps = client.get("/capabilities")

    assert first.status_code == 200
    assert second.status_code == 200
    assert first.json()["dup_suspect"] is False
    assert second.json()["dup_suspect"] is True

    rows = sqlite3.connect(db_file).execute(
        "SELECT quality_json FROM raw_ingest_meta ORDER BY received_ts ASC"
    ).fetchall()
    assert len(rows) == 2
    first_q = json.loads(rows[0][0])
    second_q = json.loads(rows[1][0])
    assert first_q["dup_suspect"] is False
    assert second_q["dup_suspect"] is True
    assert second_q["dup_reason"] in {"payload_hash_seen", "source_msg_key_seen"}

    caps_payload = caps.json()
    stats = caps_payload["ingest_stats"]
    assert stats["ingest_count"] >= 2
    assert stats["dup_suspect_count"] >= 1
    assert stats["last_5m"]["ingest_count"] >= 2
    assert stats["last_5m"]["dup_suspect_count"] >= 1


def test_gmo_trade_source_msg_key_prefers_upstream_trade_id_and_dedups_md_trades(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setenv("SILVER_ENABLED", "1")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    payload = {
        "trade_id": "t-100",
        "price": "100.1",
        "qty": "0.5",
        "side": "BUY",
    }

    with TestClient(main.app) as client:
        first = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:01Z",
                "event_ts": "2026-02-16T00:00:01Z",
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": "spot",
                "stream_name": "trades",
            },
        )
        second = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:00:02Z",
                "event_ts": "2026-02-16T00:00:01Z",
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": "spot",
                "stream_name": "trades",
            },
        )
        caps = client.get("/capabilities")

    assert first.status_code == 200
    assert second.status_code == 200
    assert second.json()["dup_suspect"] is True

    conn = sqlite3.connect(db_file)
    count = conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0]
    src_key = conn.execute("SELECT source_msg_key FROM md_trades LIMIT 1").fetchone()[0]
    assert count == 1
    assert src_key == "gmo:trade_id:t-100"

    stats = caps.json()["ingest_stats"]
    assert stats["dup_suspect_total"] >= 1


def test_gmo_trade_composite_source_msg_key_is_stable_on_reordered_arrival(monkeypatch, tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setenv("SILVER_ENABLED", "1")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    base = {
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "event_ts": "2026-02-16T00:00:10Z",
        "payload_json": {
            "price": "101.0",
            "qty": "0.75",
            "side": "sell",
            "sequence": 20,
        },
        "venue_id": "gmo",
        "market_id": "spot",
        "stream_name": "trades",
        "seq": 20,
    }

    with TestClient(main.app) as client:
        newer = client.post(
            "/raw/ingest",
            json={**base, "received_ts": "2026-02-16T00:00:20Z"},
        )
        older = client.post(
            "/raw/ingest",
            json={**base, "received_ts": "2026-02-16T00:00:05Z"},
        )

    assert newer.status_code == 200
    assert older.status_code == 200

    conn = sqlite3.connect(db_file)
    count = conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0]
    src_key = conn.execute("SELECT source_msg_key FROM md_trades LIMIT 1").fetchone()[0]
    assert count == 1
    assert src_key == "gmo:trade:v1:gmo:spot:2026-02-16T00:00:10Z:101.0:0.75:sell:20"
