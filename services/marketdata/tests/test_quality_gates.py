from __future__ import annotations

import sqlite3
from pathlib import Path

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.metrics import quality_gate_metrics
from services.marketdata.app.silver.normalizer import normalize_envelope


def _repo(path: Path) -> MarketDataMetaRepository:
    conn = sqlite3.connect(path)
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


def test_trade_negative_values_create_anomaly_event(tmp_path: Path) -> None:
    quality_gate_metrics.reset_for_tests()
    repo = _repo(tmp_path / "md.sqlite3")
    out = normalize_envelope(
        repo,
        {
            "raw_msg_id": "t-neg",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:00:01Z",
            "payload_json": {"price": -1, "qty": 1, "side": "buy"},
        },
    )
    assert out.target == "md_events_json"
    conn = repo._conn
    assert conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0] == 0
    assert conn.execute("SELECT COUNT(*) FROM md_events_json WHERE event_type='md_data_anomaly'").fetchone()[0] == 1
    assert quality_gate_metrics.summary()["anomaly_total"] >= 1


def test_ohlcv_invalid_range_creates_anomaly(tmp_path: Path) -> None:
    quality_gate_metrics.reset_for_tests()
    repo = _repo(tmp_path / "md.sqlite3")
    out = normalize_envelope(
        repo,
        {
            "raw_msg_id": "o-bad",
            "tenant_id": "tenant-a",
            "source_type": "REST",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:00:01Z",
            "payload_json": {
                "timeframe": "1m",
                "open_ts": "2026-02-16T00:00:00Z",
                "open": 1.0,
                "high": 0.9,
                "low": 0.8,
                "close": 1.1,
                "volume": 1.0,
                "is_final": True,
            },
        },
    )
    assert out.target == "md_events_json"
    conn = repo._conn
    assert conn.execute("SELECT COUNT(*) FROM md_ohlcv").fetchone()[0] == 0
    assert conn.execute("SELECT COUNT(*) FROM md_events_json WHERE event_type='md_data_anomaly'").fetchone()[0] == 1


def test_orderbook_crossed_marks_degraded_data_invalid(tmp_path: Path) -> None:
    quality_gate_metrics.reset_for_tests()
    repo = _repo(tmp_path / "md.sqlite3")
    normalize_envelope(
        repo,
        {
            "raw_msg_id": "ob-cross",
            "tenant_id": "tenant-a",
            "source_type": "WS_PUBLIC",
            "venue_id": "gmo",
            "market_id": "spot",
            "received_ts": "2026-02-16T00:00:01Z",
            "seq": 1,
            "stream_name": "orderbooks",
            "payload_json": {
                "type": "snapshot",
                "bids": [{"price": "101", "size": "1"}],
                "asks": [{"price": "100", "size": "1"}],
            },
        },
    )
    state = repo.get_orderbook_state(venue_id="gmo", market_id="spot")
    assert state is not None
    assert state["degraded"] is True
    assert state["reason"] == "DATA_INVALID"
    assert repo._conn.execute("SELECT COUNT(*) FROM md_events_json WHERE event_type='md_data_anomaly'").fetchone()[0] >= 1
