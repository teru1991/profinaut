from __future__ import annotations

import json
import sqlite3
import time
from dataclasses import dataclass
from typing import Any, Callable

from app.e_safety_adapter import append_send_intent_audit, evaluate_send_safety
from app.i_events import append_event, ensure_events_schema
from app.i_gate import GateContext, check_gate
from app.i_outbox import dequeue_next, mark_blocked, mark_failed_with_backoff, mark_inflight, mark_sent
from app.i_scheduler import LANE_PRIORITY, release, try_acquire


@dataclass(frozen=True)
class LiveSender:
    send_func: Callable[[dict[str, Any]], Any]


def worker_step(conn: sqlite3.Connection, sender: LiveSender, gate_ctx_base: GateContext) -> bool:
    item = dequeue_next(conn, LANE_PRIORITY)
    if not item:
        return False

    outbox_id, lane, venue, symbol, payload_json, attempt = item
    if not try_acquire(conn, venue, symbol, outbox_id):
        return False

    try:
        mark_inflight(conn, outbox_id)
        payload = json.loads(payload_json)

        # E safety physical enforcement (lease + audit verify + interlocks)
        safety_ok, safety_evidence = evaluate_send_safety(conn, payload, gate_ctx_base)
        if not safety_ok:
            mark_blocked(conn, outbox_id)
            append_event(conn, "OUTBOX_BLOCKED", {"outbox_id": outbox_id, "evidence": {"safety": safety_evidence}})
            return True

        action = "ORDER_INTENT"
        op = str(payload.get("op", ""))
        if op == "cancel":
            action = "CANCEL"
        elif op == "replace":
            action = "REPLACE"

        allow, gate_evidence = check_gate(
            GateContext(
                action=action,
                exchange=gate_ctx_base.exchange,
                safe_mode=gate_ctx_base.safe_mode,
                live_enabled=gate_ctx_base.live_enabled,
                live_mode=gate_ctx_base.live_mode,
                live_backoff_until_utc=gate_ctx_base.live_backoff_until_utc,
                actor_role=gate_ctx_base.actor_role,
                current_mode=gate_ctx_base.current_mode,
                metrics_ok=gate_ctx_base.metrics_ok,
                clock_ok=gate_ctx_base.clock_ok,
                audit_ok=gate_ctx_base.audit_ok,
                lease_ok=gate_ctx_base.lease_ok,
                deps_ok=gate_ctx_base.deps_ok,
            )
        )
        if not allow:
            mark_blocked(conn, outbox_id)
            append_event(conn, "OUTBOX_BLOCKED", {"outbox_id": outbox_id, "evidence": {"gate": gate_evidence}})
            return True

        # audit append must succeed before send
        audit_append_ok, audit_append_evidence = append_send_intent_audit(
            conn, outbox_id, lane, venue, symbol, payload, safety_evidence, gate_evidence
        )
        if not audit_append_ok:
            mark_blocked(conn, outbox_id)
            append_event(conn, "OUTBOX_BLOCKED", {"outbox_id": outbox_id, "evidence": {"audit_append": audit_append_evidence}})
            return True

        append_event(conn, "OUTBOX_SEND_ATTEMPT", {"outbox_id": outbox_id, "attempt": attempt, "lane": lane})
        send_result = sender.send_func(payload)
        mark_sent(conn, outbox_id)
        append_event(conn, "OUTBOX_SENT", {"outbox_id": outbox_id, "result": send_result})
        return True
    except Exception as e:
        append_event(conn, "OUTBOX_SEND_FAILED", {"outbox_id": outbox_id, "error": str(e), "attempt": attempt})
        mark_failed_with_backoff(conn, outbox_id, attempt + 1)
        return True
    finally:
        release(conn, venue, symbol, outbox_id)


def run_worker_loop(conn: sqlite3.Connection, sender: LiveSender, gate_ctx_base: GateContext, idle_sleep_sec: float = 0.2) -> None:
    ensure_events_schema(conn)
    while True:
        if not worker_step(conn, sender, gate_ctx_base):
            time.sleep(idle_sleep_sec)
