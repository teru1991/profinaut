from __future__ import annotations

import hashlib
import json
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any


@dataclass(frozen=True)
class AuditAppendResult:
    ok: bool
    seq: int | None
    record_hash: str | None
    reason: str
    evidence: dict[str, Any]


def ensure_audit_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS e_audit_chain (
          seq INTEGER PRIMARY KEY AUTOINCREMENT,
          ts_utc TEXT NOT NULL,
          prev_hash TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          record_hash TEXT NOT NULL
        )
        """
    )
    conn.execute("CREATE INDEX IF NOT EXISTS e_audit_chain_ts ON e_audit_chain(ts_utc)")
    conn.commit()


def _hash(prev_hash: str, payload_json: str) -> str:
    h = hashlib.sha256()
    h.update(prev_hash.encode("utf-8"))
    h.update(b"\n")
    h.update(payload_json.encode("utf-8"))
    return h.hexdigest()


def head_hash(conn: sqlite3.Connection) -> str:
    ensure_audit_schema(conn)
    row = conn.execute("SELECT record_hash FROM e_audit_chain ORDER BY seq DESC LIMIT 1").fetchone()
    return str(row[0]) if row else "GENESIS"


def append_audit(conn: sqlite3.Connection, payload: dict[str, Any]) -> AuditAppendResult:
    ensure_audit_schema(conn)
    ts = datetime.now(timezone.utc).isoformat()
    prev = head_hash(conn)
    payload_json = json.dumps(payload, separators=(",", ":"), sort_keys=True)
    rh = _hash(prev, payload_json)
    try:
        cur = conn.execute(
            "INSERT INTO e_audit_chain(ts_utc,prev_hash,payload_json,record_hash) VALUES(?,?,?,?)",
            (ts, prev, payload_json, rh),
        )
        conn.commit()
        return AuditAppendResult(True, int(cur.lastrowid), rh, "ok", {"prev_hash": prev})
    except Exception as e:
        return AuditAppendResult(False, None, None, "append_failed", {"error": str(e)})


def verify_chain(conn: sqlite3.Connection, limit: int | None = None) -> bool:
    ensure_audit_schema(conn)
    rows = conn.execute("SELECT seq,prev_hash,payload_json,record_hash FROM e_audit_chain ORDER BY seq ASC").fetchall()
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
