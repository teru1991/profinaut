from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from services.marketdata.app.replay import _iter_hours, run_replay


def _write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as fp:
        for row in rows:
            fp.write(json.dumps(row, separators=(",", ":"), ensure_ascii=False) + "\n")


def test_iter_hours_builds_hourly_range() -> None:
    hours = _iter_hours(
        datetime_from("2026-02-16T00:15:00Z"),
        datetime_from("2026-02-16T03:00:00Z"),
    )
    assert [h.strftime("%Y-%m-%dT%H:%M:%SZ") for h in hours] == [
        "2026-02-16T00:00:00Z",
        "2026-02-16T01:00:00Z",
        "2026-02-16T02:00:00Z",
    ]


def datetime_from(ts: str):
    from datetime import UTC, datetime

    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def test_replay_counts_and_writes(monkeypatch, tmp_path: Path) -> None:
    bronze_root = tmp_path / "bronze-root"
    db_file = tmp_path / "replay.sqlite3"

    key = (
        bronze_root
        / "bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00/part-0001.jsonl"
    )
    _write_jsonl(
        key,
        [
            {
                "raw_msg_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "venue_id": "gmo",
                "market_id": "spot",
                "received_ts": "2026-02-16T00:10:00Z",
                "payload_json": {"price": 100, "qty": 1.25, "side": "buy"},
                "parser_version": "v0.1",
            },
            {
                "raw_msg_id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
                "tenant_id": "tenant-a",
                "source_type": "WS_PUBLIC",
                "venue_id": "gmo",
                "market_id": "spot",
                "received_ts": "2026-02-16T00:11:00Z",
                "payload_json": {"mystery": True},
                "parser_version": "v0.1",
            },
        ],
    )

    dry = run_replay(
        from_ts="2026-02-16T00:00:00Z",
        to_ts="2026-02-16T01:00:00Z",
        db_dsn=f"sqlite:///{db_file}",
        bronze_root=str(bronze_root),
        dry_run=True,
        parser_version="v0.2",
    )
    assert dry.read_count == 2
    assert dry.silver_count == 1
    assert dry.events_count == 1

    live = run_replay(
        from_ts="2026-02-16T00:00:00Z",
        to_ts="2026-02-16T01:00:00Z",
        db_dsn=f"sqlite:///{db_file}",
        bronze_root=str(bronze_root),
        dry_run=False,
        parser_version="v0.2",
    )
    assert live.read_count == 2
    assert live.silver_count == 1
    assert live.events_count == 1

    conn = sqlite3.connect(db_file)
    trades = conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0]
    events = conn.execute("SELECT COUNT(*) FROM md_events_json").fetchone()[0]
    event_parser = conn.execute("SELECT parser_version FROM md_events_json LIMIT 1").fetchone()[0]
    assert trades == 1
    assert events == 1
    assert event_parser == "v0.2"
