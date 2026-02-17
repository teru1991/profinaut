from __future__ import annotations

import sqlite3
from pathlib import Path

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.metrics import ingest_metrics
from services.marketdata.app.silver.normalizer import normalize_envelope


def _repo(db_file: Path) -> MarketDataMetaRepository:
    conn = sqlite3.connect(db_file)
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


def test_trade_duplicate_message_is_deduped_by_upsert_key(tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    repo = _repo(db_file)

    base = {
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "venue_id": "gmo",
        "market_id": "spot",
        "instrument_id": "btc_jpy",
        "event_ts": "2026-02-16T00:00:01Z",
        "received_ts": "2026-02-16T00:00:01Z",
        "seq": 123,
        "payload_json": {"trade_id": "tid-100", "price": 100.0, "qty": 0.5, "side": "buy"},
    }

    normalize_envelope(repo, {**base, "raw_msg_id": "msg-1"})
    normalize_envelope(repo, {**base, "raw_msg_id": "msg-2"})

    conn = sqlite3.connect(db_file)
    trades = conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0]
    assert trades == 1

    stats = ingest_metrics.summary()
    assert int(stats["dup_suspect_total"]) >= 1


def test_trade_reordered_arrival_uses_stable_composite_key(tmp_path: Path) -> None:
    ingest_metrics.reset_for_tests()
    db_file = tmp_path / "md.sqlite3"
    repo = _repo(db_file)

    first = {
        "raw_msg_id": "msg-a",
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "venue_id": "gmo",
        "market_id": "spot",
        "instrument_id": "btc_jpy",
        "event_ts": "2026-02-16T00:00:02Z",
        "received_ts": "2026-02-16T00:00:03Z",
        "seq": 999,
        "payload_json": {"price": 101.0, "qty": 1.2, "side": "sell"},
    }
    second = {
        "raw_msg_id": "msg-b",
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "venue_id": "gmo",
        "market_id": "spot",
        "instrument_id": "btc_jpy",
        "event_ts": "2026-02-16T00:00:02Z",
        "received_ts": "2026-02-16T00:00:01Z",
        "seq": 999,
        "payload_json": {"price": 101.0, "qty": 1.2, "side": "sell"},
    }

    normalize_envelope(repo, first)
    normalize_envelope(repo, second)

    conn = sqlite3.connect(db_file)
    rows = conn.execute("SELECT source_msg_key, price, qty, side FROM md_trades").fetchall()
    assert len(rows) == 1
    key, price, qty, side = rows[0]
    assert isinstance(key, str) and key.startswith("trade|gmo|spot|2026-02-16T00:00:02Z|101.0|1.2|sell|999")
    assert (price, qty, side) == (101.0, 1.2, "sell")

    stats = ingest_metrics.summary()
    assert int(stats["dup_suspect_total"]) >= 1
