import json
import sqlite3

from app.i_outbox import dequeue_next, enqueue
from app.i_scheduler import LANE_PRIORITY


def test_lane0_not_starved_by_lane2():
    conn = sqlite3.connect(":memory:")
    for i in range(50):
        enqueue(conn, "LANE2_NEW", "gmo", "BTC_JPY", {"op": "new_order", "i": i}, f"new-{i}")
    enqueue(conn, "LANE0_CANCEL", "gmo", "BTC_JPY", {"op": "cancel"}, "cancel-1")

    item = dequeue_next(conn, LANE_PRIORITY)
    assert item is not None
    _, lane, _, _, payload_json, _ = item
    assert lane == "LANE0_CANCEL"
    assert json.loads(payload_json)["op"] == "cancel"
