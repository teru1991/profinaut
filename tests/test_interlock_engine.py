from __future__ import annotations

from datetime import UTC, datetime

from libs.safety_core.interlock_engine import InterlockEngine
from libs.safety_core.models import SafetyMode


def test_obs_unknown_triggers_safe_directive() -> None:
    engine = InterlockEngine()
    directives = engine.evaluate({"unknown_ratio": 0.25}, datetime.now(UTC))
    assert any(d.mode == SafetyMode.SAFE for d in directives)


def test_clock_critical_triggers_emergency_stop_latched() -> None:
    engine = InterlockEngine()
    directives = engine.evaluate({"clock_drift_ms": 2000}, datetime.now(UTC))
    assert any(d.mode == SafetyMode.EMERGENCY_STOP for d in directives)
    triggers = engine.last_triggers()
    assert any(t.rule_id == "CLOCK_CRITICAL" and t.latched for t in triggers)


def test_multiple_triggers_strongest_wins() -> None:
    engine = InterlockEngine()
    now = datetime.now(UTC)
    directives = engine.evaluate({"unknown_ratio": 0.5, "clock_drift_ms": 5000}, now)
    decision = engine.compose(directives, now)
    assert decision.mode == SafetyMode.EMERGENCY_STOP


def test_engine_does_not_emit_auto_downgrade() -> None:
    engine = InterlockEngine()
    directives = engine.evaluate({"unknown_ratio": 0.0, "clock_drift_ms": 0}, datetime.now(UTC))
    assert directives == []
