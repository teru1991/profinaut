import sqlite3

from app.execution_worker import LiveSender, worker_step
from app.i_gate import GateContext
from app.i_outbox import enqueue


def test_send_is_blocked_when_lease_not_ok():
    conn = sqlite3.connect(":memory:")
    enqueue(conn, "LANE2_NEW", "gmo", "BTC_JPY", {"op": "new_order"}, "dedupe-hard-1")

    called = {"n": 0}

    def send(_):
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
        metrics_ok=True,
        clock_ok=True,
        audit_ok=True,
        lease_ok=False,
        deps_ok=True,
    )

    did = worker_step(conn, LiveSender(send_func=send), ctx)
    assert did is True
    assert called["n"] == 0
