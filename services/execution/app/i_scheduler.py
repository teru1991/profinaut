from __future__ import annotations

import sqlite3

LANE_PRIORITY = ["LANE0_CANCEL", "LANE0_FLATTEN", "LANE1_REPLACE", "LANE2_NEW"]


def ensure_inflight_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS i_inflight (
          key TEXT PRIMARY KEY,
          outbox_id TEXT NOT NULL
        )
        """
    )
    conn.commit()


def key_for(venue: str, symbol: str) -> str:
    return f"{venue}::{symbol}"


def try_acquire(conn: sqlite3.Connection, venue: str, symbol: str, outbox_id: str) -> bool:
    ensure_inflight_schema(conn)
    k = key_for(venue, symbol)
    try:
        conn.execute("INSERT INTO i_inflight(key,outbox_id) VALUES(?,?)", (k, outbox_id))
        conn.commit()
        return True
    except Exception:
        return False


def release(conn: sqlite3.Connection, venue: str, symbol: str, outbox_id: str) -> None:
    ensure_inflight_schema(conn)
    conn.execute("DELETE FROM i_inflight WHERE key=? AND outbox_id=?", (key_for(venue, symbol), outbox_id))
    conn.commit()
