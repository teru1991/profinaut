import sqlite3

from app.execution_worker import LiveSender, worker_step
from app.i_gate import GateContext
from app.i_outbox import enqueue


def test_gate_deny_blocks_send():
    conn = sqlite3.connect(":memory:")
    enqueue(conn, "LANE2_NEW", "gmo", "BTC_JPY", {"op": "new_order"}, "dedupe-x")

    called = {"n": 0}

    def send(payload):
        called["n"] += 1

    ctx = GateContext(
        action="ORDER_INTENT",
        exchange="gmo",
        safe_mode="NORMAL",
        live_enabled=True,
        live_mode="live",
        live_backoff_until_utc=None,
        actor_role="oncall",
        current_mode="SAFE",
        metrics_ok=None,
        clock_ok=None,
        audit_ok=None,
        lease_ok=None,
        deps_ok=None,
    )

    did = worker_step(conn, LiveSender(send_func=send), ctx)
    assert did is True
    assert called["n"] == 0
