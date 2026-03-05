from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any


@dataclass(frozen=True)
class EventRecord:
    seq: int
    kind: str
    payload: dict[str, Any]
    ts_utc: datetime


def ensure_events_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS i_events (
          seq INTEGER PRIMARY KEY AUTOINCREMENT,
          kind TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          ts_utc TEXT NOT NULL
        )
        """
    )
    conn.commit()


def append_event(conn: sqlite3.Connection, kind: str, payload: dict[str, Any]) -> int:
    ensure_events_schema(conn)
    ts = datetime.now(timezone.utc).isoformat()
    cur = conn.execute(
        "INSERT INTO i_events(kind,payload_json,ts_utc) VALUES(?,?,?)",
        (kind, json.dumps(payload, separators=(",", ":"), sort_keys=True), ts),
    )
    conn.commit()
    return int(cur.lastrowid)
