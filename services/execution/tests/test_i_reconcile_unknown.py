import sqlite3

from app.i_events import ensure_events_schema
from app.i_reconcile import reconcile_open_orders


def test_reconcile_converges_unknown_to_canceled_event():
    conn = sqlite3.connect(":memory:")
    ensure_events_schema(conn)

    local = {
        "cid-1": {"state": "UNKNOWN"},
        "cid-2": {"state": "LIVE"},
    }
    report = reconcile_open_orders(conn, local, {"cid-2"})
    assert report.suggested_mode in {"SAFE", "CANCEL_ONLY"}
    assert len(report.diffs) == 1
    assert report.diffs[0]["client_order_id"] == "cid-1"
