from __future__ import annotations

from datetime import UTC, datetime

from libs.safety_core.engine import apply_directive
from libs.safety_core.models import ScopeKind, SafetyDirective, SafetyMode
from libs.safety_core.runtime import audit_writer, interlock_engine, slo_recorder, store


class InterlockDaemon:
    def run_once(self, snapshot: dict[str, float | int], now: datetime | None = None) -> int:
        ts = now or datetime.now(UTC)
        directives = interlock_engine.evaluate(snapshot, ts)
        applied = 0
        for directive in directives:
            detected_at = datetime.fromisoformat(directive.issued_at)
            try:
                apply_directive(
                    store,
                    audit_writer,
                    directive,
                    ts,
                    {"stable_for_seconds": 9999, "health_ok": True, "reconcile_ok": True},
                )
                slo_recorder.record_interlock_detect_to_escalate_ms(detected_at, datetime.now(UTC))
                applied += 1
            except Exception:
                fail_closed = SafetyDirective(
                    scope_kind=ScopeKind.GLOBAL,
                    selector="*",
                    mode=SafetyMode.SAFE,
                    ttl_seconds=300,
                    reason="interlock_daemon_fail_closed",
                    actor="interlock_daemon",
                    evidence={"trace_id": "interlock-fail-closed", "run_id": "interlock-daemon"},
                    issued_at=ts.isoformat(),
                )
                apply_directive(
                    store,
                    audit_writer,
                    fail_closed,
                    ts,
                    {"stable_for_seconds": 9999, "health_ok": True, "reconcile_ok": True},
                )
                applied += 1
        return applied


def run_once(snapshot: dict[str, float | int]) -> int:
    return InterlockDaemon().run_once(snapshot)
