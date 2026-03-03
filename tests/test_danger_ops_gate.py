from __future__ import annotations

import pytest

from libs.safety_core.audit_health import AuditHealth
from libs.safety_core.danger_ops import confirm, issue_challenge
from libs.safety_core.errors import SecError
from libs.safety_core.session import Actor, Session


def test_danger_ops_requires_step_up_and_valid_token(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("FILEENC_PASSPHRASE", "pw-123456")

    audit = AuditHealth()
    audit.mark_ok()

    s_basic = Session(actor=Actor(kind="human", actor_id="u1"), auth_strength="basic", session_id="s1", mode="dev")
    ch = issue_challenge(session=s_basic, op="start_live", scope="bot:example:paper")

    with pytest.raises(SecError):
        confirm(session=s_basic, op="start_live", scope="bot:example:paper", token=ch.token, audit_health=audit)

    s_step = Session(actor=Actor(kind="human", actor_id="u1"), auth_strength="step_up", session_id="s1", mode="dev")
    confirm(session=s_step, op="start_live", scope="bot:example:paper", token=ch.token, audit_health=audit)


def test_danger_ops_denied_when_audit_down(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("FILEENC_PASSPHRASE", "pw-123456")

    audit = AuditHealth()
    audit.mark_err("E_AUDIT_WRITE")

    s_step = Session(actor=Actor(kind="human", actor_id="u1"), auth_strength="step_up", session_id="s1", mode="dev")
    ch = issue_challenge(session=s_step, op="rotate_secret", scope="*")

    with pytest.raises(SecError):
        confirm(session=s_step, op="rotate_secret", scope="*", token=ch.token, audit_health=audit)
