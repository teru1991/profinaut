import sqlite3
import uuid
from datetime import datetime, timezone

from app.k_ledger_store import append_event, verify_chain
from app.k_ledger_types import LedgerEvent, LedgerRefs


def test_ledger_hash_chain_tamper_detected():
    conn = sqlite3.connect(":memory:")

    e1 = LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc1", "DEPOSIT", "JPY", "1000", refs=LedgerRefs())
    e2 = LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc1", "DEPOSIT", "JPY", "2000", refs=LedgerRefs())
    assert append_event(conn, e1).ok
    assert append_event(conn, e2).ok
    assert verify_chain(conn) is True

    conn.execute("UPDATE k_ledger_events SET payload_json='{\"x\":1}' WHERE seq=2")
    conn.commit()
    assert verify_chain(conn) is False
