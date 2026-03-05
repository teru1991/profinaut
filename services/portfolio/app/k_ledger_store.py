from __future__ import annotations

import hashlib
import json
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Optional

from app.k_ledger_types import LedgerEvent, event_to_payload


@dataclass(frozen=True)
class AppendResult:
    ok: bool
    seq: Optional[int]
    record_hash: Optional[str]
    reason: str
    evidence: dict[str, Any]


def ensure_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS k_ledger_events (
          seq INTEGER PRIMARY KEY AUTOINCREMENT,
          ts_utc TEXT NOT NULL,
          prev_hash TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          record_hash TEXT NOT NULL
        )
        """
    )
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS k_ledger_idempotency (
          event_id TEXT PRIMARY KEY,
          seq INTEGER NOT NULL
        )
        """
    )
    conn.commit()


def _hash(prev_hash: str, payload_json: str) -> str:
    h = hashlib.sha256()
    h.update(prev_hash.encode("utf-8"))
    h.update(b"\n")
    h.update(payload_json.encode("utf-8"))
    return h.hexdigest()


def head_hash(conn: sqlite3.Connection) -> str:
    ensure_schema(conn)
    row = conn.execute("SELECT record_hash FROM k_ledger_events ORDER BY seq DESC LIMIT 1").fetchone()
    return str(row[0]) if row else "GENESIS"


def append_event(conn: sqlite3.Connection, event: LedgerEvent) -> AppendResult:
    ensure_schema(conn)
    payload = event_to_payload(event)
    payload_json = json.dumps(payload, separators=(",", ":"), sort_keys=True)
    prev = head_hash(conn)
    rh = _hash(prev, payload_json)
    ts = datetime.now(timezone.utc).isoformat()

    try:
        row = conn.execute("SELECT seq FROM k_ledger_idempotency WHERE event_id=?", (event.event_id,)).fetchone()
        if row:
            return AppendResult(ok=True, seq=int(row[0]), record_hash=None, reason="idempotent_hit", evidence={"event_id": event.event_id})

        cur = conn.execute(
            "INSERT INTO k_ledger_events(ts_utc,prev_hash,payload_json,record_hash) VALUES(?,?,?,?)",
            (ts, prev, payload_json, rh),
        )
        seq = int(cur.lastrowid)
        conn.execute("INSERT INTO k_ledger_idempotency(event_id,seq) VALUES(?,?)", (event.event_id, seq))
        conn.commit()
        return AppendResult(ok=True, seq=seq, record_hash=rh, reason="ok", evidence={"prev_hash": prev})
    except Exception as ex:
        conn.rollback()
        return AppendResult(ok=False, seq=None, record_hash=None, reason="append_failed", evidence={"error": str(ex)})


def iter_payloads(conn: sqlite3.Connection) -> list[dict[str, Any]]:
    ensure_schema(conn)
    rows = conn.execute("SELECT payload_json FROM k_ledger_events ORDER BY seq ASC").fetchall()
    return [json.loads(r[0]) for r in rows]


def verify_chain(conn: sqlite3.Connection, limit: Optional[int] = None) -> bool:
    ensure_schema(conn)
    rows = conn.execute("SELECT seq,prev_hash,payload_json,record_hash FROM k_ledger_events ORDER BY seq ASC").fetchall()
    if limit is not None:
        rows = rows[-limit:]
    prev = "GENESIS"
    for _, prev_hash, payload_json, record_hash in rows:
        if prev_hash != prev:
            return False
        if _hash(prev, payload_json) != record_hash:
            return False
        prev = record_hash
    return True
