from __future__ import annotations

from datetime import UTC, datetime
import json
from pathlib import Path


from libs.safety_core.engine import compute_decision
from libs.safety_core.models import ScopeKind, SafetyDirective, SafetyMode, SafetyStateV1
from libs.safety_core.store import JsonFileSafetyStore


def test_json_file_store_persist_and_load(tmp_path: Path) -> None:
    path = tmp_path / "safety_state.json"
    store = JsonFileSafetyStore(path)
    state = SafetyStateV1(mode=SafetyMode.SAFE, reason="manual", activated_by="tester")
    store.set_current_state(state)
    store.put_directive(
        SafetyDirective(
            scope_kind=ScopeKind.GLOBAL,
            selector="*",
            mode=SafetyMode.SAFE,
            ttl_seconds=30,
            reason="safe",
            actor="tester",
            evidence={"trace_id": "t-1"},
        )
    )

    assert path.exists()
    raw = json.loads(path.read_text(encoding="utf-8"))
    assert "state" in raw and "directives" in raw

    loaded = JsonFileSafetyStore(path)
    assert loaded.get_current_state() is not None
    assert loaded.get_current_state().mode == SafetyMode.SAFE
    assert len(loaded.get_directives()) == 1


def test_corrupted_file_fails_closed_safe(tmp_path: Path) -> None:
    path = tmp_path / "broken.json"
    path.write_text("not-json", encoding="utf-8")

    store = JsonFileSafetyStore(path)
    assert store.get_current_state() is not None
    assert store.get_current_state().mode == SafetyMode.SAFE

    decision = compute_decision(store.get_directives(), datetime.now(UTC))
    assert decision.mode == SafetyMode.SAFE
