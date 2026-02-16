from __future__ import annotations

import sqlite3

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.session_recorder import SessionRecorder


def _conn() -> sqlite3.Connection:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def test_session_recorder_start_and_clean_shutdown_flow() -> None:
    conn = _conn()
    apply_migrations(conn)
    repo = MarketDataMetaRepository(conn)
    recorder = SessionRecorder(repo)

    recorder.start_session(
        session_id="sess-42",
        venue_id="gmo",
        market_id="spot",
        started_at="2026-02-16T00:00:00Z",
    )
    recorder.record_subscription(
        stream_name="ticker",
        subscribed_at="2026-02-16T00:00:01Z",
        meta_json={"symbol": "BTC_JPY"},
    )
    recorder.record_received_message(lag_ms=11)
    recorder.record_received_message(dup_suspect=True, lag_ms=9)
    recorder.record_received_message(gap_suspect=True)

    recorder.end_session(close_reason="shutdown", ended_at="2026-02-16T00:01:00Z")

    session_row = conn.execute(
        """
        SELECT ended_at, close_reason, recv_count, dup_suspect_count, gap_suspect_count, lag_stats_json
        FROM ws_sessions
        WHERE session_id = 'sess-42'
        """
    ).fetchone()
    assert session_row is not None
    assert session_row["ended_at"] == "2026-02-16T00:01:00Z"
    assert session_row["close_reason"] == "shutdown"
    assert session_row["recv_count"] == 3
    assert session_row["dup_suspect_count"] == 1
    assert session_row["gap_suspect_count"] == 1
    assert session_row["lag_stats_json"] == '{"count":2,"min_ms":9,"max_ms":11,"last_ms":9}'

    sub_row = conn.execute(
        """
        SELECT subscribed_at, unsubscribed_at
        FROM ws_subscriptions
        WHERE session_id = 'sess-42' AND stream_name = 'ticker'
        """
    ).fetchone()
    assert sub_row == ("2026-02-16T00:00:01Z", "2026-02-16T00:01:00Z")
