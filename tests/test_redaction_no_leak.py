from __future__ import annotations

from pathlib import Path

import pytest

from libs.safety_core.audit import AuditEvent, AuditLeakError, JsonlAuditWriter
from libs.safety_core.redaction import redact, redact_text, safe_str, scan_obj, scan_text


def test_redact_text_masks_authorization_and_jwt() -> None:
    s = "Authorization: Bearer abcdef0123456789TOKENVALUE and jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.aaaa.bbbb"
    out = redact_text(s)
    assert "TOKENVALUE" not in out
    assert "eyJhbGci" not in out  # masked
    assert "***" in out


def test_redact_text_masks_pem_private_key_block() -> None:
    pem = "-----BEGIN PRIVATE KEY-----\nABCDEF0123456789\n-----END PRIVATE KEY-----"
    out = redact_text(pem)
    assert out == "***REDACTED_PEM_PRIVATE_KEY***"


def test_scan_text_detects_secrets_and_near_secrets() -> None:
    s = "token=supersecretvalue1234567890 AKIA1234567890ABCD12 0123456789abcdef0123456789abcdef"
    findings = scan_text(s)
    kinds = {f.kind for f in findings}
    assert "query_token" in kinds
    assert "aws_access_key_id" in kinds
    assert "long_hex" in kinds


def test_scan_obj_detects_secret_key_names() -> None:
    obj = {"api_key": "abcdef0123456789TOKENVALUE", "nested": {"password": "pw123456"}}
    findings = scan_obj(obj)
    kinds = {f.kind for f in findings}
    assert "secret_key_name" in kinds


def test_redact_masks_values_recursively() -> None:
    obj = {"api_key": "abcdef0123456789TOKENVALUE", "x": ["Bearer abcdef0123456789", {"token": "zzzzzz"}]}
    out = redact(obj)
    # key-based
    assert out["api_key"] == "***REDACTED***"
    # content-based (Bearer masked)
    assert "abcdef0123456789" not in str(out)
    assert "***" in str(out)


def test_audit_writer_refuses_payload_with_secret_indicators(tmp_path: Path) -> None:
    w = JsonlAuditWriter(log_dir=tmp_path)

    clean = AuditEvent(
        event_type="TEST",
        actor="tester",
        scope="LEASE",
        mode_from="NORMAL",
        mode_to="NORMAL",
        reason="normal-mode transition",
        ttl=1,
        evidence_ref={"trace_id": "ok"},
    )
    w.write_event(clean)

    ev = AuditEvent(
        event_type="TEST",
        actor="tester",
        scope="LEASE",
        mode_from="NORMAL",
        mode_to="NORMAL",
        reason="Authorization: Bearer abcdef0123456789TOKENVALUE",
        ttl=1,
        evidence_ref={"trace_id": "token=supersecretvalue1234567890"},
    )
    with pytest.raises(AuditLeakError) as exc:
        w.write_event(ev)

    ev2 = AuditEvent(
        event_type="TEST",
        actor="tester",
        scope="LEASE",
        mode_from="NORMAL",
        mode_to="NORMAL",
        reason="ok",
        ttl=1,
        evidence_ref={"api_key": "abcdef"},
    )
    with pytest.raises(AuditLeakError):
        w.write_event(ev2)

    assert "abcdef" not in safe_str(str(exc.value))
