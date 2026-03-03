from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from typing import Any

from libs.safety_core.audit import AuditEvent, AuditWriter
from libs.safety_core.engine import apply_directive
from libs.safety_core.models import ScopeKind, SafetyDirective, SafetyMode, SafetyStateV1
from libs.safety_core.store import SafetyStore


@dataclass(slots=True)
class KillRequest:
    requested_mode: SafetyMode
    scope_kind: ScopeKind
    selector: str
    ttl_seconds: int
    reason: str
    actor: str
    idempotency_key: str
    evidence: dict[str, str]


def plan_halt_actions(mode: SafetyMode, now: datetime | None = None) -> list[str]:
    if mode != SafetyMode.EMERGENCY_STOP:
        return ["new_orders_block"]
    _ = now or datetime.now(UTC)
    return [
        "new_orders_block",
        "best_effort_cancel_open_orders",
        "isolate_or_stop_bots",
        "collect_support_bundle",
    ]


def apply_ui_kill(
    store: SafetyStore,
    audit: AuditWriter,
    request: KillRequest,
    idempotency_keys: set[str],
    now: datetime,
    checks: dict[str, Any],
) -> tuple[SafetyStateV1, bool]:
    if not request.idempotency_key.strip():
        raise ValueError("idempotency_key is required")
    if request.idempotency_key in idempotency_keys:
        state = store.get_current_state() or SafetyStateV1(mode=SafetyMode.SAFE, reason="idempotent_replay")
        return state, True

    directive = SafetyDirective(
        scope_kind=request.scope_kind,
        selector=request.selector,
        mode=request.requested_mode,
        ttl_seconds=request.ttl_seconds,
        reason=request.reason,
        actor=request.actor,
        evidence=request.evidence,
        issued_at=now.isoformat(),
    )

    try:
        state = apply_directive(store, audit, directive, now, checks)
    except PermissionError:
        _safe_write(
            audit,
            AuditEvent(
                event_type="KILL_DENIED",
                actor=request.actor,
                scope=f"{request.scope_kind.value}:{request.selector}",
                mode_from=(store.get_current_state().mode.value if store.get_current_state() else SafetyMode.SAFE.value),
                mode_to=request.requested_mode.value,
                reason="downgrade checks failed",
                ttl=request.ttl_seconds,
                evidence_ref=request.evidence,
            ),
        )
        raise

    _safe_write(
        audit,
        AuditEvent(
            event_type="KILL_APPLIED",
            actor=request.actor,
            scope=f"{request.scope_kind.value}:{request.selector}",
            mode_from=SafetyMode.SAFE.value,
            mode_to=state.mode.value,
            reason=request.reason,
            ttl=request.ttl_seconds,
            evidence_ref={**request.evidence, "actions": ",".join(plan_halt_actions(state.mode, now))},
        ),
    )
    idempotency_keys.add(request.idempotency_key)
    return state, False


def apply_local_kill(
    store: SafetyStore,
    audit: AuditWriter,
    reason: str,
    evidence_ref: dict[str, str],
    mode: SafetyMode = SafetyMode.EMERGENCY_STOP,
    now: datetime | None = None,
) -> SafetyStateV1:
    at = now or datetime.now(UTC)
    directive = SafetyDirective(
        scope_kind=ScopeKind.GLOBAL,
        selector="*",
        mode=mode,
        ttl_seconds=600,
        reason=reason,
        actor="local_kill",
        evidence=evidence_ref,
        issued_at=at.isoformat(),
    )
    state = apply_directive(
        store,
        audit,
        directive,
        at,
        {"stable_for_seconds": 9999, "health_ok": True, "reconcile_ok": True},
    )
    _safe_write(
        audit,
        AuditEvent(
            event_type="KILL_APPLIED",
            actor="local_kill",
            scope="GLOBAL:*",
            mode_from=SafetyMode.SAFE.value,
            mode_to=state.mode.value,
            reason=reason,
            ttl=directive.ttl_seconds,
            evidence_ref={**evidence_ref, "local_path": "true"},
        ),
    )
    return state


def _safe_write(audit: AuditWriter, event: AuditEvent) -> None:
    try:
        audit.write_event(event)
    except Exception:
        if event.mode_to == SafetyMode.EMERGENCY_STOP.value:
            return
        raise
