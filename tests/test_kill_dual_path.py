from __future__ import annotations

from datetime import UTC, datetime
import json
from pathlib import Path

import pytest

from libs.safety_core.audit import AuditEvent
from libs.safety_core.kill import KillRequest, apply_local_kill, apply_ui_kill
from libs.safety_core.models import ScopeKind, SafetyMode, SafetyStateV1
from libs.safety_core.store import InMemorySafetyStore
from worker.local_kill_runner import run_local_kill_if_requested


class DummyAudit:
    def __init__(self, should_fail: bool = False) -> None:
        self.should_fail = should_fail
        self.events: list[AuditEvent] = []

    def write_event(self, event: AuditEvent) -> None:
        if self.should_fail:
            raise OSError("audit down")
        self.events.append(event)


def test_ui_kill_requires_idempotency_reason_evidence() -> None:
    store = InMemorySafetyStore()
    req = KillRequest(
        requested_mode=SafetyMode.SAFE,
        scope_kind=ScopeKind.GLOBAL,
        selector="*",
        ttl_seconds=60,
        reason="",
        actor="tester",
        idempotency_key="",
        evidence={},
    )
    with pytest.raises(ValueError):
        apply_ui_kill(store, DummyAudit(), req, set(), datetime.now(UTC), {"stable_for_seconds": 1, "health_ok": True, "reconcile_ok": True})


def test_local_kill_succeeds_even_if_audit_fails() -> None:
    store = InMemorySafetyStore()
    state = apply_local_kill(store, DummyAudit(should_fail=True), "panic", {"trace_id": "t1"}, now=datetime.now(UTC))
    assert state.mode == SafetyMode.EMERGENCY_STOP


def test_downgrade_rejected_without_checks() -> None:
    store = InMemorySafetyStore()
    store.set_current_state(SafetyStateV1(mode=SafetyMode.EMERGENCY_STOP, reason="halt"))
    req = KillRequest(
        requested_mode=SafetyMode.NORMAL,
        scope_kind=ScopeKind.GLOBAL,
        selector="*",
        ttl_seconds=60,
        reason="resume",
        actor="tester",
        idempotency_key="k1",
        evidence={"trace_id": "t-1"},
    )
    with pytest.raises(PermissionError):
        apply_ui_kill(store, DummyAudit(), req, set(), datetime.now(UTC), {"stable_for_seconds": 0, "health_ok": False, "reconcile_ok": False})


def test_local_kill_runner_file_path(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    kill_file = tmp_path / "kill.json"
    kill_file.write_text(json.dumps({"reason": "manual", "trace_id": "local-1", "audit_id": "a-1"}), encoding="utf-8")

    from worker import local_kill_runner as runner

    class DummyStore(InMemorySafetyStore):
        pass

    dummy_store = DummyStore()
    monkeypatch.setattr(runner, "store", dummy_store)
    monkeypatch.setattr(runner, "audit_writer", DummyAudit())

    assert run_local_kill_if_requested(kill_file=kill_file)
    assert dummy_store.get_current_state() is not None
    assert dummy_store.get_current_state().mode == SafetyMode.EMERGENCY_STOP
