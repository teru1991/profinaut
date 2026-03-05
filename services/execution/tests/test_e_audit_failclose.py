import sqlite3

from app.e_audit_chain import append_audit, verify_chain


def test_audit_chain_detects_tamper():
    conn = sqlite3.connect(":memory:")
    r1 = append_audit(conn, {"a": 1})
    r2 = append_audit(conn, {"a": 2})
    assert r1.ok and r2.ok
    assert verify_chain(conn) is True

    conn.execute("UPDATE e_audit_chain SET payload_json='{\"a\":999}' WHERE seq=2")
    conn.commit()
    assert verify_chain(conn) is False
