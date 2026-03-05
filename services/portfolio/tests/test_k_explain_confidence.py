import sqlite3
import uuid
from datetime import datetime, timezone

from app.k_explain import explain
from app.k_ledger_store import append_event
from app.k_ledger_types import LedgerEvent, LedgerRefs
from app.k_sbor import replay


def test_explain_confidence_decreases_when_mark_missing():
    conn = sqlite3.connect(":memory:")

    event = LedgerEvent(1, str(uuid.uuid4()), datetime.now(timezone.utc), "test", "acc", "TRADE_FILL", "BTC", "0.1", price="6000000", quote="JPY", refs=LedgerRefs())
    assert append_event(conn, event).ok
    state = replay(conn)
    rep = explain(conn, state)
    assert rep.confidence_score < 1.0
    assert any(r.startswith("missing_mark:") for r in rep.confidence_reasons)
