import sqlite3
import uuid
from datetime import datetime, timezone

from app.k_ledger_store import append_event
from app.k_ledger_types import LedgerEvent, LedgerRefs
from app.k_pnl import compute_pnl
from app.k_sbor import replay


def test_pnl_fifo_realized():
    conn = sqlite3.connect(":memory:")

    events = [
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc", "TRADE_FILL", "AAA", "1", price="100", quote="JPY", refs=LedgerRefs()),
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc", "TRADE_FILL", "AAA", "1", price="200", quote="JPY", refs=LedgerRefs()),
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc", "TRADE_FILL", "AAA", "-1", price="300", quote="JPY", refs=LedgerRefs()),
    ]
    for event in events:
        assert append_event(conn, event).ok

    state = replay(conn)
    pnl = compute_pnl(conn, state)
    assert pnl.realized[("acc", "JPY")] == 200
