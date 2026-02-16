from __future__ import annotations

import sqlite3

from services.marketdata.app.db.repository import MarketDataMetaRepository, RawIngestMeta
from services.marketdata.app.db.schema import apply_migrations


def _conn() -> sqlite3.Connection:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def test_migrations_create_expected_tables_and_indexes() -> None:
    conn = _conn()
    apply_migrations(conn)

    tables = {
        row[0]
        for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()
    }
    assert {"raw_ingest_meta", "ws_sessions", "ws_subscriptions", "schema_migrations", "md_trades", "md_ohlcv", "md_best_bid_ask", "md_events_json"}.issubset(tables)

    indexes = {
        row[0]
        for row in conn.execute("SELECT name FROM sqlite_master WHERE type='index'").fetchall()
    }
    assert "idx_raw_ingest_meta_received_ts" in indexes
    assert "idx_raw_ingest_meta_venue_market_received_ts" in indexes
    assert "idx_raw_ingest_meta_payload_hash" in indexes
    assert "idx_ws_subscriptions_session_id" in indexes
    assert "uq_md_trades_src_msg_key" in indexes
    assert "uq_md_ohlcv_key" in indexes


def test_repository_insert_and_select_roundtrip() -> None:
    conn = _conn()
    apply_migrations(conn)
    repo = MarketDataMetaRepository(conn)

    repo.insert_raw_ingest_meta(
        RawIngestMeta(
            raw_msg_id="01ARZ3NDEKTSV4RRFFQ69G5FAV",
            tenant_id="tenant-a",
            source_type="WS_PUBLIC",
            venue_id="gmo",
            market_id="spot",
            stream_name="ticker",
            endpoint="/ws",
            event_ts="2026-02-16T00:00:00Z",
            received_ts="2026-02-16T00:00:01Z",
            seq="1001",
            source_msg_key="k1",
            payload_hash="h1",
            payload_size=123,
            object_key="bronze/source=ws_public/venue=gmo/market=spot/date=2026-02-16/hour=00/part-0001.jsonl",
            quality_json={"status": "OK"},
            parser_version="v0.1",
        )
    )

    got = repo.get_raw_ingest_meta("01ARZ3NDEKTSV4RRFFQ69G5FAV")
    assert got is not None
    assert got.raw_msg_id == "01ARZ3NDEKTSV4RRFFQ69G5FAV"
    assert got.tenant_id == "tenant-a"
    assert got.quality_json == {"status": "OK"}


def test_ws_session_and_subscription_fk() -> None:
    conn = _conn()
    apply_migrations(conn)
    repo = MarketDataMetaRepository(conn)

    repo.insert_ws_session(
        session_id="sess-1",
        venue_id="gmo",
        market_id="spot",
        started_at="2026-02-16T00:00:00Z",
        ended_at=None,
        close_reason=None,
        recv_count=10,
        dup_suspect_count=1,
        gap_suspect_count=2,
        lag_stats_json={"p50_ms": 10},
    )
    repo.insert_ws_subscription(
        session_id="sess-1",
        stream_name="ticker",
        subscribed_at="2026-02-16T00:00:01Z",
        unsubscribed_at=None,
        meta_json={"symbol": "BTC_JPY"},
    )

    count = conn.execute("SELECT COUNT(*) FROM ws_subscriptions WHERE session_id='sess-1'").fetchone()[0]
    assert count == 1
