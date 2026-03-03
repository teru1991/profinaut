from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from libs.safety_core.audit import AuditEvent, AuditWriter
from libs.safety_core.models import SafetyDecision, SafetyDirective, SafetyMode, SafetyStateV1
from libs.safety_core.store import SafetyStore

PRIORITY: dict[SafetyMode, int] = {
    SafetyMode.NORMAL: 0,
    SafetyMode.SAFE: 1,
    SafetyMode.EMERGENCY_STOP: 2,
}


def compute_decision(directives: list[SafetyDirective] | None, now: datetime) -> SafetyDecision:
    if not directives:
        return SafetyDecision(mode=SafetyMode.SAFE, sources=[], latched=False, computed_at=now.isoformat())

    strongest_by_scope: dict[tuple[str, str], SafetyDirective] = {}
    for directive in directives:
        key = (directive.scope_kind.value, directive.selector)
        cur = strongest_by_scope.get(key)
        if cur is None or PRIORITY[directive.mode] > PRIORITY[cur.mode]:
            strongest_by_scope[key] = directive

    sources = list(strongest_by_scope.values())
    mode = max((d.mode for d in sources), key=lambda m: PRIORITY[m])
    return SafetyDecision(
        mode=mode,
        sources=sources,
        latched=mode == SafetyMode.EMERGENCY_STOP,
        computed_at=now.isoformat(),
    )


def can_downgrade(from_mode: SafetyMode, to_mode: SafetyMode, checks: dict[str, Any]) -> bool:
    if PRIORITY[to_mode] >= PRIORITY[from_mode]:
        return True
    return bool(
        checks.get("stable_for_seconds", 0) > 0
        and checks.get("health_ok") is True
        and checks.get("reconcile_ok") is True
    )


def apply_directive(
    store: SafetyStore,
    audit: AuditWriter,
    directive: SafetyDirective,
    now: datetime,
    checks: dict[str, Any],
) -> SafetyStateV1:
    if directive.ttl_seconds <= 0:
        raise ValueError("ttl_seconds must be > 0")
    if not directive.reason.strip():
        raise ValueError("reason is required")
    if not directive.actor.strip():
        raise ValueError("actor is required")
    if not any(k in directive.evidence for k in ("trace_id", "run_id", "audit_id")):
        raise ValueError("evidence must include one of trace_id/run_id/audit_id")

    current = store.get_current_state() or SafetyStateV1(mode=SafetyMode.SAFE, reason="fail-closed bootstrap")
    expired = store.expire_directives(now)

    if not can_downgrade(current.mode, directive.mode, checks):
        event = AuditEvent(
            event_type="SAFETY_TRANSITION_REJECTED",
            actor=directive.actor,
            scope=f"{directive.scope_kind.value}:{directive.selector}",
            mode_from=current.mode.value,
            mode_to=directive.mode.value,
            reason="downgrade checks failed",
            ttl=directive.ttl_seconds,
            evidence_ref=directive.evidence,
        )
        try:
            audit.write_event(event)
        except Exception:
            pass
        raise PermissionError("downgrade checks failed")

    existing = store.get_directives()
    decision = compute_decision(existing + [directive], now)
    new_state = SafetyStateV1(
        mode=decision.mode,
        reason=directive.reason,
        activated_by=directive.actor,
        activated_at=now.astimezone(UTC).isoformat(),
    )
    event = AuditEvent(
        event_type="SAFETY_TRANSITION_APPLIED",
        actor=directive.actor,
        scope=f"{directive.scope_kind.value}:{directive.selector}",
        mode_from=current.mode.value,
        mode_to=new_state.mode.value,
        reason=directive.reason,
        ttl=directive.ttl_seconds,
        evidence_ref={**directive.evidence, "expired_count": str(len(expired))},
    )

    if new_state.mode == SafetyMode.EMERGENCY_STOP:
        try:
            audit.write_event(event)
        except Exception:
            pass
    else:
        audit.write_event(event)

    store.put_directive(directive)
    store.set_current_state(new_state)
    return new_state
