from __future__ import annotations

import sqlite3
from datetime import datetime, timezone


def ensure_inbox_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS i_inbox (
          source TEXT NOT NULL,
          payload_hash TEXT NOT NULL,
          received_at_utc TEXT NOT NULL,
          PRIMARY KEY (source, payload_hash)
        )
        """
    )
    conn.commit()


def seen_before(conn: sqlite3.Connection, source: str, payload_hash: str) -> bool:
    ensure_inbox_schema(conn)
    row = conn.execute(
        "SELECT 1 FROM i_inbox WHERE source=? AND payload_hash=?",
        (source, payload_hash),
    ).fetchone()
    return row is not None


def mark_seen(conn: sqlite3.Connection, source: str, payload_hash: str) -> None:
    ensure_inbox_schema(conn)
    ts = datetime.now(timezone.utc).isoformat()
    conn.execute(
        "INSERT OR IGNORE INTO i_inbox(source,payload_hash,received_at_utc) VALUES(?,?,?)",
        (source, payload_hash, ts),
    )
    conn.commit()
