import sqlite3
import uuid
from datetime import datetime, timezone

from app.k_ledger_store import append_event
from app.k_ledger_types import LedgerEvent, LedgerRefs
from app.k_sbor import replay


def test_replay_is_deterministic():
    conn = sqlite3.connect(":memory:")

    events = [
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc1", "DEPOSIT", "JPY", "10000", refs=LedgerRefs()),
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc1", "PRICE_MARK", "BTC", "0", price="6000000", quote="JPY", refs=LedgerRefs()),
        LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc1", "TRADE_FILL", "BTC", "0.1", price="6000000", quote="JPY", refs=LedgerRefs()),
    ]
    for event in events:
        assert append_event(conn, event).ok

    s1 = replay(conn)
    s2 = replay(conn)
    assert s1.cash == s2.cash
    assert s1.marks == s2.marks
    assert list(s1.positions.keys()) == list(s2.positions.keys())
