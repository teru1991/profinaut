from __future__ import annotations

import json
import sqlite3
import uuid
from datetime import datetime, timedelta, timezone
from typing import Any, Optional

from app.i_types import Lane


def ensure_outbox_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS i_outbox (
          outbox_id TEXT PRIMARY KEY,
          lane TEXT NOT NULL,
          venue TEXT NOT NULL,
          symbol TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          status TEXT NOT NULL,
          dedupe_key TEXT NOT NULL,
          attempt INTEGER NOT NULL,
          next_attempt_at_utc TEXT NOT NULL
        )
        """
    )
    conn.execute("CREATE INDEX IF NOT EXISTS i_outbox_status_next ON i_outbox(status,next_attempt_at_utc)")
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS i_outbox_dedupe ON i_outbox(dedupe_key)")
    conn.commit()


def enqueue(conn: sqlite3.Connection, lane: Lane, venue: str, symbol: str, payload: dict[str, Any], dedupe_key: str) -> str:
    ensure_outbox_schema(conn)
    outbox_id = str(uuid.uuid4())
    now = datetime.now(timezone.utc).isoformat()
    conn.execute(
        """
        INSERT OR IGNORE INTO i_outbox(outbox_id,lane,venue,symbol,payload_json,status,dedupe_key,attempt,next_attempt_at_utc)
        VALUES(?,?,?,?,?,'PENDING',?,0,?)
        """,
        (outbox_id, lane, venue, symbol, json.dumps(payload, separators=(",", ":"), sort_keys=True), dedupe_key, now),
    )
    conn.commit()
    row = conn.execute("SELECT outbox_id FROM i_outbox WHERE dedupe_key=?", (dedupe_key,)).fetchone()
    return str(row[0])


def mark_blocked(conn: sqlite3.Connection, outbox_id: str) -> None:
    ensure_outbox_schema(conn)
    conn.execute("UPDATE i_outbox SET status='BLOCKED' WHERE outbox_id=?", (outbox_id,))
    conn.commit()


def mark_inflight(conn: sqlite3.Connection, outbox_id: str) -> None:
    ensure_outbox_schema(conn)
    conn.execute("UPDATE i_outbox SET status='INFLIGHT' WHERE outbox_id=?", (outbox_id,))
    conn.commit()


def mark_sent(conn: sqlite3.Connection, outbox_id: str) -> None:
    ensure_outbox_schema(conn)
    conn.execute("UPDATE i_outbox SET status='SENT' WHERE outbox_id=?", (outbox_id,))
    conn.commit()


def mark_failed_with_backoff(conn: sqlite3.Connection, outbox_id: str, attempt: int) -> None:
    ensure_outbox_schema(conn)
    delay = min(60, max(1, 2 ** min(attempt, 6)))
    next_at = (datetime.now(timezone.utc) + timedelta(seconds=delay)).isoformat()
    conn.execute(
        "UPDATE i_outbox SET status='FAILED', attempt=?, next_attempt_at_utc=? WHERE outbox_id=?",
        (attempt, next_at, outbox_id),
    )
    conn.commit()


def dequeue_next(conn: sqlite3.Connection, lanes_priority: list[str]) -> Optional[tuple[str, str, str, str, str, int]]:
    ensure_outbox_schema(conn)
    now = datetime.now(timezone.utc).isoformat()
    for lane in lanes_priority:
        row = conn.execute(
            """
            SELECT outbox_id,lane,venue,symbol,payload_json,attempt
            FROM i_outbox
            WHERE status IN ('PENDING','FAILED') AND lane=? AND next_attempt_at_utc<=?
            ORDER BY next_attempt_at_utc ASC
            LIMIT 1
            """,
            (lane, now),
        ).fetchone()
        if row:
            return (str(row[0]), str(row[1]), str(row[2]), str(row[3]), str(row[4]), int(row[5]))
    return None
