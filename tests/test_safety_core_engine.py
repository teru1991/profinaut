from __future__ import annotations

from datetime import UTC, datetime, timedelta

import pytest

from libs.safety_core.audit import AuditEvent
from libs.safety_core.engine import apply_directive, can_downgrade, compute_decision
from libs.safety_core.models import ScopeKind, SafetyDirective, SafetyMode, SafetyStateV1
from libs.safety_core.store import InMemorySafetyStore


class DummyAudit:
    def __init__(self, should_fail: bool = False) -> None:
        self.should_fail = should_fail
        self.events: list[AuditEvent] = []

    def write_event(self, event: AuditEvent) -> None:
        if self.should_fail:
            raise OSError("audit unavailable")
        self.events.append(event)


def _directive(mode: SafetyMode, ttl_seconds: int = 60) -> SafetyDirective:
    return SafetyDirective(
        scope_kind=ScopeKind.GLOBAL,
        selector="*",
        mode=mode,
        ttl_seconds=ttl_seconds,
        reason=f"set-{mode.value}",
        actor="tester",
        evidence={"trace_id": "trace-1"},
    )


def test_composition_emergency_stop_wins() -> None:
    now = datetime.now(UTC)
    decision = compute_decision([_directive(SafetyMode.SAFE), _directive(SafetyMode.EMERGENCY_STOP)], now)
    assert decision.mode == SafetyMode.EMERGENCY_STOP
    assert decision.latched is True


def test_unknown_or_empty_directives_fail_closed_safe() -> None:
    now = datetime.now(UTC)
    decision = compute_decision([], now)
    assert decision.mode == SafetyMode.SAFE


def test_downgrade_requires_all_checks() -> None:
    assert not can_downgrade(
        SafetyMode.EMERGENCY_STOP,
        SafetyMode.NORMAL,
        {"stable_for_seconds": 0, "health_ok": False, "reconcile_ok": False},
    )


def test_apply_directive_rejects_downgrade_without_checks() -> None:
    store = InMemorySafetyStore()
    store.set_current_state(SafetyStateV1(mode=SafetyMode.EMERGENCY_STOP, reason="halted", activated_by="system"))
    with pytest.raises(PermissionError):
        apply_directive(
            store,
            DummyAudit(),
            _directive(SafetyMode.NORMAL),
            datetime.now(UTC),
            {"stable_for_seconds": 0, "health_ok": False, "reconcile_ok": False},
        )


def test_ttl_expiry_does_not_auto_normalize_state() -> None:
    store = InMemorySafetyStore()
    audit = DummyAudit()
    now = datetime.now(UTC)
    apply_directive(
        store,
        audit,
        _directive(SafetyMode.EMERGENCY_STOP, ttl_seconds=1),
        now,
        {"stable_for_seconds": 300, "health_ok": True, "reconcile_ok": True},
    )

    expired = store.expire_directives(now + timedelta(seconds=2))
    assert len(expired) == 1
    assert store.get_current_state() is not None
    assert store.get_current_state().mode == SafetyMode.EMERGENCY_STOP


def test_emergency_stop_applies_even_when_audit_unavailable() -> None:
    store = InMemorySafetyStore()
    state = apply_directive(
        store,
        DummyAudit(should_fail=True),
        _directive(SafetyMode.EMERGENCY_STOP),
        datetime.now(UTC),
        {"stable_for_seconds": 300, "health_ok": True, "reconcile_ok": True},
    )
    assert state.mode == SafetyMode.EMERGENCY_STOP
