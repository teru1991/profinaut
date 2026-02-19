from __future__ import annotations

import gzip
import json
from pathlib import Path

from fastapi.testclient import TestClient

from services.marketdata.app import main


async def _idle_poller() -> None:
    return None


def test_ingest_n_records_persisted_once_with_dedupe(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))
    monkeypatch.setenv("BRONZE_IDEMPOTENCY_DB", str(tmp_path / "idempotency.sqlite3"))
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    payload = {"symbol": "BTC_JPY", "price": "100", "qty": "0.1", "side": "buy"}
    with TestClient(main.app) as client:
        for i in range(5):
            resp = client.post(
                "/raw/ingest",
                json={
                    "tenant_id": "tenant-a",
                    "source_type": "WS_PUBLIC",
                    "received_ts": f"2026-02-16T00:00:0{i}Z",
                    "event_ts": f"2026-02-16T00:00:0{i}Z",
                    "payload_json": payload,
                    "venue_id": "gmo",
                    "market_id": "BTC_JPY",
                    "source_event_id": f"evt-{i}",
                    "idempotency_key": f"gmo:BTC_JPY:evt-{i}",
                },
            )
            assert resp.status_code == 200

        dup = client.post(
            "/raw/ingest",
            json={
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "received_ts": "2026-02-16T00:10:00Z",
                "event_ts": "2026-02-16T00:10:00Z",
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": "BTC_JPY",
                "source_event_id": "evt-0",
                "idempotency_key": "gmo:BTC_JPY:evt-0",
            },
        )
        assert dup.status_code == 200
        assert dup.json()["object_key"] == "dedupe://dropped"

    files = list(bronze_root.rglob("*.jsonl.gz"))
    assert files
    total = 0
    for path in files:
        total += len([ln for ln in gzip.decompress(path.read_bytes()).decode("utf-8").splitlines() if ln.strip()])
    assert total == 5

    first_line = gzip.decompress(files[0].read_bytes()).decode("utf-8").splitlines()[0]
    parsed = json.loads(first_line)
    assert parsed["meta"]["raw_ref"].startswith("raw://bronze/gmo/")
