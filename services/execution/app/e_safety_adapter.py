from __future__ import annotations

import sqlite3
from dataclasses import asdict
from typing import Any

from app.e_audit_chain import append_audit, verify_chain
from app.e_interlocks import InterlockInputs, decide as decide_interlocks
from app.e_lease import default_lease_reader


def evaluate_send_safety(conn: sqlite3.Connection, payload: dict[str, Any], gate_ctx_base: Any) -> tuple[bool, dict[str, Any]]:
    lease_status = default_lease_reader().read()
    audit_ok = verify_chain(conn, limit=50)
    inter = decide_interlocks(
        InterlockInputs(
            metrics_ok=getattr(gate_ctx_base, "metrics_ok", None),
            clock_ok=getattr(gate_ctx_base, "clock_ok", None),
            deps_ok=getattr(gate_ctx_base, "deps_ok", None),
            lease_ok=lease_status.ok,
            audit_ok=audit_ok,
            reconcile_ok=True,
        )
    )

    op = str(payload.get("op", ""))
    is_new_replace = op in {"new_order", "replace"}
    block = False
    if inter.mode == "HALT":
        block = True
    elif inter.mode == "CANCEL_ONLY" and is_new_replace:
        block = True

    return (not block), {
        "lease": asdict(lease_status),
        "audit_ok": audit_ok,
        "interlock": asdict(inter),
    }


def append_send_intent_audit(conn: sqlite3.Connection, outbox_id: str, lane: str, venue: str, symbol: str, payload: dict[str, Any], safety_evidence: dict[str, Any], gate_evidence: dict[str, Any]) -> tuple[bool, dict[str, Any]]:
    ap = append_audit(
        conn,
        {
            "kind": "EXEC_SEND_INTENT",
            "outbox_id": outbox_id,
            "lane": lane,
            "venue": venue,
            "symbol": symbol,
            "payload": payload,
            "safety": safety_evidence,
            "gate": gate_evidence,
        },
    )
    return ap.ok, asdict(ap)
