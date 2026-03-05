from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Literal

from app.j_policy_decision import GateInput, decide
from app.j_policy_ssot import JPolicySSOT, JPolicySSOTError, ssot_root_from_repo

PolicyDecision = Literal["ALLOW", "BLOCK", "THROTTLE", "REDUCE_ONLY", "CLOSE_ONLY", "FLATTEN", "HALT"]
PolicyAction = Literal["ORDER_INTENT", "CANCEL", "REPLACE"]


@dataclass(frozen=True)
class PolicyGateInput:
    action: PolicyAction
    exchange: str
    safe_mode: str
    live_enabled: bool
    live_mode: str
    live_backoff_until_utc: datetime | None
    degraded_reason: str | None = None
    actor_role: str | None = None
    current_mode: str | None = None
    metrics_ok: bool | None = None
    clock_ok: bool | None = None
    audit_ok: bool | None = None
    lease_ok: bool | None = None
    deps_ok: bool | None = None


@dataclass(frozen=True)
class PolicyGateResult:
    decision: PolicyDecision
    reason_code: str
    evidence: dict[str, Any] | None = None


def _repo_root_from_here() -> Path:
    return Path(__file__).resolve().parents[3]


def _op_from_action(action: PolicyAction) -> str:
    if action == "ORDER_INTENT":
        return "live_send_new_order"
    if action == "REPLACE":
        return "live_send_replace"
    if action == "CANCEL":
        return "live_send_cancel"
    return "unknown_op"


def _map_gate_decision_to_policy(decision: str, op: str) -> PolicyDecision:
    if decision == "ALLOW":
        return "ALLOW"
    if decision == "DENY":
        return "BLOCK"
    if decision == "HALT":
        return "HALT"
    if decision == "CANCEL_ONLY":
        if op in {"live_send_cancel", "live_send_flatten"}:
            return "ALLOW"
        return "CLOSE_ONLY"
    return "BLOCK"


def evaluate_policy_gate(gate_input: PolicyGateInput) -> PolicyGateResult:
    """Evaluate a centralized execution policy gate for all live/upstream entrypoints."""
    is_live_exchange = gate_input.exchange == "gmo"
    now = datetime.now(timezone.utc)

    if not is_live_exchange:
        return PolicyGateResult(decision="ALLOW", reason_code="POLICY_ALLOW", evidence={"note": "non-live-exchange"})

    if gate_input.safe_mode in {"SAFE_MODE", "HALTED"} and gate_input.action in {"ORDER_INTENT", "REPLACE"}:
        return PolicyGateResult(decision="BLOCK", reason_code="SAFE_MODE_BLOCKED", evidence={"safe_mode": gate_input.safe_mode})

    if not gate_input.live_enabled:
        return PolicyGateResult(decision="BLOCK", reason_code="LIVE_DISABLED")

    if gate_input.live_mode.strip().lower() != "live":
        return PolicyGateResult(decision="BLOCK", reason_code="DRY_RUN_ONLY")

    backoff_active = bool(gate_input.live_backoff_until_utc is not None and now < gate_input.live_backoff_until_utc)
    if backoff_active:
        return PolicyGateResult(
            decision="THROTTLE",
            reason_code="LIVE_DEGRADED",
            evidence={"degraded_reason": gate_input.degraded_reason or "Execution service degraded"},
        )

    repo_root = _repo_root_from_here()
    ssot_root = ssot_root_from_repo(repo_root)
    try:
        ssot = JPolicySSOT.load(ssot_root)
    except JPolicySSOTError as e:
        return PolicyGateResult(
            decision="HALT",
            reason_code="J_POLICY_DENY_UNKNOWN_INPUT",
            evidence={"ssot_error": str(e), "ssot_root": str(ssot_root)},
        )

    op = _op_from_action(gate_input.action)
    role = gate_input.actor_role or "oncall"
    current_mode = gate_input.current_mode or "SAFE"
    lease_ok = gate_input.lease_ok if gate_input.lease_ok is not None else True

    res = decide(
        ssot,
        GateInput(
            op=op,
            role=role,
            current_mode=current_mode,
            safe_mode=gate_input.safe_mode,
            metrics_ok=gate_input.metrics_ok if gate_input.metrics_ok is not None else True,
            clock_ok=gate_input.clock_ok if gate_input.clock_ok is not None else True,
            audit_ok=gate_input.audit_ok if gate_input.audit_ok is not None else True,
            lease_ok=lease_ok,
            deps_ok=gate_input.deps_ok if gate_input.deps_ok is not None else True,
            backoff_active=False,
            now_utc=now,
        ),
    )

    evidence = dict(res.evidence)
    evidence["legacy_backoff_active"] = backoff_active
    evidence["derived_lease_ok"] = lease_ok

    return PolicyGateResult(decision=_map_gate_decision_to_policy(res.decision, op), reason_code=res.reason_code, evidence=evidence)
