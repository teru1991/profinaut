from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any

from app.policy_gate import PolicyGateInput, evaluate_policy_gate


@dataclass(frozen=True)
class GateContext:
    action: str
    exchange: str
    safe_mode: str
    live_enabled: bool
    live_mode: str
    live_backoff_until_utc: datetime | None
    actor_role: str | None
    current_mode: str | None
    metrics_ok: bool | None
    clock_ok: bool | None
    audit_ok: bool | None
    lease_ok: bool | None
    deps_ok: bool | None


def check_gate(ctx: GateContext) -> tuple[bool, dict[str, Any]]:
    required = [ctx.metrics_ok, ctx.clock_ok, ctx.audit_ok, ctx.lease_ok, ctx.deps_ok]
    if any(v is None for v in required):
        return False, {
            "decision": "HALT",
            "reason_code": "J_POLICY_DENY_MISSING_REQUIRED_INPUT",
            "evidence": {"missing_required": True},
            "checked_at_utc": datetime.now(timezone.utc).isoformat(),
            "safety_preconditions": {"lease": ctx.lease_ok, "audit": ctx.audit_ok},
        }

    result = evaluate_policy_gate(
        PolicyGateInput(
            action=ctx.action,  # type: ignore[arg-type]
            exchange=ctx.exchange,
            safe_mode=ctx.safe_mode,
            live_enabled=ctx.live_enabled,
            live_mode=ctx.live_mode,
            live_backoff_until_utc=ctx.live_backoff_until_utc,
            actor_role=ctx.actor_role,
            current_mode=ctx.current_mode,
            metrics_ok=ctx.metrics_ok,
            clock_ok=ctx.clock_ok,
            audit_ok=ctx.audit_ok,
            lease_ok=ctx.lease_ok,
            deps_ok=ctx.deps_ok,
        )
    )
    return result.decision == "ALLOW", {
        "decision": result.decision,
        "reason_code": result.reason_code,
        "evidence": result.evidence or {},
        "checked_at_utc": datetime.now(timezone.utc).isoformat(),
        "safety_preconditions": {"lease": ctx.lease_ok, "audit": ctx.audit_ok},
    }
