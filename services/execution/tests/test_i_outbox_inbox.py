import json
import sqlite3

from app.i_outbox import dequeue_next, enqueue
from app.i_scheduler import LANE_PRIORITY


def test_outbox_dedupe_and_dequeue_lane_priority():
    conn = sqlite3.connect(":memory:")
    id1 = enqueue(conn, "LANE2_NEW", "gmo", "BTC_JPY", {"op": "new_order", "x": 1}, "dup-1")
    id2 = enqueue(conn, "LANE2_NEW", "gmo", "BTC_JPY", {"op": "new_order", "x": 2}, "dup-1")
    assert id1 == id2

    enqueue(conn, "LANE0_CANCEL", "gmo", "BTC_JPY", {"op": "cancel", "x": 9}, "dup-cancel-1")
    item = dequeue_next(conn, LANE_PRIORITY)
    assert item is not None
    _, lane, _, _, payload_json, _ = item
    assert lane == "LANE0_CANCEL"
    assert json.loads(payload_json)["op"] == "cancel"
