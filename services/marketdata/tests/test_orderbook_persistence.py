from __future__ import annotations

import sqlite3
from pathlib import Path

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.silver import normalizer
from services.marketdata.app.silver.normalizer import normalize_envelope


def _repo(db_file: Path) -> MarketDataMetaRepository:
    conn = sqlite3.connect(db_file)
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


def _reset_orderbook_runtime_state() -> None:
    normalizer._ORDERBOOK_ENGINES.clear()
    normalizer._ORDERBOOK_LAST_SEQ.clear()
    normalizer._ORDERBOOK_REQUIRE_SNAPSHOT.clear()


def test_orderbook_warm_start_keeps_bbo_continuity_after_restart(tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    repo = _repo(db_file)
    _reset_orderbook_runtime_state()

    normalize_envelope(
        repo,
        {
            "raw_msg_id": "snap-1",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:00:01Z",
            "event_ts": "2026-02-16T00:00:01Z",
            "seq": 10,
            "stream_name": "orderbooks",
            "payload_json": {
                "type": "snapshot",
                "bids": [{"price": "100", "size": "1"}],
                "asks": [{"price": "101", "size": "2"}],
            },
        },
    )

    # simulate process restart
    _reset_orderbook_runtime_state()

    normalize_envelope(
        repo,
        {
            "raw_msg_id": "delta-1",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:00:02Z",
            "event_ts": "2026-02-16T00:00:02Z",
            "seq": 11,
            "stream_name": "orderbooks",
            "payload_json": {
                "type": "delta",
                "changes": {
                    "bids": [{"price": "100.5", "size": "1.1"}],
                    "asks": [],
                },
            },
        },
    )

    state = repo.get_orderbook_state(venue_id="gmo", market_id="spot")
    assert state is not None
    assert state["last_seq"] == "11"
    assert state["bid_px"] == 100.5
    # ask continuity comes from warm-started persisted book
    assert state["ask_px"] == 101.0


def test_orderbook_warm_start_marks_stale_until_snapshot(tmp_path: Path, monkeypatch) -> None:
    db_file = tmp_path / "md.sqlite3"
    repo = _repo(db_file)
    _reset_orderbook_runtime_state()

    repo.upsert_orderbook_state(
        venue_id="gmo",
        market_id="spot",
        bid_px=100.0,
        bid_qty=1.0,
        ask_px=101.0,
        ask_qty=1.0,
        as_of="2026-02-16T00:00:01Z",
        last_update_ts="2026-02-16T00:00:01Z",
        last_seq="10",
        degraded=False,
        reason=None,
    )

    monkeypatch.setenv("ORDERBOOK_WARM_START_MAX_AGE_SECONDS", "1")
    monkeypatch.setenv("ORDERBOOK_WARM_START_NOW_TS", "2026-02-16T00:10:00Z")

    normalize_envelope(
        repo,
        {
            "raw_msg_id": "delta-old-1",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:10:01Z",
            "event_ts": "2026-02-16T00:10:01Z",
            "seq": 11,
            "stream_name": "orderbooks",
            "payload_json": {
                "type": "delta",
                "changes": {
                    "bids": [{"price": "100.2", "size": "1.2"}],
                    "asks": [],
                },
            },
        },
    )

    stale_state = repo.get_orderbook_state(venue_id="gmo", market_id="spot")
    assert stale_state is not None
    assert stale_state["degraded"] is True
    assert stale_state["reason"] == "ORDERBOOK_STATE_STALE"

    normalize_envelope(
        repo,
        {
            "raw_msg_id": "snap-new-1",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:10:02Z",
            "event_ts": "2026-02-16T00:10:02Z",
            "seq": 12,
            "stream_name": "orderbooks",
            "payload_json": {
                "type": "snapshot",
                "bids": [{"price": "100.3", "size": "1.3"}],
                "asks": [{"price": "101.3", "size": "1.3"}],
            },
        },
    )

    recovered_state = repo.get_orderbook_state(venue_id="gmo", market_id="spot")
    assert recovered_state is not None
    assert recovered_state["degraded"] is False
    assert recovered_state["reason"] is None
    assert recovered_state["last_seq"] == "12"
